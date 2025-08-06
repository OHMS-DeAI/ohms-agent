use crate::domain::*;
use crate::services::{with_state, with_state_mut};
use ic_cdk::api::time;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

pub struct BindingService;

impl BindingService {
    pub async fn bind_model(model_id: String) -> Result<(), String> {
        // TODO: In real implementation, fetch manifest from ohms-model canister
        // For now, create a mock binding for the bootstrap milestone
        let manifest_digest = Self::compute_manifest_digest(&model_id)?;
        
        let binding = ModelBinding {
            model_id: model_id.clone(),
            bound_at: time(),
            manifest_digest,
            chunks_loaded: 0,
            total_chunks: 8, // Mock value
        };
        
        with_state_mut(|state| {
            state.binding = Some(binding);
            state.metrics.last_activity = time();
        });
        
        Ok(())
    }
    
    pub fn set_config(config: AgentConfig) -> Result<(), String> {
        with_state_mut(|state| {
            state.config = config;
        });
        Ok(())
    }
    
    pub fn get_config() -> Result<AgentConfig, String> {
        Ok(with_state(|state| state.config.clone()))
    }
    
    pub fn get_health() -> AgentHealth {
        with_state(|state| {
            let cache_hits = state.metrics.cache_hits;
            let cache_misses = state.metrics.cache_misses;
            let total_requests = cache_hits + cache_misses;
            
            let hit_rate = if total_requests > 0 {
                cache_hits as f32 / total_requests as f32
            } else {
                0.0
            };
            
            let warm_set_utilization = state.cache_entries.len() as f32 / 100.0; // Mock calculation
            
            AgentHealth {
                model_bound: state.binding.is_some(),
                cache_hit_rate: hit_rate,
                warm_set_utilization,
                queue_depth: 0, // TODO: Implement proper queue tracking
                last_inference_timestamp: state.metrics.last_activity,
            }
        })
    }
    
    fn compute_manifest_digest(model_id: &str) -> Result<String, String> {
        let mut hasher = Sha256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(time().to_be_bytes());
        Ok(general_purpose::STANDARD.encode(hasher.finalize()))
    }
}