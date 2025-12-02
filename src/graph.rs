use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub type NodeId = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: HashMap<NodeId, CodeNode>,
    pub edges: Vec<DependencyEdge>,
    next_id: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub file_path: PathBuf,
    pub line_range: (usize, usize),
    pub is_exported: bool,
    pub is_used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Function,
    Class,
    Method,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Calls,
    References,
    Instantiates,
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, node: CodeNode) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, edge: DependencyEdge) {
        self.edges.push(edge);
    }

    #[allow(dead_code)]
    pub fn get_node(&self, id: NodeId) -> Option<&CodeNode> {
        self.nodes.get(&id)
    }
}
