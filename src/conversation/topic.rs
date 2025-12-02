// Topic detection
use anyhow::Result;
use std::collections::HashMap;

use super::graph::{ConversationGraph, Topic};

/// Topic detector
pub struct TopicDetector {
    min_cluster_size: usize,
}

impl TopicDetector {
    /// Create a new topic detector
    pub fn new() -> Self {
        Self {
            min_cluster_size: 3,
        }
    }

    /// Detect topics in conversation graph
    pub fn detect_topics(&self, graph: &mut ConversationGraph) -> Result<()> {
        // Simple keyword-based topic detection
        let mut keyword_groups: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            let keywords = self.extract_keywords(&node.content);

            for keyword in keywords {
                keyword_groups
                    .entry(keyword.clone())
                    .or_insert_with(Vec::new)
                    .push(node.id.clone());
            }
        }

        // Create topics from keyword groups
        let mut topic_id = 0;
        for (keyword, message_ids) in keyword_groups {
            if message_ids.len() >= self.min_cluster_size {
                let topic = Topic {
                    id: format!("topic_{}", topic_id),
                    name: keyword.clone(),
                    keywords: vec![keyword],
                    message_ids: message_ids.clone(),
                };

                // Assign topic to nodes
                for msg_id in &message_ids {
                    if let Some(node) = graph.nodes.iter_mut().find(|n| &n.id == msg_id) {
                        node.topic_id = Some(topic.id.clone());
                    }
                }

                graph.add_topic(topic);
                topic_id += 1;
            }
        }

        Ok(())
    }

    /// Extract keywords from text
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let stop_words = vec!["the", "a", "an", "and", "or", "but", "in", "on", "at", "to"];

        text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3 && !stop_words.contains(&w.as_ref()))
            .map(|w| w.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_detector() {
        let detector = TopicDetector::new();
        let keywords = detector.extract_keywords("This is a test function");
        assert!(keywords.contains(&"test".to_string()));
        assert!(keywords.contains(&"function".to_string()));
    }
}
