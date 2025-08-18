use ic_cdk_macros::*;
use crate::domain::{AgentConfig, AgentHealth, InferenceRequest, InferenceResponse};
use crate::domain::instruction::*;
use crate::services::{BindingService, InferenceService, MemoryService, CacheService, InstructionAnalyzer, AgentFactory, with_state, AgentTaskResult, AgentStatusInfo, AgentSummary, AgentTask};
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