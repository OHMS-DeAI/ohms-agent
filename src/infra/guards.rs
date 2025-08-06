use ic_cdk::api::{caller, time};
use candid::Principal;
use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    static RATE_LIMITS: RefCell<HashMap<Principal, RateLimit>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone)]
struct RateLimit {
    requests: u32,
    window_start: u64,
    blocked_until: u64,
}

pub struct Guards;

impl Guards {
    pub fn require_caller_authenticated() -> Result<(), String> {
        let caller = caller();
        if caller == Principal::anonymous() {
            return Err("Authentication required".to_string());
        }
        Ok(())
    }
    
    pub fn require_admin() -> Result<(), String> {
        Self::require_caller_authenticated()?;
        // TODO: Implement proper admin check with governance canister
        Ok(())
    }
    
    pub fn rate_limit_check() -> Result<(), String> {
        let caller = caller();
        let now = time();
        let window_duration = 60 * 1_000_000_000; // 1 minute in nanoseconds
        let max_requests_per_window = 100;
        
        RATE_LIMITS.with(|limits| {
            let mut limits = limits.borrow_mut();
            let limit = limits.entry(caller).or_insert(RateLimit {
                requests: 0,
                window_start: now,
                blocked_until: 0,
            });
            
            // Check if still blocked
            if now < limit.blocked_until {
                return Err(format!("Rate limited. Try again in {} seconds", 
                    (limit.blocked_until - now) / 1_000_000_000));
            }
            
            // Reset window if expired
            if now - limit.window_start > window_duration {
                limit.requests = 0;
                limit.window_start = now;
            }
            
            limit.requests += 1;
            
            if limit.requests > max_requests_per_window {
                limit.blocked_until = now + window_duration;
                return Err("Rate limit exceeded. Try again later".to_string());
            }
            
            Ok(())
        })
    }
    
    pub fn validate_prompt_length(prompt: &str) -> Result<(), String> {
        const MAX_PROMPT_LENGTH: usize = 10_000; // 10k characters
        
        if prompt.len() > MAX_PROMPT_LENGTH {
            return Err(format!("Prompt too long. Max length: {}", MAX_PROMPT_LENGTH));
        }
        
        Ok(())
    }
    
    pub fn validate_msg_id(msg_id: &str) -> Result<(), String> {
        if msg_id.is_empty() || msg_id.len() > 64 {
            return Err("Invalid msg_id format".to_string());
        }
        
        // Check for valid characters (alphanumeric + underscore/dash)
        if !msg_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("msg_id contains invalid characters".to_string());
        }
        
        Ok(())
    }
    
    pub fn check_memory_limits() -> Result<(), String> {
        // TODO: Implement actual memory usage checks
        // For now, just return Ok for bootstrap milestone
        Ok(())
    }
}