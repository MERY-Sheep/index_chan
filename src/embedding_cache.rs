// Embedding cache for semantic search
// Pre-computes and stores embeddings for code nodes

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::graph::{CodeGraph, NodeId};

#[cfg(feature = "semantic-search")]
use crate::embedding::EmbeddingGenerator;

/// Cached embeddings for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCache {
    /// Project directory path
    pub project_path: PathBuf,
    /// Node ID -> embedding vector
    pub embeddings: HashMap<NodeId, Vec<f32>>,
    /// Node ID -> text that was embedded (for debugging)
    pub node_texts: HashMap<NodeId, String>,
    /// Cache version
    pub version: String,
}

impl EmbeddingCache {
    /// Create a new empty cache
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            embeddings: HashMap::new(),
            node_texts: HashMap::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Build embeddings for all nodes in the graph
    #[cfg(feature = "semantic-search")]
    pub fn build_from_graph(graph: &CodeGraph, project_path: &Path) -> Result<Self> {
        let mut cache = Self::new(project_path);

        eprintln!("Building embeddings for {} nodes...", graph.nodes.len());

        let generator = EmbeddingGenerator::new()?;

        let mut count = 0;
        for (&node_id, node) in &graph.nodes {
            // Create text representation for embedding
            let text = format!(
                "{} {} {}",
                node.name,
                node.file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(""),
                format!("{:?}", node.node_type).to_lowercase()
            );

            match generator.embed(&text) {
                Ok(embedding) => {
                    cache.embeddings.insert(node_id, embedding);
                    cache.node_texts.insert(node_id, text);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to embed node {}: {}", node.name, e);
                }
            }

            // Progress indicator
            if count % 50 == 0 {
                eprintln!("Progress: {}/{} nodes", count, graph.nodes.len());
            }
        }

        eprintln!("Completed: {} embeddings generated", count);

        Ok(cache)
    }

    /// Stub for when semantic-search feature is disabled
    #[cfg(not(feature = "semantic-search"))]
    pub fn build_from_graph(_graph: &CodeGraph, project_path: &Path) -> Result<Self> {
        Ok(Self::new(project_path))
    }

    /// Get cache file path for a project
    pub fn cache_path(project_path: &Path) -> PathBuf {
        project_path.join(".index-chan").join("embeddings.json")
    }

    /// Save cache to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load cache from disk
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let cache: Self = serde_json::from_str(&json)?;
        Ok(cache)
    }

    /// Check if cache exists and is valid for a project
    pub fn is_valid(project_path: &Path) -> bool {
        let cache_path = Self::cache_path(project_path);
        cache_path.exists()
    }

    /// Get or create cache for a project
    #[cfg(feature = "semantic-search")]
    pub fn get_or_create(graph: &CodeGraph, project_path: &Path) -> Result<Self> {
        let cache_path = Self::cache_path(project_path);

        if cache_path.exists() {
            match Self::load(&cache_path) {
                Ok(cache) => {
                    // Check if cache has all nodes
                    if cache.embeddings.len() >= graph.nodes.len() / 2 {
                        eprintln!("Using cached embeddings: {} nodes", cache.embeddings.len());
                        return Ok(cache);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load cache: {}", e);
                }
            }
        }

        // Build new cache
        let cache = Self::build_from_graph(graph, project_path)?;
        cache.save(&cache_path)?;
        Ok(cache)
    }

    /// Stub for when semantic-search feature is disabled
    #[cfg(not(feature = "semantic-search"))]
    pub fn get_or_create(_graph: &CodeGraph, project_path: &Path) -> Result<Self> {
        Ok(Self::new(project_path))
    }
}
