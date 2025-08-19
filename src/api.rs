use ic_cdk_macros::*;
use crate::domain::{AgentConfig, AgentHealth, InferenceRequest, InferenceResponse};
use crate::domain::instruction::*;
use crate::services::{BindingService, InferenceService, MemoryService, CacheService, InstructionAnalyzer, AgentFactory, with_state, AgentTaskResult, AgentStatusInfo, AgentSummary, AgentTask, ModelRepoClient, NOVAQValidationResult, NOVAQModelMeta};
use crate::services::agent_factory::TaskPriority;
use crate::infra::{Guards, Metrics};
use std::collections::HashMap;

#[update]
async fn bind_model(model_id: String) -> Result<(), String> {
    Guards::require_caller_authenticated()?;
    BindingService::bind_model(model_id).await
}

#[update] 
async fn infer(request: InferenceRequest) -> Result<InferenceResponse, String> {
    Guards::require_caller_authenticated()?;
    Guards::rate_limit_check()?;
    Guards::validate_prompt_length(&request.prompt)?;
    Guards::validate_msg_id(&request.msg_id)?;
    
    let result = InferenceService::process_inference(request).await?;
    Metrics::increment_inference_count();
    Ok(result)
}

#[update]
fn set_config(config: AgentConfig) -> Result<(), String> {
    Guards::require_caller_authenticated()?;
    BindingService::set_config(config)
}

#[query]
fn get_config() -> Result<AgentConfig, String> {
    Guards::require_caller_authenticated()?;
    BindingService::get_config()
}

#[query]
fn health() -> AgentHealth {
    BindingService::get_health()
}

#[query]
fn repo_canister() -> Result<String, String> {
    Guards::require_caller_authenticated()?;
    Ok(crate::services::with_state(|s| s.config.model_repo_canister_id.clone()))
}

#[update]
async fn prefetch_next(n: u32) -> Result<u32, String> {
    Guards::require_caller_authenticated()?;
    BindingService::prefetch_next(n).await
}

#[query]
fn get_loader_stats() -> Result<String, String> {
    let (bound, loaded, total, cache_util, cache_entries) = with_state(|s| {
        let bound = s.binding.is_some();
        let (loaded, total) = s.binding.as_ref().map(|b| (b.chunks_loaded, b.total_chunks)).unwrap_or((0,0));
        let util = CacheService::get_utilization();
        let entries = s.cache_entries.len();
        (bound, loaded, total, util, entries)
    });
    Ok(serde_json::json!({
        "model_bound": bound,
        "chunks_loaded": loaded,
        "total_chunks": total,
        "cache_utilization": cache_util,
        "cache_entries": cache_entries
    }).to_string())
}

#[query]
fn get_memory_stats() -> Result<String, String> {
    Guards::require_caller_authenticated()?;
    Ok(MemoryService::get_stats().to_string())
}

#[update]
fn clear_memory() -> Result<(), String> {
    Guards::require_caller_authenticated()?;
    MemoryService::clear_expired();
    Ok(())
}

// Phase 2: Instruction Analysis and Agent Factory APIs

#[update]
async fn analyze_instruction(instruction: UserInstruction) -> Result<AnalyzedInstruction, String> {
    Guards::require_caller_authenticated()?;
    InstructionAnalyzer::analyze_instruction(instruction)
}

#[update]
async fn create_agent(instruction: UserInstruction) -> Result<String, String> {
    Guards::require_caller_authenticated()?;
    
    // Analyze the instruction
    let analysis = InstructionAnalyzer::analyze_instruction(instruction.clone())?;
    
    // Create the agent
    let user_id = instruction.user_id.clone();
    let agent = AgentFactory::create_agent(user_id, instruction, analysis).await?;
    
    Ok(agent.agent_id)
}

// Compatible endpoint for UI (maps to create_agent)
#[derive(serde::Deserialize, candid::CandidType)]
pub struct AgentCreationRequest {
    pub instruction: String,
    pub agent_count: Option<u32>,
    pub capabilities: Option<Vec<String>>,
    pub priority: Option<String>,
}

#[derive(serde::Serialize, candid::CandidType)]
pub struct AgentCreationResult {
    pub agent_id: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub estimated_completion: Option<u64>,
}

#[update]
async fn create_agent_from_instruction(request: AgentCreationRequest) -> Result<AgentCreationResult, String> {
    Guards::require_caller_authenticated()?;
    
    // Convert to UserInstruction format
    let user_instruction = UserInstruction {
        instruction_text: request.instruction,
        user_id: ic_cdk::api::caller().to_string(),
        subscription_tier: SubscriptionTier::Basic, // Will be validated by coordinator
        context: Some(InstructionContext {
            domain: None,
            complexity: None,
            urgency: Some(match request.priority.as_deref() {
                Some("low") => UrgencyLevel::Low,
                Some("high") => UrgencyLevel::High,
                Some("critical") => UrgencyLevel::Critical,
                _ => UrgencyLevel::Normal,
            }),
            collaboration_needed: request.agent_count.unwrap_or(1) > 1,
            external_tools_required: vec![],
        }),
        preferences: Some(AgentPreferences {
            response_style: ResponseStyle::Conversational,
            detail_level: DetailLevel::Standard,
            creativity_level: CreativityLevel::Balanced,
            safety_level: SafetyLevel::Standard,
            language: "en".to_string(),
        }),
    };
    
    // Analyze the instruction
    let analysis = InstructionAnalyzer::analyze_instruction(user_instruction.clone())?;
    
    // Create the agent(s)
    let agent_count = request.agent_count.unwrap_or(1);
    let user_id = user_instruction.user_id.clone();
    
    if agent_count == 1 {
        let agent = AgentFactory::create_agent(user_id, user_instruction, analysis).await?;
        Ok(AgentCreationResult {
            agent_id: agent.agent_id,
            status: "Ready".to_string(),
            capabilities: request.capabilities.unwrap_or_else(|| vec!["General Assistant".to_string()]),
            estimated_completion: Some(ic_cdk::api::time() + 30_000_000_000), // 30 seconds from now
        })
    } else {
        let agents = AgentFactory::create_coordinated_agents(user_id, user_instruction, analysis).await?;
        // Return first agent ID (coordinator)
        let primary_agent = agents.first().ok_or("Failed to create coordinated agents")?;
        Ok(AgentCreationResult {
            agent_id: primary_agent.agent_id.clone(),
            status: "Ready".to_string(),
            capabilities: request.capabilities.unwrap_or_else(|| vec!["Coordinated Team".to_string()]),
            estimated_completion: Some(ic_cdk::api::time() + 60_000_000_000), // 60 seconds for coordinated
        })
    }
}

#[update]
async fn create_coordinated_agents(instruction: UserInstruction) -> Result<Vec<String>, String> {
    Guards::require_caller_authenticated()?;
    
    // Analyze the instruction
    let analysis = InstructionAnalyzer::analyze_instruction(instruction.clone())?;
    
    // Create coordinated agents
    let user_id = instruction.user_id.clone();
    let agents = AgentFactory::create_coordinated_agents(user_id, instruction, analysis).await?;
    
    Ok(agents.into_iter().map(|a| a.agent_id).collect())
}

#[update]
async fn execute_agent_task(agent_id: String, task_description: String) -> Result<AgentTaskResult, String> {
    Guards::require_caller_authenticated()?;
    
    let task = AgentTask {
        task_id: format!("task-{}", ic_cdk::api::time()),
        description: task_description,
        priority: TaskPriority::Normal,
        deadline: None,
        context: HashMap::new(),
    };
    
    AgentFactory::execute_task(&agent_id, task).await
}

#[query]
async fn get_agent_status(agent_id: String) -> Result<AgentStatusInfo, String> {
    Guards::require_caller_authenticated()?;
    AgentFactory::get_agent_status(&agent_id).await
}

#[query]
async fn list_user_agents(user_id: String) -> Result<Vec<AgentSummary>, String> {
    Guards::require_caller_authenticated()?;
    AgentFactory::list_user_agents(&user_id).await
}

// NOVAQ Validation APIs

#[update]
async fn validate_novaq_model(model_id: String, model_data: Vec<u8>) -> Result<NOVAQValidationResult, String> {
    Guards::require_caller_authenticated()?;
    ModelRepoClient::validate_novaq_model(&model_id, &model_data).await
}

#[query]
async fn extract_novaq_metadata(model_data: Vec<u8>) -> Result<NOVAQModelMeta, String> {
    Guards::require_caller_authenticated()?;
    ModelRepoClient::extract_novaq_metadata(&model_data).await
}

#[query]
fn is_novaq_model(model_data: Vec<u8>) -> bool {
    ModelRepoClient::is_novaq_model(&model_data)
}

#[query]
fn get_novaq_quality_score(model_data: Vec<u8>) -> Result<f64, String> {
    ModelRepoClient::get_novaq_quality_score(&model_data)
}