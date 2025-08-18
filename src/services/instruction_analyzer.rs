use crate::domain::instruction::*;
use std::collections::HashMap;

/// Service for analyzing user instructions and generating agent configurations
pub struct InstructionAnalyzer;

impl InstructionAnalyzer {
    /// Analyze a user instruction and generate comprehensive agent configuration
    pub fn analyze_instruction(instruction: UserInstruction) -> Result<AnalyzedInstruction, String> {
        let extracted_capabilities = Self::extract_capabilities(&instruction)?;
        let model_requirements = Self::determine_model_requirements(&instruction, &extracted_capabilities)?;
        let agent_configuration = Self::generate_agent_configuration(&instruction, &extracted_capabilities)?;
        let coordination_requirements = Self::analyze_coordination_needs(&instruction, &extracted_capabilities)?;
        let estimated_complexity = Self::estimate_complexity(&instruction, &extracted_capabilities);
        let estimated_duration = Self::estimate_duration(&instruction, &extracted_capabilities);
        let confidence_score = Self::calculate_confidence(&instruction, &extracted_capabilities);

        Ok(AnalyzedInstruction {
            original_instruction: instruction,
            extracted_capabilities,
            model_requirements,
            agent_configuration,
            coordination_requirements,
            estimated_complexity,
            estimated_duration,
            confidence_score,
        })
    }

    /// Extract capabilities from instruction text using keyword analysis
    fn extract_capabilities(instruction: &UserInstruction) -> Result<Vec<Capability>, String> {
        let text = instruction.instruction_text.to_lowercase();
        let mut capabilities = Vec::new();

        // Code generation capabilities
        if Self::contains_keywords(&text, &["code", "program", "script", "function", "class", "api", "database"]) {
            capabilities.push(Capability {
                name: "Code Generation".to_string(),
                description: "Generate code in various programming languages".to_string(),
                category: CapabilityCategory::CodeGeneration,
                priority: CapabilityPriority::Essential,
                required_tools: vec!["code_editor".to_string(), "syntax_checker".to_string()],
                estimated_tokens: 2048,
            });
        }

        // Text generation capabilities
        if Self::contains_keywords(&text, &["write", "create", "generate", "compose", "draft", "content"]) {
            capabilities.push(Capability {
                name: "Text Generation".to_string(),
                description: "Generate human-like text content".to_string(),
                category: CapabilityCategory::TextGeneration,
                priority: CapabilityPriority::Essential,
                required_tools: vec!["text_processor".to_string()],
                estimated_tokens: 1024,
            });
        }

        // Data analysis capabilities
        if Self::contains_keywords(&text, &["analyze", "data", "statistics", "chart", "graph", "report", "insights"]) {
            capabilities.push(Capability {
                name: "Data Analysis".to_string(),
                description: "Analyze data and generate insights".to_string(),
                category: CapabilityCategory::DataAnalysis,
                priority: CapabilityPriority::Essential,
                required_tools: vec!["data_processor".to_string(), "visualization_tool".to_string()],
                estimated_tokens: 3072,
            });
        }

        // Content creation capabilities
        if Self::contains_keywords(&text, &["content", "article", "blog", "social media", "marketing", "creative"]) {
            capabilities.push(Capability {
                name: "Content Creation".to_string(),
                description: "Create engaging content for various platforms".to_string(),
                category: CapabilityCategory::ContentCreation,
                priority: CapabilityPriority::Essential,
                required_tools: vec!["content_editor".to_string(), "plagiarism_checker".to_string()],
                estimated_tokens: 2048,
            });
        }

        // Problem solving capabilities
        if Self::contains_keywords(&text, &["solve", "problem", "issue", "debug", "fix", "optimize", "improve"]) {
            capabilities.push(Capability {
                name: "Problem Solving".to_string(),
                description: "Analyze and solve complex problems".to_string(),
                category: CapabilityCategory::ProblemSolving,
                priority: CapabilityPriority::Essential,
                required_tools: vec!["debugger".to_string(), "optimizer".to_string()],
                estimated_tokens: 4096,
            });
        }

        // Research capabilities
        if Self::contains_keywords(&text, &["research", "find", "search", "investigate", "explore", "discover"]) {
            capabilities.push(Capability {
                name: "Research".to_string(),
                description: "Conduct research and gather information".to_string(),
                category: CapabilityCategory::Research,
                priority: CapabilityPriority::Important,
                required_tools: vec!["web_search".to_string(), "document_analyzer".to_string()],
                estimated_tokens: 2048,
            });
        }

        // Planning capabilities
        if Self::contains_keywords(&text, &["plan", "strategy", "roadmap", "timeline", "schedule", "organize"]) {
            capabilities.push(Capability {
                name: "Planning".to_string(),
                description: "Create plans and strategies".to_string(),
                category: CapabilityCategory::Planning,
                priority: CapabilityPriority::Important,
                required_tools: vec!["planner".to_string(), "scheduler".to_string()],
                estimated_tokens: 1536,
            });
        }

        // If no specific capabilities detected, add general assistance
        if capabilities.is_empty() {
            capabilities.push(Capability {
                name: "General Assistance".to_string(),
                description: "Provide general help and support".to_string(),
                category: CapabilityCategory::TextGeneration,
                priority: CapabilityPriority::Essential,
                required_tools: vec![],
                estimated_tokens: 1024,
            });
        }

        Ok(capabilities)
    }

    /// Determine model requirements based on instruction and capabilities
    fn determine_model_requirements(
        instruction: &UserInstruction,
        capabilities: &[Capability],
    ) -> Result<ModelRequirements, String> {
        let mut recommended_models = Vec::new();
        let mut min_context_length = 2048;
        let mut reasoning_level = ReasoningLevel::Basic;
        let mut creativity_requirement = CreativityRequirement::None;

        // Determine model recommendations based on capabilities
        for capability in capabilities {
            match capability.category {
                CapabilityCategory::CodeGeneration => {
                    recommended_models.push("codellama-7b-novaq".to_string());
                    recommended_models.push("wizardcoder-15b-novaq".to_string());
                    min_context_length = min_context_length.max(8192);
                    reasoning_level = ReasoningLevel::Advanced;
                }
                CapabilityCategory::DataAnalysis => {
                    recommended_models.push("llama-2-70b-novaq".to_string());
                    recommended_models.push("gpt4all-13b-novaq".to_string());
                    min_context_length = min_context_length.max(16384);
                    reasoning_level = ReasoningLevel::Expert;
                }
                CapabilityCategory::ContentCreation => {
                    recommended_models.push("llama-2-13b-novaq".to_string());
                    recommended_models.push("vicuna-13b-novaq".to_string());
                    creativity_requirement = CreativityRequirement::Medium;
                }
                CapabilityCategory::ProblemSolving => {
                    recommended_models.push("llama-2-70b-novaq".to_string());
                    recommended_models.push("wizardlm-30b-novaq".to_string());
                    min_context_length = min_context_length.max(8192);
                    reasoning_level = ReasoningLevel::Expert;
                }
                _ => {
                    recommended_models.push("llama-2-7b-novaq".to_string());
                }
            }
        }

        // Remove duplicates and limit to top 3
        recommended_models.sort();
        recommended_models.dedup();
        recommended_models.truncate(3);

        // Determine precision based on subscription tier
        let preferred_precision = match instruction.subscription_tier {
            SubscriptionTier::Basic => ModelPrecision::INT4,
            SubscriptionTier::Pro => ModelPrecision::INT8,
            SubscriptionTier::Enterprise => ModelPrecision::FP16,
        };

        Ok(ModelRequirements {
            recommended_models,
            minimum_context_length: min_context_length,
            preferred_precision,
            specialized_requirements: Self::extract_specialized_requirements(instruction),
            reasoning_capability: reasoning_level,
            creativity_requirement,
        })
    }

    /// Generate agent configuration based on instruction analysis
    fn generate_agent_configuration(
        instruction: &UserInstruction,
        capabilities: &[Capability],
    ) -> Result<AgentConfiguration, String> {
        let agent_type = Self::determine_agent_type(capabilities);
        let personality = Self::generate_personality(instruction);
        let behavior_rules = Self::generate_behavior_rules(instruction, capabilities);
        let communication_style = Self::determine_communication_style(instruction);
        let decision_making = Self::determine_decision_making(instruction);
        let memory_configuration = Self::generate_memory_config(instruction);
        let tool_access = Self::determine_tool_access(capabilities);
        let safety_constraints = Self::generate_safety_constraints(instruction);

        Ok(AgentConfiguration {
            agent_type,
            personality,
            behavior_rules,
            communication_style,
            decision_making,
            memory_configuration,
            tool_access,
            safety_constraints,
        })
    }

    /// Analyze coordination requirements for multi-agent tasks
    fn analyze_coordination_needs(
        instruction: &UserInstruction,
        capabilities: &[Capability],
    ) -> Result<CoordinationRequirements, String> {
        let text = instruction.instruction_text.to_lowercase();
        let requires_coordination = capabilities.len() > 1 || 
            Self::contains_keywords(&text, &["multiple", "team", "coordinate", "collaborate", "together"]);

        let coordination_type = if !requires_coordination {
            CoordinationType::None
        } else if Self::contains_keywords(&text, &["sequence", "step by step", "pipeline"]) {
            CoordinationType::Sequential
        } else if Self::contains_keywords(&text, &["parallel", "simultaneous", "at the same time"]) {
            CoordinationType::Parallel
        } else if Self::contains_keywords(&text, &["hierarchy", "manager", "lead"]) {
            CoordinationType::Hierarchical
        } else {
            CoordinationType::Collaborative
        };

        let agent_count = if requires_coordination {
            capabilities.len().max(2) as u32
        } else {
            1
        };

        Ok(CoordinationRequirements {
            requires_coordination,
            coordination_type,
            agent_count,
            communication_protocol: CommunicationProtocol::Direct,
            task_distribution: TaskDistributionStrategy::CapabilityBased,
        })
    }

    /// Estimate task complexity
    fn estimate_complexity(instruction: &UserInstruction, capabilities: &[Capability]) -> ComplexityLevel {
        let text = instruction.instruction_text.to_lowercase();
        let capability_count = capabilities.len();
        let has_complex_keywords = Self::contains_keywords(&text, &["complex", "advanced", "expert", "sophisticated"]);

        match (capability_count, has_complex_keywords) {
            (0, false) => ComplexityLevel::Simple,
            (1, false) => ComplexityLevel::Simple,
            (1..=2, false) => ComplexityLevel::Moderate,
            (3..=4, _) => ComplexityLevel::Complex,
            (5.., _) | (_, true) => ComplexityLevel::Expert,
        }
    }

    /// Estimate task duration
    fn estimate_duration(instruction: &UserInstruction, capabilities: &[Capability]) -> DurationEstimate {
        let base_tokens: u32 = capabilities.iter().map(|c| c.estimated_tokens).sum();
        let base_seconds = (base_tokens as f64 / 100.0).max(30.0) as u64; // Rough estimate

        DurationEstimate {
            min_duration_seconds: base_seconds / 2,
            expected_duration_seconds: base_seconds,
            max_duration_seconds: base_seconds * 3,
            confidence: 0.7,
        }
    }

    /// Calculate confidence score for analysis
    fn calculate_confidence(instruction: &UserInstruction, capabilities: &[Capability]) -> f32 {
        let mut confidence: f32 = 0.8; // Base confidence

        // Increase confidence for specific keywords
        let text = instruction.instruction_text.to_lowercase();
        if Self::contains_keywords(&text, &["code", "write", "analyze", "create", "solve"]) {
            confidence += 0.1;
        }

        // Decrease confidence for vague instructions
        if Self::contains_keywords(&text, &["something", "anything", "whatever", "maybe"]) {
            confidence -= 0.2;
        }

        // Adjust based on capability count
        if capabilities.len() == 1 {
            confidence += 0.05;
        } else if capabilities.len() > 3 {
            confidence -= 0.1;
        }

        confidence.max(0.3_f32).min(1.0_f32)
    }

    // Helper methods
    fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|&keyword| text.contains(keyword))
    }

    fn extract_specialized_requirements(instruction: &UserInstruction) -> Vec<String> {
        let text = instruction.instruction_text.to_lowercase();
        let mut requirements = Vec::new();

        if Self::contains_keywords(&text, &["real-time", "live", "streaming"]) {
            requirements.push("real_time_processing".to_string());
        }
        if Self::contains_keywords(&text, &["secure", "encrypted", "private"]) {
            requirements.push("security_focused".to_string());
        }
        if Self::contains_keywords(&text, &["multilingual", "translate", "language"]) {
            requirements.push("multilingual_support".to_string());
        }

        requirements
    }

    fn determine_agent_type(capabilities: &[Capability]) -> AgentType {
        for capability in capabilities {
            match capability.category {
                CapabilityCategory::CodeGeneration => return AgentType::CodeAssistant,
                CapabilityCategory::DataAnalysis => return AgentType::DataAnalyst,
                CapabilityCategory::ContentCreation => return AgentType::ContentCreator,
                CapabilityCategory::ProblemSolving => return AgentType::ProblemSolver,
                CapabilityCategory::Research => return AgentType::Researcher,
                CapabilityCategory::Planning => return AgentType::Planner,
                _ => continue,
            }
        }
        AgentType::GeneralAssistant
    }

    fn generate_personality(instruction: &UserInstruction) -> AgentPersonality {
        let preferences = instruction.preferences.as_ref();
        let mut personality = AgentPersonality::default();

        if let Some(prefs) = preferences {
            match prefs.creativity_level {
                CreativityLevel::Conservative => personality.creativity = 0.3,
                CreativityLevel::Balanced => personality.creativity = 0.5,
                CreativityLevel::Creative => personality.creativity = 0.7,
                CreativityLevel::Experimental => personality.creativity = 0.9,
            }

            match prefs.detail_level {
                DetailLevel::Summary => personality.thoroughness = 0.4,
                DetailLevel::Standard => personality.thoroughness = 0.6,
                DetailLevel::Comprehensive => personality.thoroughness = 0.8,
                DetailLevel::Expert => personality.thoroughness = 0.9,
            }

            match prefs.response_style {
                ResponseStyle::Concise => personality.efficiency = 0.8,
                ResponseStyle::Detailed => personality.thoroughness = 0.8,
                ResponseStyle::Conversational => personality.formality = 0.3,
                ResponseStyle::Technical => personality.formality = 0.8,
            }
        }

        personality
    }

    fn generate_behavior_rules(instruction: &UserInstruction, capabilities: &[Capability]) -> Vec<String> {
        let mut rules = vec![
            "Always prioritize user safety and ethical considerations".to_string(),
            "Provide accurate and helpful responses".to_string(),
            "Ask for clarification when instructions are unclear".to_string(),
        ];

        // Add capability-specific rules
        for capability in capabilities {
            match capability.category {
                CapabilityCategory::CodeGeneration => {
                    rules.push("Follow best practices and coding standards".to_string());
                    rules.push("Include comments and documentation in code".to_string());
                }
                CapabilityCategory::DataAnalysis => {
                    rules.push("Validate data sources and assumptions".to_string());
                    rules.push("Provide clear explanations of analysis methods".to_string());
                }
                CapabilityCategory::ContentCreation => {
                    rules.push("Ensure content is original and engaging".to_string());
                    rules.push("Consider target audience and platform requirements".to_string());
                }
                _ => {}
            }
        }

        rules
    }

    fn determine_communication_style(instruction: &UserInstruction) -> CommunicationStyle {
        if let Some(preferences) = &instruction.preferences {
            match preferences.response_style {
                ResponseStyle::Concise => CommunicationStyle::Direct,
                ResponseStyle::Detailed => CommunicationStyle::Technical,
                ResponseStyle::Conversational => CommunicationStyle::Conversational,
                ResponseStyle::Technical => CommunicationStyle::Technical,
            }
        } else {
            CommunicationStyle::Friendly
        }
    }

    fn determine_decision_making(instruction: &UserInstruction) -> DecisionMakingStyle {
        if let Some(preferences) = &instruction.preferences {
            match preferences.safety_level {
                SafetyLevel::Strict => DecisionMakingStyle::Conservative,
                SafetyLevel::Standard => DecisionMakingStyle::Balanced,
                SafetyLevel::Flexible => DecisionMakingStyle::Aggressive,
                SafetyLevel::Experimental => DecisionMakingStyle::Aggressive,
            }
        } else {
            DecisionMakingStyle::Balanced
        }
    }

    fn generate_memory_config(instruction: &UserInstruction) -> MemoryConfiguration {
        let mut config = MemoryConfiguration::default();

        // Adjust based on subscription tier
        match instruction.subscription_tier {
            SubscriptionTier::Basic => {
                config.short_term_capacity = 2048;
                config.long_term_capacity = 8192;
            }
            SubscriptionTier::Pro => {
                config.short_term_capacity = 4096;
                config.long_term_capacity = 16384;
            }
            SubscriptionTier::Enterprise => {
                config.short_term_capacity = 8192;
                config.long_term_capacity = 32768;
                config.sharing_enabled = true;
            }
        }

        config
    }

    fn determine_tool_access(capabilities: &[Capability]) -> Vec<String> {
        let mut tools = Vec::new();
        
        for capability in capabilities {
            tools.extend(capability.required_tools.clone());
        }

        tools.sort();
        tools.dedup();
        tools
    }

    fn generate_safety_constraints(instruction: &UserInstruction) -> Vec<String> {
        let mut constraints = vec![
            "No harmful or malicious content".to_string(),
            "Respect privacy and confidentiality".to_string(),
        ];

        if let Some(preferences) = &instruction.preferences {
            match preferences.safety_level {
                SafetyLevel::Strict => {
                    constraints.push("Conservative approach to all decisions".to_string());
                    constraints.push("Require explicit user approval for significant actions".to_string());
                }
                SafetyLevel::Standard => {
                    constraints.push("Follow standard safety protocols".to_string());
                }
                SafetyLevel::Flexible => {
                    constraints.push("Allow creative solutions within ethical bounds".to_string());
                }
                SafetyLevel::Experimental => {
                    constraints.push("User assumes responsibility for experimental approaches".to_string());
                }
            }
        }

        constraints
    }
}
