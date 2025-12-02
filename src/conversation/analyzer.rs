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

    /// Find related messages based on query
    pub fn find_related_messages(
        &self,
        graph: &ConversationGraph,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RelatedMessage>> {
        let query_embedding = self.embedding_model.encode(query)?;
        let mut results = Vec::new();

        for node in &graph.nodes {
            if let Some(node_embedding) = &node.embedding {
                let similarity = EmbeddingModel::cosine_similarity(&query_embedding, node_embedding);
                
                results.push(RelatedMessage {
                    id: node.id.clone(),
                    content: node.content.clone(),
                    role: node.role.clone(),
                    timestamp: node.timestamp.clone(),
                    similarity,
                    topic_id: node.topic_id.clone(),
                });
            }
        }

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        
        // Take top K
        results.truncate(top_k);

        Ok(results)
    }

    /// Calculate token reduction with smart context selection
    pub fn calculate_token_reduction(&self, graph: &ConversationGraph, query: Option<&str>) -> TokenReduction {
        // Estimate tokens: roughly 1.3 tokens per word for English/Japanese
        let estimate_tokens = |text: &str| -> usize {
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            // For Japanese text (more characters than words), use character-based estimation
            if char_count > word_count * 3 {
                (char_count as f32 * 0.5) as usize // Japanese: ~2 chars per token
            } else {
                (word_count as f32 * 1.3) as usize // English: ~1.3 tokens per word
            }
        };

        let total_tokens: usize = graph
            .nodes
            .iter()
            .map(|n| estimate_tokens(&n.content))
            .sum();

        let relevant_tokens = if let Some(q) = query {
            // Use semantic search to find relevant messages
            if let Ok(related) = self.find_related_messages(graph, q, 10) {
                related
                    .iter()
                    .map(|m| estimate_tokens(&m.content))
                    .sum()
            } else {
                total_tokens / 2
            }
        } else {
            // Use topic-based reduction
            let unique_topics: std::collections::HashSet<_> = graph
                .nodes
                .iter()
                .filter_map(|n| n.topic_id.as_ref())
                .collect();

            if !unique_topics.is_empty() {
                // Keep messages from the most recent topics
                let recent_topics: Vec<_> = unique_topics.into_iter().take(3).collect();
                graph
                    .nodes
                    .iter()
                    .filter(|n| {
                        n.topic_id
                            .as_ref()
                            .map(|t| recent_topics.contains(&t))
                            .unwrap_or(false)
                    })
                    .map(|n| estimate_tokens(&n.content))
                    .sum()
            } else {
                // Fallback: keep recent messages
                graph
                    .nodes
                    .iter()
                    .rev()
                    .take(10)
                    .map(|n| estimate_tokens(&n.content))
                    .sum()
            }
        };

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

/// Related message result
#[derive(Debug, Clone)]
pub struct RelatedMessage {
    pub id: String,
    pub content: String,
    pub role: String,
    pub timestamp: String,
    pub similarity: f32,
    pub topic_id: Option<String>,
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
        let _analyzer = ConversationAnalyzer::new().unwrap();
        // Test with actual file would go here
    }
}
