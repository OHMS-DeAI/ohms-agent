use ic_cdk_macros::*;
use crate::domain::{AgentConfig, AgentHealth, InferenceRequest, InferenceResponse};
use crate::services::{BindingService, InferenceService, MemoryService};
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