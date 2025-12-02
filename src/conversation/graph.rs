// Conversation graph data structures
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conversation graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationGraph {
    pub nodes: Vec<ConversationNode>,
    pub edges: Vec<ConversationEdge>,
    pub topics: Vec<Topic>,
}

/// Conversation node (message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationNode {
    pub id: String,
    pub timestamp: String,
    pub role: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub topic_id: Option<String>,
}

/// Conversation edge (relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEdge {
    pub from: String,
    pub to: String,
    pub weight: f32,
    pub relation_type: RelationType,
}

/// Relation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    Sequential,   // Time-based continuation
    Semantic,     // Semantic similarity
    Reference,    // Explicit reference
    CodeRelated,  // Code dependency
}

/// Topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub message_ids: Vec<String>,
}

impl ConversationGraph {
    /// Create a new conversation graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            topics: Vec::new(),
        }
    }

    /// Add a node
    pub fn add_node(&mut self, node: ConversationNode) {
        self.nodes.push(node);
    }

    /// Add an edge
    pub fn add_edge(&mut self, edge: ConversationEdge) {
        self.edges.push(edge);
    }

    /// Add a topic
    pub fn add_topic(&mut self, topic: Topic) {
        self.topics.push(topic);
    }

    /// Get node by id
    pub fn get_node(&self, id: &str) -> Option<&ConversationNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get related nodes
    pub fn get_related_nodes(&self, id: &str) -> Vec<&ConversationNode> {
        let related_ids: Vec<String> = self
            .edges
            .iter()
            .filter(|e| e.from == id || e.to == id)
            .flat_map(|e| vec![e.from.clone(), e.to.clone()])
            .filter(|i| i != id)
            .collect();

        self.nodes
            .iter()
            .filter(|n| related_ids.contains(&n.id))
            .collect()
    }

    /// Get nodes by topic
    pub fn get_nodes_by_topic(&self, topic_id: &str) -> Vec<&ConversationNode> {
        self.nodes
            .iter()
            .filter(|n| n.topic_id.as_ref() == Some(&topic_id.to_string()))
            .collect()
    }

    /// Calculate statistics
    pub fn stats(&self) -> ConversationStats {
        let total_messages = self.nodes.len();
        let total_topics = self.topics.len();
        let total_edges = self.edges.len();

        let avg_edges_per_node = if total_messages > 0 {
            total_edges as f32 / total_messages as f32
        } else {
            0.0
        };

        ConversationStats {
            total_messages,
            total_topics,
            total_edges,
            avg_edges_per_node,
        }
    }
}

/// Conversation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStats {
    pub total_messages: usize,
    pub total_topics: usize,
    pub total_edges: usize,
    pub avg_edges_per_node: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_graph() {
        let mut graph = ConversationGraph::new();

        let node1 = ConversationNode {
            id: "1".to_string(),
            timestamp: "2024-12-02T10:00:00Z".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            embedding: None,
            topic_id: None,
        };

        graph.add_node(node1);
        assert_eq!(graph.nodes.len(), 1);

        let edge = ConversationEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            weight: 1.0,
            relation_type: RelationType::Sequential,
        };

        graph.add_edge(edge);
        assert_eq!(graph.edges.len(), 1);
    }
}
