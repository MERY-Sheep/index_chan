use crate::graph::{CodeGraph, CodeNode};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCode {
    pub node: CodeNode,
    pub safety_level: SafetyLevel,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    DefinitelySafe,
    ProbablySafe,
    NeedsReview,
}

pub fn detect_dead_code(graph: &CodeGraph) -> Vec<DeadCode> {
    let mut dead_code = Vec::new();
    
    // Mark nodes as used based on edges
    let mut used_nodes = std::collections::HashSet::new();
    for edge in &graph.edges {
        used_nodes.insert(edge.to);
    }
    
    for (id, node) in &graph.nodes {
        // Skip entry points
        if is_entry_point(node) {
            continue;
        }
        
        // Skip exported functions (they might be used externally)
        if node.is_exported {
            continue;
        }
        
        // Check if used
        if !used_nodes.contains(id) {
            let (safety, reason) = evaluate_safety(node, graph);
            dead_code.push(DeadCode {
                node: node.clone(),
                safety_level: safety,
                reason,
            });
        }
    }
    
    dead_code
}

fn is_entry_point(node: &CodeNode) -> bool {
    // Check for common entry point patterns
    let name = node.name.as_str();
    matches!(name, "main" | "index" | "app" | "start")
}

fn evaluate_safety(node: &CodeNode, _graph: &CodeGraph) -> (SafetyLevel, String) {
    // Exported functions need review
    if node.is_exported {
        return (
            SafetyLevel::NeedsReview,
            "Exported function - may be used externally".to_string(),
        );
    }
    
    // Test files need review
    let path_str = node.file_path.to_string_lossy();
    if path_str.contains("test") || path_str.contains("spec") {
        return (
            SafetyLevel::NeedsReview,
            "Test file - may be used in tests".to_string(),
        );
    }
    
    // Check for dynamic call risks
    if has_dynamic_call_risk(node) {
        return (
            SafetyLevel::ProbablySafe,
            "Possible dynamic call pattern".to_string(),
        );
    }
    
    (
        SafetyLevel::DefinitelySafe,
        "Not exported, no references found".to_string(),
    )
}

fn has_dynamic_call_risk(node: &CodeNode) -> bool {
    // Simple heuristic: check if name suggests dynamic usage
    let name = node.name.to_lowercase();
    name.contains("dynamic") || name.contains("eval") || name.contains("reflect")
}
