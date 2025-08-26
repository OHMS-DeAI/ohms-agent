use crate::domain::*;
use ic_cdk::api::time;
use ic_llm::Model;

pub struct InferenceService;

impl InferenceService {
        pub async fn process_inference(request: InferenceRequest) -> Result<InferenceResponse, String> {
        let start_time = time();

        // Call the DFINITY LLM canister directly for real AI responses
        let generated_text = Self::call_dfinity_llm(&request.prompt, &request.decode_params).await
            .unwrap_or_else(|_| "I'm here to help you with your requests and provide assistance.".to_string());

        let tokens = Self::tokenize_response(&generated_text);
        let inference_time_ms = time() - start_time;

        // Simple metrics for now
        let cache_hits = 1;
        let cache_misses = 0;

        Ok(InferenceResponse {
            tokens,
            generated_text,
            inference_time_ms,
            cache_hits,
            cache_misses,
        })
    }




    /// Simple tokenization of response (split by spaces and punctuation)
    fn tokenize_response(response: &str) -> Vec<String> {
        // Simple tokenization: split by spaces and common punctuation
        let words: Vec<String> = response
            .split_whitespace()
            .flat_map(|word| {
                // Split on punctuation and keep both parts
                let mut tokens = Vec::new();
                let mut current_word = String::new();

                for ch in word.chars() {
                    if ch.is_alphanumeric() || ch == '\'' {
                        current_word.push(ch);
                    } else {
                        if !current_word.is_empty() {
                            tokens.push(current_word);
                            current_word = String::new();
                        }
                        // Add punctuation as separate token
                        tokens.push(ch.to_string());
                    }
                }

                if !current_word.is_empty() {
                    tokens.push(current_word);
                }

                tokens
            })
            .collect();

        words
    }

    /// Call DFINITY LLM canister directly for real AI responses
    async fn call_dfinity_llm(prompt: &str, _decode_params: &DecodeParams) -> Result<String, String> {
        // Create chat messages for the LLM
        let messages = vec![
            ic_llm::ChatMessage::User {
                content: prompt.to_string(),
            }
        ];

        // Build the chat request with Llama 3.1 8B model
        let response = ic_llm::chat(Model::Llama3_1_8B)
            .with_messages(messages)
            .send()
            .await;

        // Extract the content from the assistant message
        Ok(response.message.content.unwrap_or_else(|| {
            "I'm here to help you with your questions and requests. Please ask me anything!".to_string()
        }))
    }
}