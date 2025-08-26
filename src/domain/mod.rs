use serde::{Deserialize, Serialize};
use candid::CandidType;

pub mod instruction;
pub use instruction::*;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AgentConfig {
    pub warm_set_target: f32,
    pub prefetch_depth: u32,
    pub max_tokens: u32,
    pub concurrency_limit: u32,
    pub ttl_seconds: u64,
    pub model_repo_canister_id: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            warm_set_target: 0.6,
            prefetch_depth: 2,
            max_tokens: 2048,
            concurrency_limit: 4,
            ttl_seconds: 3600,
            model_repo_canister_id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AgentHealth {
    pub model_bound: bool,
    pub cache_hit_rate: f32,
    pub warm_set_utilization: f32,
    pub queue_depth: u32,
    pub last_inference_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InferenceRequest {
    pub seed: u64,
    pub prompt: String,
    pub decode_params: DecodeParams,
    pub msg_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DecodeParams {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub repetition_penalty: Option<f32>,
}

impl Default for DecodeParams {
    fn default() -> Self {
        Self {
            max_tokens: Some(512),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(50),
            repetition_penalty: Some(1.1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InferenceResponse {
    pub tokens: Vec<String>,
    pub generated_text: String,
    pub inference_time_ms: u64,
    pub cache_hits: u32,
    pub cache_misses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ModelBinding {
    pub model_id: String,
    pub bound_at: u64,
    pub manifest_digest: String,
    pub chunks_loaded: u32,
    pub total_chunks: u32,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct MemoryEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub created_at: u64,
    pub expires_at: u64,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]  
pub struct CacheEntry {
    pub layer_id: String,
    pub data: Vec<u8>,
    pub last_accessed: u64,
    pub access_count: u32,
    pub size_bytes: usize,
}