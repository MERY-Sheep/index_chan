use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

use crate::graph::{CodeGraph, CodeNode, DependencyEdge, EdgeType, NodeType};
use crate::parser::{CodeParser, Language};

/// File hash cache for incremental scanning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanCache {
    /// Map of file path -> content hash
    pub file_hashes: HashMap<String, String>,
    /// Cached graph (serializable form)
    pub cached_nodes: Vec<CachedNode>,
    /// Version for cache invalidation
    pub version: u32,
}

const CACHE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedNode {
    pub name: String,
    pub node_type: String,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub is_exported: bool,
    pub signature: String,
}

impl ScanCache {
    /// Load cache from file
    pub fn load(cache_path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(cache_path).ok()?;
        let cache: ScanCache = serde_json::from_str(&content).ok()?;
        if cache.version != CACHE_VERSION {
            return None; // Invalidate old cache
        }
        Some(cache)
    }

    /// Save cache to file
    pub fn save(&self, cache_path: &Path) -> Result<()> {
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(cache_path, content)?;
        Ok(())
    }

    /// Calculate file hash
    pub fn hash_file(path: &Path) -> Result<String> {
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check if file has changed
    pub fn is_file_changed(&self, path: &Path) -> bool {
        let path_str = path.display().to_string();
        match (self.file_hashes.get(&path_str), Self::hash_file(path)) {
            (Some(cached), Ok(current)) => cached != &current,
            _ => true, // If we can't determine, assume changed
        }
    }
}

pub struct Scanner {
    /// Enable incremental scanning
    pub incremental: bool,
}

impl Scanner {
    pub fn new() -> Result<Self> {
        Ok(Self { incremental: true })
    }

    /// Create scanner with specific incremental setting
    pub fn with_incremental(incremental: bool) -> Result<Self> {
        Ok(Self { incremental })
    }

    /// Get cache path for a directory
    fn cache_path(dir: &Path) -> PathBuf {
        dir.join(".index-chan").join("scan_cache.json")
    }

    pub fn scan_directory(&mut self, dir: &Path) -> Result<CodeGraph> {
        let cache_path = Self::cache_path(dir);
        let cache = if self.incremental {
            ScanCache::load(&cache_path)
        } else {
            None
        };

        let mut graph = CodeGraph::new();
        let mut new_cache = ScanCache {
            version: CACHE_VERSION,
            ..Default::default()
        };
        let mut file_count = 0;
        let mut cached_count = 0;
        let mut changed_count = 0;

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
            let file_changed = cache.as_ref()
                .map(|c| c.is_file_changed(file_path))
                .unwrap_or(true);

            if file_changed {
                // File changed, rescan
                if let Err(e) = self.scan_file(file_path, *language, &mut graph) {
                    eprintln!("⚠️  Failed to scan {}: {}", file_path.display(), e);
                } else {
                    file_count += 1;
                    changed_count += 1;
                }
            } else {
                // File unchanged, use cached nodes
                if let Some(ref c) = cache {
                    let path_str = file_path.display().to_string();
                    for cached_node in c.cached_nodes.iter().filter(|n| n.file_path == path_str) {
                        let node = CodeNode {
                            id: 0,
                            name: cached_node.name.clone(),
                            node_type: NodeType::Function, // Simplified
                            file_path: PathBuf::from(&cached_node.file_path),
                            line_range: cached_node.line_range,
                            is_exported: cached_node.is_exported,
                            is_used: false,
                            signature: cached_node.signature.clone(),
                        };
                        graph.add_node(node);
                    }
                    cached_count += 1;
                    file_count += 1;
                }
            }

            // Update hash cache
            if let Ok(hash) = ScanCache::hash_file(file_path) {
                new_cache.file_hashes.insert(file_path.display().to_string(), hash);
            }
        }

        if self.incremental && cache.is_some() {
            println!("✅ Scanned {} files ({} changed, {} cached)", file_count, changed_count, cached_count);
        } else {
            println!("✅ Scanned {} files (full scan)", file_count);
        }
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

        // Save cache for next scan
        if self.incremental {
            // Convert nodes to cached format
            for node in graph.nodes.values() {
                new_cache.cached_nodes.push(CachedNode {
                    name: node.name.clone(),
                    node_type: format!("{:?}", node.node_type),
                    file_path: node.file_path.display().to_string(),
                    line_range: node.line_range,
                    is_exported: node.is_exported,
                    signature: node.signature.clone(),
                });
            }
            if let Err(e) = new_cache.save(&cache_path) {
                eprintln!("⚠️  Failed to save scan cache: {}", e);
            }
        }

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
                signature: func.signature,
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
