// Conversation analyzer
use anyhow::Result;
use std::path::PathBuf;

use super::graph::{ConversationGraph, ConversationNode, ConversationEdge, RelationType};
use crate::search::embeddings::EmbeddingModel;

/// Conversation analyzer
pub struct ConversationAnalyzer {
    embedding_model: EmbeddingModel,
}

impl ConversationAnalyzer {
    /// Create a new conversation analyzer
    pub fn new() -> Result<Self> {
        let config = crate::search::embeddings::EmbeddingConfig::default();
        let embedding_model = EmbeddingModel::new(config)?;

        Ok(Self { embedding_model })
    }

    /// Analyze chat history from JSON file
    pub fn analyze_file(&self, path: &PathBuf) -> Result<ConversationGraph> {
        let content = std::fs::read_to_string(path)?;
        let messages: Vec<serde_json::Value> = serde_json::from_str(&content)?;

        let mut graph = ConversationGraph::new();

        // Parse messages
        for (i, msg) in messages.iter().enumerate() {
            let id = format!("{}", i);
            let timestamp = msg["timestamp"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            let role = msg["role"].as_str().unwrap_or("user").to_string();
            let content = msg["content"].as_str().unwrap_or("").to_string();

            // Generate embedding
            let embedding = self.embedding_model.encode(&content).ok();

            let node = ConversationNode {
                id: id.clone(),
                timestamp,
                role,
                content,
                embedding,
                topic_id: None,
            };

            graph.add_node(node);

            // Add sequential edge
            if i > 0 {
                let edge = ConversationEdge {
                    from: format!("{}", i - 1),
                    to: id,
                    weight: 1.0,
                    relation_type: RelationType::Sequential,
                };
                graph.add_edge(edge);
            }
        }

        // Add semantic edges
        self.add_semantic_edges(&mut graph)?;

        Ok(graph)
    }

    /// Add semantic edges based on similarity
    fn add_semantic_edges(&self, graph: &mut ConversationGraph) -> Result<()> {
        let threshold = 0.7; // Similarity threshold

        for i in 0..graph.nodes.len() {
            for j in (i + 1)..graph.nodes.len() {
                let node_i = &graph.nodes[i];
                let node_j = &graph.nodes[j];

                if let (Some(emb_i), Some(emb_j)) = (&node_i.embedding, &node_j.embedding) {
                    let similarity = EmbeddingModel::cosine_similarity(emb_i, emb_j);

                    if similarity > threshold {
                        let edge = ConversationEdge {
                            from: node_i.id.clone(),
                            to: node_j.id.clone(),
                            weight: similarity,
                            relation_type: RelationType::Semantic,
                        };
                        graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculate token reduction
    pub fn calculate_token_reduction(&self, graph: &ConversationGraph) -> TokenReduction {
        let total_tokens: usize = graph
            .nodes
            .iter()
            .map(|n| n.content.split_whitespace().count())
            .sum();

        // Simple heuristic: keep only related messages
        let relevant_tokens: usize = total_tokens / 2; // Placeholder

        let reduction_rate = if total_tokens > 0 {
            (total_tokens - relevant_tokens) as f32 / total_tokens as f32
        } else {
            0.0
        };

        TokenReduction {
            total_tokens,
            relevant_tokens,
            reduction_rate,
        }
    }
}

/// Token reduction statistics
#[derive(Debug, Clone)]
pub struct TokenReduction {
    pub total_tokens: usize,
    pub relevant_tokens: usize,
    pub reduction_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_analyzer() {
        let analyzer = ConversationAnalyzer::new().unwrap();
        // Test with actual file would go here
    }
}
