// Code index for simple text search
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata for indexed code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub file_path: PathBuf,
    pub function_name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub code_snippet: String,
    pub dependencies: Vec<String>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub metadata: CodeMetadata,
    pub score: f32,
}

/// Code index for simple text search
pub struct CodeIndex {
    metadata: Vec<CodeMetadata>,
}

impl CodeIndex {
    /// Create a new code index
    pub fn new() -> Result<Self> {
        Ok(Self {
            metadata: Vec::new(),
        })
    }

    /// Add code to the index
    pub fn add(&mut self, metadata: CodeMetadata) -> Result<()> {
        self.metadata.push(metadata);
        Ok(())
    }

    /// Add multiple codes to the index
    pub fn add_batch(&mut self, metadata_list: Vec<CodeMetadata>) -> Result<()> {
        for metadata in metadata_list {
            self.add(metadata)?;
        }
        Ok(())
    }

    /// Search for similar code using simple text matching
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<(usize, f32)> = self
            .metadata
            .iter()
            .enumerate()
            .map(|(i, meta)| {
                let text = format!(
                    "{} {} {}",
                    meta.function_name,
                    meta.code_snippet,
                    meta.dependencies.join(" ")
                ).to_lowercase();
                
                // Simple text matching score
                let score = if text.contains(&query_lower) {
                    1.0
                } else {
                    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
                    let matches = query_words.iter()
                        .filter(|qw| text.contains(*qw))
                        .count();
                    
                    if query_words.is_empty() {
                        0.0
                    } else {
                        matches as f32 / query_words.len() as f32
                    }
                };
                
                (i, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top k
        let top_results: Vec<SearchResult> = results
            .iter()
            .take(top_k)
            .map(|(i, score)| SearchResult {
                metadata: self.metadata[*i].clone(),
                score: *score,
            })
            .collect();
        
        Ok(top_results)
    }

    /// Get total number of indexed items
    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    /// Check if index is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    /// Save index to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.metadata)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Load index from file
    pub fn load(&mut self, path: &PathBuf) -> Result<()> {
        let data = std::fs::read_to_string(path)?;
        let metadata_list: Vec<CodeMetadata> = serde_json::from_str(&data)?;
        
        self.metadata.clear();
        
        self.add_batch(metadata_list)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_index() {
        let mut index = CodeIndex::new().unwrap();
        
        let metadata = CodeMetadata {
            file_path: PathBuf::from("test.ts"),
            function_name: "testFunction".to_string(),
            start_line: 1,
            end_line: 10,
            code_snippet: "function testFunction() { return 42; }".to_string(),
            dependencies: vec![],
        };
        
        index.add(metadata).unwrap();
        assert_eq!(index.len(), 1);
        
        let results = index.search("test function", 5).unwrap();
        assert_eq!(results.len(), 1);
    }
}
