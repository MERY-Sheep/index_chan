// Semantic embedding module using Candle + all-MiniLM-L6-v2
// Phase 7 GraphRAG: Concept Transformer integration

#[cfg(feature = "semantic-search")]
use anyhow::{anyhow, Result};
#[cfg(feature = "semantic-search")]
use candle_core::{DType, Device, Tensor};
#[cfg(feature = "semantic-search")]
use candle_nn::VarBuilder;
#[cfg(feature = "semantic-search")]
use candle_transformers::models::bert::{BertModel, Config};
#[cfg(feature = "semantic-search")]
use hf_hub::{api::sync::Api, Repo, RepoType};
#[cfg(feature = "semantic-search")]
use tokenizers::Tokenizer;

#[cfg(not(feature = "semantic-search"))]
use anyhow::{anyhow, Result};

const EMBEDDING_DIM: usize = 384;
const MODEL_ID: &str = "sentence-transformers/all-MiniLM-L6-v2";

/// Sentence embedding generator using all-MiniLM-L6-v2
#[cfg(feature = "semantic-search")]
pub struct EmbeddingGenerator {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

#[cfg(not(feature = "semantic-search"))]
pub struct EmbeddingGenerator;

#[cfg(feature = "semantic-search")]
impl EmbeddingGenerator {
    /// Create a new embedding generator (downloads model on first use)
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;

        eprintln!("Loading model: {}", MODEL_ID);

        // Download model files from HuggingFace Hub
        let api = Api::new()?;
        let repo = api.repo(Repo::new(MODEL_ID.to_string(), RepoType::Model));

        let config_path = repo.get("config.json")?;
        let tokenizer_path = repo.get("tokenizer.json")?;
        let weights_path = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))?;

        // Load config
        let config_content = std::fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_content)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Load model weights
        let vb = if weights_path
            .extension()
            .map(|e| e == "safetensors")
            .unwrap_or(false)
        {
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)? }
        } else {
            return Err(anyhow!("Only safetensors format is supported"));
        };

        let model = BertModel::load(vb, &config)?;

        eprintln!("Model loaded successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Generate embedding for a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text])?;
        Ok(embeddings.into_iter().next().unwrap())
    }

    /// Generate embeddings for multiple texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings = Vec::new();

        for text in texts {
            let encoding = self
                .tokenizer
                .encode(*text, true)
                .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

            let input_ids: Vec<u32> = encoding.get_ids().to_vec();
            let attention_mask: Vec<u32> = encoding.get_attention_mask().to_vec();
            let token_type_ids: Vec<u32> = encoding.get_type_ids().to_vec();

            let seq_len = input_ids.len();

            let input_ids = Tensor::new(&input_ids[..], &self.device)?.reshape((1, seq_len))?;
            let attention_mask =
                Tensor::new(&attention_mask[..], &self.device)?.reshape((1, seq_len))?;
            let token_type_ids =
                Tensor::new(&token_type_ids[..], &self.device)?.reshape((1, seq_len))?;

            // Forward pass
            let output = self
                .model
                .forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

            // Mean pooling over sequence dimension
            let embedding = self.mean_pooling(&output, &attention_mask)?;

            // Normalize
            let embedding = self.normalize(&embedding)?;

            let embedding_vec: Vec<f32> = embedding.squeeze(0)?.to_vec1()?;
            all_embeddings.push(embedding_vec);
        }

        Ok(all_embeddings)
    }

    /// Mean pooling: average token embeddings weighted by attention mask
    fn mean_pooling(&self, output: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        let mask = attention_mask.unsqueeze(2)?.to_dtype(DType::F32)?;
        let masked = output.broadcast_mul(&mask)?;
        let sum = masked.sum(1)?;
        let mask_sum = mask.sum(1)?;
        let result = sum.broadcast_div(&mask_sum)?;

        Ok(result)
    }

    /// L2 normalize the embedding
    fn normalize(&self, embedding: &Tensor) -> Result<Tensor> {
        let norm = embedding.sqr()?.sum(1)?.sqrt()?.unsqueeze(1)?;
        let normalized = embedding.broadcast_div(&norm)?;
        Ok(normalized)
    }

    /// Compute cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        EMBEDDING_DIM
    }
}

#[cfg(not(feature = "semantic-search"))]
impl EmbeddingGenerator {
    pub fn new() -> Result<Self> {
        Err(anyhow!("semantic-search feature is not enabled. Build with: cargo build --features semantic-search"))
    }

    pub fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow!("semantic-search feature is not enabled"))
    }

    pub fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Err(anyhow!("semantic-search feature is not enabled"))
    }

    pub fn cosine_similarity(_a: &[f32], _b: &[f32]) -> f32 {
        0.0
    }

    pub fn dimension(&self) -> usize {
        EMBEDDING_DIM
    }
}
