use crate::domain::*;
use crate::services::{with_state, with_state_mut};
use ic_cdk::api::time;
use serde_json::Value;

pub struct MemoryService;

impl MemoryService {
    pub fn store(key: String, data: Vec<u8>, ttl_seconds: u64, encrypt: bool) -> Result<(), String> {
        let now = time();
        let expires_at = now + ttl_seconds * 1_000_000_000; // Convert to nanoseconds
        
        let encrypted_data = if encrypt {
            Self::encrypt_data(&data)?
        } else {
            data
        };
        
        let entry = MemoryEntry {
            key: key.clone(),
            data: encrypted_data,
            created_at: now,
            expires_at,
            encrypted: encrypt,
        };
        
        with_state_mut(|state| {
            state.memory_entries.insert(key, entry);
        });
        
        Ok(())
    }
    
    pub fn retrieve(key: &str) -> Result<Vec<u8>, String> {
        let now = time();
        
        with_state_mut(|state| {
            if let Some(entry) = state.memory_entries.get(key) {
                if entry.expires_at > now {
                    let data = if entry.encrypted {
                        Self::decrypt_data(&entry.data)?
                    } else {
                        entry.data.clone()
                    };
                    Ok(data)
                } else {
                    // Entry expired, remove it
                    state.memory_entries.remove(key);
                    Err("Entry expired".to_string())
                }
            } else {
                Err("Entry not found".to_string())
            }
        })
    }
    
    pub fn clear_expired() {
        let now = time();
        
        with_state_mut(|state| {
            state.memory_entries.retain(|_, entry| entry.expires_at > now);
        });
    }
    
    pub fn get_stats() -> Value {
        with_state(|state| {
            let now = time();
            let active_entries = state.memory_entries
                .values()
                .filter(|entry| entry.expires_at > now)
                .count();
            
            let total_size: usize = state.memory_entries
                .values()
                .map(|entry| entry.data.len())
                .sum();
            
            serde_json::json!({
                "active_entries": active_entries,
                "total_entries": state.memory_entries.len(),
                "total_size_bytes": total_size,
                "encrypted_entries": state.memory_entries
                    .values()
                    .filter(|entry| entry.encrypted)
                    .count()
            })
        })
    }
    
    fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
        // Simple XOR encryption for demo - in production use proper encryption
        let key = b"ohms_agent_key_32_bytes_exactly!";
        let mut encrypted = Vec::with_capacity(data.len());
        
        for (i, byte) in data.iter().enumerate() {
            encrypted.push(byte ^ key[i % key.len()]);
        }
        
        Ok(encrypted)
    }
    
    fn decrypt_data(encrypted: &[u8]) -> Result<Vec<u8>, String> {
        // Same XOR operation for decryption
        Self::encrypt_data(encrypted)
    }
}