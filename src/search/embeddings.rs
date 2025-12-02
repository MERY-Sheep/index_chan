// Embedding model for code vectorization using Candle
use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

/// Configuration for embedding model
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimension: usize,
    pub max_length: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            // Use the correct HuggingFace model ID
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            max_length: 512,
        }
    }
}

/// Embedding model for converting code to vectors
pub struct EmbeddingModel {
    model: Option<BertModel>,
    tokenizer: Tokenizer,
    device: Device,
    config: EmbeddingConfig,
    use_simple_hash: bool,
}

impl EmbeddingModel {
    /// Create a new embedding model
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        Self::new_with_fallback(config, true)
    }

    /// Create a new embedding model with optional fallback to simple hash
    pub fn new_with_fallback(config: EmbeddingConfig, allow_fallback: bool) -> Result<Self> {
        let device = Device::Cpu;

        // Try to load the actual model
        match Self::load_model(&config, &device) {
            Ok((model, tokenizer)) => {
                println!("✅ Embeddingモデルをロード完了: {}", config.model_name);
                Ok(Self {
                    model: Some(model),
                    tokenizer,
                    device,
                    config,
                    use_simple_hash: false,
                })
            }
            Err(e) => {
                if allow_fallback {
                    eprintln!("⚠️  Embeddingモデルのロードに失敗、シンプルハッシュにフォールバック: {}", e);
                    
                    // Create a dummy tokenizer for fallback mode
                    let tokenizer = Self::create_dummy_tokenizer()?;
                    
                    Ok(Self {
                        model: None,
                        tokenizer,
                        device,
                        config,
                        use_simple_hash: true,
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    fn load_model(config: &EmbeddingConfig, device: &Device) -> Result<(BertModel, Tokenizer)> {
        println!("📥 Embeddingモデルをロード中: {}", config.model_name);

        // Check local directory first
        let local_model_dir = std::path::PathBuf::from("models/embeddings");
        let use_local = local_model_dir.exists()
            && local_model_dir.join("config.json").exists()
            && local_model_dir.join("tokenizer.json").exists()
            && local_model_dir.join("model.safetensors").exists();

        let (config_file, tokenizer_file, model_file) = if use_local {
            println!("  ローカルモデルを使用: ./models/embeddings/");
            (
                local_model_dir.join("config.json"),
                local_model_dir.join("tokenizer.json"),
                local_model_dir.join("model.safetensors"),
            )
        } else {
            println!("  HuggingFace Hubからダウンロード中...");
            println!("  💡 初回実行時は数分かかる場合があります");
            
            // Use the same approach as LLM model
            let api = Api::new().context(
                "HuggingFace APIの初期化に失敗しました\n\
                 💡 トラブルシューティング:\n\
                    1. インターネット接続を確認してください\n\
                    2. ファイアウォール内の場合はプロキシ設定を確認してください\n\
                    3. HuggingFace Hubがダウンしている場合は後で再試行してください"
            )?;
            
            // Create model repo reference
            let model_repo = api.model(config.model_name.clone());

            println!("    - config.json");
            let config_file = model_repo
                .get("config.json")
                .context("config.jsonのダウンロードに失敗しました\n\
                         💡 モデルが存在しないか、ネットワーク接続に失敗しました")?;

            println!("    - tokenizer.json");
            let tokenizer_file = model_repo
                .get("tokenizer.json")
                .context("tokenizer.jsonのダウンロードに失敗しました")?;

            println!("    - model.safetensors (~90MB)");
            let model_file = model_repo
                .get("model.safetensors")
                .context("model.safetensorsのダウンロードに失敗しました\n\
                         💡 このファイルは大きいです(~90MB)。以下を確認してください:\n\
                            - 安定したインターネット接続\n\
                            - 十分なディスク容量")?;

            (config_file, tokenizer_file, model_file)
        };

        let tokenizer = Tokenizer::from_file(tokenizer_file)
            .map_err(|e| anyhow::anyhow!("トークナイザーのロードに失敗: {}", e))?;

        let bert_config: BertConfig = serde_json::from_reader(std::fs::File::open(config_file)?)?;

        let dtype = candle_core::DType::F32;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], dtype, device)? };

        let model = BertModel::load(vb, &bert_config)?;

        Ok((model, tokenizer))
    }

    fn create_dummy_tokenizer() -> Result<Tokenizer> {
        // Create a minimal tokenizer for fallback mode
        use tokenizers::models::bpe::BPE;
        use tokenizers::Tokenizer as TokenizerBuilder;
        
        let bpe = BPE::default();
        Ok(TokenizerBuilder::new(bpe))
    }

    /// Encode text into a vector
    pub fn encode(&self, text: &str) -> Result<Vec<f32>> {
        if self.use_simple_hash {
            return Ok(self.simple_hash(text));
        }

        let model = self.model.as_ref().context("モデルが初期化されていません")?;

        // Tokenize
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("トークン化に失敗: {}", e))?;

        let tokens = encoding.get_ids();
        let token_ids = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;
        
        // Create token type IDs (all zeros for single sentence)
        let token_type_ids = Tensor::zeros_like(&token_ids)?;

        // Forward pass
        let embeddings = model.forward(&token_ids, &token_type_ids, None)?;

        // Mean pooling
        let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
        let embeddings = embeddings.squeeze(0)?;

        // Normalize
        let embeddings = Self::normalize_l2(&embeddings)?;

        // Convert to Vec<f32>
        let embedding_vec = embeddings.to_vec1::<f32>()?;

        Ok(embedding_vec)
    }

    /// Encode multiple texts into vectors
    pub fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.encode(t)).collect()
    }

    /// L2 normalization
    fn normalize_l2(v: &Tensor) -> Result<Tensor> {
        let norm = v.sqr()?.sum_all()?.sqrt()?;
        Ok(v.broadcast_div(&norm)?)
    }

    /// Simple hash-based vector (fallback)
    fn simple_hash(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0; self.config.dimension];
        
        for (i, byte) in text.bytes().enumerate() {
            let idx = (i * byte as usize) % self.config.dimension;
            vec[idx] += 1.0;
        }
        
        // Normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        
        vec
    }

    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_model() {
        let config = EmbeddingConfig::default();
        let model = EmbeddingModel::new(config).unwrap();
        
        let vec = model.encode("function test() {}").unwrap();
        assert_eq!(vec.len(), 384);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        assert!((EmbeddingModel::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((EmbeddingModel::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}
