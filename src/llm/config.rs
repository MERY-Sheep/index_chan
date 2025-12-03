use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model_name: String,
    pub model_path: Option<PathBuf>,
    pub temperature: f32,
    pub max_tokens: usize,
    pub confidence_threshold: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model_name: "gemini-2.0-flash".to_string(),
            model_path: None,
            temperature: 0.7,
            max_tokens: 2048,
            confidence_threshold: 0.85,
        }
    }
}

impl LLMConfig {
    // Gemini APIを使用するため、ローカルモデルパスは不要
}
