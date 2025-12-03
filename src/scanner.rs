use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use ignore::WalkBuilder;

use crate::graph::{CodeGraph, CodeNode, DependencyEdge, EdgeType, NodeType};
use crate::parser::{CodeParser, Language};

pub struct Scanner {
    // No longer holds a single parser
}

impl Scanner {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn scan_directory(&mut self, dir: &Path) -> Result<CodeGraph> {
        let mut graph = CodeGraph::new();
        let mut file_count = 0;

        // Collect all supported files (TypeScript and Rust) using ignore crate
        let code_files: Vec<(PathBuf, Language)> = WalkBuilder::new(dir)
            .add_custom_ignore_filename(".indexchanignore")
            .git_ignore(true)      // .gitignoreも尊重
            .git_global(true)      // グローバル.gitignoreも
            .git_exclude(true)     // .git/info/excludeも
            .build()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if !path.is_file() {
                    return None;
                }
                let ext = path.extension()?.to_str()?;
                let lang = Language::from_extension(ext)?;
                Some((path.to_path_buf(), lang))
            })
            .collect();

        let ts_count = code_files.iter().filter(|(_, lang)| *lang == Language::TypeScript).count();
        let rs_count = code_files.iter().filter(|(_, lang)| *lang == Language::Rust).count();
        
        println!("📂 Found {} files (TypeScript: {}, Rust: {})", code_files.len(), ts_count, rs_count);

        // First pass: collect all function/class definitions
        for (file_path, language) in &code_files {
            if let Err(e) = self.scan_file(file_path, *language, &mut graph) {
                eprintln!("⚠️  Failed to scan {}: {}", file_path.display(), e);
            } else {
                file_count += 1;
            }
        }

        println!("✅ Scanned {} files", file_count);
        println!("📊 Found {} nodes", graph.nodes.len());

        // Second pass: build dependency edges
        for (file_path, language) in &code_files {
            if let Err(e) = self.build_dependencies(file_path, *language, &mut graph) {
                eprintln!(
                    "⚠️  Failed to build dependencies for {}: {}",
                    file_path.display(),
                    e
                );
            }
        }

        println!("🔗 Found {} edges", graph.edges.len());

        Ok(graph)
    }

    fn scan_file(&mut self, path: &Path, language: Language, graph: &mut CodeGraph) -> Result<()> {
        let source = std::fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let mut parser = CodeParser::new(language)?;
        let tree = parser
            .parse_file(path)
            .context("Failed to parse file")?;

        let functions = parser.extract_functions(&tree, &source);

        for func in functions {
            let node = CodeNode {
                id: 0, // Will be set by add_node
                name: func.name,
                node_type: NodeType::Function,
                file_path: path.to_path_buf(),
                line_range: func.line_range,
                is_exported: func.is_exported,
                is_used: false,
            };
            graph.add_node(node);
        }

        Ok(())
    }

    fn build_dependencies(&mut self, path: &Path, language: Language, graph: &mut CodeGraph) -> Result<()> {
        let source = std::fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let mut parser = CodeParser::new(language)?;
        let tree = parser
            .parse_file(path)
            .context("Failed to parse file")?;

        // Extract function calls
        let calls = parser.extract_calls(&tree, &source);

        // Find matching nodes and create edges
        for call in calls {
            // Find the caller node (if inside a function)
            let caller_id = self.find_node_at_line(graph, path, call.caller_line);

            // Find the callee node by name
            if let Some(callee_id) = self.find_node_by_name(graph, &call.callee_name) {
                if let Some(caller_id) = caller_id {
                    // Call from within a function
                    graph.add_edge(DependencyEdge {
                        from: caller_id,
                        to: callee_id,
                        edge_type: EdgeType::Calls,
                    });
                } else {
                    // Call from top-level (entry point)
                    // Create a dummy edge to mark the callee as used
                    graph.add_edge(DependencyEdge {
                        from: usize::MAX, // Special marker for top-level
                        to: callee_id,
                        edge_type: EdgeType::Calls,
                    });
                }
            }
        }

        Ok(())
    }

    fn find_node_at_line(&self, graph: &CodeGraph, path: &Path, line: usize) -> Option<usize> {
        graph.nodes.iter().find_map(|(id, node)| {
            if node.file_path == path && line >= node.line_range.0 && line <= node.line_range.1 {
                Some(*id)
            } else {
                None
            }
        })
    }

    fn find_node_by_name(&self, graph: &CodeGraph, name: &str) -> Option<usize> {
        graph
            .nodes
            .iter()
            .find_map(|(id, node)| if node.name == name { Some(*id) } else { None })
    }
}
