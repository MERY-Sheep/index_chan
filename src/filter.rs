// Graph filtering module for Phase 3.3
use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::graph::{CodeGraph, CodeNode, DependencyEdge, NodeId};

#[cfg(feature = "search")]
use crate::search::CodeIndex;

#[cfg(feature = "llm")]
use crate::llm::LLMAnalyzer;

/// Filter intent parsed from natural language query
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FilterIntent {
    ShowRelated,  // "○○関連を表示"
    ShowOnly,     // "○○だけ表示"
    Hide,         // "○○を非表示"
    Highlight,    // "○○を強調"
}

/// Filter scope
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FilterScope {
    Functions,        // 関数名で検索
    Files,            // ファイル名で検索
    Dependencies,     // 依存関係を含む
    DeadCode,         // デッドコードのみ
}

/// Parsed filter query
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilterQuery {
    pub intent: FilterIntent,
    pub keywords: Vec<String>,
    pub scope: FilterScope,
}

/// Graph filter
#[allow(dead_code)]
pub struct GraphFilter {
    #[cfg(feature = "search")]
    search_index: Option<CodeIndex>,
    
    #[cfg(feature = "llm")]
    llm: Option<LLMAnalyzer>,
}

impl GraphFilter {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "search")]
            search_index: None,
            
            #[cfg(feature = "llm")]
            llm: None,
        }
    }

    #[cfg(feature = "search")]
    pub fn with_search_index(mut self, index: CodeIndex) -> Self {
        self.search_index = Some(index);
        self
    }

    #[cfg(feature = "llm")]
    pub fn with_llm(mut self, llm: LLMAnalyzer) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Filter graph by simple keyword search
    pub fn filter_by_keywords(
        &self,
        graph: &CodeGraph,
        keywords: &[String],
        include_dependencies: bool,
    ) -> Result<CodeGraph> {
        let mut relevant_nodes = HashSet::new();

        // Find nodes matching keywords
        for (id, node) in &graph.nodes {
            if self.matches_keywords(node, keywords) {
                relevant_nodes.insert(*id);
            }
        }

        // Include dependencies if requested
        if include_dependencies {
            let deps = self.collect_dependencies(graph, &relevant_nodes);
            relevant_nodes.extend(deps);
        }

        // Build filtered graph
        self.build_filtered_graph(graph, &relevant_nodes)
    }

    /// Filter graph by dead code
    pub fn filter_dead_code(&self, graph: &CodeGraph) -> Result<CodeGraph> {
        let mut dead_nodes = HashSet::new();

        for (id, node) in &graph.nodes {
            if !node.is_used {
                dead_nodes.insert(*id);
            }
        }

        self.build_filtered_graph(graph, &dead_nodes)
    }

    /// Filter graph by file path
    pub fn filter_by_file(
        &self,
        graph: &CodeGraph,
        file_pattern: &str,
        include_dependencies: bool,
    ) -> Result<CodeGraph> {
        let mut relevant_nodes = HashSet::new();

        for (id, node) in &graph.nodes {
            if let Some(path_str) = node.file_path.to_str() {
                if path_str.contains(file_pattern) {
                    relevant_nodes.insert(*id);
                }
            }
        }

        if include_dependencies {
            let deps = self.collect_dependencies(graph, &relevant_nodes);
            relevant_nodes.extend(deps);
        }

        self.build_filtered_graph(graph, &relevant_nodes)
    }

    /// Filter graph using semantic search (requires search feature)
    #[cfg(feature = "search")]
    pub fn filter_by_semantic_search(
        &self,
        graph: &CodeGraph,
        query: &str,
        top_k: usize,
        include_dependencies: bool,
    ) -> Result<CodeGraph> {
        let search_index = self.search_index.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Search index not initialized"))?;

        // Perform semantic search
        let results = search_index.search(query, top_k)?;

        // Extract node IDs from search results
        let mut relevant_nodes = HashSet::new();
        for result in results {
            // Assuming search results contain node IDs in metadata
            // This needs to be adapted based on actual search implementation
            if let Some(node_id) = self.find_node_by_name(graph, &result.text) {
                relevant_nodes.insert(node_id);
            }
        }

        if include_dependencies {
            let deps = self.collect_dependencies(graph, &relevant_nodes);
            relevant_nodes.extend(deps);
        }

        self.build_filtered_graph(graph, &relevant_nodes)
    }

    /// Parse natural language query using LLM (requires llm feature)
    #[cfg(feature = "llm")]
    pub fn parse_query_with_llm(&mut self, query: &str) -> Result<FilterQuery> {
        let llm = self.llm.as_mut()
            .ok_or_else(|| anyhow::anyhow!("LLM not initialized"))?;

        let prompt = format!(
            r#"以下のクエリを解析して、フィルタリングの意図とキーワードを抽出してください。

クエリ: "{}"

以下のJSON形式で返してください:
{{
    "intent": "show_related" | "show_only" | "hide" | "highlight",
    "keywords": ["キーワード1", "キーワード2"],
    "scope": "functions" | "files" | "dependencies" | "dead_code"
}}

例:
クエリ: "LLM関連の機能を表示"
{{
    "intent": "show_related",
    "keywords": ["LLM", "llm", "analyzer", "inference"],
    "scope": "functions"
}}
"#,
            query
        );

        let response = llm.generate(&prompt)?;
        
        // Parse JSON response
        self.parse_llm_response(&response)
    }

    #[cfg(feature = "llm")]
    fn parse_llm_response(&self, response: &str) -> Result<FilterQuery> {
        // Extract JSON from response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        #[derive(serde::Deserialize)]
        struct LLMResponse {
            intent: String,
            keywords: Vec<String>,
            scope: String,
        }

        let parsed: LLMResponse = serde_json::from_str(json_str)?;

        let intent = match parsed.intent.as_str() {
            "show_related" => FilterIntent::ShowRelated,
            "show_only" => FilterIntent::ShowOnly,
            "hide" => FilterIntent::Hide,
            "highlight" => FilterIntent::Highlight,
            _ => FilterIntent::ShowRelated,
        };

        let scope = match parsed.scope.as_str() {
            "functions" => FilterScope::Functions,
            "files" => FilterScope::Files,
            "dependencies" => FilterScope::Dependencies,
            "dead_code" => FilterScope::DeadCode,
            _ => FilterScope::Functions,
        };

        Ok(FilterQuery {
            intent,
            keywords: parsed.keywords,
            scope,
        })
    }

    // Helper methods

    fn matches_keywords(&self, node: &CodeNode, keywords: &[String]) -> bool {
        let node_name_lower = node.name.to_lowercase();
        let file_path_lower = node.file_path.to_string_lossy().to_lowercase();

        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            if node_name_lower.contains(&keyword_lower) 
                || file_path_lower.contains(&keyword_lower) {
                return true;
            }
        }

        false
    }

    fn collect_dependencies(
        &self,
        graph: &CodeGraph,
        seed_nodes: &HashSet<NodeId>,
    ) -> HashSet<NodeId> {
        let mut dependencies = HashSet::new();

        for edge in &graph.edges {
            if seed_nodes.contains(&edge.from) {
                dependencies.insert(edge.to);
            }
            if seed_nodes.contains(&edge.to) {
                dependencies.insert(edge.from);
            }
        }

        dependencies
    }

    fn build_filtered_graph(
        &self,
        graph: &CodeGraph,
        node_ids: &HashSet<NodeId>,
    ) -> Result<CodeGraph> {
        let mut filtered = CodeGraph::new();
        let mut id_mapping = HashMap::new();

        // Add nodes
        for id in node_ids {
            if let Some(node) = graph.nodes.get(id) {
                let mut new_node = node.clone();
                new_node.id = filtered.nodes.len();
                let new_id = filtered.add_node(new_node);
                id_mapping.insert(*id, new_id);
            }
        }

        // Add edges (only between filtered nodes)
        for edge in &graph.edges {
            if let (Some(&new_from), Some(&new_to)) = 
                (id_mapping.get(&edge.from), id_mapping.get(&edge.to)) {
                filtered.add_edge(DependencyEdge {
                    from: new_from,
                    to: new_to,
                    edge_type: edge.edge_type,
                });
            }
        }

        Ok(filtered)
    }

    #[cfg(feature = "search")]
    fn find_node_by_name(&self, graph: &CodeGraph, name: &str) -> Option<NodeId> {
        for (id, node) in &graph.nodes {
            if node.name == name {
                return Some(*id);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CodeNode, NodeType};
    use std::path::PathBuf;

    #[test]
    fn test_filter_by_keywords() {
        let mut graph = CodeGraph::new();
        
        let node1 = CodeNode {
            id: 0,
            name: "llm_analyzer".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/llm/analyzer.rs"),
            line_range: (1, 10),
            is_exported: true,
            is_used: true,
            signature: "fn llm_analyzer()".to_string(),
        };

        let node2 = CodeNode {
            id: 1,
            name: "scan_file".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/scanner.rs"),
            line_range: (1, 10),
            is_exported: true,
            is_used: true,
            signature: "fn scan_file()".to_string(),
        };

        graph.add_node(node1);
        graph.add_node(node2);

        let filter = GraphFilter::new();
        let keywords = vec!["llm".to_string()];
        let filtered = filter.filter_by_keywords(&graph, &keywords, false).unwrap();

        assert_eq!(filtered.nodes.len(), 1);
    }

    #[test]
    fn test_filter_dead_code() {
        let mut graph = CodeGraph::new();
        
        let node1 = CodeNode {
            id: 0,
            name: "used_function".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/main.rs"),
            line_range: (1, 10),
            is_exported: true,
            is_used: true,
            signature: "fn used_function()".to_string(),
        };

        let node2 = CodeNode {
            id: 1,
            name: "unused_function".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/old.rs"),
            line_range: (1, 10),
            is_exported: false,
            is_used: false,
            signature: "fn unused_function()".to_string(),
        };

        graph.add_node(node1);
        graph.add_node(node2);

        let filter = GraphFilter::new();
        let filtered = filter.filter_dead_code(&graph).unwrap();

        assert_eq!(filtered.nodes.len(), 1);
        assert_eq!(filtered.nodes.values().next().unwrap().name, "unused_function");
    }
}
