use crate::domain::*;
use crate::domain::instruction::*;
use std::collections::HashMap;
use std::cell::RefCell;

pub mod binding;
pub mod inference;
pub mod memory;
pub mod cache;
pub mod modelrepo;
pub mod instruction_analyzer;
pub mod agent_factory;
pub mod novaq_validation;
pub mod dfinity_llm;

pub use binding::BindingService;
pub use inference::InferenceService;
pub use memory::MemoryService;
pub use cache::CacheService;
pub use modelrepo::ModelRepoClient;
pub use instruction_analyzer::InstructionAnalyzer;
pub use agent_factory::{AgentFactory, AutonomousAgent, AgentTask, AgentTaskResult, AgentStatusInfo, AgentSummary};
pub use novaq_validation::{NOVAQValidationService, NOVAQValidationResult, NOVAQModelMeta};
// Note: Currently supports only Llama 3.1 8B
// Architecture is designed to easily add new models when they become available
pub use dfinity_llm::{DfinityLlmService, QuantizedModel, ChatMessage, MessageRole, ConversationSession, TokenUsage, UserQuota, LlmError};
use modelrepo::ModelManifest;

thread_local! {
    static STATE: RefCell<AgentState> = RefCell::new(AgentState::default());
}

#[derive(Debug, Default)]
pub struct AgentState {
    pub config: AgentConfig,
    pub binding: Option<ModelBinding>,
    pub manifest: Option<ModelManifest>,
    pub memory_entries: HashMap<String, MemoryEntry>,
    pub cache_entries: HashMap<String, CacheEntry>,
    pub metrics: AgentMetrics,
    pub agents: HashMap<String, AutonomousAgent>,
    pub llm_service: DfinityLlmService,
}

#[derive(Debug, Default)]
pub struct AgentMetrics {
    pub total_inferences: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_inference_time_ms: f64,
    pub last_activity: u64,
}

pub fn with_state<R>(f: impl FnOnce(&AgentState) -> R) -> R {
    STATE.with(|s| f(&*s.borrow()))
}

pub fn with_state_mut<R>(f: impl FnOnce(&mut AgentState) -> R) -> R {
    STATE.with(|s| f(&mut *s.borrow_mut()))
}