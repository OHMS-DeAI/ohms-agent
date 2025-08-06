use crate::domain::*;
use crate::services::{with_state, with_state_mut, CacheService};
use ic_cdk::api::time;
use rand::{SeedableRng, Rng};
use rand_chacha::ChaCha8Rng;

pub struct InferenceService;

impl InferenceService {
    pub async fn process_inference(request: InferenceRequest) -> Result<InferenceResponse, String> {
        let start_time = time();
        
        // Verify model is bound
        let model_bound = with_state(|state| state.binding.is_some());
        if !model_bound {
            return Err("No model bound to agent".to_string());
        }
        
        // Deterministic seed generation from msg_id
        let seed = Self::derive_seed(&request.msg_id, request.seed);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        
        // Mock token generation with deterministic randomness
        let tokens = Self::generate_tokens(&mut rng, &request.prompt, &request.decode_params)?;
        let generated_text = tokens.join("");
        
        let inference_time_ms = time() - start_time;
        
        // Update metrics
        let (cache_hits, cache_misses) = with_state_mut(|state| {
            state.metrics.total_inferences += 1;
            state.metrics.last_activity = time();
            
            // Mock cache stats - in real implementation this would be based on actual cache access
            let hits = rng.gen_range(5..15);
            let misses = rng.gen_range(0..5);
            
            state.metrics.cache_hits += hits;
            state.metrics.cache_misses += misses;
            
            (hits as u32, misses as u32)
        });
        
        Ok(InferenceResponse {
            tokens,
            generated_text,
            inference_time_ms,
            cache_hits,
            cache_misses,
        })
    }
    
    fn derive_seed(msg_id: &str, user_seed: u64) -> u64 {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(msg_id.as_bytes());
        hasher.update(user_seed.to_be_bytes());
        let hash = hasher.finalize();
        u64::from_be_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7]
        ])
    }
    
    fn generate_tokens(
        rng: &mut ChaCha8Rng, 
        prompt: &str, 
        params: &DecodeParams
    ) -> Result<Vec<String>, String> {
        let max_tokens = params.max_tokens.unwrap_or(512);
        let mut tokens = Vec::new();
        
        // Simple mock tokenization - in real implementation this would use the actual model
        let words = [
            "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog.",
            "This", "is", "a", "deterministic", "inference", "response", "for", "testing.",
            "OHMS", "agent", "system", "working", "correctly", "with", "seeded", "generation.",
        ];
        
        let prompt_hash = prompt.len() % words.len();
        
        for i in 0..max_tokens.min(50) {
            let word_idx = (prompt_hash + i as usize + rng.gen_range(0..words.len())) % words.len();
            tokens.push(words[word_idx].to_string());
            
            if tokens.len() >= max_tokens as usize {
                break;
            }
        }
        
        Ok(tokens)
    }
}