// Graph data exporter for UI visualization
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::graph::{ConversationGraph, RelationType};

/// Graph node for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub is_reduced: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<String>,
}

/// Graph edge for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub weight: f32,
    pub edge_type: String,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub reduced_nodes: usize,
    pub reduction_rate: f32,
}

/// Complete graph data for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

impl GraphData {
    /// Create graph data from conversation graph
    pub fn from_conversation_graph(graph: &ConversationGraph, reduced_node_ids: &[String]) -> Self {
        let nodes: Vec<GraphNode> = graph
            .nodes
            .iter()
            .map(|node| {
                let is_reduced = reduced_node_ids.contains(&node.id);
                // 日本語対応：文字数でカット（バイト境界を考慮）
                let content_preview = node.content.chars().take(20).collect::<String>();
                GraphNode {
                    id: node.id.clone(),
                    label: format!("{}\n{}", node.role, content_preview),
                    role: node.role.clone(),
                    content: node.content.clone(),
                    timestamp: node.timestamp.clone(),
                    is_reduced,
                    topic_id: node.topic_id.clone(),
                }
            })
            .collect();

        let edges: Vec<GraphEdge> = graph
            .edges
            .iter()
            .map(|edge| GraphEdge {
                from: edge.from.clone(),
                to: edge.to.clone(),
                weight: edge.weight,
                edge_type: match edge.relation_type {
                    RelationType::Sequential => "sequential".to_string(),
                    RelationType::Semantic => "semantic".to_string(),
                    RelationType::Reference => "reference".to_string(),
                    RelationType::CodeRelated => "code_related".to_string(),
                },
            })
            .collect();

        let reduced_nodes = reduced_node_ids.len();
        let total_nodes = nodes.len();
        let reduction_rate = if total_nodes > 0 {
            (reduced_nodes as f32 / total_nodes as f32) * 100.0
        } else {
            0.0
        };

        let metadata = GraphMetadata {
            total_nodes,
            total_edges: edges.len(),
            reduced_nodes,
            reduction_rate,
        };

        GraphData {
            nodes,
            edges,
            metadata,
        }
    }

    /// Save graph data to JSON file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load graph data from JSON file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let data: GraphData = serde_json::from_str(&content)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::graph::{ConversationNode, ConversationEdge};

    #[test]
    fn test_graph_data_export() {
        let mut graph = ConversationGraph::new();
        
        graph.add_node(ConversationNode {
            id: "0".to_string(),
            timestamp: "10:00:00".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            embedding: None,
            topic_id: None,
        });

        graph.add_node(ConversationNode {
            id: "1".to_string(),
            timestamp: "10:01:00".to_string(),
            role: "assistant".to_string(),
            content: "Hi there".to_string(),
            embedding: None,
            topic_id: None,
        });

        graph.add_edge(ConversationEdge {
            from: "0".to_string(),
            to: "1".to_string(),
            weight: 1.0,
            relation_type: RelationType::Sequential,
        });

        let reduced_nodes = vec!["1".to_string()];
        let graph_data = GraphData::from_conversation_graph(&graph, &reduced_nodes);

        assert_eq!(graph_data.nodes.len(), 2);
        assert_eq!(graph_data.edges.len(), 1);
        assert_eq!(graph_data.metadata.reduced_nodes, 1);
        assert_eq!(graph_data.metadata.reduction_rate, 50.0);
    }
}
