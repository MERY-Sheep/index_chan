// Search query builder

/// Search query configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchQuery {
    pub text: String,
    pub top_k: usize,
    pub min_score: f32,
    pub include_context: bool,
}

impl SearchQuery {
    /// Create a new search query
    #[allow(dead_code)]
    pub fn new(text: String) -> Self {
        Self {
            text,
            top_k: 10,
            min_score: 0.0,
            include_context: false,
        }
    }

    /// Set top k results
    #[allow(dead_code)]
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set minimum score threshold
    #[allow(dead_code)]
    pub fn with_min_score(mut self, min_score: f32) -> Self {
        self.min_score = min_score;
        self
    }

    /// Include context in results
    #[allow(dead_code)]
    pub fn with_context(mut self, include_context: bool) -> Self {
        self.include_context = include_context;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query() {
        let query = SearchQuery::new("test".to_string())
            .with_top_k(5)
            .with_min_score(0.5)
            .with_context(true);
        
        assert_eq!(query.text, "test");
        assert_eq!(query.top_k, 5);
        assert_eq!(query.min_score, 0.5);
        assert_eq!(query.include_context, true);
    }
}
