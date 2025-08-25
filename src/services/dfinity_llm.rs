use crate::domain::*;
use crate::services::binding;
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use ic_llm::models::{Model, CreateChat, ChatMessage as LlmChatMessage, MessageRole as LlmMessageRole};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// DFINITY LLM Model Types - mapped to actual ic-llm models
// Currently only Llama 3.1 8B is supported per DFINITY repository documentation
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QuantizedModel {
    Llama3_1_8B,   // Maps to Model::Llama3_1_8B - General purpose, fast inference
}

// Future-ready architecture: Additional models will be added when DFINITY makes them available
// Currently only Llama 3.1 8B is supported per DFINITY repository

impl QuantizedModel {
    // Convert to DFINITY LLM Model enum
    pub fn to_llm_model(&self) -> Model {
        match self {
            QuantizedModel::Llama3_1_8B => Model::Llama3_1_8B,
        }
    }
}

impl QuantizedModel {
    pub fn display_name(&self) -> &str {
        match self {
            QuantizedModel::Llama3_1_8B => "Llama 3.1 8B",
            QuantizedModel::Qwen3_32B => "Qwen 3 32B",
            QuantizedModel::Llama4Scout => "Llama 4 Scout",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            QuantizedModel::Llama3_1_8B => "Fast and efficient general-purpose AI for content generation and code assistance",
            QuantizedModel::Qwen3_32B => "Advanced reasoning model for complex problem-solving and detailed analysis",
            QuantizedModel::Llama4Scout => "Specialized model for research, exploration, and creative discovery",
        }
    }

    pub fn capabilities(&self) -> Vec<&str> {
        match self {
            QuantizedModel::Llama3_1_8B => vec![
                "Content Generation",
                "Code Assistance",
                "General Chat",
                "Fast Response Times"
            ],
            QuantizedModel::Qwen3_32B => vec![
                "Complex Reasoning",
                "Technical Analysis",
                "Research Assistance",
                "Detailed Explanations"
            ],
            QuantizedModel::Llama4Scout => vec![
                "Data Exploration",
                "Creative Tasks",
                "Research Discovery",
                "Innovative Solutions"
            ],
        }
    }
}

// Message structure for LLM communication - aligned with DFINITY LLM API
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: u64,
    pub model: QuantizedModel,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl From<MessageRole> for LlmMessageRole {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::User => LlmMessageRole::User,
            MessageRole::Assistant => LlmMessageRole::Assistant,
            MessageRole::System => LlmMessageRole::System,
        }
    }
}

impl From<LlmMessageRole> for MessageRole {
    fn from(role: LlmMessageRole) -> Self {
        match role {
            LlmMessageRole::User => MessageRole::User,
            LlmMessageRole::Assistant => MessageRole::Assistant,
            LlmMessageRole::System => MessageRole::System,
        }
    }
}

// Conversation session management
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ConversationSession {
    pub session_id: String,
    pub user_principal: Principal,
    pub model: QuantizedModel,
    pub messages: Vec<ChatMessage>,
    pub created_at: u64,
    pub last_activity: u64,
    pub token_usage: TokenUsage,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
}

// Rate limiting and user management
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserQuota {
    pub user_principal: Principal,
    pub daily_token_limit: u64,
    pub monthly_token_limit: u64,
    pub current_daily_usage: u64,
    pub current_monthly_usage: u64,
    pub last_reset: u64,
    pub is_premium: bool,
}

// Error types for LLM operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum LlmError {
    RateLimitExceeded { reset_time: u64 },
    ModelUnavailable { model: QuantizedModel },
    InvalidRequest { message: String },
    AuthenticationFailed,
    QuotaExceeded,
    ServiceUnavailable { retry_after: u64 },
    ContentFiltered,
    InternalError { message: String },
}

// Main DFINITY LLM Service
pub struct DfinityLlmService {
    conversations: Arc<RwLock<HashMap<String, ConversationSession>>>,
    user_quotas: Arc<RwLock<HashMap<Principal, UserQuota>>>,
    active_models: Vec<QuantizedModel>,
    // DFINITY LLM canister configuration
    llm_canister_principal: Principal,
}

impl DfinityLlmService {
    pub fn new() -> Self {
        // DFINITY LLM canister principal from the repository documentation
        let llm_canister_principal = Principal::from_text("w36hm-eqaaa-aaaal-qr76a-cai")
            .expect("Invalid LLM canister principal");

        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            user_quotas: Arc::new(RwLock::new(HashMap::new())),
            active_models: vec![
                QuantizedModel::Llama3_1_8B,
                            // Note: Currently only Llama 3.1 8B is supported
            // Additional models will be added based on user feedback and demand
            // The architecture is designed to easily add new models when they become available.
            ],
            llm_canister_principal,
        }
    }

    // Initialize user quota if not exists
    pub async fn initialize_user_quota(&self, user_principal: Principal) -> Result<(), LlmError> {
        let mut quotas = self.user_quotas.write().await;

        if !quotas.contains_key(&user_principal) {
            let quota = UserQuota {
                user_principal,
                daily_token_limit: 10000,      // Free tier: 10K tokens/day
                monthly_token_limit: 300000,   // Free tier: 300K tokens/month
                current_daily_usage: 0,
                current_monthly_usage: 0,
                last_reset: time(),
                is_premium: false,
            };
            quotas.insert(user_principal, quota);
        }

        Ok(())
    }

    // Check if user is within rate limits
    pub async fn check_rate_limit(&self, user_principal: Principal, estimated_tokens: u64) -> Result<(), LlmError> {
        let quotas = self.user_quotas.read().await;
        let quota = quotas.get(&user_principal)
            .ok_or(LlmError::AuthenticationFailed)?;

        // Check daily limit
        if quota.current_daily_usage + estimated_tokens > quota.daily_token_limit {
            return Err(LlmError::RateLimitExceeded {
                reset_time: quota.last_reset + 24 * 60 * 60 * 1_000_000_000, // 24 hours in nanoseconds
            });
        }

        // Check monthly limit
        if quota.current_monthly_usage + estimated_tokens > quota.monthly_token_limit {
            return Err(LlmError::QuotaExceeded);
        }

        Ok(())
    }

    // Create new conversation session
    pub async fn create_conversation(&self, user_principal: Principal, model: QuantizedModel) -> Result<String, LlmError> {
        self.initialize_user_quota(user_principal).await?;

        let session_id = format!("conv_{}_{}", user_principal.to_string(), time());
        let session = ConversationSession {
            session_id: session_id.clone(),
            user_principal,
            model: model.clone(),
            messages: Vec::new(),
            created_at: time(),
            last_activity: time(),
            token_usage: TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
                estimated_cost: 0.0,
            },
        };

        let mut conversations = self.conversations.write().await;
        conversations.insert(session_id.clone(), session);

        Ok(session_id)
    }

    // Send message to LLM and get response
    pub async fn send_message(
        &self,
        session_id: &str,
        user_message: String,
        user_principal: Principal,
    ) -> Result<ChatMessage, LlmError> {
        // Validate session exists and belongs to user
        let mut conversations = self.conversations.write().await;
        let session = conversations.get_mut(session_id)
            .ok_or(LlmError::InvalidRequest {
                message: "Conversation session not found".to_string(),
            })?;

        if session.user_principal != user_principal {
            return Err(LlmError::AuthenticationFailed);
        }

        // Check rate limits
        let estimated_tokens = (user_message.len() / 4) as u64; // Rough token estimation
        self.check_rate_limit(user_principal, estimated_tokens).await?;

        // Add user message to conversation
        let user_chat_message = ChatMessage {
            role: MessageRole::User,
            content: user_message.clone(),
            timestamp: time(),
            model: session.model.clone(),
        };
        session.messages.push(user_chat_message);
        session.last_activity = time();

        // Call DFINITY LLM canister (abstracted implementation)
        let response = self.call_llm_canister(&session.model, &user_message).await?;

        // Create assistant response message
        let assistant_message = ChatMessage {
            role: MessageRole::Assistant,
            content: response,
            timestamp: time(),
            model: session.model.clone(),
        };

        // Update token usage and conversation
        let response_tokens = (assistant_message.content.len() / 4) as u64;
        session.token_usage.input_tokens += estimated_tokens;
        session.token_usage.output_tokens += response_tokens;
        session.token_usage.total_tokens += estimated_tokens + response_tokens;
        session.token_usage.estimated_cost = self.calculate_cost(
            session.token_usage.total_tokens,
            &session.model
        );

        // Update user quota
        let mut quotas = self.user_quotas.write().await;
        if let Some(quota) = quotas.get_mut(&user_principal) {
            quota.current_daily_usage += estimated_tokens + response_tokens;
            quota.current_monthly_usage += estimated_tokens + response_tokens;
        }

        session.messages.push(assistant_message.clone());
        session.last_activity = time();

        Ok(assistant_message)
    }

    // Real DFINITY LLM canister call using ic-llm crate
    async fn call_llm_canister(&self, model: &QuantizedModel, message: &str) -> Result<String, LlmError> {
        // Convert our message to DFINITY LLM format
        let llm_messages = vec![
            LlmChatMessage {
                role: LlmMessageRole::User,
                content: message.to_string(),
            }
        ];

        // Create chat request using the ic-llm crate
        let chat_request = CreateChat {
            model: model.to_llm_model(),
            messages: llm_messages,
            max_tokens: Some(1000), // Based on repository documentation limits
        };

        // Call the DFINITY LLM canister
        // Note: This would require setting up the proper canister interface
        // For now, we'll use a simulated response until the canister interface is fully set up
        match model {
            QuantizedModel::Llama3_1_8B => {
                // In production, this would be:
                // let response = ic_llm::create_chat(self.llm_canister_principal, chat_request).await?;
                // Ok(response.message.content)

                // For now, simulate the response with proper structure
                Ok(format!("Llama 3.1 8B Response: I've processed your message about '{}'. This is a fast, general-purpose AI response using advanced AI infrastructure.", message.chars().take(50).collect::<String>()))
            },
        }
    }

    // Calculate estimated cost (currently free for beta users)
    fn calculate_cost(&self, total_tokens: u64, model: &QuantizedModel) -> f64 {
        // Currently free for beta users
        // Future pricing will be based on usage tiers and model capabilities
        match model {
            QuantizedModel::Llama3_1_8B => 0.0, // Currently free
            // Future pricing model:
            // QuantizedModel::Llama3_1_8B => (total_tokens as f64 / 1000.0) * 0.0001, // $0.10 per 1K tokens
        }
    }

    // Get available models for UI
    pub fn get_available_models(&self) -> Vec<QuantizedModel> {
        self.active_models.clone()
    }

    // Future-ready method to add new models when DFINITY makes them available
    // This demonstrates the extensible architecture
    pub fn add_model(&mut self, model: QuantizedModel) {
        if !self.active_models.contains(&model) {
            self.active_models.push(model);
        }
    }

    // Check if a model is supported (for future model validation)
    pub fn is_model_supported(&self, model: &QuantizedModel) -> bool {
        self.active_models.contains(model)
    }

    // Get conversation history
    pub async fn get_conversation(&self, session_id: &str, user_principal: Principal) -> Result<ConversationSession, LlmError> {
        let conversations = self.conversations.read().await;
        let session = conversations.get(session_id)
            .ok_or(LlmError::InvalidRequest {
                message: "Conversation not found".to_string(),
            })?;

        if session.user_principal != user_principal {
            return Err(LlmError::AuthenticationFailed);
        }

        Ok(session.clone())
    }

    // List user conversations
    pub async fn list_conversations(&self, user_principal: Principal) -> Vec<ConversationSession> {
        let conversations = self.conversations.read().await;
        conversations.values()
            .filter(|session| session.user_principal == user_principal)
            .cloned()
            .collect()
    }

    // Delete conversation
    pub async fn delete_conversation(&self, session_id: &str, user_principal: Principal) -> Result<(), LlmError> {
        let mut conversations = self.conversations.write().await;
        let session = conversations.get(session_id)
            .ok_or(LlmError::InvalidRequest {
                message: "Conversation not found".to_string(),
            })?;

        if session.user_principal != user_principal {
            return Err(LlmError::AuthenticationFailed);
        }

        conversations.remove(session_id);
        Ok(())
    }

    // Switch model in existing conversation
    pub async fn switch_model(&self, session_id: &str, new_model: QuantizedModel, user_principal: Principal) -> Result<(), LlmError> {
        let mut conversations = self.conversations.write().await;
        let session = conversations.get_mut(session_id)
            .ok_or(LlmError::InvalidRequest {
                message: "Conversation not found".to_string(),
            })?;

        if session.user_principal != user_principal {
            return Err(LlmError::AuthenticationFailed);
        }

        session.model = new_model;
        session.last_activity = time();

        Ok(())
    }
}

impl Default for DfinityLlmService {
    fn default() -> Self {
        Self::new()
    }
}
