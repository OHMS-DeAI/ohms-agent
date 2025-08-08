use crate::domain::*;
use crate::services::{with_state, with_state_mut, ModelRepoClient, CacheService};
use ic_cdk::api::time;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

pub struct BindingService;

impl BindingService {
    pub async fn bind_model(model_id: String) -> Result<(), String> {
        // Real binding: fetch manifest and prefetch chunks from ohms-model canister
        let repo_canister = with_state(|s| s.config.model_repo_canister_id.clone());
        if repo_canister.is_empty() { return Err("model_repo_canister_id not configured".to_string()); }

        let manifest = ModelRepoClient::get_manifest(&repo_canister, &model_id).await?;
        // Ensure Active state (avoid binding Pending/Deprecated)
        match manifest.state {
            crate::services::modelrepo::ModelState::Active => {},
            _ => return Err("model is not Active".to_string()),
        }

        // Prefetch first N chunks
        let prefetch_n = with_state(|s| s.config.prefetch_depth);
        let mut loaded: u32 = 0;
        for chunk in manifest.chunks.iter().take(prefetch_n as usize) {
            let bytes = ModelRepoClient::get_chunk(&repo_canister, &model_id, &chunk.id).await?;
            CacheService::put(chunk.id.clone(), bytes)?;
            loaded += 1;
        }

        let binding = ModelBinding {
            model_id: model_id.clone(),
            bound_at: time(),
            manifest_digest: manifest.digest.clone(),
            chunks_loaded: loaded,
            total_chunks: manifest.chunks.len() as u32,
            version: manifest.version.clone(),
        };

        with_state_mut(|state| {
            state.manifest = Some(manifest);
            state.binding = Some(binding);
            state.metrics.last_activity = time();
        });
        Ok(())
    }
    
    pub async fn prefetch_next(n: u32) -> Result<u32, String> {
        let (repo_canister, model_id, already_loaded, manifest_opt) = with_state(|s| {
            (s.config.model_repo_canister_id.clone(),
             s.binding.as_ref().map(|b| b.model_id.clone()),
             s.binding.as_ref().map(|b| b.chunks_loaded).unwrap_or(0),
             s.manifest.clone())
        });
        if repo_canister.is_empty() { return Err("model_repo_canister_id not configured".into()); }
        let model_id = model_id.ok_or_else(|| "no model bound".to_string())?;
        let manifest = manifest_opt.ok_or_else(|| "manifest not loaded".to_string())?;
        let mut loaded = 0u32;
        for chunk in manifest.chunks.iter().skip(already_loaded as usize).take(n as usize) {
            let bytes = ModelRepoClient::get_chunk(&repo_canister, &model_id, &chunk.id).await?;
            CacheService::put(chunk.id.clone(), bytes)?;
            loaded += 1;
        }
        with_state_mut(|s| {
            if let Some(b) = &mut s.binding {
                b.chunks_loaded += loaded;
            }
        });
        Ok(loaded)
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