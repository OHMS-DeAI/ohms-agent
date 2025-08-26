use crate::domain::instruction::*;
use crate::domain::{AgentConfig, ModelBinding};
use crate::services::{BindingService, with_state, with_state_mut};
use std::collections::HashMap;
use candid::CandidType;

/// Service for creating autonomous agents from analyzed instructions
pub struct AgentFactory;

/// Autonomous agent instance with full configuration
#[derive(Debug, Clone)]
pub struct AutonomousAgent {
    pub agent_id: String,
    pub user_id: String,
    pub instruction: UserInstruction,
    pub analysis: AnalyzedInstruction,
    pub config: AgentConfig,
    pub model_binding: Option<ModelBinding>,
    pub status: AgentStatus,
    pub created_at: u64,
    pub last_active: u64,
    pub memory: HashMap<String, Vec<u8>>,
    pub performance_metrics: AgentPerformanceMetrics,
}

/// Agent status tracking
#[derive(Debug, Clone, CandidType)]
pub enum AgentStatus {
    Creating,       // Agent is being initialized
    Ready,          // Agent is ready to receive tasks
    Active,         // Agent is actively working
    Paused,         // Agent is paused by user
    Completed,      // Agent has completed its task
    Error(String),  // Agent encountered an error
}

/// Performance metrics for agent monitoring
#[derive(Debug, Clone, Default, CandidType)]
pub struct AgentPerformanceMetrics {
    pub tasks_completed: u32,
    pub total_tokens_used: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f32,
    pub last_task_timestamp: u64,
}

impl AgentFactory {
    /// Create a new autonomous agent from analyzed instruction
    pub async fn create_agent(
        user_id: String,
        instruction: UserInstruction,
        analysis: AnalyzedInstruction,
    ) -> Result<AutonomousAgent, String> {
        // Validate user subscription and quotas
        Self::validate_user_quotas(&user_id, &instruction.subscription_tier).await?;

        // Generate unique agent ID
        let agent_id = Self::generate_agent_id(&user_id);

        // Create agent configuration
        let config = Self::create_agent_config(&analysis)?;

        // Initialize agent
        let mut agent = AutonomousAgent {
            agent_id: agent_id.clone(),
            user_id,
            instruction,
            analysis,
            config,
            model_binding: None,
            status: AgentStatus::Creating,
            created_at: ic_cdk::api::time(),
            last_active: ic_cdk::api::time(),
            memory: HashMap::new(),
            performance_metrics: AgentPerformanceMetrics::default(),
        };

        // Bind to appropriate NOVAQ model
        agent.model_binding = Self::bind_novaq_model(&agent).await?;

        // Update agent status
        agent.status = AgentStatus::Ready;

        // Store agent in state
        Self::store_agent(agent.clone()).await?;

        Ok(agent)
    }

    /// Create multiple coordinated agents for complex tasks
    pub async fn create_coordinated_agents(
        user_id: String,
        instruction: UserInstruction,
        analysis: AnalyzedInstruction,
    ) -> Result<Vec<AutonomousAgent>, String> {
        if !analysis.coordination_requirements.requires_coordination {
            return Err("No coordination required for this instruction".to_string());
        }

        let mut agents = Vec::new();
        let agent_count = analysis.coordination_requirements.agent_count;

        // Create specialized agents based on capabilities
        for (index, capability) in analysis.extracted_capabilities.iter().enumerate() {
            if index >= agent_count as usize {
                break;
            }

            // Create specialized instruction for this agent
            let specialized_instruction = Self::create_specialized_instruction(
                &instruction,
                capability,
                index,
                agent_count,
            );

            // Create specialized analysis
            let specialized_analysis = Self::create_specialized_analysis(
                &analysis,
                capability,
                index,
                agent_count,
            );

            // Create the agent
            let agent = Self::create_agent(
                user_id.clone(),
                specialized_instruction,
                specialized_analysis,
            ).await?;

            agents.push(agent);
        }

        Ok(agents)
    }

    /// Execute a task with the autonomous agent
    pub async fn execute_task(
        agent_id: &str,
        task: AgentTask,
    ) -> Result<AgentTaskResult, String> {
        let mut agent = Self::get_agent(agent_id).await?;

        // Update agent status
        agent.status = AgentStatus::Active;
        agent.last_active = ic_cdk::api::time();
        Self::update_agent(&agent).await?;

        // Execute the task based on agent type and capabilities
        let result = match agent.analysis.agent_configuration.agent_type {
            AgentType::CodeAssistant => Self::execute_code_task(&agent, &task).await?,
            AgentType::DataAnalyst => Self::execute_data_task(&agent, &task).await?,
            AgentType::ContentCreator => Self::execute_content_task(&agent, &task).await?,
            AgentType::ProblemSolver => Self::execute_problem_task(&agent, &task).await?,
            AgentType::Researcher => Self::execute_research_task(&agent, &task).await?,
            AgentType::Planner => Self::execute_planning_task(&agent, &task).await?,
            _ => Self::execute_general_task(&agent, &task).await?,
        };

        // Update performance metrics
        agent.performance_metrics.tasks_completed += 1;
        agent.performance_metrics.total_tokens_used += result.tokens_used;
        agent.performance_metrics.last_task_timestamp = ic_cdk::api::time();
        agent.status = AgentStatus::Ready;

        Self::update_agent(&agent).await?;

        Ok(result)
    }

    /// Get agent status and performance
    pub async fn get_agent_status(agent_id: &str) -> Result<AgentStatusInfo, String> {
        let agent = Self::get_agent(agent_id).await?;

        Ok(AgentStatusInfo {
            agent_id: agent.agent_id.clone(),
            status: agent.status.clone(),
            performance_metrics: agent.performance_metrics.clone(),
            model_bound: agent.model_binding.is_some(),
            created_at: agent.created_at,
            last_active: agent.last_active,
        })
    }

    /// List all agents for a user
    pub async fn list_user_agents(user_id: &str) -> Result<Vec<AgentSummary>, String> {
        Ok(with_state(|state| {
            state.agents
                .iter()
                .filter(|(_, agent)| agent.user_id == user_id)
                .map(|(id, agent)| AgentSummary {
                    agent_id: id.clone(),
                    agent_type: agent.analysis.agent_configuration.agent_type.clone(),
                    status: agent.status.clone(),
                    created_at: agent.created_at,
                    last_active: agent.last_active,
                })
                .collect::<Vec<_>>()
        }))
    }

    // Private helper methods

    async fn validate_user_quotas(user_id: &str, _tier: &SubscriptionTier) -> Result<(), String> {
        // Call the economics canister to validate subscription quotas
        // This will be implemented when we integrate with the economics canister
        // For now, we'll use a simple validation
        
        // Check agent creation limits
        let user_agents = Self::list_user_agents(user_id).await?;
        
        // Get user subscription from economics canister
        // TODO: Implement cross-canister call to economics canister
        // let subscription = econ_canister::get_user_subscription(user_id).await?;
        
        // For now, use a default limit
        let max_agents = 25; // Default to Pro tier limit
        
        if user_agents.len() >= max_agents {
            return Err(format!("Agent limit reached. Maximum: {}", max_agents));
        }

        Ok(())
    }

    fn generate_agent_id(user_id: &str) -> String {
        let timestamp = ic_cdk::api::time();
        format!("agent-{}-{}", user_id, timestamp)
    }

    fn create_agent_config(analysis: &AnalyzedInstruction) -> Result<AgentConfig, String> {
        let model_repo_id = with_state(|state| state.config.model_repo_canister_id.clone());
        
        Ok(AgentConfig {
            warm_set_target: 0.7,
            prefetch_depth: 3,
            max_tokens: analysis.model_requirements.minimum_context_length,
            concurrency_limit: match analysis.coordination_requirements.agent_count {
                1 => 2,
                2..=5 => 4,
                _ => 8,
            },
            ttl_seconds: 7200, // 2 hours
            model_repo_canister_id: model_repo_id,
        })
    }

    async fn bind_novaq_model(agent: &AutonomousAgent) -> Result<Option<ModelBinding>, String> {
        // Select the best available NOVAQ model
        let recommended_model = agent.analysis.model_requirements.recommended_models
            .first()
            .ok_or("No recommended models available")?;

        // Try to bind to the recommended model
        match BindingService::bind_model(recommended_model.clone()).await {
            Ok(_) => {
                // Get the binding details
                Ok(with_state(|state| {
                    state.binding.clone()
                }))
            }
            Err(_) => {
                // Fallback to any available NOVAQ model
                let fallback_models = vec![
                    "llama-2-7b-novaq".to_string(),
                    "codellama-7b-novaq".to_string(),
                    "vicuna-7b-novaq".to_string(),
                ];

                for model in fallback_models {
                    if BindingService::bind_model(model.clone()).await.is_ok() {
                        return Ok(with_state(|state| state.binding.clone()));
                    }
                }

                Err("No NOVAQ models available for binding".to_string())
            }
        }
    }

    async fn store_agent(agent: AutonomousAgent) -> Result<(), String> {
        with_state_mut(|state| {
            state.agents.insert(agent.agent_id.clone(), agent);
        });
        Ok(())
    }

    async fn get_agent(agent_id: &str) -> Result<AutonomousAgent, String> {
        with_state(|state| {
            state.agents.get(agent_id)
                .cloned()
                .ok_or_else(|| format!("Agent {} not found", agent_id))
        })
    }

    async fn update_agent(agent: &AutonomousAgent) -> Result<(), String> {
        with_state_mut(|state| {
            state.agents.insert(agent.agent_id.clone(), agent.clone());
        });
        Ok(())
    }

    fn create_specialized_instruction(
        original: &UserInstruction,
        capability: &Capability,
        index: usize,
        total: u32,
    ) -> UserInstruction {
        let specialized_text = format!(
            "Specialized agent {} of {}: {} - {}",
            index + 1,
            total,
            capability.name,
            original.instruction_text
        );

        UserInstruction {
            instruction_text: specialized_text,
            user_id: original.user_id.clone(),
            subscription_tier: original.subscription_tier.clone(),
            context: original.context.clone(),
            preferences: original.preferences.clone(),
        }
    }

    fn create_specialized_analysis(
        original: &AnalyzedInstruction,
        capability: &Capability,
        _index: usize,
        total: u32,
    ) -> AnalyzedInstruction {
        let mut specialized = original.clone();
        specialized.extracted_capabilities = vec![capability.clone()];
        specialized.coordination_requirements.agent_count = total;
        specialized.estimated_complexity = ComplexityLevel::Simple;
        specialized.confidence_score = 0.9;
        specialized
    }

    // Task execution methods for different agent types
    async fn execute_code_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        // Use the agent's model binding to generate code
        let prompt = format!(
            "You are a specialized code assistant. {}",
            task.description
        );

        // Execute inference using the bound model
        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }

    async fn execute_data_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        let prompt = format!(
            "You are a data analyst. Analyze and provide insights for: {}",
            task.description
        );

        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }

    async fn execute_content_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        let prompt = format!(
            "You are a content creator. Create engaging content for: {}",
            task.description
        );

        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }

    async fn execute_problem_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        let prompt = format!(
            "You are a problem solver. Analyze and solve: {}",
            task.description
        );

        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }

    async fn execute_research_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        let prompt = format!(
            "You are a researcher. Research and provide information about: {}",
            task.description
        );

        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }

    async fn execute_planning_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        let prompt = format!(
            "You are a planner. Create a plan for: {}",
            task.description
        );

        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }

    async fn execute_general_task(_agent: &AutonomousAgent, task: &AgentTask) -> Result<AgentTaskResult, String> {
        let prompt = format!(
            "You are a helpful assistant. Help with: {}",
            task.description
        );

        let inference_request = crate::domain::InferenceRequest {
            seed: task.task_id.parse().unwrap_or(0),
            prompt,
            decode_params: crate::domain::DecodeParams::default(),
            msg_id: task.task_id.clone(),
        };

        let response = crate::services::InferenceService::process_inference(inference_request).await?;

        Ok(AgentTaskResult {
            task_id: task.task_id.clone(),
            success: true,
            result: response.generated_text,
            tokens_used: response.tokens.len() as u64,
            execution_time_ms: response.inference_time_ms,
            error_message: None,
        })
    }
}

// Additional data structures for agent management

#[derive(Debug, Clone, CandidType)]
pub struct AgentTask {
    pub task_id: String,
    pub description: String,
    pub priority: TaskPriority,
    pub deadline: Option<u64>,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, CandidType)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, CandidType)]
pub struct AgentTaskResult {
    pub task_id: String,
    pub success: bool,
    pub result: String,
    pub tokens_used: u64,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, CandidType)]
pub struct AgentStatusInfo {
    pub agent_id: String,
    pub status: AgentStatus,
    pub performance_metrics: AgentPerformanceMetrics,
    pub model_bound: bool,
    pub created_at: u64,
    pub last_active: u64,
}

#[derive(Debug, Clone, CandidType)]
pub struct AgentSummary {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub created_at: u64,
    pub last_active: u64,
}
