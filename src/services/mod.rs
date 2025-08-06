use crate::domain::*;
use ic_cdk::api::time;
use std::collections::HashMap;
use std::cell::RefCell;

pub mod binding;
pub mod inference;
pub mod memory;
pub mod cache;

pub use binding::BindingService;
pub use inference::InferenceService; 
pub use memory::MemoryService;
pub use cache::CacheService;

thread_local! {
    static STATE: RefCell<AgentState> = RefCell::new(AgentState::default());
}

#[derive(Debug, Default)]
pub struct AgentState {
    pub config: AgentConfig,
    pub binding: Option<ModelBinding>,
    pub memory_entries: HashMap<String, MemoryEntry>,
    pub cache_entries: HashMap<String, CacheEntry>,
    pub metrics: AgentMetrics,
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