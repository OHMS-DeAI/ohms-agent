use serde::{Deserialize, Serialize};
use candid::CandidType;

/// User instruction for creating autonomous agents
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UserInstruction {
    pub instruction_text: String,
    pub user_id: String,
    pub subscription_tier: SubscriptionTier,
    pub context: Option<InstructionContext>,
    pub preferences: Option<AgentPreferences>,
}

/// Context information for instruction analysis
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstructionContext {
    pub domain: Option<String>,           // e.g., "coding", "content_creation", "data_analysis"
    pub complexity: Option<ComplexityLevel>,
    pub urgency: Option<UrgencyLevel>,
    pub collaboration_needed: bool,
    pub external_tools_required: Vec<String>,
}

/// User preferences for agent behavior
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AgentPreferences {
    pub response_style: ResponseStyle,
    pub detail_level: DetailLevel,
    pub creativity_level: CreativityLevel,
    pub safety_level: SafetyLevel,
    pub language: String,
}

/// Subscription tier information
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum SubscriptionTier {
    Basic,      // $29/month - 5 agents, 100k tokens
    Pro,        // $99/month - 25 agents, 500k tokens  
    Enterprise, // $299/month - 100 agents, 2M tokens
}

/// Complexity levels for task analysis
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ComplexityLevel {
    Simple,     // Single task, straightforward
    Moderate,   // Multiple steps, some coordination
    Complex,    // Multi-agent coordination required
    Expert,     // Advanced reasoning, multiple domains
}

/// Urgency levels for task prioritization
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum UrgencyLevel {
    Low,        // Can be done over time
    Normal,     // Standard priority
    High,       // Needs attention soon
    Critical,   // Immediate attention required
}

/// Response style preferences
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ResponseStyle {
    Concise,    // Brief, to-the-point responses
    Detailed,   // Comprehensive explanations
    Conversational, // Natural, chat-like responses
    Technical,  // Formal, technical language
}

/// Detail level preferences
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum DetailLevel {
    Summary,    // High-level overview
    Standard,   // Balanced detail
    Comprehensive, // Full details and explanations
    Expert,     // Technical deep-dive
}

/// Creativity level preferences
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CreativityLevel {
    Conservative, // Follow established patterns
    Balanced,     // Mix of conventional and creative
    Creative,     // Innovative approaches
    Experimental, // Novel, cutting-edge solutions
}

/// Safety level preferences
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum SafetyLevel {
    Strict,     // Conservative, safety-first
    Standard,   // Balanced safety and capability
    Flexible,   // Allow more creative solutions
    Experimental, // Maximum capability, user responsible
}

/// Analyzed instruction with extracted capabilities and requirements
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AnalyzedInstruction {
    pub original_instruction: UserInstruction,
    pub extracted_capabilities: Vec<Capability>,
    pub model_requirements: ModelRequirements,
    pub agent_configuration: AgentConfiguration,
    pub coordination_requirements: CoordinationRequirements,
    pub estimated_complexity: ComplexityLevel,
    pub estimated_duration: DurationEstimate,
    pub confidence_score: f32,
}

/// Specific capabilities needed for the task
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub category: CapabilityCategory,
    pub priority: CapabilityPriority,
    pub required_tools: Vec<String>,
    pub estimated_tokens: u32,
}

/// Capability categories for classification
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CapabilityCategory {
    TextGeneration,
    CodeGeneration,
    DataAnalysis,
    ContentCreation,
    ProblemSolving,
    Coordination,
    Communication,
    Research,
    Planning,
    Execution,
    Custom(String),
}

/// Priority levels for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CapabilityPriority {
    Essential,      // Must have for task completion
    Important,      // Strongly recommended
    Helpful,        // Nice to have
    Optional,       // Low priority
}

/// Model requirements based on instruction analysis
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ModelRequirements {
    pub recommended_models: Vec<String>,
    pub minimum_context_length: u32,
    pub preferred_precision: ModelPrecision,
    pub specialized_requirements: Vec<String>,
    pub reasoning_capability: ReasoningLevel,
    pub creativity_requirement: CreativityRequirement,
}

/// Model precision requirements
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ModelPrecision {
    FP32,       // Full precision
    FP16,       // Half precision
    INT8,       // 8-bit quantization
    INT4,       // 4-bit quantization
    Mixed,      // Mixed precision
}

/// Reasoning capability requirements
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ReasoningLevel {
    Basic,      // Simple pattern matching
    Intermediate, // Logical reasoning
    Advanced,   // Complex problem solving
    Expert,     // Multi-step reasoning
}

/// Creativity requirements
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CreativityRequirement {
    None,       // No creativity needed
    Low,        // Minor variations
    Medium,     // Creative solutions
    High,       // Novel approaches
}

/// Agent configuration generated from instruction analysis
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AgentConfiguration {
    pub agent_type: AgentType,
    pub personality: AgentPersonality,
    pub behavior_rules: Vec<String>,
    pub communication_style: CommunicationStyle,
    pub decision_making: DecisionMakingStyle,
    pub memory_configuration: MemoryConfiguration,
    pub tool_access: Vec<String>,
    pub safety_constraints: Vec<String>,
}

/// Types of agents that can be created
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum AgentType {
    GeneralAssistant,
    CodeAssistant,
    ContentCreator,
    DataAnalyst,
    ProblemSolver,
    Coordinator,
    Researcher,
    Planner,
    Executor,
    Custom(String),
}

/// Agent personality traits
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AgentPersonality {
    pub helpfulness: f32,      // 0.0 to 1.0
    pub creativity: f32,       // 0.0 to 1.0
    pub thoroughness: f32,     // 0.0 to 1.0
    pub efficiency: f32,       // 0.0 to 1.0
    pub formality: f32,        // 0.0 to 1.0
    pub assertiveness: f32,    // 0.0 to 1.0
}

/// Communication style preferences
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CommunicationStyle {
    Direct,         // Straightforward, no-nonsense
    Friendly,       // Warm, approachable
    Professional,   // Formal, business-like
    Technical,      // Detailed, technical
    Conversational, // Natural, chat-like
}

/// Decision making style
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum DecisionMakingStyle {
    Conservative,   // Safe, proven approaches
    Balanced,       // Mix of safe and innovative
    Aggressive,     // Fast, innovative approaches
    Collaborative,  // Consult with other agents
}

/// Memory configuration for agent persistence
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct MemoryConfiguration {
    pub short_term_capacity: u32,    // Tokens for immediate context
    pub long_term_capacity: u32,     // Tokens for persistent memory
    pub retention_policy: RetentionPolicy,
    pub sharing_enabled: bool,       // Share memory with other agents
}

/// Memory retention policies
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum RetentionPolicy {
    Session,        // Clear after session ends
    Daily,          // Clear daily
    Weekly,         // Clear weekly
    Persistent,     // Keep until explicitly cleared
}

/// Coordination requirements for multi-agent tasks
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CoordinationRequirements {
    pub requires_coordination: bool,
    pub coordination_type: CoordinationType,
    pub agent_count: u32,
    pub communication_protocol: CommunicationProtocol,
    pub task_distribution: TaskDistributionStrategy,
}

/// Types of coordination needed
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CoordinationType {
    None,           // Single agent task
    Sequential,     // Agents work in sequence
    Parallel,       // Agents work simultaneously
    Collaborative,  // Agents work together
    Hierarchical,   // One agent coordinates others
}

/// Communication protocols for agent coordination
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CommunicationProtocol {
    Direct,         // Direct agent-to-agent communication
    Centralized,    // Through coordinator
    Broadcast,      // Broadcast to all agents
    Hierarchical,   // Through chain of command
}

/// Task distribution strategies
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum TaskDistributionStrategy {
    RoundRobin,     // Distribute tasks evenly
    CapabilityBased, // Assign based on agent capabilities
    LoadBalanced,   // Based on current workload
    PriorityBased,  // Based on task priority
}

/// Duration estimates for task completion
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DurationEstimate {
    pub min_duration_seconds: u64,
    pub expected_duration_seconds: u64,
    pub max_duration_seconds: u64,
    pub confidence: f32,  // 0.0 to 1.0
}

impl Default for AgentPersonality {
    fn default() -> Self {
        Self {
            helpfulness: 0.8,
            creativity: 0.5,
            thoroughness: 0.7,
            efficiency: 0.6,
            formality: 0.5,
            assertiveness: 0.5,
        }
    }
}

impl Default for MemoryConfiguration {
    fn default() -> Self {
        Self {
            short_term_capacity: 4096,
            long_term_capacity: 16384,
            retention_policy: RetentionPolicy::Session,
            sharing_enabled: false,
        }
    }
}
