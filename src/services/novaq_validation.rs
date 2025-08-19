use candid::{CandidType, Deserialize};
use serde::Serialize;

/// NOVAQ validation service for OHMS agent
pub struct NOVAQValidationService;

/// NOVAQ model validation result
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct NOVAQValidationResult {
    pub model_id: String,
    pub compression_ratio: f64,
    pub bit_accuracy: f64,
    pub quality_score: f64,
    pub validation_passed: bool,
    pub issues: Vec<String>,
    pub validation_timestamp: u64,
}

/// NOVAQ model metadata
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct NOVAQModelMeta {
    pub target_bits: f32,
    pub num_subspaces: u32,
    pub l1_codebook_size: u32,
    pub l2_codebook_size: u32,
    pub compression_ratio: f64,
    pub bit_accuracy: f64,
    pub quality_score: f64,
}

impl NOVAQValidationService {
    /// Validate a NOVAQ compressed model
    pub async fn validate_novaq_model(
        model_id: &str,
        model_data: &[u8],
    ) -> Result<NOVAQValidationResult, String> {
        // Parse the NOVAQ model data
        let novaq_model = Self::parse_novaq_model(model_data)?;
        
        // Extract validation metrics
        let compression_ratio = novaq_model.compression_ratio as f64;
        let bit_accuracy = novaq_model.bit_accuracy as f64;
        let quality_score = (compression_ratio / 100.0 + bit_accuracy) / 2.0;
        
        // Apply validation thresholds based on bit depth
        let (validation_passed, issues) = Self::apply_validation_thresholds(
            &novaq_model.config,
            compression_ratio,
            bit_accuracy,
        );
        
        Ok(NOVAQValidationResult {
            model_id: model_id.to_string(),
            compression_ratio,
            bit_accuracy,
            quality_score,
            validation_passed,
            issues,
            validation_timestamp: ic_cdk::api::time(),
        })
    }
    
    /// Extract NOVAQ model metadata
    pub async fn extract_novaq_metadata(
        model_data: &[u8],
    ) -> Result<NOVAQModelMeta, String> {
        let novaq_model = Self::parse_novaq_model(model_data)?;
        
        Ok(NOVAQModelMeta {
            target_bits: novaq_model.config.target_bits,
            num_subspaces: novaq_model.config.num_subspaces as u32,
            l1_codebook_size: novaq_model.config.codebook_size_l1 as u32,
            l2_codebook_size: novaq_model.config.codebook_size_l2 as u32,
            compression_ratio: novaq_model.compression_ratio as f64,
            bit_accuracy: novaq_model.bit_accuracy as f64,
            quality_score: (novaq_model.compression_ratio as f64 / 100.0 + novaq_model.bit_accuracy as f64) / 2.0,
        })
    }
    
    /// Check if model data is NOVAQ compressed
    pub fn is_novaq_model(model_data: &[u8]) -> bool {
        // Try to parse as NOVAQ model - if it succeeds, it's a NOVAQ model
        Self::parse_novaq_model(model_data).is_ok()
    }
    
    /// Get NOVAQ model quality score
    pub fn get_quality_score(model_data: &[u8]) -> Result<f64, String> {
        let novaq_model = Self::parse_novaq_model(model_data)?;
        let compression_ratio = novaq_model.compression_ratio as f64;
        let bit_accuracy = novaq_model.bit_accuracy as f64;
        Ok((compression_ratio / 100.0 + bit_accuracy) / 2.0)
    }
    
    /// Parse NOVAQ model from binary data
    fn parse_novaq_model(model_data: &[u8]) -> Result<NOVAQModelStruct, String> {
        // Use bincode to deserialize the NOVAQ model
        bincode::deserialize::<NOVAQModelStruct>(model_data)
            .map_err(|e| format!("Failed to parse NOVAQ model: {}", e))
    }
    
    /// Apply validation thresholds based on bit depth
    fn apply_validation_thresholds(
        config: &NOVAQConfigStruct,
        compression_ratio: f64,
        bit_accuracy: f64,
    ) -> (bool, Vec<String>) {
        let mut issues = Vec::new();
        
        // Minimum compression ratio check
        if compression_ratio < 2.0 {
            issues.push("Compression ratio below minimum threshold (2.0x)".to_string());
        }
        
        // Bit accuracy thresholds based on target bits
        let min_bit_accuracy = match config.target_bits {
            b if b <= 1.0 => 0.85,  // 1-bit: 85% accuracy is excellent
            b if b <= 2.0 => 0.90,  // 2-bit: 90% accuracy is excellent
            b if b <= 4.0 => 0.95,  // 4-bit: 95% accuracy is excellent
            _ => 0.98,              // Higher bits: 98% accuracy expected
        };
        
        if bit_accuracy < min_bit_accuracy {
            issues.push(format!(
                "Bit accuracy {:.1}% below threshold {:.1}% for {:.1}-bit quantization",
                bit_accuracy * 100.0,
                min_bit_accuracy * 100.0,
                config.target_bits
            ));
        }
        
        // Subspace validation
        if config.num_subspaces == 0 {
            issues.push("Invalid number of subspaces (must be > 0)".to_string());
        }
        
        // Codebook size validation
        if config.codebook_size_l1 == 0 || config.codebook_size_l2 == 0 {
            issues.push("Invalid codebook sizes (must be > 0)".to_string());
        }
        
        let validation_passed = issues.is_empty();
        (validation_passed, issues)
    }
}

// Internal structures for NOVAQ model parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NOVAQModelStruct {
    pub config: NOVAQConfigStruct,
    pub compression_ratio: f32,
    pub bit_accuracy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NOVAQConfigStruct {
    pub target_bits: f32,
    pub num_subspaces: usize,
    pub codebook_size_l1: usize,
    pub codebook_size_l2: usize,
    pub outlier_threshold: f32,
    pub teacher_model_path: Option<String>,
    pub refinement_iterations: usize,
    pub kl_weight: f32,
    pub cosine_weight: f32,
    pub learning_rate: f32,
    pub seed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_thresholds() {
        let config = NOVAQConfigStruct {
            target_bits: 1.5,
            num_subspaces: 2,
            codebook_size_l1: 16,
            codebook_size_l2: 4,
            outlier_threshold: 0.01,
            teacher_model_path: None,
            refinement_iterations: 50,
            kl_weight: 1.0,
            cosine_weight: 0.5,
            learning_rate: 0.001,
            seed: 42,
        };
        
        // Test good compression
        let (passed, issues) = NOVAQValidationService::apply_validation_thresholds(
            &config,
            383.3,  // High compression ratio
            0.95,   // Good accuracy
        );
        assert!(passed, "Should pass with good metrics: {:?}", issues);
        
        // Test poor compression
        let (passed, issues) = NOVAQValidationService::apply_validation_thresholds(
            &config,
            1.5,    // Low compression ratio
            0.80,   // Poor accuracy
        );
        assert!(!passed, "Should fail with poor metrics");
        assert!(!issues.is_empty(), "Should have validation issues");
    }
}
