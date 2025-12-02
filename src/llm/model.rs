use anyhow::{Context, Result};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config as Qwen2Config, ModelForCausalLM as Qwen2Model};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

use super::config::LLMConfig;

#[allow(dead_code)]
pub struct LLMModel {
    model: Qwen2Model,
    tokenizer: Tokenizer,
    device: Device,
    config: LLMConfig,
}

impl LLMModel {
    pub fn new(config: LLMConfig) -> Result<Self> {
        println!("📥 モデルをロード中: {}", config.model_name);

        let device = Device::Cpu;

        // Check if local model directory exists
        let local_model_dir = std::path::PathBuf::from("models");
        let tokenizer_path = if local_model_dir.join("tokenizer.json").exists() {
            local_model_dir.join("tokenizer.json")
        } else {
            local_model_dir.join("tokenizer_config.json")
        };

        let use_local = local_model_dir.exists()
            && local_model_dir.join("config.json").exists()
            && tokenizer_path.exists()
            && local_model_dir.join("model.safetensors").exists();

        let (config_file, tokenizer_file, model_file) = if use_local {
            println!("  ローカルモデルを使用: ./models/");
            (
                local_model_dir.join("config.json"),
                tokenizer_path,
                local_model_dir.join("model.safetensors"),
            )
        } else {
            println!("  HuggingFace Hubからダウンロード中...");
            println!("  💡 初回実行時は数分かかる場合があります");

            // Download model from HuggingFace Hub using model() method
            let api = Api::new().context(
                "HuggingFace APIの初期化に失敗しました\n\
                 💡 トラブルシューティング:\n\
                    1. インターネット接続を確認してください\n\
                    2. ファイアウォール内の場合はプロキシ設定を確認してください\n\
                    3. HuggingFace Hubがダウンしている場合は後で再試行してください"
            )?;
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

            println!("    - model.safetensors (~3GB)");
            let model_file = model_repo
                .get("model.safetensors")
                .context("model.safetensorsのダウンロードに失敗しました\n\
                         💡 このファイルは大きいです(~3GB)。以下を確認してください:\n\
                            - 安定したインターネット接続\n\
                            - 十分なディスク容量\n\
                            - 時間がかかります(5-10分程度)")?;

            (config_file, tokenizer_file, model_file)
        };

        println!("  トークナイザーをロード中...");
        let tokenizer = Tokenizer::from_file(tokenizer_file)
            .map_err(|e| anyhow::anyhow!("トークナイザーのロードに失敗: {}", e))?;

        println!("  モデル設定をロード中...");
        let model_config: Qwen2Config = serde_json::from_reader(std::fs::File::open(config_file)?)
            .context("モデル設定の解析に失敗しました")?;

        println!("  モデルの重みをロード中...");
        println!("  💡 約3GBのRAMが必要です");
        // Use F32 for better compatibility
        let dtype = candle_core::DType::F32;
        let vb = unsafe { 
            VarBuilder::from_mmaped_safetensors(&[model_file], dtype, &device)
                .context("モデルの重みのロードに失敗しました\n\
                         💡 考えられる原因:\n\
                            - メモリ不足(約3GBのRAMが必要)\n\
                            - モデルファイルの破損(~/.cache/huggingfaceを削除してみてください)\n\
                            - 互換性のないモデル形式")?
        };

        let model = Qwen2Model::new(&model_config, vb)
            .context("モデルの初期化に失敗しました")?;

        println!("✅ モデルのロード完了");

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
        })
    }

    pub fn generate(&mut self, prompt: &str) -> Result<String> {
        // Format prompt for Qwen2.5-Coder-Instruct
        let formatted_prompt = format!(
            "<|im_start|>system\nYou are a helpful code analysis assistant.<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            prompt
        );

        // Tokenize input
        let encoding = self
            .tokenizer
            .encode(formatted_prompt.as_str(), true)
            .map_err(|e| anyhow::anyhow!("トークン化に失敗: {}", e))?;

        let tokens = encoding.get_ids();
        println!("  入力トークン数: {}", tokens.len());

        if tokens.len() > 2000 {
            anyhow::bail!(
                "入力が長すぎます: {}トークン (最大2000)\n\
                 💡 コンテキストを減らすか、分析を分割してください",
                tokens.len()
            );
        }

        // Generate tokens
        let mut generated_tokens = tokens.to_vec();
        let max_new_tokens = self.config.max_tokens.min(50); // Limit for testing

        for step in 0..max_new_tokens {
            // For first step, use all tokens; for subsequent steps, use only the last token
            let input_tokens = if step == 0 {
                &generated_tokens[..]
            } else {
                &generated_tokens[generated_tokens.len() - 1..]
            };

            let input = Tensor::new(input_tokens, &self.device)?.unsqueeze(0)?;

            let start_pos = if step == 0 {
                0
            } else {
                generated_tokens.len() - 1
            };

            // Forward pass (ModelForCausalLM already returns logits for the last position)
            let logits = self.model.forward(&input, start_pos)?;
            let last_logits = logits.squeeze(0)?.squeeze(0)?; // shape: [vocab_size]

            // Sample next token (greedy for now)
            let next_token = last_logits.argmax(0)?.to_scalar::<u32>()?;

            // Check for EOS token
            let eos_tokens = vec![
                self.tokenizer.token_to_id("<|endoftext|>"),
                self.tokenizer.token_to_id("<|im_end|>"),
                self.tokenizer.token_to_id("</s>"),
            ];

            if eos_tokens.iter().any(|&t| t == Some(next_token)) {
                println!("  EOS検出 (ステップ {})", step + 1);
                break;
            }

            generated_tokens.push(next_token);

            if (step + 1) % 10 == 0 {
                println!("  生成中... {}トークン", step + 1);
            }
        }

        println!(
            "  生成完了: {}トークン",
            generated_tokens.len() - tokens.len()
        );

        // Decode output
        let output = self
            .tokenizer
            .decode(&generated_tokens[tokens.len()..], true)
            .map_err(|e| anyhow::anyhow!("デコードに失敗: {}", e))?;

        Ok(output)
    }
}
