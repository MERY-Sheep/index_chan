use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use cleaner::Cleaner;
use detector::detect_dead_code;
use reporter::{generate_json_report, print_report};
use scanner::Scanner;

mod annotator;
mod cleaner;
mod detector;
mod graph;
mod llm;
mod parser;
mod reporter;
mod scanner;

#[derive(Parser)]
#[command(name = "index-chan")]
#[command(about = "TypeScript dead code detection CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan directory for dead code
    Scan {
        /// Target directory to scan
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Output report to JSON file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Use LLM for advanced analysis
        #[arg(long)]
        llm: bool,
    },

    /// Clean dead code (interactive or automatic)
    Clean {
        /// Target directory to clean
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Dry run (don't actually delete)
        #[arg(long)]
        dry_run: bool,

        /// Automatic mode (only delete definitely safe code)
        #[arg(long)]
        auto: bool,

        /// Only delete definitely safe code
        #[arg(long)]
        safe_only: bool,
    },

    /// Annotate code that should be kept (suppress warnings)
    Annotate {
        /// Target directory to annotate
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Use LLM for advanced analysis
        #[arg(long)]
        llm: bool,

        /// Dry run (don't actually modify files)
        #[arg(long)]
        dry_run: bool,
    },

    /// Test LLM inference with a simple prompt
    TestLlm {
        /// Custom prompt to test (optional)
        #[arg(short, long)]
        prompt: Option<String>,

        /// List available files in the model repository
        #[arg(long)]
        list_files: bool,

        /// Test tokenizer only (no inference)
        #[arg(long)]
        tokenizer_only: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            directory,
            output,
            llm,
        } => {
            println!("🔍 Scanning directory: {}", directory.display());
            if llm {
                println!("🤖 LLM分析モード有効");
            }
            println!();

            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            let total_files = walkdir::WalkDir::new(&directory)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().and_then(|s| s.to_str()) == Some("ts")
                        || e.path().extension().and_then(|s| s.to_str()) == Some("tsx")
                })
                .count();

            let total_functions = graph.nodes.len();
            let mut dead_code = detect_dead_code(&graph);

            // LLM analysis if requested
            if llm {
                println!("🤖 LLMで分析中...");
                let llm_config = llm::LLMConfig::default();
                let mut llm_analyzer = llm::LLMAnalyzer::new(llm_config, true)?;
                let context_collector = llm::ContextCollector::new(&directory);

                for code in &mut dead_code {
                    let context = context_collector.collect_context(&code.node);
                    match llm_analyzer.analyze(&code.node, &context) {
                        Ok(analysis) => {
                            // Update reason with LLM analysis
                            code.reason = format!(
                                "{} (確信度: {:.0}%)",
                                analysis.reason,
                                analysis.confidence * 100.0
                            );

                            // Update safety level based on LLM analysis
                            if analysis.should_delete && analysis.confidence > 0.9 {
                                code.safety_level = detector::SafetyLevel::DefinitelySafe;
                            } else if !analysis.should_delete && analysis.confidence > 0.8 {
                                code.safety_level = detector::SafetyLevel::NeedsReview;
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  LLM分析エラー ({}): {}", code.node.name, e);
                        }
                    }
                }
                println!("✅ LLM分析完了\n");
            }

            print_report(&dead_code, total_files, total_functions);

            if let Some(output_path) = output {
                let report = generate_json_report(&dead_code, total_files, total_functions);
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&output_path, json)?;
                println!("\n📄 Report saved to: {}", output_path.display());
            }

            Ok(())
        }
        Commands::Clean {
            directory,
            dry_run,
            auto,
            safe_only,
        } => {
            println!("🧹 Cleaning directory: {}", directory.display());
            if dry_run {
                println!("(Dry run mode)");
            }
            println!();

            // スキャン
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            let dead_code = detect_dead_code(&graph);

            if dead_code.is_empty() {
                println!("✨ デッドコードは見つかりませんでした");
                return Ok(());
            }

            println!("\n削除候補: {}個", dead_code.len());

            // クリーニング実行
            let cleaner = Cleaner::new(dry_run, auto, safe_only);
            let result = cleaner.clean(&dead_code)?;

            println!("\n📊 結果:");
            println!(
                "  削除: {}個 ({}行)",
                result.deleted_count, result.deleted_lines
            );
            println!("  スキップ: {}個", result.skipped_count);

            if dry_run {
                println!("\n💡 実際に削除するには --dry-run を外してください");
            }

            Ok(())
        }
        Commands::Annotate {
            directory,
            llm,
            dry_run,
        } => {
            println!("📝 アノテーション追加: {}", directory.display());
            if llm {
                println!("🤖 LLM分析モード有効");
            }
            if dry_run {
                println!("(ドライランモード)");
            }
            println!();

            // スキャン
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            let dead_code = detect_dead_code(&graph);

            if dead_code.is_empty() {
                println!("✨ デッドコードは見つかりませんでした");
                return Ok(());
            }

            println!("📊 検出結果: {}個の未使用関数", dead_code.len());

            // LLM analysis if requested
            let mut annotator = annotator::Annotator::new(dry_run);

            if llm {
                println!("🤖 LLMで分析中...");
                let llm_config = llm::LLMConfig::default();
                let mut llm_analyzer = llm::LLMAnalyzer::new(llm_config, true)?;
                let context_collector = llm::ContextCollector::new(&directory);

                let mut analyses = std::collections::HashMap::new();

                for code in &dead_code {
                    let context = context_collector.collect_context(&code.node);
                    match llm_analyzer.analyze(&code.node, &context) {
                        Ok(analysis) => {
                            let key =
                                format!("{}:{}", code.node.file_path.display(), code.node.name);
                            analyses.insert(
                                key,
                                annotator::LLMAnalysisData {
                                    should_delete: analysis.should_delete,
                                    confidence: analysis.confidence,
                                    reason: analysis.reason,
                                    category: format!("{:?}", analysis.category),
                                },
                            );
                        }
                        Err(e) => {
                            eprintln!("⚠️  LLM分析エラー ({}): {}", code.node.name, e);
                        }
                    }
                }

                annotator = annotator.with_llm_analyses(analyses);
                println!("✅ LLM分析完了\n");
            }

            // アノテーション追加
            let result = annotator.annotate(&dead_code)?;

            println!("\n📝 結果:");
            println!("  アノテーション追加: {}個", result.annotated_count);
            println!("  スキップ: {}個", result.skipped_count);

            if dry_run {
                println!("\n💡 実際に追加するには --dry-run を外してください");
            } else {
                println!("\n✅ アノテーションを追加しました");
            }

            Ok(())
        }
        Commands::TestLlm {
            prompt,
            list_files,
            tokenizer_only,
        } => {
            println!("🤖 LLM推論テスト開始\n");

            let config = llm::LLMConfig::default();

            if list_files {
                println!("📂 モデルリポジトリのファイル一覧を確認中...");
                println!("  モデル: {}\n", config.model_name);

                use hf_hub::api::sync::Api;
                let api = Api::new()?;
                let model_repo = api.model(config.model_name.clone());

                println!("💡 以下のファイルをダウンロード試行します:");
                let files = vec!["config.json", "tokenizer.json", "model.safetensors"];
                for file in files {
                    print!("  {} ... ", file);
                    match model_repo.get(file) {
                        Ok(path) => println!("✅ 存在 ({})", path.display()),
                        Err(e) => println!("❌ エラー: {}", e),
                    }
                }
                return Ok(());
            }

            let test_prompt = prompt.unwrap_or_else(|| {
                "この関数は削除しても安全ですか？\n\nfunction unusedHelper() {\n  return 42;\n}"
                    .to_string()
            });

            println!("📝 プロンプト:");
            println!("{}\n", test_prompt);

            println!("🔧 モデル設定:");
            println!("  モデル名: {}", config.model_name);
            println!("  最大トークン数: {}", config.max_tokens);
            println!("  温度: {}", config.temperature);
            println!();

            if tokenizer_only {
                println!("🔧 トークナイザーのみテスト\n");

                use tokenizers::Tokenizer;
                let tokenizer_path = std::path::PathBuf::from("models/tokenizer.json");

                if !tokenizer_path.exists() {
                    eprintln!(
                        "❌ tokenizer.jsonが見つかりません: {}",
                        tokenizer_path.display()
                    );
                    return Ok(());
                }

                println!("📥 トークナイザーをロード中...");
                let tokenizer = Tokenizer::from_file(tokenizer_path)
                    .map_err(|e| anyhow::anyhow!("トークナイザーのロードに失敗: {}", e))?;

                println!("✅ トークナイザーのロード完了\n");

                println!("🔤 エンコードテスト:");
                let encoding = tokenizer
                    .encode(test_prompt.as_str(), true)
                    .map_err(|e| anyhow::anyhow!("エンコードに失敗: {}", e))?;

                let tokens = encoding.get_ids();
                println!("  トークン数: {}", tokens.len());
                println!("  トークンID: {:?}", &tokens[..tokens.len().min(10)]);

                println!("\n🔤 デコードテスト:");
                let decoded = tokenizer
                    .decode(tokens, true)
                    .map_err(|e| anyhow::anyhow!("デコードに失敗: {}", e))?;
                println!("  デコード結果: {}", decoded);

                println!("\n✅ トークナイザーは正常に動作しています");
                return Ok(());
            }

            println!("📥 モデルをロード中...");
            println!("  (初回実行時は数分かかる場合があります)");
            println!("  💡 ファイル確認: cargo run -- test-llm --list-files");
            println!("  💡 トークナイザーのみテスト: cargo run -- test-llm --tokenizer-only\n");

            match llm::LLMModel::new(config) {
                Ok(mut model) => {
                    println!("\n🚀 推論実行中...");

                    match model.generate(&test_prompt) {
                        Ok(response) => {
                            println!("\n✅ 推論成功！\n");
                            println!("📤 応答:");
                            println!("{}", response);
                        }
                        Err(e) => {
                            eprintln!("\n❌ 推論エラー: {}", e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ モデルロードエラー: {}", e);
                    eprintln!("\n💡 トラブルシューティング:");
                    eprintln!("  1. インターネット接続を確認してください");
                    eprintln!("  2. HuggingFace Hubへのアクセスが可能か確認してください");
                    eprintln!("  3. ディスク容量を確認してください（約2GB必要）");
                    return Err(e);
                }
            }

            Ok(())
        }
    }
}
