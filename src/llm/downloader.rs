use anyhow::{Context, Result};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::PathBuf;

pub struct ModelDownloader {
    api: Api,
}

impl ModelDownloader {
    pub fn new() -> Result<Self> {
        let api = Api::new()?;
        Ok(Self { api })
    }
    
    pub fn download_model(&self, model_name: &str) -> Result<ModelFiles> {
        println!("📥 モデルをダウンロード中: {}", model_name);
        
        let repo = self.api.repo(Repo::new(
            model_name.to_string(),
            RepoType::Model,
        ));
        
        println!("  モデルファイルをダウンロード中...");
        let model_file = repo.get("model.safetensors")
            .context("モデルファイルのダウンロードに失敗しました")?;
        
        println!("  トークナイザーをダウンロード中...");
        let tokenizer_file = repo.get("tokenizer.json")
            .context("トークナイザーのダウンロードに失敗しました")?;
        
        println!("  設定ファイルをダウンロード中...");
        let config_file = repo.get("config.json")
            .context("設定ファイルのダウンロードに失敗しました")?;
        
        println!("✅ ダウンロード完了");
        
        Ok(ModelFiles {
            model_file,
            tokenizer_file,
            config_file,
        })
    }
    
    pub fn is_model_cached(&self, model_name: &str) -> bool {
        let repo = self.api.repo(Repo::new(
            model_name.to_string(),
            RepoType::Model,
        ));
        
        // Check if all required files exist in cache
        repo.get("model.safetensors").is_ok()
            && repo.get("tokenizer.json").is_ok()
            && repo.get("config.json").is_ok()
    }
}

pub struct ModelFiles {
    pub model_file: PathBuf,
    pub tokenizer_file: PathBuf,
    pub config_file: PathBuf,
}
