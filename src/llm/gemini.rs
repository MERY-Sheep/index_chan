// Gemini API integration with Function Calling support
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::function_calling::{Tool, FunctionCall};

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text { text: String },
    FunctionCall { function_call: FunctionCallPart },
    FunctionResponse { function_response: FunctionResponsePart },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallPart {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponsePart {
    pub name: String,
    pub response: Value,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}


#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePartRaw>,
}

#[derive(Debug, Deserialize)]
struct ResponsePartRaw {
    #[serde(default)]
    text: Option<String>,
    #[serde(default, rename = "functionCall")]
    function_call: Option<FunctionCallPart>,
}

/// Result from Gemini API call
#[derive(Debug)]
pub enum GeminiResult {
    Text(String),
    FunctionCall(FunctionCall),
}

pub struct GeminiClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Result<Self> {
        Ok(Self {
            api_key,
            model: "gemini-2.0-flash".to_string(),
            client: reqwest::Client::new(),
        })
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// Simple text generation (backward compatible)
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let contents = vec![Content {
            role: "user".to_string(),
            parts: vec![Part::Text { text: prompt.to_string() }],
        }];
        
        match self.generate_with_contents(contents, None).await? {
            GeminiResult::Text(text) => Ok(text),
            GeminiResult::FunctionCall(_) => {
                anyhow::bail!("Unexpected function call in simple generate")
            }
        }
    }

    /// Generate with function calling support
    pub async fn generate_with_tools(
        &self,
        contents: Vec<Content>,
        tools: Option<Vec<Tool>>,
    ) -> Result<GeminiResult> {
        self.generate_with_contents(contents, tools).await
    }

    /// Continue conversation after function response
    pub async fn continue_with_function_response(
        &self,
        mut contents: Vec<Content>,
        function_name: &str,
        response: Value,
        tools: Option<Vec<Tool>>,
    ) -> Result<GeminiResult> {
        contents.push(Content {
            role: "function".to_string(),
            parts: vec![Part::FunctionResponse {
                function_response: FunctionResponsePart {
                    name: function_name.to_string(),
                    response,
                },
            }],
        });
        
        self.generate_with_contents(contents, tools).await
    }


    /// Internal method for API calls
    async fn generate_with_contents(
        &self,
        contents: Vec<Content>,
        tools: Option<Vec<Tool>>,
    ) -> Result<GeminiResult> {
        use std::time::Instant;
        
        let start = Instant::now();
        let encoded_key = urlencoding::encode(&self.api_key);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model,
            encoded_key
        );
        
        println!("  🔍 Gemini API呼び出し (model: {})", self.model);

        let request = GeminiRequest {
            contents,
            generation_config: Some(GenerationConfig {
                temperature: 0.7,
                max_output_tokens: 2048,
            }),
            tools,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Gemini APIへのリクエストに失敗しました")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Gemini APIエラー ({}): {}\n\
                 💡 APIキーを確認してください: https://makersuite.google.com/app/apikey",
                status,
                error_text
            );
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Gemini APIのレスポンス解析に失敗しました")?;

        let part = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .context("Gemini APIのレスポンスが空です")?;

        let result = if let Some(fc) = &part.function_call {
            println!("  🔧 Function Call検出: {}", fc.name);
            GeminiResult::FunctionCall(FunctionCall {
                name: fc.name.clone(),
                args: fc.args.clone(),
            })
        } else if let Some(text) = &part.text {
            println!("  ✅ テキスト応答受信 ({}文字)", text.len());
            GeminiResult::Text(text.clone())
        } else {
            anyhow::bail!("Gemini APIのレスポンスが不正です")
        };

        println!("  ⏱️  合計時間: {:.2}秒", start.elapsed().as_secs_f64());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_gemini_client() {
        let api_key = std::env::var("GEMINI_API_KEY").unwrap();
        let client = GeminiClient::new(api_key).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(client.generate("Hello")).unwrap();
        assert!(!response.is_empty());
    }
}
