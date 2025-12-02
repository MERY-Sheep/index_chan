use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use cleaner::Cleaner;
use detector::detect_dead_code;
use reporter::{generate_json_report, print_report};
use scanner::Scanner;

mod annotator;
mod cleaner;
mod conversation;
mod detector;
mod exporter;
mod graph;
mod llm;
mod parser;
mod reporter;
mod scanner;
mod search;

#[cfg(feature = "web")]
mod web_server;

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

    /// Test embedding model
    TestEmbedding {
        /// Text to encode (optional)
        #[arg(short, long)]
        text: Option<String>,

        /// Compare similarity between two texts
        #[arg(long)]
        compare: bool,
    },

    /// Create search index for code
    Index {
        /// Target directory to index
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Output index file
        #[arg(short, long, value_name = "FILE", default_value = ".index-chan/index.json")]
        output: PathBuf,
    },

    /// Search for code
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Index file to search
        #[arg(short, long, value_name = "FILE", default_value = ".index-chan/index.json")]
        index: PathBuf,

        /// Number of results to return
        #[arg(short = 'k', long, default_value = "10")]
        top_k: usize,

        /// Include context in results
        #[arg(long)]
        context: bool,
    },

    /// Analyze chat history
    AnalyzeChat {
        /// Chat history JSON file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output graph file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Extract topics from chat history
    Topics {
        /// Chat history JSON file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Use LLM for advanced topic detection
        #[arg(long)]
        llm: bool,
    },

    /// Find related messages in chat history
    Related {
        /// Chat history JSON file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Query to find related messages
        #[arg(value_name = "QUERY")]
        query: String,

        /// Number of results to return
        #[arg(short = 'k', long, default_value = "5")]
        top_k: usize,

        /// Show context window around each result
        #[arg(long)]
        context: bool,
    },

    /// Export dependency graph for visualization
    Export {
        /// Target directory to analyze
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Export format (graphml, dot, json)
        #[arg(short, long, default_value = "graphml")]
        format: String,
    },

    /// Visualize dependency graph in 3D (web server)
    #[cfg(feature = "web")]
    Visualize {
        /// Target directory to analyze
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Server port
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Open browser automatically
        #[arg(long)]
        open: bool,
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
                println!("🤖 LLM analysis mode enabled");
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
                println!("🤖 Analyzing with LLM...");
                let llm_config = llm::LLMConfig::default();
                let mut llm_analyzer = llm::LLMAnalyzer::new(llm_config, true)?;
                let context_collector = llm::ContextCollector::new(&directory);

                for code in &mut dead_code {
                    let context = context_collector.collect_context(&code.node);
                    match llm_analyzer.analyze(&code.node, &context) {
                        Ok(analysis) => {
                            // Update reason with LLM analysis
                            code.reason = format!(
                                "{} (confidence: {:.0}%)",
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
                            eprintln!("⚠️  LLM analysis error ({}): {}", code.node.name, e);
                        }
                    }
                }
                println!("✅ LLM analysis completed\n");
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
                println!("✨ No dead code found");
                return Ok(());
            }

            println!("\nDeletion candidates: {} items", dead_code.len());

            // Execute cleaning
            let cleaner = Cleaner::new(dry_run, auto, safe_only);
            let result = cleaner.clean(&dead_code)?;

            println!("\n📊 Results:");
            println!(
                "  Deleted: {} items ({} lines)",
                result.deleted_count, result.deleted_lines
            );
            println!("  Skipped: {} items", result.skipped_count);

            if dry_run {
                println!("\n💡 Remove --dry-run flag to actually delete");
            }

            Ok(())
        }
        Commands::Annotate {
            directory,
            llm,
            dry_run,
        } => {
            println!("📝 Adding annotations: {}", directory.display());
            if llm {
                println!("🤖 LLM analysis mode enabled");
            }
            if dry_run {
                println!("(Dry run mode)");
            }
            println!();

            // スキャン
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            let dead_code = detect_dead_code(&graph);

            if dead_code.is_empty() {
                println!("✨ No dead code found");
                return Ok(());
            }

            println!("📊 Detection results: {} unused functions", dead_code.len());

            // LLM analysis if requested
            let mut annotator = annotator::Annotator::new(dry_run);

            if llm {
                println!("🤖 Analyzing with LLM...");
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
                            eprintln!("⚠️  LLM analysis error ({}): {}", code.node.name, e);
                        }
                    }
                }

                annotator = annotator.with_llm_analyses(analyses);
                println!("✅ LLM analysis completed\n");
            }

            // アノテーション追加
            let result = annotator.annotate(&dead_code)?;

            println!("\n📝 Results:");
            println!("  Annotations added: {} items", result.annotated_count);
            println!("  Skipped: {} items", result.skipped_count);

            if dry_run {
                println!("\n💡 Remove --dry-run flag to actually add annotations");
            } else {
                println!("\n✅ Annotations added successfully");
            }

            Ok(())
        }
        Commands::TestLlm {
            prompt,
            list_files,
            tokenizer_only,
        } => {
            println!("🤖 Starting LLM inference test\n");

            let config = llm::LLMConfig::default();

            if list_files {
                println!("📂 Checking model repository files...");
                println!("  Model: {}\n", config.model_name);

                use hf_hub::api::sync::Api;
                let api = Api::new()?;
                let model_repo = api.model(config.model_name.clone());

                println!("💡 Attempting to download the following files:");
                let files = vec!["config.json", "tokenizer.json", "model.safetensors"];
                for file in files {
                    print!("  {} ... ", file);
                    match model_repo.get(file) {
                        Ok(path) => println!("✅ Exists ({})", path.display()),
                        Err(e) => println!("❌ Error: {}", e),
                    }
                }
                return Ok(());
            }

            let test_prompt = prompt.unwrap_or_else(|| {
                "Is this function safe to delete?\n\nfunction unusedHelper() {\n  return 42;\n}"
                    .to_string()
            });

            println!("📝 Prompt:");
            println!("{}\n", test_prompt);

            println!("🔧 Model configuration:");
            println!("  Model name: {}", config.model_name);
            println!("  Max tokens: {}", config.max_tokens);
            println!("  Temperature: {}", config.temperature);
            println!();

            if tokenizer_only {
                println!("🔧 Testing tokenizer only\n");

                use tokenizers::Tokenizer;
                let tokenizer_path = std::path::PathBuf::from("models/tokenizer.json");

                if !tokenizer_path.exists() {
                    eprintln!(
                        "❌ tokenizer.json not found: {}",
                        tokenizer_path.display()
                    );
                    return Ok(());
                }

                println!("📥 Loading tokenizer...");
                let tokenizer = Tokenizer::from_file(tokenizer_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

                println!("✅ Tokenizer loaded successfully\n");

                println!("🔤 Encoding test:");
                let encoding = tokenizer
                    .encode(test_prompt.as_str(), true)
                    .map_err(|e| anyhow::anyhow!("Failed to encode: {}", e))?;

                let tokens = encoding.get_ids();
                println!("  Token count: {}", tokens.len());
                println!("  Token IDs: {:?}", &tokens[..tokens.len().min(10)]);

                println!("\n🔤 Decoding test:");
                let decoded = tokenizer
                    .decode(tokens, true)
                    .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;
                println!("  Decoded result: {}", decoded);

                println!("\n✅ Tokenizer is working correctly");
                return Ok(());
            }

            println!("📥 Loading model...");
            println!("  (First run may take several minutes)");
            println!("  💡 Check files: cargo run -- test-llm --list-files");
            println!("  💡 Test tokenizer only: cargo run -- test-llm --tokenizer-only\n");

            match llm::LLMModel::new(config) {
                Ok(mut model) => {
                    println!("\n🚀 Running inference...");

                    match model.generate(&test_prompt) {
                        Ok(response) => {
                            println!("\n✅ Inference successful!\n");
                            println!("📤 Response:");
                            println!("{}", response);
                        }
                        Err(e) => {
                            eprintln!("\n❌ Inference error: {}", e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Model loading error: {}", e);
                    eprintln!("\n💡 Troubleshooting:");
                    eprintln!("  1. Check your internet connection");
                    eprintln!("  2. Verify access to HuggingFace Hub");
                    eprintln!("  3. Check disk space (approximately 2GB required)");
                    return Err(e);
                }
            }

            Ok(())
        }
        Commands::Index { directory, output } => {
            println!("📚 Creating index: {}", directory.display());
            println!();

            // Scan directory
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            println!("📊 Found {} functions", graph.nodes.len());

            // Create index
            let mut index = search::CodeIndex::new()?;

            for (_id, node) in &graph.nodes {
                // Get dependencies
                let dependencies: Vec<String> = graph
                    .edges
                    .iter()
                    .filter(|e| e.from == node.id)
                    .filter_map(|e| graph.nodes.get(&e.to).map(|n| n.name.clone()))
                    .collect();

                let metadata = search::index::CodeMetadata {
                    file_path: node.file_path.clone(),
                    function_name: node.name.clone(),
                    start_line: node.line_range.0,
                    end_line: node.line_range.1,
                    code_snippet: format!("{:?}", node.node_type), // TODO: Get actual code snippet
                    dependencies,
                };
                index.add(metadata)?;
            }

            println!("✅ Indexed {} items", index.len());

            // Save index
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            index.save(&output)?;

            println!("💾 Index saved to: {}", output.display());

            Ok(())
        }
        Commands::Search {
            query,
            index: index_path,
            top_k,
            context,
        } => {
            println!("🔍 Searching: {}", query);
            println!();

            // Load index
            let mut index = search::CodeIndex::new()?;
            
            if !index_path.exists() {
                eprintln!("❌ Index file not found: {}", index_path.display());
                eprintln!("💡 Create index first: index-chan index <directory>");
                return Ok(());
            }

            index.load(&index_path)?;
            println!("📚 Loaded index: {} items", index.len());
            println!();

            // Search
            let results = index.search(&query, top_k)?;

            if results.is_empty() {
                println!("No results found");
                return Ok(());
            }

            println!("📊 Found {} results:\n", results.len());

            for (i, result) in results.iter().enumerate() {
                println!("{}. {} (score: {:.2})", i + 1, result.metadata.function_name, result.score);
                println!("   📄 {}:{}:{}", 
                    result.metadata.file_path.display(),
                    result.metadata.start_line,
                    result.metadata.end_line
                );
                
                if context {
                    println!("   📝 Code:");
                    for line in result.metadata.code_snippet.lines().take(5) {
                        println!("      {}", line);
                    }
                    if result.metadata.code_snippet.lines().count() > 5 {
                        println!("      ...");
                    }
                }
                
                if !result.metadata.dependencies.is_empty() {
                    println!("   🔗 Dependencies: {}", result.metadata.dependencies.join(", "));
                }
                
                println!();
            }

            Ok(())
        }
        Commands::AnalyzeChat { file, output } => {
            println!("💬 Analyzing chat history: {}", file.display());
            println!();

            if !file.exists() {
                eprintln!("❌ File not found: {}", file.display());
                return Ok(());
            }

            // Analyze chat
            let analyzer = conversation::ConversationAnalyzer::new()?;
            let mut graph = analyzer.analyze_file(&file)?;

            println!("📊 Chat statistics:");
            let stats = graph.stats();
            println!("  Messages: {}", stats.total_messages);
            println!("  Edges: {}", stats.total_edges);
            println!("  Avg edges per message: {:.2}", stats.avg_edges_per_node);
            println!();

            // Detect topics
            let mut topic_detector = conversation::TopicDetector::new();
            topic_detector.detect_topics(&mut graph)?;

            println!("📚 Topics detected: {}", graph.topics.len());
            for topic in &graph.topics {
                println!("  - {} ({} messages)", topic.name, topic.message_ids.len());
            }
            println!();

            // Calculate token reduction
            let reduction = analyzer.calculate_token_reduction(&graph, None);
            println!("🎯 Token reduction:");
            println!("  Total tokens: {}", reduction.total_tokens);
            println!("  Relevant tokens: {}", reduction.relevant_tokens);
            println!("  Reduction rate: {:.1}%", reduction.reduction_rate * 100.0);

            // Save graph
            if let Some(output_path) = output {
                let json = serde_json::to_string_pretty(&graph)?;
                std::fs::write(&output_path, json)?;
                println!("\n💾 Graph saved to: {}", output_path.display());
            }

            Ok(())
        }
        Commands::Topics { file, llm } => {
            println!("📚 トピック抽出: {}", file.display());
            if llm {
                println!("🤖 LLM分析モード有効");
            }
            println!();

            if !file.exists() {
                eprintln!("❌ ファイルが見つかりません: {}", file.display());
                return Ok(());
            }

            // Analyze chat
            let analyzer = conversation::ConversationAnalyzer::new()?;
            let mut graph = analyzer.analyze_file(&file)?;

            // Detect topics
            let mut topic_detector = if llm {
                println!("🤖 LLMでトピックを分析中...");
                let llm_config = llm::LLMConfig::default();
                conversation::TopicDetector::with_llm(llm_config)?
            } else {
                conversation::TopicDetector::new()
            };
            
            topic_detector.detect_topics(&mut graph)?;

            if graph.topics.is_empty() {
                println!("トピックが見つかりませんでした");
                return Ok(());
            }

            println!("📊 {}個のトピックを検出:\n", graph.topics.len());

            for (i, topic) in graph.topics.iter().enumerate() {
                println!("{}. {}", i + 1, topic.name);
                println!("   メッセージ数: {}", topic.message_ids.len());
                println!("   キーワード: {}", topic.keywords.join(", "));
                println!();
            }

            Ok(())
        }
        Commands::TestEmbedding { text, compare } => {
            println!("🧪 Embeddingモデルのテスト\n");

            let config = search::embeddings::EmbeddingConfig::default();
            println!("📝 設定:");
            println!("  モデル: {}", config.model_name);
            println!("  次元数: {}", config.dimension);
            println!("  最大長: {}\n", config.max_length);

            println!("📥 モデルをロード中...");
            let model = search::embeddings::EmbeddingModel::new(config)?;
            println!();

            if compare {
                let text1 = "function authenticate(user, password) { return true; }";
                let text2 = "function login(username, pwd) { return checkCredentials(username, pwd); }";
                let text3 = "function calculateTotal(items) { return items.reduce((sum, item) => sum + item.price, 0); }";

                println!("📊 類似度比較テスト:\n");
                println!("テキスト1: {}", text1);
                println!("テキスト2: {}", text2);
                println!("テキスト3: {}\n", text3);

                println!("🔄 エンコード中...");
                let vec1 = model.encode(text1)?;
                let vec2 = model.encode(text2)?;
                let vec3 = model.encode(text3)?;

                let sim_1_2 = search::embeddings::EmbeddingModel::cosine_similarity(&vec1, &vec2);
                let sim_1_3 = search::embeddings::EmbeddingModel::cosine_similarity(&vec1, &vec3);
                let sim_2_3 = search::embeddings::EmbeddingModel::cosine_similarity(&vec2, &vec3);

                println!("\n📈 類似度スコア:");
                println!("  テキスト1 vs テキスト2 (認証関連): {:.4}", sim_1_2);
                println!("  テキスト1 vs テキスト3 (異なる機能): {:.4}", sim_1_3);
                println!("  テキスト2 vs テキスト3 (異なる機能): {:.4}", sim_2_3);

                println!("\n💡 期待される結果:");
                println!("  - 認証関連の関数同士（1 vs 2）の類似度が高い");
                println!("  - 異なる機能の関数（1 vs 3, 2 vs 3）の類似度が低い");
            } else {
                let test_text = text.unwrap_or_else(|| {
                    "function getUserById(id) { return database.query('SELECT * FROM users WHERE id = ?', [id]); }".to_string()
                });

                println!("📝 テキスト:");
                println!("{}\n", test_text);

                println!("🔄 エンコード中...");
                let vector = model.encode(&test_text)?;

                println!("\n✅ エンコード成功!");
                println!("  ベクトル次元: {}", vector.len());
                println!("  最初の10要素: {:?}", &vector[..10.min(vector.len())]);

                // Calculate L2 norm
                let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                println!("  L2ノルム: {:.6}", norm);
                println!("\n💡 L2ノルムが1.0に近い場合、正規化されています");
            }

            Ok(())
        }
        Commands::Related { file, query, top_k, context } => {
            println!("🔍 関連メッセージ検索: {}", file.display());
            println!("📝 クエリ: {}\n", query);

            if !file.exists() {
                eprintln!("❌ ファイルが見つかりません: {}", file.display());
                return Ok(());
            }

            // Analyze chat
            let analyzer = conversation::ConversationAnalyzer::new()?;
            let graph = analyzer.analyze_file(&file)?;

            println!("📊 会話統計:");
            let stats = graph.stats();
            println!("  メッセージ数: {}", stats.total_messages);
            println!();

            // Find related messages
            println!("🔍 関連メッセージを検索中...");
            let related = analyzer.find_related_messages(&graph, &query, top_k)?;

            if related.is_empty() {
                println!("関連メッセージが見つかりませんでした");
                return Ok(());
            }

            println!("📊 {}件の関連メッセージを発見:\n", related.len());

            for (i, msg) in related.iter().enumerate() {
                println!("{}. [{}] {} (類似度: {:.3})", 
                    i + 1, 
                    msg.role, 
                    msg.timestamp,
                    msg.similarity
                );
                
                if let Some(topic_id) = &msg.topic_id {
                    if let Some(topic) = graph.topics.iter().find(|t| &t.id == topic_id) {
                        println!("   🏷️  トピック: {}", topic.name);
                    }
                }
                
                println!("   💬 {}", msg.content);
                
                if context {
                    let context_msgs = graph.get_context_window(&msg.id, 1);
                    if context_msgs.len() > 1 {
                        println!("   📖 コンテキスト:");
                        for ctx_msg in context_msgs {
                            if ctx_msg.id != msg.id {
                                println!("      [{}] {}", ctx_msg.role, 
                                    ctx_msg.content.chars().take(60).collect::<String>());
                            }
                        }
                    }
                }
                
                println!();
            }

            // Calculate token reduction
            let reduction = analyzer.calculate_token_reduction(&graph, Some(&query));
            println!("🎯 トークン削減効果:");
            println!("  全体トークン数: {}", reduction.total_tokens);
            println!("  関連トークン数: {}", reduction.relevant_tokens);
            println!("  削減率: {:.1}%", reduction.reduction_rate * 100.0);

            Ok(())
        }
        Commands::Export { directory, output, format } => {
            println!("📊 グラフをエクスポート中: {}", directory.display());
            println!("📁 出力先: {}", output.display());
            println!("📋 形式: {}\n", format);

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // Scan directory
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            println!("📊 グラフ統計:");
            println!("  ノード数: {}", graph.nodes.len());
            println!("  エッジ数: {}", graph.edges.len());
            println!();

            // Export based on format
            match format.to_lowercase().as_str() {
                "graphml" => {
                    exporter::GraphExporter::export_graphml(&graph, &output)?;
                    println!("✅ GraphML形式でエクスポート完了");
                    println!("💡 Gephi、yEd、Cytoscapeで開けます");
                }
                "dot" => {
                    exporter::GraphExporter::export_dot(&graph, &output)?;
                    println!("✅ DOT形式でエクスポート完了");
                    println!("💡 Graphvizで可視化:");
                    println!("   dot -Tsvg {} -o graph.svg", output.display());
                    println!("   neato -Tpng {} -o graph.png", output.display());
                }
                "json" => {
                    exporter::GraphExporter::export_json(&graph, &output)?;
                    println!("✅ JSON形式でエクスポート完了");
                    println!("💡 カスタム可視化ツールで使用できます");
                }
                _ => {
                    eprintln!("❌ 未対応の形式: {}", format);
                    eprintln!("💡 対応形式: graphml, dot, json");
                    return Ok(());
                }
            }

            println!("\n📄 ファイルサイズ: {} bytes", std::fs::metadata(&output)?.len());

            Ok(())
        }
        #[cfg(feature = "web")]
        Commands::Visualize {
            directory,
            port,
            open,
        } => {
            println!("📊 依存関係グラフを可視化中: {}", directory.display());
            println!();

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // Scan directory
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;

            println!("📊 グラフ統計:");
            println!("  ノード数: {}", graph.nodes.len());
            println!("  エッジ数: {}", graph.edges.len());
            println!();

            // Open browser if requested
            if open {
                let url = format!("http://localhost:{}", port);
                println!("🌐 ブラウザを開いています: {}", url);
                #[cfg(feature = "web")]
                {
                    use std::process::Command;
                    let _ = Command::new("cmd")
                        .args(&["/C", "start", &url])
                        .spawn();
                }
            }

            // Start web server (requires tokio runtime)
            #[cfg(feature = "web")]
            {
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(async {
                    web_server::server::start_server(graph, port).await
                })?;
            }

            Ok(())
        }
    }
}
