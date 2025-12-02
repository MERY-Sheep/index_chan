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
            model_name: "Qwen/Qwen2.5-Coder-1.5B-Instruct".to_string(),
            model_path: None,
            temperature: 0.7,
            max_tokens: 512,
            confidence_threshold: 0.85,
        }
    }
}

impl LLMConfig {
    pub fn get_cache_dir() -> PathBuf {
        let home = dirs::home_dir().expect("Failed to get home directory");
        home.join(".cache").join("index-chan").join("models")
    }
    
    pub fn get_model_path(&self) -> PathBuf {
        if let Some(path) = &self.model_path {
            path.clone()
        } else {
            Self::get_cache_dir().join(&self.model_name)
        }
    }
}
