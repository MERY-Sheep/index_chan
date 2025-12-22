// Context Generation for MCP
// gather_context の実装

use std::path::Path;
use std::collections::{HashMap, HashSet};
use anyhow::Result;

use crate::graph::{CodeGraph, CodeNode, NodeId, NodeType};
use crate::scanner::Scanner;

/// Context output mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextMode {
    Full,     // Complete code
    Skeleton, // Signatures only
}

/// Output format for context
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ContextFormat {
    #[default]
    Standard,  // Current format with comments
    LlmEdit,   // Format optimized for LLM editing with clear markers
}

impl Default for ContextMode {
    fn default() -> Self {
        Self::Full
    }
}

/// Context generation result
#[derive(Debug, Clone)]
pub struct ContextResult {
    pub content: String,
    pub files_count: usize,
    pub functions_count: usize,
    pub total_lines: usize,
    /// Context quality metrics
    pub quality: ContextQuality,
}

/// Context quality metrics based on S/N ratio analysis
/// Inspired by Concept Transformer Phase 9b findings
#[derive(Debug, Clone, Default)]
pub struct ContextQuality {
    /// Estimated token count
    pub estimated_tokens: usize,
    /// Ratio of meaningful identifiers (3+ chars) to short variables (1-2 chars)
    pub sn_ratio: f32,
    /// Quality level: "high" (S/N > 2.0), "medium" (1.0-2.0), "low" (< 1.0)
    pub quality_level: String,
    /// Recommendation for improvement
    pub recommendation: Option<String>,
    /// Concept density score (0.0-1.0): ratio of type definitions, interfaces, etc.
    pub concept_density: f32,
    /// Number of dependencies collected
    pub dependency_count: usize,
    /// Context explosion warning (true if dependencies > 100)
    pub context_explosion_warning: bool,
    /// Entry point ratio: signal (entry point code) / noise (dependencies)
    pub entry_point_ratio: f32,
}

/// Context generator
pub struct ContextGenerator {
    graph: CodeGraph,
}

impl ContextGenerator {
    /// Create from directory scan
    pub fn from_directory(directory: &Path) -> Result<Self> {
        let mut scanner = Scanner::new()?;
        let graph = scanner.scan_directory(directory)?;
        Ok(Self { graph })
    }

    /// Create from existing graph
    pub fn from_graph(graph: CodeGraph) -> Self {
        Self { graph }
    }

    /// Gather context for a function with dependencies
    pub fn gather_context(
        &self,
        entry_point: Option<&str>,
        query: Option<&str>,
        depth: usize,
        mode: ContextMode,
        format: ContextFormat,
    ) -> Result<ContextResult> {
        let mut collected_nodes: Vec<&CodeNode> = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();

        // Find entry point node(s)
        let start_nodes: Vec<&CodeNode> = if let Some(entry) = entry_point {
            // Support qualified names like "context.rs::gather_context"
            if let Some((file_part, func_part)) = entry.split_once("::") {
                self.graph.nodes.values()
                    .filter(|n| {
                        let file_matches = n.file_path.to_string_lossy().contains(file_part);
                        let name_matches = n.name == func_part || n.name.contains(func_part);
                        file_matches && name_matches
                    })
                    .collect()
            } else {
                self.graph.nodes.values()
                    .filter(|n| n.name == entry || n.name.contains(entry))
                    .collect()
            }
        } else if let Some(_q) = query {
            // TODO: Use search index for query-based lookup
            self.graph.nodes.values().take(10).collect()
        } else {
            return Ok(ContextResult {
                content: "// No entry point or query specified".to_string(),
                files_count: 0,
                functions_count: 0,
                total_lines: 0,
                quality: ContextQuality::default(),
            });
        };

        // BFS to collect dependencies
        for node in &start_nodes {
            self.collect_dependencies(node, depth, &mut collected_nodes, &mut visited);
        }

        // Sort by importance for skeleton mode (prioritize high-density nodes)
        // This is based on Concept Transformer insights: type definitions > core logic > getters
        if mode == ContextMode::Skeleton {
            let entry_point_ids: HashSet<NodeId> = start_nodes.iter().map(|n| n.id).collect();
            self.sort_by_importance(&mut collected_nodes, &entry_point_ids);
        }

        // Generate output
        let content = self.format_context(&collected_nodes, mode, format, entry_point, query);
        let files: HashSet<_> = collected_nodes.iter().map(|n| &n.file_path).collect();
        let total_lines: usize = collected_nodes.iter().map(|n| n.line_range.1 - n.line_range.0 + 1).sum();

        // Calculate entry point lines (signal vs noise ratio)
        let entry_point_lines: usize = start_nodes.iter()
            .map(|n| n.line_range.1 - n.line_range.0 + 1)
            .sum();

        // Calculate context quality with enhanced metrics
        let quality = self.calculate_quality(&content, total_lines, mode, &collected_nodes, entry_point_lines);

        Ok(ContextResult {
            content,
            files_count: files.len(),
            functions_count: collected_nodes.len(),
            total_lines,
            quality,
        })
    }

    /// Calculate context quality metrics based on S/N ratio
    /// Enhanced with Concept Transformer Phase 9b insights
    fn calculate_quality(
        &self,
        content: &str,
        total_lines: usize,
        mode: ContextMode,
        collected_nodes: &[&CodeNode],
        entry_point_lines: usize,
    ) -> ContextQuality {
        // Estimate tokens (rough: ~4 chars per token for code)
        let estimated_tokens = content.len() / 4;

        // Count short identifiers (1-2 chars) vs meaningful names (3+ chars)
        // This is based on Concept Transformer's finding that short variables are "noise"
        let mut short_vars = 0;
        let mut meaningful_names = 0;

        // Concept density tracking
        let mut type_definitions = 0;
        let mut total_definitions = 0;

        // Simple heuristic: split by non-alphanumeric and count
        for word in content.split(|c: char| !c.is_alphanumeric() && c != '_') {
            let word = word.trim();
            if word.is_empty() {
                continue;
            }
            // Skip keywords and common tokens
            if matches!(word, "fn" | "let" | "mut" | "pub" | "struct" | "impl" | "if" | "else"
                | "for" | "while" | "return" | "use" | "const" | "type" | "self" | "Self"
                | "true" | "false" | "function" | "var" | "export" | "import") {
                continue;
            }
            if word.len() <= 2 {
                short_vars += 1;
            } else {
                meaningful_names += 1;
            }
        }

        // Calculate concept density from node types
        // High density: type definitions, interfaces, structs, traits
        // Low density: simple functions, getters/setters
        for node in collected_nodes {
            total_definitions += 1;
            match node.node_type {
                NodeType::Class => type_definitions += 2, // Classes are high-value
                NodeType::Function | NodeType::Method => {
                    // Check signature for high-density patterns
                    let sig = node.signature.to_lowercase();
                    if sig.contains("trait ") || sig.contains("interface ")
                       || sig.contains("impl ") || sig.contains("struct ")
                       || sig.contains("enum ") || sig.contains("type ") {
                        type_definitions += 2;
                    } else if sig.contains("get") || sig.contains("set")
                              || sig.contains("is_") || sig.contains("has_") {
                        // Low density - getter/setter patterns
                        type_definitions += 0;
                    } else {
                        type_definitions += 1;
                    }
                }
                _ => type_definitions += 1,
            }
        }

        let concept_density = if total_definitions > 0 {
            (type_definitions as f32 / (total_definitions * 2) as f32).min(1.0)
        } else {
            0.5
        };

        // Calculate S/N ratio
        let sn_ratio = if short_vars > 0 {
            meaningful_names as f32 / short_vars as f32
        } else if meaningful_names > 0 {
            10.0 // Perfect signal
        } else {
            1.0 // Neutral
        };

        // Dependency count and context explosion warning
        // Based on Concept Transformer finding: >100 tokens of context degrades S/N significantly
        let dependency_count = collected_nodes.len();
        let context_explosion_warning = dependency_count > 100 || estimated_tokens > 4000;

        // Entry point ratio: signal (entry point) / noise (dependencies)
        // Higher is better - indicates focused context
        let entry_point_ratio = if total_lines > 0 && entry_point_lines > 0 {
            entry_point_lines as f32 / total_lines as f32
        } else {
            0.0
        };

        // Determine quality level and recommendation
        let (quality_level, mut recommendation) = if sn_ratio >= 2.0 {
            ("high".to_string(), None)
        } else if sn_ratio >= 1.0 {
            ("medium".to_string(),
             if estimated_tokens > 2000 {
                 Some("Consider using skeleton mode to reduce context size".to_string())
             } else {
                 None
             })
        } else {
            let rec = match mode {
                ContextMode::Full if total_lines > 100 =>
                    Some("Low S/N ratio detected. Consider: 1) Use skeleton mode, 2) Reduce depth, 3) Use more specific entry point".to_string()),
                _ =>
                    Some("Low S/N ratio - context contains many short variable names".to_string()),
            };
            ("low".to_string(), rec)
        };

        // Add context explosion warning to recommendation
        if context_explosion_warning {
            let warning = format!(
                "⚠️ Context explosion detected ({} deps, ~{} tokens). Consider: reduce depth or use skeleton mode",
                dependency_count, estimated_tokens
            );
            recommendation = Some(match recommendation {
                Some(existing) => format!("{}. {}", existing, warning),
                None => warning,
            });
        }

        // Add low entry point ratio warning
        if entry_point_ratio < 0.1 && dependency_count > 10 {
            let warning = "Low signal ratio: entry point code is <10% of context. Consider more specific entry point";
            recommendation = Some(match recommendation {
                Some(existing) => format!("{}. {}", existing, warning),
                None => warning.to_string(),
            });
        }

        ContextQuality {
            estimated_tokens,
            sn_ratio,
            quality_level,
            recommendation,
            concept_density,
            dependency_count,
            context_explosion_warning,
            entry_point_ratio,
        }
    }

    /// Collect dependencies recursively
    fn collect_dependencies<'a>(
        &'a self,
        node: &'a CodeNode,
        depth: usize,
        collected: &mut Vec<&'a CodeNode>,
        visited: &mut HashSet<NodeId>,
    ) {
        if visited.contains(&node.id) || depth == 0 {
            return;
        }
        visited.insert(node.id);
        collected.push(node);

        if depth > 0 {
            // Find edges from this node
            for edge in &self.graph.edges {
                if edge.from == node.id {
                    if let Some(target) = self.graph.nodes.get(&edge.to) {
                        self.collect_dependencies(target, depth - 1, collected, visited);
                    }
                }
            }
        }
    }

    /// Calculate importance score for a node (higher = more important)
    /// Based on Concept Transformer insights: type definitions > core logic > getters/setters
    fn calculate_node_importance(&self, node: &CodeNode, is_entry_point: bool) -> i32 {
        let mut score = 0;

        // Entry points are always most important
        if is_entry_point {
            score += 1000;
        }

        // Node type scoring
        match node.node_type {
            NodeType::Class => score += 100,  // Classes/structs are high-value
            NodeType::Function => score += 50,
            NodeType::Method => score += 40,
            NodeType::Variable => score += 10,
        }

        // Signature-based scoring (concept density heuristics)
        let sig = node.signature.to_lowercase();

        // High-density patterns
        if sig.contains("trait ") || sig.contains("interface ") {
            score += 80;
        }
        if sig.contains("impl ") {
            score += 60;
        }
        if sig.contains("struct ") || sig.contains("enum ") || sig.contains("type ") {
            score += 70;
        }
        if sig.contains("pub ") {
            score += 20;  // Exported items are more important
        }

        // Low-density patterns (getters/setters/boilerplate)
        if sig.contains("get_") || sig.contains("set_") || sig.starts_with("get") || sig.starts_with("set") {
            score -= 30;
        }
        if sig.contains("is_") || sig.contains("has_") {
            score -= 20;
        }
        if node.name == "new" || node.name == "default" || node.name == "clone" {
            score -= 40;  // Common constructors are less unique
        }

        // Penalty for very short functions (likely trivial)
        let line_count = node.line_range.1.saturating_sub(node.line_range.0);
        if line_count <= 3 {
            score -= 10;
        } else if line_count > 50 {
            score += 30;  // Longer functions contain more logic
        }

        score
    }

    /// Sort collected nodes by importance for better context quality
    fn sort_by_importance<'a>(&self, nodes: &mut Vec<&'a CodeNode>, entry_point_ids: &HashSet<NodeId>) {
        nodes.sort_by(|a, b| {
            let score_a = self.calculate_node_importance(a, entry_point_ids.contains(&a.id));
            let score_b = self.calculate_node_importance(b, entry_point_ids.contains(&b.id));
            score_b.cmp(&score_a)  // Descending order
        });
    }

    /// Format collected nodes into context string
    fn format_context(
        &self,
        nodes: &[&CodeNode],
        mode: ContextMode,
        format: ContextFormat,
        entry_point: Option<&str>,
        query: Option<&str>,
    ) -> String {
        match format {
            ContextFormat::Standard => self.format_standard(nodes, mode, entry_point, query),
            ContextFormat::LlmEdit => self.format_llm_edit(nodes, mode, entry_point, query),
        }
    }

    /// Standard format with comments
    fn format_standard(
        &self,
        nodes: &[&CodeNode],
        mode: ContextMode,
        entry_point: Option<&str>,
        query: Option<&str>,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str("// ===== CONTEXT FILE =====\n");
        output.push_str("// Generated by index-chan\n");
        if let Some(entry) = entry_point {
            output.push_str(&format!("// Entry point: {}\n", entry));
        }
        if let Some(q) = query {
            output.push_str(&format!("// Query: {}\n", q));
        }

        // Group by file
        let mut by_file: HashMap<&std::path::PathBuf, Vec<&CodeNode>> = HashMap::new();
        for node in nodes {
            by_file.entry(&node.file_path).or_default().push(*node);
        }

        output.push_str(&format!("// Files: {}, Functions: {}\n\n", by_file.len(), nodes.len()));

        // Output each file
        for (file_path, file_nodes) in by_file {
            output.push_str(&format!("// ===== FILE: {} =====\n", file_path.display()));

            for node in file_nodes {
                output.push_str(&format!("// Lines: {}-{}\n\n", node.line_range.0, node.line_range.1));

                match mode {
                    ContextMode::Full => {
                        // Read actual code from file
                        if let Ok(code) = self.read_code_range(file_path, node.line_range) {
                            output.push_str(&code);
                        } else {
                            output.push_str(&format!("// {} {:?}\n", node.name, node.node_type));
                        }
                    }
                    ContextMode::Skeleton => {
                        // Output signature if available, otherwise just name
                        if !node.signature.is_empty() {
                            output.push_str(&node.signature);
                            output.push_str("\n");
                        } else {
                            output.push_str(&format!("{} {:?}\n", node.name, node.node_type));
                        }
                    }
                }
                output.push_str("\n");
            }
        }

        output.push_str("// ===== END CONTEXT =====\n");
        output
    }

    /// LLM-optimized format for editing
    /// Format:
    /// <<<FILE: path/to/file.rs:10-50>>>
    /// [code content]
    /// <<<END FILE>>>
    fn format_llm_edit(
        &self,
        nodes: &[&CodeNode],
        mode: ContextMode,
        entry_point: Option<&str>,
        query: Option<&str>,
    ) -> String {
        let mut output = String::new();

        // Compact header
        output.push_str("# CONTEXT FOR LLM EDITING\n");
        if let Some(entry) = entry_point {
            output.push_str(&format!("# Entry: {}\n", entry));
        }
        if let Some(q) = query {
            output.push_str(&format!("# Query: {}\n", q));
        }
        output.push_str("#\n");
        output.push_str("# Instructions: Edit the code blocks below. Keep the <<<FILE>>> markers intact.\n");
        output.push_str("# The markers contain file path and line numbers for applying changes.\n\n");

        // Group by file and sort nodes by line number within each file
        let mut by_file: HashMap<&std::path::PathBuf, Vec<&CodeNode>> = HashMap::new();
        for node in nodes {
            by_file.entry(&node.file_path).or_default().push(*node);
        }

        // Sort nodes within each file by line number
        for file_nodes in by_file.values_mut() {
            file_nodes.sort_by_key(|n| n.line_range.0);
        }

        // Output each file with LLM-friendly markers
        for (file_path, file_nodes) in by_file {
            for node in file_nodes {
                // Use <<< >>> markers that are easy for LLM to parse
                output.push_str(&format!(
                    "<<<FILE: {}:{}-{}>>>\n",
                    file_path.display(),
                    node.line_range.0,
                    node.line_range.1
                ));

                match mode {
                    ContextMode::Full => {
                        if let Ok(code) = self.read_code_range(file_path, node.line_range) {
                            output.push_str(&code);
                            output.push_str("\n");
                        } else {
                            output.push_str(&format!("// {} {:?}\n", node.name, node.node_type));
                        }
                    }
                    ContextMode::Skeleton => {
                        if !node.signature.is_empty() {
                            output.push_str(&node.signature);
                            output.push_str("\n");
                        } else {
                            output.push_str(&format!("{} {:?}\n", node.name, node.node_type));
                        }
                    }
                }

                output.push_str("<<<END FILE>>>\n\n");
            }
        }

        output
    }

    /// Read code from file at specific line range
    fn read_code_range(&self, file_path: &Path, range: (usize, usize)) -> Result<String> {
        let content = std::fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = range.0.saturating_sub(1);
        let end = range.1.min(lines.len());

        Ok(lines[start..end].join("\n"))
    }

    /// Find a node by qualified name
    /// Supports formats:
    /// - "function_name" - matches by function name only
    /// - "file.rs::function_name" - matches by file name and function name
    /// - "Type::function_name" - matches by signature containing "Type::" or "impl Type"
    fn find_node_by_qualified_name(&self, qualified_name: &str) -> Option<&CodeNode> {
        if let Some((qualifier, func_name)) = qualified_name.split_once("::") {
            // Qualified search
            self.graph.nodes.values().find(|n| {
                if n.name != func_name {
                    return false;
                }
                // Check if qualifier matches file name
                let file_name = n.file_path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("");
                if file_name == qualifier || file_name.starts_with(&format!("{}.", qualifier)) {
                    return true;
                }
                // Check if qualifier matches type in signature (e.g., "impl Type" or "Type::")
                if n.signature.contains(&format!("impl {}", qualifier))
                    || n.signature.contains(&format!("{}::", qualifier)) {
                    return true;
                }
                false
            })
        } else {
            // Simple name search
            self.graph.nodes.values().find(|n| n.name == qualified_name)
        }
    }

    /// Get dependencies of a function
    pub fn get_dependencies(&self, function_name: &str, depth: usize) -> Vec<DependencyInfo> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut seen_results = HashSet::new();

        // Find the function using qualified name search
        let start_node = self.find_node_by_qualified_name(function_name);

        if let Some(node) = start_node {
            self.collect_dependency_info(node, depth, &mut result, &mut visited, &mut seen_results, true);
        }

        result
    }

    /// Get dependents (reverse dependencies) of a function
    pub fn get_dependents(&self, function_name: &str, depth: usize) -> Vec<DependencyInfo> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut seen_results = HashSet::new();

        // Find the function using qualified name search
        let start_node = self.find_node_by_qualified_name(function_name);

        if let Some(node) = start_node {
            self.collect_dependency_info(node, depth, &mut result, &mut visited, &mut seen_results, false);
        }

        result
    }

    fn collect_dependency_info<'a>(
        &'a self,
        node: &'a CodeNode,
        depth: usize,
        result: &mut Vec<DependencyInfo>,
        visited: &mut HashSet<NodeId>,
        seen_results: &mut HashSet<(String, String, usize)>, // (name, file_path, line) for dedup
        forward: bool, // true = dependencies, false = dependents
    ) {
        // depth == 0 は「このノードからの依存を取得しない」を意味する
        // visited チェックは重複防止のため
        if visited.contains(&node.id) {
            return;
        }
        visited.insert(node.id);

        // depth が 0 より大きい場合のみエッジを辿る
        if depth == 0 {
            return;
        }

        for edge in &self.graph.edges {
            let (source, target) = if forward {
                (&edge.from, &edge.to)
            } else {
                (&edge.to, &edge.from)
            };

            if source == &node.id {
                if let Some(target_node) = self.graph.nodes.get(target) {
                    // 重複チェック: 同じ(name, file_path, line)は追加しない
                    let key = (
                        target_node.name.clone(),
                        target_node.file_path.display().to_string(),
                        target_node.line_range.0,
                    );
                    if !seen_results.contains(&key) {
                        seen_results.insert(key);
                        // 依存先のノードを結果に追加
                        result.push(DependencyInfo {
                            name: target_node.name.clone(),
                            file_path: target_node.file_path.display().to_string(),
                            line: target_node.line_range.0,
                            node_type: format!("{:?}", target_node.node_type),
                        });
                    }

                    // 再帰的に深堀りする (depth を減らして)
                    if depth > 1 {
                        self.collect_dependency_info(target_node, depth - 1, result, visited, seen_results, forward);
                    }
                }
            }
        }
    }
}

/// Dependency information
#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyInfo {
    pub name: String,
    pub file_path: String,
    pub line: usize,
    pub node_type: String,
}

/// Call chain step
#[derive(Debug, Clone, serde::Serialize)]
pub struct CallChainStep {
    pub name: String,
    pub file_path: String,
    pub line: usize,
    pub node_type: String,
}

/// Call chain result
#[derive(Debug, Clone, serde::Serialize)]
pub struct CallChainResult {
    pub from: String,
    pub to: String,
    pub chain: Vec<CallChainStep>,
    pub found: bool,
}

impl ContextGenerator {
    /// Get call chain from one function to another (BFS)
    pub fn get_call_chain(&self, from: &str, to: &str, max_depth: usize) -> CallChainResult {
        use std::collections::VecDeque;

        let from_node = self.find_node_by_qualified_name(from);
        let to_node = self.find_node_by_qualified_name(to);

        if from_node.is_none() || to_node.is_none() {
            return CallChainResult {
                from: from.to_string(),
                to: to.to_string(),
                chain: vec![],
                found: false,
            };
        }

        let from_node = from_node.unwrap();
        let to_node = to_node.unwrap();
        let to_id = to_node.id;

        // BFS to find shortest path
        let mut queue: VecDeque<(NodeId, Vec<NodeId>)> = VecDeque::new();
        let mut visited: HashSet<NodeId> = HashSet::new();

        queue.push_back((from_node.id, vec![from_node.id]));
        visited.insert(from_node.id);

        while let Some((current_id, path)) = queue.pop_front() {
            if path.len() > max_depth + 1 {
                continue;
            }

            if current_id == to_id {
                // Found path, convert to CallChainStep
                let chain: Vec<CallChainStep> = path.iter()
                    .filter_map(|id| self.graph.nodes.get(id))
                    .map(|node| CallChainStep {
                        name: node.name.clone(),
                        file_path: node.file_path.display().to_string(),
                        line: node.line_range.0,
                        node_type: format!("{:?}", node.node_type),
                    })
                    .collect();

                return CallChainResult {
                    from: from.to_string(),
                    to: to.to_string(),
                    chain,
                    found: true,
                };
            }

            // Find edges from current node
            for edge in &self.graph.edges {
                if edge.from == current_id && !visited.contains(&edge.to) {
                    visited.insert(edge.to);
                    let mut new_path = path.clone();
                    new_path.push(edge.to);
                    queue.push_back((edge.to, new_path));
                }
            }
        }

        CallChainResult {
            from: from.to_string(),
            to: to.to_string(),
            chain: vec![],
            found: false,
        }
    }

    /// Get all call paths from a function (call tree visualization)
    pub fn get_call_tree(&self, function_name: &str, depth: usize, direction: &str) -> Vec<CallTreeNode> {
        let start_node = self.find_node_by_qualified_name(function_name);

        if start_node.is_none() {
            return vec![];
        }

        let start_node = start_node.unwrap();
        let forward = direction == "callee" || direction == "down";

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        self.build_call_tree(start_node, depth, forward, 0, &mut result, &mut visited);

        result
    }

    fn build_call_tree(
        &self,
        node: &CodeNode,
        max_depth: usize,
        forward: bool,
        current_depth: usize,
        result: &mut Vec<CallTreeNode>,
        visited: &mut HashSet<NodeId>,
    ) {
        if visited.contains(&node.id) || current_depth > max_depth {
            return;
        }
        visited.insert(node.id);

        result.push(CallTreeNode {
            name: node.name.clone(),
            file_path: node.file_path.display().to_string(),
            line: node.line_range.0,
            depth: current_depth,
            node_type: format!("{:?}", node.node_type),
        });

        for edge in &self.graph.edges {
            let (source, target) = if forward {
                (&edge.from, &edge.to)
            } else {
                (&edge.to, &edge.from)
            };

            if source == &node.id {
                if let Some(next_node) = self.graph.nodes.get(target) {
                    self.build_call_tree(next_node, max_depth, forward, current_depth + 1, result, visited);
                }
            }
        }
    }
}

/// Call tree node
#[derive(Debug, Clone, serde::Serialize)]
pub struct CallTreeNode {
    pub name: String,
    pub file_path: String,
    pub line: usize,
    pub depth: usize,
    pub node_type: String,
}

/// Parsed file change from LLM output
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedFileChange {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub modified_content: String,
}

/// Parse LLM-edited content back into file changes
/// Expects format:
/// <<<FILE: path/to/file.rs:10-50>>>
/// [edited code]
/// <<<END FILE>>>
pub fn parse_llm_edits(content: &str) -> Result<Vec<ParsedFileChange>> {
    use regex::Regex;

    let mut changes = Vec::new();

    // Match <<<FILE: path:start-end>>> blocks
    let file_re = Regex::new(r"<<<FILE:\s*([^:]+):(\d+)-(\d+)>>>\n([\s\S]*?)<<<END FILE>>>")?;

    for cap in file_re.captures_iter(content) {
        let file_path = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let start_line: usize = cap.get(2).map(|m| m.as_str().parse().unwrap_or(1)).unwrap_or(1);
        let end_line: usize = cap.get(3).map(|m| m.as_str().parse().unwrap_or(1)).unwrap_or(1);
        let code = cap.get(4).map(|m| m.as_str()).unwrap_or("");

        // Remove trailing newline if present
        let modified_content = code.trim_end_matches('\n').to_string();

        changes.push(ParsedFileChange {
            file_path: file_path.to_string(),
            start_line,
            end_line,
            modified_content,
        });
    }

    if changes.is_empty() {
        anyhow::bail!("No valid <<<FILE>>> blocks found in input");
    }

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CodeGraph, CodeNode, DependencyEdge, EdgeType, NodeType};
    use std::path::PathBuf;

    fn create_test_graph() -> CodeGraph {
        let mut graph = CodeGraph::new();

        // ノード追加: main -> scan_directory -> detect_dead_code
        let main_id = graph.add_node(CodeNode {
            id: 0,
            name: "main".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/main.rs"),
            line_range: (1, 100),
            is_exported: false,
            is_used: false,
            signature: "fn main()".to_string(),
        });

        let scan_id = graph.add_node(CodeNode {
            id: 0,
            name: "scan_directory".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/scanner.rs"),
            line_range: (10, 50),
            is_exported: true,
            is_used: false,
            signature: "pub fn scan_directory(&mut self, dir: &Path) -> Result<CodeGraph>".to_string(),
        });

        let detect_id = graph.add_node(CodeNode {
            id: 0,
            name: "detect_dead_code".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("src/detector.rs"),
            line_range: (20, 80),
            is_exported: true,
            is_used: false,
            signature: "pub fn detect_dead_code(graph: &CodeGraph) -> Vec<DeadCode>".to_string(),
        });

        // エッジ: main -> scan_directory
        graph.add_edge(DependencyEdge {
            from: main_id,
            to: scan_id,
            edge_type: EdgeType::Calls,
        });

        // エッジ: main -> detect_dead_code
        graph.add_edge(DependencyEdge {
            from: main_id,
            to: detect_id,
            edge_type: EdgeType::Calls,
        });

        // エッジ: scan_directory -> detect_dead_code (間接的)
        graph.add_edge(DependencyEdge {
            from: scan_id,
            to: detect_id,
            edge_type: EdgeType::Calls,
        });

        graph
    }

    #[test]
    fn test_get_dependencies() {
        let graph = create_test_graph();
        let generator = ContextGenerator::from_graph(graph);

        // main の依存関係を取得 (depth=1)
        let deps = generator.get_dependencies("main", 1);
        assert_eq!(deps.len(), 2, "main should have 2 direct dependencies");

        let dep_names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(dep_names.contains(&"scan_directory"));
        assert!(dep_names.contains(&"detect_dead_code"));
    }

    #[test]
    fn test_get_dependencies_depth2() {
        let graph = create_test_graph();
        let generator = ContextGenerator::from_graph(graph);

        // main の依存関係を取得 (depth=2)
        let deps = generator.get_dependencies("main", 2);
        // main -> scan_directory, detect_dead_code
        // scan_directory -> detect_dead_code (ただしこれは既にvisitedなのでスキップ)
        assert!(deps.len() >= 2, "main should have at least 2 dependencies with depth=2");
    }

    #[test]
    fn test_get_dependents() {
        let graph = create_test_graph();
        let generator = ContextGenerator::from_graph(graph);

        // detect_dead_code の逆依存関係を取得 (depth=1)
        let deps = generator.get_dependents("detect_dead_code", 1);
        assert_eq!(deps.len(), 2, "detect_dead_code should have 2 dependents (main and scan_directory)");

        let dep_names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(dep_names.contains(&"main"));
        assert!(dep_names.contains(&"scan_directory"));
    }

    #[test]
    fn test_get_dependencies_nonexistent() {
        let graph = create_test_graph();
        let generator = ContextGenerator::from_graph(graph);

        // 存在しない関数の依存関係
        let deps = generator.get_dependencies("nonexistent_function", 1);
        assert_eq!(deps.len(), 0, "nonexistent function should have no dependencies");
    }

    #[test]
    fn test_scanner_graph_edges() {
        // test_rust_projectをスキャンしてエッジとノードを確認
        use crate::scanner::Scanner;
        use std::path::Path;

        let test_dir = Path::new("test_rust_project");
        if !test_dir.exists() {
            eprintln!("test_rust_project does not exist, skipping test");
            return;
        }

        let mut scanner = Scanner::new().unwrap();
        let graph = scanner.scan_directory(test_dir).unwrap();

        eprintln!("=== Nodes ===");
        for (id, node) in &graph.nodes {
            eprintln!("  Node[{}]: {} (node.id={})", id, node.name, node.id);
            assert_eq!(*id, node.id, "HashMap key should match node.id");
        }

        eprintln!("=== Edges ===");
        for edge in &graph.edges {
            eprintln!("  Edge: {} -> {}", edge.from, edge.to);
        }

        // ノードIDとエッジのfrom/toが一致することを確認
        for edge in &graph.edges {
            if edge.from != usize::MAX {
                assert!(
                    graph.nodes.contains_key(&edge.from),
                    "Edge from={} should reference an existing node",
                    edge.from
                );
            }
            assert!(
                graph.nodes.contains_key(&edge.to),
                "Edge to={} should reference an existing node",
                edge.to
            );
        }

        // ContextGeneratorで依存関係を取得
        let generator = ContextGenerator::from_graph(graph);

        // another_used -> used_function の依存関係を確認
        let deps = generator.get_dependencies("another_used", 1);
        eprintln!("=== Dependencies of another_used ===");
        for dep in &deps {
            eprintln!("  -> {}", dep.name);
        }
        assert!(deps.len() > 0, "another_used should have dependencies");
        assert!(deps.iter().any(|d| d.name == "used_function"), "another_used should depend on used_function");

        // used_function -> helper_function の依存関係を確認
        let deps = generator.get_dependencies("used_function", 1);
        eprintln!("=== Dependencies of used_function ===");
        for dep in &deps {
            eprintln!("  -> {}", dep.name);
        }
        assert!(deps.len() > 0, "used_function should have dependencies");
        assert!(deps.iter().any(|d| d.name == "helper_function"), "used_function should depend on helper_function");
    }
}
