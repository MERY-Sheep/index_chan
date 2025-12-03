// Prompt history management
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

/// A prompt sent to the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub timestamp: String,
    pub system_prompt: String,
    pub conversation_history: Vec<Message>,
    pub current_query: String,
    pub token_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
}

/// Prompt history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHistory {
    pub prompts: Vec<Prompt>,
}

impl PromptHistory {
    /// Create a new empty prompt history
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    /// Load prompt history from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let history: PromptHistory = serde_json::from_str(&content)?;
        Ok(history)
    }

    /// Save prompt history to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a new prompt
    pub fn add_prompt(&mut self, prompt: Prompt) {
        self.prompts.push(prompt);
        
        // Auto-rotate: keep only last 1000 prompts
        if self.prompts.len() > 1000 {
            self.prompts.drain(0..self.prompts.len() - 1000);
        }
    }

    /// Get prompt by ID
    pub fn get_prompt(&self, id: &str) -> Option<&Prompt> {
        self.prompts.iter().find(|p| p.id == id)
    }

    /// Get all prompts
    pub fn get_all_prompts(&self) -> &[Prompt] {
        &self.prompts
    }

    /// Get prompts containing a specific node ID
    pub fn get_prompts_with_node(&self, node_id: &str) -> Vec<&Prompt> {
        self.prompts
            .iter()
            .filter(|p| {
                p.conversation_history
                    .iter()
                    .any(|m| m.node_id.as_ref().map(|id| id == node_id).unwrap_or(false))
            })
            .collect()
    }

    /// Calculate total token count
    pub fn total_tokens(&self) -> usize {
        self.prompts.iter().map(|p| p.token_count).sum()
    }

    /// Get statistics
    pub fn stats(&self) -> PromptStats {
        let total_prompts = self.prompts.len();
        let total_tokens = self.total_tokens();
        let avg_tokens = if total_prompts > 0 {
            total_tokens / total_prompts
        } else {
            0
        };

        PromptStats {
            total_prompts,
            total_tokens,
            avg_tokens,
        }
    }
}

impl Default for PromptHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptStats {
    pub total_prompts: usize,
    pub total_tokens: usize,
    pub avg_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_history() {
        let mut history = PromptHistory::new();
        
        let prompt = Prompt {
            id: "test_1".to_string(),
            timestamp: "2025-12-03T10:00:00Z".to_string(),
            system_prompt: "You are a coding assistant".to_string(),
            conversation_history: vec![
                Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    node_id: Some("0".to_string()),
                },
            ],
            current_query: "How are you?".to_string(),
            token_count: 100,
            response: Some("I'm fine".to_string()),
        };

        history.add_prompt(prompt);
        assert_eq!(history.prompts.len(), 1);
        assert_eq!(history.total_tokens(), 100);
    }

    #[test]
    fn test_auto_rotate() {
        let mut history = PromptHistory::new();
        
        // Add 1100 prompts
        for i in 0..1100 {
            let prompt = Prompt {
                id: format!("test_{}", i),
                timestamp: "2025-12-03T10:00:00Z".to_string(),
                system_prompt: "Test".to_string(),
                conversation_history: vec![],
                current_query: "Test".to_string(),
                token_count: 10,
                response: None,
            };
            history.add_prompt(prompt);
        }

        // Should keep only last 1000
        assert_eq!(history.prompts.len(), 1000);
        assert_eq!(history.prompts[0].id, "test_100");
    }
}
