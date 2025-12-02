use super::{LLMConfig, LLMModel};
use crate::graph::CodeNode;
use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysis {
    pub should_delete: bool,
    pub confidence: f32,
    pub reason: String,
    pub category: AnalysisCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisCategory {
    SafeToDelete,   // 削除推奨
    KeepForFuture,  // 将来使う予定
    Experimental,   // 実験的機能
    WorkInProgress, // WIP
    NeedsReview,    // 要確認
}

pub struct LLMAnalyzer {
    model: Option<LLMModel>,
    config: LLMConfig,
}

impl LLMAnalyzer {
    pub fn new(config: LLMConfig, enable_llm: bool) -> Result<Self> {
        let model = if enable_llm {
            Some(LLMModel::new(config.clone())?)
        } else {
            None
        };

        Ok(Self { model, config })
    }

    pub fn analyze(&mut self, node: &CodeNode, context: &str) -> Result<LLMAnalysis> {
        if self.model.is_some() {
            self.analyze_with_llm(node, context)
        } else {
            // Fallback to rule-based analysis
            Ok(self.analyze_rule_based(node))
        }
    }

    fn analyze_with_llm(&mut self, node: &CodeNode, context: &str) -> Result<LLMAnalysis> {
        let prompt = self.build_prompt(node, context);
        let response = self.model.as_mut().unwrap().generate(&prompt)?;

        // Parse LLM response
        self.parse_llm_response(&response)
    }

    fn build_prompt(&self, node: &CodeNode, context: &str) -> String {
        format!(
            r#"You are a code analysis expert. Analyze if the following unused function should be deleted or kept.

Function: {}
File: {}
Lines: {}-{}
Exported: {}

Context:
{}

Analyze carefully:
1. Is this function actually used? (Check for dynamic calls, reflection, etc.)
2. Is it likely to be used in the future? (Check for WIP, TODO, recent commits)
3. Is it experimental or under development?
4. Does it have historical significance?
5. Should it be deleted or kept?

Categories:
- SafeToDelete: Old, deprecated, or replaced code
- KeepForFuture: Recently added, marked as TODO/WIP
- Experimental: Experimental features, prototypes
- WorkInProgress: Active development
- NeedsReview: Uncertain, needs human review

Respond ONLY with valid JSON (no markdown, no extra text):
{{
  "should_delete": true,
  "confidence": 0.95,
  "reason": "This function was replaced 2 years ago",
  "category": "SafeToDelete"
}}
"#,
            node.name,
            node.file_path.display(),
            node.line_range.0,
            node.line_range.1,
            node.is_exported,
            context
        )
    }

    fn parse_llm_response(&self, response: &str) -> Result<LLMAnalysis> {
        // Try to extract JSON from response (LLM might add extra text)
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        // Parse JSON
        match serde_json::from_str::<LLMAnalysis>(json_str) {
            Ok(analysis) => Ok(analysis),
            Err(e) => {
                // Fallback: try to extract information manually
                eprintln!("⚠️  Failed to parse LLM response as JSON: {}", e);
                eprintln!("Response: {}", response);

                let should_delete = response.to_lowercase().contains("should_delete\": true")
                    || response.to_lowercase().contains("safe to delete");

                let confidence = extract_confidence(response).unwrap_or(0.5);

                let reason = extract_reason(response)
                    .unwrap_or_else(|| "Failed to parse LLM response".to_string());

                let category = if should_delete {
                    AnalysisCategory::SafeToDelete
                } else {
                    AnalysisCategory::NeedsReview
                };

                Ok(LLMAnalysis {
                    should_delete,
                    confidence,
                    reason,
                    category,
                })
            }
        }
    }

    fn analyze_rule_based(&self, node: &CodeNode) -> LLMAnalysis {
        // Fallback to simple rule-based analysis
        let should_delete = !node.is_exported;
        let confidence = if should_delete { 0.8 } else { 0.3 };
        let reason = if should_delete {
            "Not exported and not used".to_string()
        } else {
            "Exported function - may be used externally".to_string()
        };
        let category = if should_delete {
            AnalysisCategory::SafeToDelete
        } else {
            AnalysisCategory::NeedsReview
        };

        LLMAnalysis {
            should_delete,
            confidence,
            reason,
            category,
        }
    }
}

// Helper functions for parsing LLM responses
fn extract_confidence(text: &str) -> Option<f32> {
    // Try to find "confidence": 0.XX pattern
    if let Some(start) = text.find("\"confidence\"") {
        let rest = &text[start..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon + 1..];
            // Extract number
            let num_str: String = after_colon
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_numeric() || *c == '.')
                .collect();

            return num_str.parse::<f32>().ok();
        }
    }
    None
}

fn extract_reason(text: &str) -> Option<String> {
    // Try to find "reason": "..." pattern
    if let Some(start) = text.find("\"reason\"") {
        let rest = &text[start..];
        if let Some(quote_start) = rest
            .find('"')
            .and_then(|i| rest[i + 1..].find('"').map(|j| i + j + 2))
        {
            let after_quote = &rest[quote_start..];
            if let Some(quote_end) = after_quote.find('"') {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }
    None
}
