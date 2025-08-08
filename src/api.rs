use ic_cdk_macros::*;
use crate::domain::{AgentConfig, AgentHealth, InferenceRequest, InferenceResponse};
use crate::services::{BindingService, InferenceService, MemoryService, CacheService, with_state};
use crate::infra::{Guards, Metrics};

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