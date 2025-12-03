// Conversation analyzer
use anyhow::Result;
use std::path::PathBuf;

use super::graph::{ConversationGraph, ConversationNode, ConversationEdge, RelationType};

/// Conversation analyzer
pub struct ConversationAnalyzer {}

impl ConversationAnalyzer {
    /// Create a new conversation analyzer
    pub fn new() -> Result<Self> {
        Ok(Self {})
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

            let node = ConversationNode {
                id: id.clone(),
                timestamp,
                role,
                content,
                embedding: None, // Embeddingは使用しない
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

        Ok(graph)
    }

    /// Find related messages based on query (simple text matching)
    pub fn find_related_messages(
        &self,
        graph: &ConversationGraph,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RelatedMessage>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for node in &graph.nodes {
            let content_lower = node.content.to_lowercase();
            
            // シンプルなテキストマッチングで類似度を計算
            let similarity = if content_lower.contains(&query_lower) {
                1.0
            } else {
                // 単語の一致数で類似度を計算
                let query_words: Vec<&str> = query_lower.split_whitespace().collect();
                let content_words: Vec<&str> = content_lower.split_whitespace().collect();
                
                let matches = query_words.iter()
                    .filter(|qw| content_words.iter().any(|cw| cw.contains(*qw)))
                    .count();
                
                if query_words.is_empty() {
                    0.0
                } else {
                    matches as f32 / query_words.len() as f32
                }
            };
            
            if similarity > 0.0 {
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
    /// 
    /// Returns statistics about how much context can be reduced by selecting only relevant messages.
    /// - reduction_rate: The percentage of tokens that can be SAVED (not used)
    ///   - 0% = no reduction possible (all messages are relevant)
    ///   - 50% = half the tokens can be saved
    ///   - Target: 40-60% reduction
    /// 
    /// The algorithm:
    /// 1. Find semantically related messages based on query
    /// 2. Add context window (1 message before/after) for coherence
    /// 3. No blind inclusion of recent messages - relevance is key
    pub fn calculate_token_reduction(&self, graph: &ConversationGraph, query: Option<&str>) -> TokenReduction {
        // Estimate tokens: roughly 1.3 tokens per word for English/Japanese
        let estimate_tokens = |text: &str| -> usize {
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            // For Japanese text (more characters than words), use character-based estimation
            if char_count > word_count * 3 {
                // Japanese: ~2 chars per token, minimum 1 token
                std::cmp::max(1, (char_count as f32 * 0.5) as usize)
            } else {
                // English: ~1.3 tokens per word, minimum 1 token
                std::cmp::max(1, (word_count as f32 * 1.3) as usize)
            }
        };

        let total_tokens: usize = graph
            .nodes
            .iter()
            .map(|n| estimate_tokens(&n.content))
            .sum();

        // Collect indices of messages to include (based on relevance only)
        let mut included_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

        if let Some(q) = query {
            // Find related messages based on query
            if let Ok(related) = self.find_related_messages(graph, q, 10) {
                for rel in &related {
                    // Find index of related message
                    if let Some(idx) = graph.nodes.iter().position(|n| n.id == rel.id) {
                        included_indices.insert(idx);
                        // Add context window (1 message before and after) for coherence
                        if idx > 0 {
                            included_indices.insert(idx - 1);
                        }
                        if idx + 1 < graph.nodes.len() {
                            included_indices.insert(idx + 1);
                        }
                    }
                }
            }
        } else {
            // No query: use topic-based selection
            let unique_topics: std::collections::HashSet<_> = graph
                .nodes
                .iter()
                .filter_map(|n| n.topic_id.as_ref())
                .collect();

            if !unique_topics.is_empty() {
                // Keep messages from the most recent topics
                let recent_topics: Vec<_> = unique_topics.into_iter().take(3).collect();
                for (idx, node) in graph.nodes.iter().enumerate() {
                    if node.topic_id.as_ref().map(|t| recent_topics.contains(&t)).unwrap_or(false) {
                        included_indices.insert(idx);
                    }
                }
            } else {
                // Fallback: if no topics and no query, include all (no reduction)
                for idx in 0..graph.nodes.len() {
                    included_indices.insert(idx);
                }
            }
        }

        // Calculate tokens for included messages
        let relevant_tokens: usize = if included_indices.is_empty() {
            // No related messages found - this means high reduction is possible
            // but we should return 0 to indicate "nothing relevant found"
            0
        } else {
            included_indices
                .iter()
                .filter_map(|&idx| graph.nodes.get(idx))
                .map(|n| estimate_tokens(&n.content))
                .sum()
        };

        // Calculate reduction rate
        let reduction_rate = if total_tokens > 0 && relevant_tokens > 0 {
            let saved = total_tokens.saturating_sub(relevant_tokens);
            saved as f32 / total_tokens as f32
        } else if total_tokens > 0 && relevant_tokens == 0 {
            // No relevant messages found - report as 0% reduction
            // (we can't reduce if we found nothing useful)
            0.0
        } else {
            0.0
        };

        // Clamp to target range (0% - 80%)
        let reduction_rate = reduction_rate.clamp(0.0, 0.8);

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
