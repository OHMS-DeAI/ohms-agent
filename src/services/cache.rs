use crate::domain::*;
use crate::services::{with_state, with_state_mut};
use ic_cdk::api::time;

pub struct CacheService;

impl CacheService {
    pub fn get(layer_id: &str) -> Option<Vec<u8>> {
        let now = time();
        
        with_state_mut(|state| {
            if let Some(entry) = state.cache_entries.get_mut(layer_id) {
                entry.last_accessed = now;
                entry.access_count += 1;
                Some(entry.data.clone())
            } else {
                None
            }
        })
    }
    
    pub fn put(layer_id: String, data: Vec<u8>) -> Result<(), String> {
        let now = time();
        let size_bytes = data.len();
        
        let entry = CacheEntry {
            layer_id: layer_id.clone(),
            data,
            last_accessed: now,
            access_count: 1,
            size_bytes,
        };
        
        with_state_mut(|state| {
            // Simple LRU eviction - check if we need to make space
            let current_size: usize = state.cache_entries
                .values()
                .map(|e| e.size_bytes)
                .sum();
            
            let max_cache_size = 100 * 1024 * 1024; // 100MB limit for demo
            
            if current_size + size_bytes > max_cache_size {
                Self::evict_lru(state, size_bytes);
            }
            
            state.cache_entries.insert(layer_id, entry);
        });
        
        Ok(())
    }
    
    pub fn prefetch_layers(layer_ids: &[String]) -> Result<(), String> {
        // Mock prefetch - in real implementation this would load from model repo
        for layer_id in layer_ids {
            if !with_state(|state| state.cache_entries.contains_key(layer_id)) {
                let mock_data = vec![0u8; 1024 * 1024]; // 1MB mock layer data
                Self::put(layer_id.clone(), mock_data)?;
            }
        }
        Ok(())
    }
    
    fn evict_lru(state: &mut crate::services::AgentState, needed_space: usize) {
        let mut entries: Vec<_> = state.cache_entries
            .iter()
            .map(|(k, v)| (k.clone(), v.last_accessed, v.size_bytes))
            .collect();
            
        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|(_, accessed, _)| *accessed);
        
        let mut freed_space = 0;
        for (key, _, size) in entries {
            if freed_space >= needed_space {
                break;
            }
            
            state.cache_entries.remove(&key);
            freed_space += size;
        }
    }
    
    pub fn get_hit_rate() -> f32 {
        with_state(|state| {
            let total_requests = state.metrics.cache_hits + state.metrics.cache_misses;
            if total_requests > 0 {
                state.metrics.cache_hits as f32 / total_requests as f32
            } else {
                0.0
            }
        })
    }
    
    pub fn get_utilization() -> f32 {
        with_state(|state| {
            let current_size: usize = state.cache_entries
                .values()
                .map(|e| e.size_bytes)
                .sum();
            
            let max_size = 100 * 1024 * 1024; // 100MB
            current_size as f32 / max_size as f32
        })
    }
}