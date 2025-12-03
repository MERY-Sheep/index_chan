// Topic detection
use anyhow::Result;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::graph::{ConversationGraph, Topic};
use crate::llm::GeminiClient;

/// Word type for Japanese keyword extraction
#[derive(Debug, Clone, PartialEq)]
enum WordType {
    None,
    Kanji,
    Katakana,
}

/// Topic detection result from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicAnalysis {
    pub topics: Vec<TopicInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub name: String,
    pub keywords: Vec<String>,
    pub message_indices: Vec<usize>,
    pub summary: String,
}

/// Topic detector
pub struct TopicDetector {
    min_cluster_size: usize,
    gemini_client: Option<GeminiClient>,
}

impl TopicDetector {
    /// Create a new topic detector
    pub fn new() -> Self {
        Self {
            min_cluster_size: 2, // Lower threshold for small conversations
            gemini_client: None,
        }
    }

    /// Create a new topic detector with Gemini support
    pub fn with_gemini(api_key: String) -> Result<Self> {
        let gemini_client = GeminiClient::new(api_key).ok();
        
        Ok(Self {
            min_cluster_size: 2,
            gemini_client,
        })
    }

    /// Detect topics in conversation graph
    pub async fn detect_topics(&self, graph: &mut ConversationGraph) -> Result<()> {
        if self.gemini_client.is_some() {
            self.detect_topics_with_gemini(graph).await
        } else {
            self.detect_topics_keyword_based(graph)
        }
    }

    /// Detect topics using Gemini
    async fn detect_topics_with_gemini(&self, graph: &mut ConversationGraph) -> Result<()> {
        let prompt = self.build_topic_prompt(graph);
        
        let response = self.gemini_client.as_ref().unwrap().generate(&prompt).await?;
        
        let analysis = self.parse_topic_response(&response)?;
        
        // Create topics from Gemini analysis
        for (topic_id, topic_info) in analysis.topics.iter().enumerate() {
            let message_ids: Vec<String> = topic_info.message_indices
                .iter()
                .map(|i| format!("{}", i))
                .collect();
            
            let topic = Topic {
                id: format!("topic_{}", topic_id),
                name: topic_info.name.clone(),
                keywords: topic_info.keywords.clone(),
                message_ids: message_ids.clone(),
            };
            
            // Assign topic to nodes
            for msg_id in &message_ids {
                if let Some(node) = graph.nodes.iter_mut().find(|n| &n.id == msg_id) {
                    node.topic_id = Some(topic.id.clone());
                }
            }
            
            graph.add_topic(topic);
        }
        
        Ok(())
    }

    /// Detect topics using keyword-based approach (fallback)
    fn detect_topics_keyword_based(&self, graph: &mut ConversationGraph) -> Result<()> {
        let mut keyword_groups: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            let keywords = self.extract_keywords(&node.content);

            for keyword in keywords {
                keyword_groups
                    .entry(keyword.clone())
                    .or_insert_with(Vec::new)
                    .push(node.id.clone());
            }
        }

        // Create topics from keyword groups
        let mut topic_id = 0;
        for (keyword, message_ids) in keyword_groups {
            if message_ids.len() >= self.min_cluster_size {
                let topic = Topic {
                    id: format!("topic_{}", topic_id),
                    name: keyword.clone(),
                    keywords: vec![keyword],
                    message_ids: message_ids.clone(),
                };

                // Assign topic to nodes
                for msg_id in &message_ids {
                    if let Some(node) = graph.nodes.iter_mut().find(|n| &n.id == msg_id) {
                        node.topic_id = Some(topic.id.clone());
                    }
                }

                graph.add_topic(topic);
                topic_id += 1;
            }
        }

        Ok(())
    }

    /// Build prompt for LLM topic detection
    fn build_topic_prompt(&self, graph: &ConversationGraph) -> String {
        let mut messages_text = String::new();
        
        for (i, node) in graph.nodes.iter().enumerate() {
            messages_text.push_str(&format!(
                "[{}] {}: {}\n",
                i, node.role, node.content
            ));
        }
        
        format!(
            r#"You are a conversation analyst. Analyze the following chat history and identify distinct topics.

Chat History:
{}

Analyze the conversation and group messages by topic. Consider:
1. What are the main themes discussed?
2. Which messages belong to the same topic?
3. What keywords represent each topic?
4. Provide a brief summary for each topic

Respond ONLY with valid JSON (no markdown, no extra text):
{{
  "topics": [
    {{
      "name": "Topic name",
      "keywords": ["keyword1", "keyword2"],
      "message_indices": [0, 1, 5],
      "summary": "Brief summary of this topic"
    }}
  ]
}}
"#,
            messages_text
        )
    }

    /// Parse LLM response for topic detection
    fn parse_topic_response(&self, response: &str) -> Result<TopicAnalysis> {
        // Extract JSON from response
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };
        
        serde_json::from_str::<TopicAnalysis>(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse topic response: {}", e))
    }

    /// Extract keywords from text (supports English and Japanese)
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // English stop words
        let stop_words_en = vec!["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "is", "are", "was", "were", "for", "with"];
        
        // Japanese stop words (common particles and auxiliary verbs)
        let stop_words_ja = vec!["の", "に", "は", "を", "が", "で", "と", "から", "まで", "より", "も", "ね", "よ", "な", "だ", "です", "ます", "した", "する", "ある", "いる", "なる", "お", "ご"];

        let mut keywords = Vec::new();
        
        // For Japanese text, try to extract meaningful words using simple heuristics
        let has_japanese = text.chars().any(|c| {
            ('\u{3040}'..='\u{309F}').contains(&c) || // Hiragana
            ('\u{30A0}'..='\u{30FF}').contains(&c) || // Katakana
            ('\u{4E00}'..='\u{9FFF}').contains(&c)    // Kanji
        });
        
        if has_japanese {
            // Simple Japanese keyword extraction: extract Kanji sequences and Katakana words
            let mut current_word = String::new();
            let mut word_type = WordType::None;
            
            for c in text.chars() {
                if ('\u{4E00}'..='\u{9FFF}').contains(&c) {
                    // Kanji
                    if word_type == WordType::Kanji {
                        current_word.push(c);
                    } else {
                        // Save previous word
                        if !current_word.is_empty() && current_word.chars().count() >= 2 {
                            let word = current_word.trim().to_string();
                            if !stop_words_ja.contains(&word.as_str()) {
                                keywords.push(word);
                            }
                        }
                        current_word.clear();
                        current_word.push(c);
                        word_type = WordType::Kanji;
                    }
                } else if ('\u{30A0}'..='\u{30FF}').contains(&c) {
                    // Katakana
                    if word_type == WordType::Katakana {
                        current_word.push(c);
                    } else {
                        // Save previous word
                        if !current_word.is_empty() && current_word.chars().count() >= 2 {
                            let word = current_word.trim().to_string();
                            if !stop_words_ja.contains(&word.as_str()) {
                                keywords.push(word);
                            }
                        }
                        current_word.clear();
                        current_word.push(c);
                        word_type = WordType::Katakana;
                    }
                } else {
                    // End of word (Hiragana, punctuation, etc.)
                    if !current_word.is_empty() && current_word.chars().count() >= 2 {
                        let word = current_word.trim().to_string();
                        if !stop_words_ja.contains(&word.as_str()) {
                            keywords.push(word);
                        }
                    }
                    current_word.clear();
                    word_type = WordType::None;
                }
            }
            
            // Don't forget the last word
            if !current_word.is_empty() && current_word.chars().count() >= 2 {
                let word = current_word.trim().to_string();
                if !stop_words_ja.contains(&word.as_str()) {
                    keywords.push(word);
                }
            }
        } else {
            // English text: split by whitespace and punctuation
            for word in text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
                let word_lower = word.to_lowercase();
                
                if word.len() >= 3 && !stop_words_en.contains(&word_lower.as_str()) {
                    keywords.push(word_lower);
                }
            }
        }
        
        keywords
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_detector() {
        let detector = TopicDetector::new();
        let keywords = detector.extract_keywords("This is a test function");
        assert!(keywords.contains(&"test".to_string()));
        assert!(keywords.contains(&"function".to_string()));
    }
}
