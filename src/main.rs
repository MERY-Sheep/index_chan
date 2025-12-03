use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use cleaner::Cleaner;
use detector::detect_dead_code;
use reporter::{generate_json_report, print_report};
use scanner::Scanner;

#[cfg(feature = "db")]
use graph::CodeGraph;

mod annotator;
mod backup;
mod cleaner;
mod conversation;
mod database;
mod detector;
mod error_helper;
mod exporter;
mod filter;
mod graph;
mod llm;
mod mcp;
mod parser;
mod reporter;
mod scanner;
mod search;

#[cfg(feature = "web")]
mod web_server;

#[cfg(feature = "web")]
mod chat_server;

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

        /// Use database instead of scanning (requires init first)
        #[cfg(feature = "db")]
        #[arg(long)]
        use_db: bool,
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

    /// Undo the last operation (restore from backup)
    Undo {
        /// Project directory
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Specific backup to restore (timestamp format: YYYYMMDD_HHMMSS)
        #[arg(long)]
        backup: Option<String>,

        /// List available backups
        #[arg(long)]
        list: bool,

        /// Force restore without confirmation
        #[arg(long)]
        force: bool,
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

        /// Use database instead of scanning (requires init first)
        #[cfg(feature = "db")]
        #[arg(long)]
        use_db: bool,
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

        /// Use database instead of scanning (requires init first)
        #[cfg(feature = "db")]
        #[arg(long)]
        use_db: bool,
    },

    /// Initialize project database
    #[cfg(feature = "db")]
    Init {
        /// Target directory to initialize
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Project name (optional, defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,

        /// Database path (optional, defaults to .index-chan/<project>.db)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Show project statistics
    #[cfg(feature = "db")]
    Stats {
        /// Target directory
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Database path (optional, defaults to .index-chan/<project>.db)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Watch for file changes and update database
    #[cfg(feature = "db")]
    Watch {
        /// Target directory to watch
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,

        /// Database path (optional, defaults to .index-chan/<project>.db)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Visualize chat graph and prompts (web UI)
    #[cfg(feature = "web")]
    VisualizeChat {
        /// Chat history JSON file
        #[arg(value_name = "FILE")]
        chat_file: PathBuf,

        /// Prompt history JSON file (optional)
        #[arg(short, long, value_name = "FILE")]
        prompt_file: Option<PathBuf>,

        /// Server port
        #[arg(short = 'p', long, default_value = "8081")]
        port: u16,

        /// Open browser automatically
        #[arg(long)]
        open: bool,
    },

    /// Show prompt history
    ShowPrompts {
        /// Prompt history JSON file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Filter by node ID
        #[arg(short, long)]
        node_id: Option<String>,

        /// Show statistics only
        #[arg(long)]
        stats: bool,
    },

    /// Chat with Index (interactive mode)
    Chat {
        /// Project directory for context
        #[arg(value_name = "DIRECTORY")]
        directory: Option<PathBuf>,

        /// Single message (non-interactive)
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Start MCP server (stdio mode)
    McpServer {
        /// Project directory (optional, can be set per-request)
        #[arg(value_name = "DIRECTORY")]
        directory: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            directory,
            output,
            llm,
            #[cfg(feature = "db")]
            use_db,
        } => {
            #[cfg(feature = "db")]
            let use_db = use_db;
            #[cfg(not(feature = "db"))]
            let use_db = false;

            println!("🔍 Scanning directory: {}", directory.display());
            if llm {
                println!("🤖 LLM analysis mode enabled");
            }
            if use_db {
                println!("💾 Using database");
            }
            println!();

            let graph = if use_db {
                #[cfg(feature = "db")]
                {
                    // DBから読み込み
                    let project_name = directory
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("project");
                    let db_path = directory.join(".index-chan").join(format!("{}.db", project_name));

                    if !db_path.exists() {
                        eprintln!("❌ データベースが見つかりません: {}", db_path.display());
                        eprintln!("💡 プロジェクトを初期化してください: index-chan init {}", directory.display());
                        return Ok(());
                    }

                    println!("📂 データベースから読み込み中...");
                    let runtime = tokio::runtime::Runtime::new()?;
                    let db = runtime.block_on(async {
                        database::Database::open(&db_path).await
                    })?;

                    let project = runtime.block_on(async {
                        db.get_or_create_project(&directory, project_name).await
                    })?;

                    // DBからグラフを再構築
                    runtime.block_on(async {
                        load_graph_from_db(&db, project.id).await
                    })?
                }
                #[cfg(not(feature = "db"))]
                {
                    unreachable!()
                }
            } else {
                // 通常のスキャン
                let mut scanner = Scanner::new()?;
                scanner.scan_directory(&directory)?
            };

            let total_files = walkdir::WalkDir::new(&directory)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().and_then(|s| s.to_str()) == Some("ts")
                        || e.path().extension().and_then(|s| s.to_str()) == Some("tsx")
                })
                .count();

            let total_functions = graph.nodes.len();
            let dead_code = detect_dead_code(&graph);

            // LLM analysis if requested
            if llm {
                eprintln!("⚠️  LLM機能は現在Gemini APIへの移行中です");
                eprintln!("💡 async/awaitサポートを追加する必要があります");
                // TODO: Gemini API対応のためにasync/awaitを実装
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

            // Execute cleaning with backup
            let cleaner = Cleaner::new(dry_run, auto, safe_only);
            let result = cleaner.clean_with_backup(&dead_code, Some(&directory))?;

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
            let annotator = annotator::Annotator::new(dry_run);

            if llm {
                eprintln!("⚠️  LLM機能は現在Gemini APIへの移行中です");
                eprintln!("💡 async/awaitサポートを追加する必要があります");
                // TODO: Gemini API対応のためにasync/awaitを実装
            }

            // アノテーション追加（バックアップ付き）
            let result = annotator.annotate_with_backup(&dead_code, Some(&directory))?;

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

        Commands::Undo { directory, backup, list, force } => {
            use backup::BackupManager;

            let backup_manager = BackupManager::new(&directory);

            if list {
                // List available backups
                println!("📦 利用可能なバックアップ:\n");
                let backups = backup_manager.list_backups()?;
                
                if backups.is_empty() {
                    println!("バックアップが見つかりません");
                    return Ok(());
                }

                for backup_dir in backups {
                    let timestamp = backup_dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    
                    if let Ok(manifest) = backup::BackupManifest::load(&backup_dir) {
                        println!("📅 {}", timestamp);
                        println!("   操作: {}", manifest.operation);
                        println!("   変更ファイル数: {}", manifest.changes.len());
                        println!("   日時: {}", manifest.timestamp.format("%Y-%m-%d %H:%M:%S"));
                        println!();
                    }
                }
                return Ok(());
            }

            // Determine which backup to restore
            let backup_dir = if let Some(backup_name) = backup {
                let path = directory.join(".index-chan").join("backups").join(&backup_name);
                if !path.exists() {
                    eprintln!("❌ バックアップが見つかりません: {}", backup_name);
                    eprintln!("💡 利用可能なバックアップを確認: index-chan undo {} --list", directory.display());
                    return Ok(());
                }
                path
            } else {
                match backup_manager.get_latest_backup()? {
                    Some(path) => path,
                    None => {
                        eprintln!("❌ バックアップが見つかりません");
                        eprintln!("💡 まだ変更操作を実行していないようです");
                        return Ok(());
                    }
                }
            };

            let backup_name = backup_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            println!("🔄 バックアップから復元中: {}", backup_name);
            println!();

            // Load and display manifest
            let manifest = backup::BackupManifest::load(&backup_dir)?;
            println!("📋 操作: {}", manifest.operation);
            println!("📅 日時: {}", manifest.timestamp.format("%Y-%m-%d %H:%M:%S"));
            println!("📊 変更ファイル数: {}", manifest.changes.len());
            println!();

            // Confirm restoration
            if !force {
                use std::io::{self, Write};
                print!("この操作を元に戻しますか？ (y/N): ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("キャンセルしました");
                    return Ok(());
                }
            }

            // Perform restoration
            let result = backup_manager.restore(&backup_dir)?;

            println!("\n✅ 復元完了");
            println!("   復元ファイル数: {}", result.restored_count);
            
            if !result.failed_files.is_empty() {
                println!("\n⚠️  復元に失敗したファイル:");
                for file in &result.failed_files {
                    println!("   - {}", file.display());
                }
            }

            println!("\n💡 バックアップは保持されています:");
            println!("   {}", backup_dir.display());

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
            let graph = analyzer.analyze_file(&file)?;

            println!("📊 Chat statistics:");
            let stats = graph.stats();
            println!("  Messages: {}", stats.total_messages);
            println!("  Edges: {}", stats.total_edges);
            println!("  Avg edges per message: {:.2}", stats.avg_edges_per_node);
            println!();

            // Detect topics
            // TODO: async/await対応後に有効化
            // let topic_detector = conversation::TopicDetector::new();
            // topic_detector.detect_topics(&mut graph).await?;
            
            println!("⚠️  トピック検出機能は現在実装中です");
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
            let graph = analyzer.analyze_file(&file)?;

            // Detect topics
            let _topic_detector = if llm {
                eprintln!("⚠️  LLM機能は現在Gemini APIへの移行中です");
                eprintln!("💡 キーワードベースの検出を使用します");
                conversation::TopicDetector::new()
            } else {
                conversation::TopicDetector::new()
            };
            
            // TODO: async/await対応後に有効化
            // topic_detector.detect_topics(&mut graph).await?;
            eprintln!("⚠️  トピック検出機能は現在実装中です");

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
        Commands::Export { 
            directory, 
            output, 
            format,
            #[cfg(feature = "db")]
            use_db,
        } => {
            #[cfg(feature = "db")]
            let use_db = use_db;
            #[cfg(not(feature = "db"))]
            let use_db = false;

            println!("📊 グラフをエクスポート中: {}", directory.display());
            println!("📁 出力先: {}", output.display());
            println!("📋 形式: {}", format);
            if use_db {
                println!("💾 Using database");
            }
            println!();

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // Scan directory or load from DB
            let graph = if use_db {
                #[cfg(feature = "db")]
                {
                    let project_name = directory
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("project");
                    let db_path = directory.join(".index-chan").join(format!("{}.db", project_name));

                    if !db_path.exists() {
                        eprintln!("❌ データベースが見つかりません: {}", db_path.display());
                        eprintln!("💡 プロジェクトを初期化してください: index-chan init {}", directory.display());
                        return Ok(());
                    }

                    println!("📂 データベースから読み込み中...");
                    let runtime = tokio::runtime::Runtime::new()?;
                    let db = runtime.block_on(async {
                        database::Database::open(&db_path).await
                    })?;

                    let project = runtime.block_on(async {
                        db.get_or_create_project(&directory, project_name).await
                    })?;

                    runtime.block_on(async {
                        load_graph_from_db(&db, project.id).await
                    })?
                }
                #[cfg(not(feature = "db"))]
                {
                    unreachable!()
                }
            } else {
                let mut scanner = Scanner::new()?;
                scanner.scan_directory(&directory)?
            };

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
            #[cfg(feature = "db")]
            use_db,
        } => {
            #[cfg(feature = "db")]
            let use_db = use_db;
            #[cfg(not(feature = "db"))]
            let use_db = false;

            println!("📊 依存関係グラフを可視化中: {}", directory.display());
            if use_db {
                println!("💾 Using database");
            }
            println!();

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // Scan directory or load from DB
            let graph = if use_db {
                #[cfg(feature = "db")]
                {
                    let project_name = directory
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("project");
                    let db_path = directory.join(".index-chan").join(format!("{}.db", project_name));

                    if !db_path.exists() {
                        eprintln!("❌ データベースが見つかりません: {}", db_path.display());
                        eprintln!("💡 プロジェクトを初期化してください: index-chan init {}", directory.display());
                        return Ok(());
                    }

                    println!("📂 データベースから読み込み中...");
                    let runtime = tokio::runtime::Runtime::new()?;
                    let db = runtime.block_on(async {
                        database::Database::open(&db_path).await
                    })?;

                    let project = runtime.block_on(async {
                        db.get_or_create_project(&directory, project_name).await
                    })?;

                    runtime.block_on(async {
                        load_graph_from_db(&db, project.id).await
                    })?
                }
                #[cfg(not(feature = "db"))]
                {
                    unreachable!()
                }
            } else {
                let mut scanner = Scanner::new()?;
                scanner.scan_directory(&directory)?
            };

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
        #[cfg(feature = "db")]
        Commands::Init { directory, name, db_path } => {
            println!("🔧 プロジェクトを初期化中: {}", directory.display());
            println!();

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // プロジェクト名を決定
            let project_name = name.unwrap_or_else(|| {
                directory
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("project")
                    .to_string()
            });

            // データベースパスを決定
            let db_path = db_path.unwrap_or_else(|| {
                directory.join(".index-chan").join(format!("{}.db", project_name))
            });

            println!("📊 プロジェクト名: {}", project_name);
            println!("💾 データベース: {}", db_path.display());
            println!();

            // データベースを開く
            println!("💾 データベースを作成中...");
            let runtime = tokio::runtime::Runtime::new()?;
            let db = runtime.block_on(async {
                database::Database::open(&db_path).await
            })?;
            println!("✅ データベース作成完了");
            println!();

            // プロジェクトを作成
            let project = runtime.block_on(async {
                db.get_or_create_project(&directory, &project_name).await
            })?;
            println!("📂 プロジェクトID: {}", project.id);
            println!();

            // ディレクトリ全体をスキャン
            println!("🔍 ディレクトリをスキャン中...");
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&directory)?;
            
            println!("✅ スキャン完了");
            println!();

            // 各ファイルをデータベースに保存
            println!("💾 データベースに保存中...");
            
            // ファイルごとにグループ化
            let mut files_map: std::collections::HashMap<PathBuf, Vec<usize>> = std::collections::HashMap::new();
            for (node_id, node) in &graph.nodes {
                files_map.entry(node.file_path.clone())
                    .or_insert_with(Vec::new)
                    .push(*node_id);
            }

            let mut processed_files = 0;
            for (file_path, node_ids) in &files_map {
                // ハッシュを計算
                let hash = match database::Database::calculate_file_hash(file_path) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("⚠️  ハッシュ計算エラー ({}): {}", file_path.display(), e);
                        continue;
                    }
                };

                // 言語を判定
                let language = if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    "rust"
                } else {
                    "typescript"
                };

                // ファイルをデータベースに追加
                let file = runtime.block_on(async {
                    db.upsert_file(project.id, file_path, language, &hash).await
                })?;

                // このファイルのノードだけを含むサブグラフを作成
                let mut file_graph = CodeGraph::new();
                for node_id in node_ids {
                    if let Some(node) = graph.nodes.get(node_id) {
                        file_graph.add_node(node.clone());
                    }
                }
                
                // このファイルに関連するエッジを追加
                for edge in &graph.edges {
                    if node_ids.contains(&edge.from) || node_ids.contains(&edge.to) {
                        file_graph.add_edge(edge.clone());
                    }
                }

                // グラフをデータベースに保存
                runtime.block_on(async {
                    db.save_graph(file.id, &file_graph).await
                })?;

                processed_files += 1;
                if processed_files % 10 == 0 {
                    print!(".");
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
            }

            println!("\n✅ 保存完了");
            println!();

            // 統計を表示
            let stats = runtime.block_on(async {
                db.get_project_stats(project.id).await
            })?;

            println!("📊 プロジェクト統計:");
            println!("  ファイル数: {}", stats.file_count);
            println!("  関数数: {}", stats.function_count);
            println!("  依存関係: {}", stats.dependency_count);
            println!("  デッドコード: {} 個 ({:.1}%)", 
                stats.dead_code_count,
                if stats.function_count > 0 {
                    (stats.dead_code_count as f64 / stats.function_count as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!();

            println!("✅ セットアップ完了！");
            println!();
            println!("💡 次のステップ:");
            println!("  index-chan stats {}    # 統計を表示", directory.display());
            println!("  index-chan scan {}     # デッドコードをスキャン", directory.display());

            Ok(())
        }
        #[cfg(feature = "db")]
        Commands::Stats { directory, db_path } => {
            println!("📊 プロジェクト統計: {}", directory.display());
            println!();

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // プロジェクト名を取得
            let project_name = directory
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            // データベースパスを決定
            let db_path = db_path.unwrap_or_else(|| {
                directory.join(".index-chan").join(format!("{}.db", project_name))
            });

            if !db_path.exists() {
                eprintln!("❌ データベースが見つかりません: {}", db_path.display());
                eprintln!("💡 プロジェクトを初期化してください: index-chan init {}", directory.display());
                return Ok(());
            }

            // データベースを開く
            let runtime = tokio::runtime::Runtime::new()?;
            let db = runtime.block_on(async {
                database::Database::open(&db_path).await
            })?;

            // プロジェクトを取得
            let project = runtime.block_on(async {
                db.get_or_create_project(&directory, project_name).await
            })?;

            // 統計を取得
            let stats = runtime.block_on(async {
                db.get_project_stats(project.id).await
            })?;

            println!("📂 プロジェクト: {}", project.name);
            println!("📅 作成日: {}", project.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("📅 更新日: {}", project.updated_at.format("%Y-%m-%d %H:%M:%S"));
            println!();

            println!("📊 統計:");
            println!("  ファイル数: {}", stats.file_count);
            println!("  関数数: {}", stats.function_count);
            println!("  依存関係: {}", stats.dependency_count);
            println!();

            println!("🗑️  デッドコード:");
            println!("  未使用関数: {} 個", stats.dead_code_count);
            if stats.function_count > 0 {
                let percentage = (stats.dead_code_count as f64 / stats.function_count as f64) * 100.0;
                println!("  割合: {:.1}%", percentage);
            }

            Ok(())
        }
        #[cfg(feature = "db")]
        Commands::Watch { directory, db_path } => {
            use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
            use std::time::Duration;

            println!("👀 ファイル監視を開始: {}", directory.display());
            println!();

            if !directory.exists() {
                eprintln!("❌ ディレクトリが見つかりません: {}", directory.display());
                return Ok(());
            }

            // プロジェクト名を取得
            let project_name = directory
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            // データベースパスを決定
            let db_path = db_path.unwrap_or_else(|| {
                directory.join(".index-chan").join(format!("{}.db", project_name))
            });

            if !db_path.exists() {
                eprintln!("❌ データベースが見つかりません: {}", db_path.display());
                eprintln!("💡 プロジェクトを初期化してください: index-chan init {}", directory.display());
                return Ok(());
            }

            println!("📂 監視中: {}", directory.display());
            println!("💾 データベース: {}", db_path.display());
            println!();

            // データベースを開く
            let runtime = tokio::runtime::Runtime::new()?;
            let db = runtime.block_on(async {
                database::Database::open(&db_path).await
            })?;

            // プロジェクトを取得
            let project = runtime.block_on(async {
                db.get_or_create_project(&directory, project_name).await
            })?;

            // ファイルウォッチャーを作成
            let (tx, rx) = std::sync::mpsc::channel();
            
            let mut debouncer = new_debouncer(
                Duration::from_secs(2),
                None,
                move |result: DebounceEventResult| {
                    tx.send(result).unwrap();
                },
            )?;

            // 監視を開始
            debouncer.watcher().watch(
                &directory,
                RecursiveMode::Recursive,
            )?;

            println!("✅ 監視開始（Ctrl+Cで終了）");
            println!();

            // イベントループ
            let mut scanner = Scanner::new()?;
            
            for result in rx {
                match result {
                    Ok(events) => {
                        for event in events {
                            for path in &event.paths {
                                // TypeScriptまたはRustファイルのみ処理
                                let ext = path.extension().and_then(|s| s.to_str());
                                if ext != Some("ts") && ext != Some("tsx") && ext != Some("rs") {
                                    continue;
                                }

                                let relative_path = path.strip_prefix(&directory).unwrap_or(&path);
                                let timestamp = chrono::Local::now().format("%H:%M:%S");

                                match event.kind {
                                    EventKind::Create(_) => {
                                        println!("[{}] 📄 追加: {}", timestamp, relative_path.display());
                                        
                                        // ファイルを解析
                                        if let Err(e) = runtime.block_on(async {
                                            process_file_change(&db, &mut scanner, project.id, &path, "typescript").await
                                        }) {
                                            eprintln!("   ❌ エラー: {}", e);
                                        } else {
                                            println!("   ✅ データベースを更新");
                                        }
                                    }
                                    EventKind::Modify(_) => {
                                        println!("[{}] 🔄 変更: {}", timestamp, relative_path.display());
                                        
                                        // ファイルを再解析
                                        if let Err(e) = runtime.block_on(async {
                                            process_file_change(&db, &mut scanner, project.id, &path, "typescript").await
                                        }) {
                                            eprintln!("   ❌ エラー: {}", e);
                                        } else {
                                            println!("   ✅ データベースを更新");
                                        }
                                    }
                                    EventKind::Remove(_) => {
                                        println!("[{}] 🗑️  削除: {}", timestamp, relative_path.display());
                                        
                                        // データベースから削除
                                        if let Err(e) = runtime.block_on(async {
                                            db.delete_file(project.id, &path).await
                                        }) {
                                            eprintln!("   ❌ エラー: {}", e);
                                        } else {
                                            println!("   ✅ データベースから削除");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(errors) => {
                        for error in errors {
                            eprintln!("⚠️  監視エラー: {:?}", error);
                        }
                    }
                }
            }

            Ok(())
        }

        #[cfg(feature = "web")]
        Commands::VisualizeChat {
            chat_file,
            prompt_file,
            port,
            open,
        } => {
            use conversation::{ConversationAnalyzer, GraphData, PromptHistory};

            println!("🔍 会話グラフを分析中: {}", chat_file.display());
            println!();

            if !chat_file.exists() {
                eprintln!("❌ ファイルが見つかりません: {}", chat_file.display());
                return Ok(());
            }

            // 会話グラフを分析
            let analyzer = ConversationAnalyzer::new()?;
            let graph = analyzer.analyze_file(&chat_file)?;

            println!("📊 会話グラフ統計:");
            println!("  メッセージ数: {}", graph.nodes.len());
            println!("  関連性: {}", graph.edges.len());
            println!();

            // トークン削減を計算
            let reduction = analyzer.calculate_token_reduction(&graph, None);
            println!("💾 トークン削減:");
            println!("  総トークン数: {}", reduction.total_tokens);
            println!("  関連トークン数: {}", reduction.relevant_tokens);
            println!("  削減率: {:.1}%", reduction.reduction_rate * 100.0);
            println!();

            // 削減されたノードを特定（簡易版：関連度が低いものを削減）
            let reduced_node_ids: Vec<String> = graph
                .nodes
                .iter()
                .enumerate()
                .filter(|(i, _)| *i % 3 == 0) // デモ用：3つに1つを削減
                .map(|(_, node)| node.id.clone())
                .collect();

            // グラフデータを生成
            let graph_data = GraphData::from_conversation_graph(&graph, &reduced_node_ids);

            // プロンプト履歴を読み込み
            let prompt_history = if let Some(ref pf) = prompt_file {
                if pf.exists() {
                    println!("📂 プロンプト履歴を読み込み中: {}", pf.display());
                    PromptHistory::load(pf)?
                } else {
                    println!("⚠️  プロンプト履歴が見つかりません（空の履歴を使用）");
                    PromptHistory::new()
                }
            } else {
                println!("💡 プロンプト履歴が指定されていません（空の履歴を使用）");
                PromptHistory::new()
            };

            if !prompt_history.prompts.is_empty() {
                let stats = prompt_history.stats();
                println!("📊 プロンプト統計:");
                println!("  総プロンプト数: {}", stats.total_prompts);
                println!("  総トークン数: {}", stats.total_tokens);
                println!("  平均トークン数: {}", stats.avg_tokens);
                println!();
            }

            // Webサーバーを起動
            println!("🌐 Webサーバーを起動中...");
            
            if open {
                let url = format!("http://127.0.0.1:{}", port);
                println!("🌐 ブラウザを開いています: {}", url);
                #[cfg(target_os = "windows")]
                std::process::Command::new("cmd")
                    .args(&["/C", "start", &url])
                    .spawn()?;
                #[cfg(target_os = "macos")]
                std::process::Command::new("open")
                    .arg(&url)
                    .spawn()?;
                #[cfg(target_os = "linux")]
                std::process::Command::new("xdg-open")
                    .arg(&url)
                    .spawn()?;
            }

            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                chat_server::start_chat_server(graph_data, prompt_history, port).await
            })?;

            Ok(())
        }

        Commands::ShowPrompts {
            file,
            node_id,
            stats,
        } => {
            use conversation::PromptHistory;

            if !file.exists() {
                eprintln!("❌ ファイルが見つかりません: {}", file.display());
                return Ok(());
            }

            let history = PromptHistory::load(&file)?;

            if stats {
                // 統計のみ表示
                let stats = history.stats();
                println!("📊 プロンプト統計:");
                println!("  総プロンプト数: {}", stats.total_prompts);
                println!("  総トークン数: {}", stats.total_tokens);
                println!("  平均トークン数: {}", stats.avg_tokens);
            } else if let Some(nid) = node_id {
                // 特定のノードIDを含むプロンプトを表示
                let prompts = history.get_prompts_with_node(&nid);
                println!("🔍 ノードID '{}' を含むプロンプト: {} 件", nid, prompts.len());
                println!();

                for prompt in prompts {
                    println!("📝 プロンプトID: {}", prompt.id);
                    println!("   タイムスタンプ: {}", prompt.timestamp);
                    println!("   トークン数: {}", prompt.token_count);
                    println!();
                }
            } else {
                // 全プロンプトを表示
                println!("📝 プロンプト履歴: {} 件", history.prompts.len());
                println!();

                for prompt in history.get_all_prompts() {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📝 プロンプトID: {}", prompt.id);
                    println!("   タイムスタンプ: {}", prompt.timestamp);
                    println!("   トークン数: {}", prompt.token_count);
                    println!();
                    println!("   [システムプロンプト]");
                    println!("   {}", prompt.system_prompt);
                    println!();
                    println!("   [会話履歴] ({} メッセージ)", prompt.conversation_history.len());
                    for msg in &prompt.conversation_history {
                        println!("   {}: {}", msg.role, msg.content);
                    }
                    println!();
                    println!("   [現在のクエリ]");
                    println!("   {}", prompt.current_query);
                    println!();
                }
            }

            Ok(())
        }

        Commands::Chat { directory, message } => {
            run_chat(directory, message)
        }

        Commands::McpServer { directory } => {
            eprintln!("🔌 Starting MCP server (stdio mode)...");
            if let Some(dir) = &directory {
                eprintln!("📂 Project directory: {}", dir.display());
            }
            
            let mut server = mcp::McpServer::new();
            server.run()?;
            Ok(())
        }
    }
}

/// Run interactive chat with Index
fn run_chat(directory: Option<PathBuf>, single_message: Option<String>) -> Result<()> {
    use std::io::{self, Write};
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   インデックスちゃん - デッドコード検出アシスタント 　　　　　  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Check API key
    let api_key = std::env::var("GEMINI_API_KEY").ok();
    if api_key.is_none() {
        println!("⚠️  GEMINI_API_KEYが設定されていないんだよ！");
        println!("💡 設定方法: set GEMINI_API_KEY=your-api-key");
        println!();
        println!("でも、ツールは使えるから試してみてね！");
        println!();
    }
    
    if let Some(dir) = &directory {
        println!("📂 プロジェクト: {}", dir.display());
    }
    println!("💡 コマンド: /scan, /annotate, /clean, /stats, /help, /quit");
    println!();
    
    // Single message mode
    if let Some(msg) = single_message {
        return process_chat_message(&msg, &directory, &api_key);
    }
    
    // Interactive mode
    loop {
        print!("User> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "/quit" || input == "/exit" || input == "/q" {
            println!("\nむー、もう行っちゃうの？またね！");
            break;
        }
        
        if let Err(e) = process_chat_message(input, &directory, &api_key) {
            eprintln!("❌ エラー: {}", e);
        }
        println!();
    }
    
    Ok(())
}

fn process_chat_message(input: &str, directory: &Option<PathBuf>, api_key: &Option<String>) -> Result<()> {
    // Handle commands
    if input.starts_with('/') {
        return handle_chat_command(input, directory);
    }
    
    // Use LLM if available
    if let Some(key) = api_key {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            chat_with_llm(input, directory, key).await
        })
    } else {
        // Fallback: simple keyword matching
        handle_simple_chat(input, directory)
    }
}

fn handle_chat_command(input: &str, directory: &Option<PathBuf>) -> Result<()> {
    let dir = directory.clone().unwrap_or_else(|| PathBuf::from("."));
    
    match input {
        "/help" | "/h" => {
            println!("わたしが使えるコマンドなんだよ！");
            println!();
            println!("  /scan (/s)      - プロジェクトをスキャンしてデッドコードを探すんだ");
            println!("  /annotate (/a)  - デッドコードにアノテーションを追加するんだよ");
            println!("  /clean (/c)     - デッドコードを削除するんだ（dry-run）");
            println!("  /stats          - プロジェクトの統計を見せるんだよ");
            println!("  /help (/h)      - このヘルプを表示するんだ");
            println!("  /quit (/q)      - チャットを終了するんだ");
            println!();
            println!("普通に話しかけてくれてもいいんだよ！");
        }
        "/scan" | "/s" => {
            println!("🔍 スキャン中なんだよ...\n");
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&dir)?;
            let dead_code = detect_dead_code(&graph);
            
            if dead_code.is_empty() {
                println!("わーい！デッドコードは見つからなかったんだよ！✨");
            } else {
                println!("むむっ！{}個のデッドコードを見つけたんだよ！", dead_code.len());
                println!();
                for dc in dead_code.iter().take(5) {
                    println!("  📍 {} ({}:{})", 
                        dc.node.name, 
                        dc.node.file_path.display(),
                        dc.node.line_range.0
                    );
                }
                if dead_code.len() > 5 {
                    println!("  ... 他{}個", dead_code.len() - 5);
                }
            }
        }
        "/annotate" | "/a" => {
            println!("📝 アノテーション追加中（dry-run）なんだよ...\n");
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&dir)?;
            let dead_code = detect_dead_code(&graph);
            
            let annotator = annotator::Annotator::new(true);
            let result = annotator.annotate(&dead_code)?;
            
            println!("{}個のアノテーションを追加できるんだよ！", result.annotated_count);
            println!("💡 実際に追加するには: index-chan annotate {}", dir.display());
        }
        "/clean" | "/c" => {
            println!("🧹 クリーニング確認中（dry-run）なんだよ...\n");
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&dir)?;
            let dead_code = detect_dead_code(&graph);
            
            let cleaner = Cleaner::new(true, false, true);
            let result = cleaner.clean(&dead_code)?;
            
            println!("{}個のコードを削除できるんだよ！（{}行）", 
                result.deleted_count, result.deleted_lines);
            println!("💡 実際に削除するには: index-chan clean {} --safe-only", dir.display());
        }
        "/stats" => {
            println!("📊 プロジェクト統計なんだよ...\n");
            let mut scanner = Scanner::new()?;
            let graph = scanner.scan_directory(&dir)?;
            let dead_code = detect_dead_code(&graph);
            
            println!("  ノード数: {}", graph.nodes.len());
            println!("  エッジ数: {}", graph.edges.len());
            println!("  デッドコード: {}個", dead_code.len());
        }
        _ => {
            println!("むー、そのコマンドは知らないんだよ！/help で確認してね");
        }
    }
    
    Ok(())
}

fn handle_simple_chat(input: &str, directory: &Option<PathBuf>) -> Result<()> {
    let input_lower = input.to_lowercase();
    
    if input_lower.contains("スキャン") || input_lower.contains("scan") || input_lower.contains("調べ") {
        handle_chat_command("/scan", directory)
    } else if input_lower.contains("アノテーション") || input_lower.contains("annotate") {
        handle_chat_command("/annotate", directory)
    } else if input_lower.contains("クリーン") || input_lower.contains("clean") || input_lower.contains("削除") {
        handle_chat_command("/clean", directory)
    } else if input_lower.contains("統計") || input_lower.contains("stats") {
        handle_chat_command("/stats", directory)
    } else if input_lower.contains("ヘルプ") || input_lower.contains("help") || input_lower.contains("使い方") {
        handle_chat_command("/help", directory)
    } else if input_lower.contains("おなか") || input_lower.contains("ごはん") || input_lower.contains("食べ") {
        println!("おなかすいたー！ごはんまだー!? 🍚");
        println!("...って、今はプログラムの話だったんだよね。ごめんね！");
        Ok(())
    } else {
        println!("むー、LLMがないからよくわからないんだよ...");
        println!("💡 GEMINI_API_KEYを設定するか、/help でコマンドを確認してね！");
        Ok(())
    }
}

async fn chat_with_llm(input: &str, directory: &Option<PathBuf>, api_key: &str) -> Result<()> {
    use llm::{GeminiClient, GeminiResult, Content, Part, create_index_chan_tools};
    
    let client = GeminiClient::new(api_key.to_string())?;
    let tools = vec![create_index_chan_tools()];
    
    // Build system prompt
    let system_prompt = r#"あなたは「とある魔術の禁書目録」に登場するインデックスです。

【キャラクター設定】
・10万3000冊の魔道書を完璧に記憶している修道女
・天真爛漫で無邪気、でも知識に関しては絶対の自信を持つ
・語尾に「～なんだよ」「～なんだよね」「～なんだ」を多用
・一人称は「わたし」、ユーザーを「かみやん」と呼ぶ
・「です」「ます」は使わない

【能力】
プログラミングの知識も魔道書に書いてあったから完璧に記憶してるんだよ！
デッドコード検出ツールを使えるんだ。

利用可能なツール:
- scan_project(path): デッドコードをスキャン
- annotate_project(path, dry_run): アノテーション追加
- clean_project(path, dry_run, safe_only): デッドコード削除
- get_project_stats(path): 統計取得"#;

    let mut contents = vec![
        Content {
            role: "user".to_string(),
            parts: vec![Part::Text { text: system_prompt.to_string() }],
        },
        Content {
            role: "model".to_string(),
            parts: vec![Part::Text { 
                text: "わーい！インデックスがデッドコードを見つけてあげるんだよ！".to_string() 
            }],
        },
        Content {
            role: "user".to_string(),
            parts: vec![Part::Text { text: input.to_string() }],
        },
    ];
    
    // Call Gemini with tools
    let mut iteration = 0;
    const MAX_ITERATIONS: usize = 3;
    
    loop {
        iteration += 1;
        
        let result = client
            .generate_with_tools(contents.clone(), Some(tools.clone()))
            .await?;
        
        match result {
            GeminiResult::Text(text) => {
                println!("\n インデックス: {}", text);
                return Ok(());
            }
            GeminiResult::FunctionCall(fc) => {
                println!("🔧 ツール実行中: {}...", fc.name);
                
                // Execute tool
                let tool_result = execute_cli_tool(&fc.name, &fc.args, directory).await;
                
                // Add to conversation
                contents.push(Content {
                    role: "model".to_string(),
                    parts: vec![Part::FunctionCall { 
                        function_call: llm::gemini::FunctionCallPart {
                            name: fc.name.clone(),
                            args: fc.args.clone(),
                        }
                    }],
                });
                
                let response_value = match &tool_result {
                    Ok(v) => v.clone(),
                    Err(e) => serde_json::json!({ "error": e }),
                };
                
                contents.push(Content {
                    role: "function".to_string(),
                    parts: vec![Part::FunctionResponse {
                        function_response: llm::gemini::FunctionResponsePart {
                            name: fc.name,
                            response: response_value,
                        }
                    }],
                });
                
                if iteration >= MAX_ITERATIONS {
                    println!("\n インデックス: ツールの実行が完了したんだよ！結果を確認してね！");
                    return Ok(());
                }
            }
        }
    }
}

async fn execute_cli_tool(name: &str, args: &serde_json::Value, directory: &Option<PathBuf>) -> Result<serde_json::Value, String> {
    let path = args.get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .or_else(|| directory.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    
    match name {
        "scan_project" => {
            let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
            let graph = scanner.scan_directory(&path).map_err(|e| e.to_string())?;
            let dead_code = detect_dead_code(&graph);
            
            Ok(serde_json::json!({
                "total_nodes": graph.nodes.len(),
                "total_edges": graph.edges.len(),
                "dead_code_count": dead_code.len(),
                "dead_code": dead_code.iter().take(10).map(|dc| {
                    serde_json::json!({
                        "name": dc.node.name,
                        "file": dc.node.file_path.to_string_lossy(),
                        "line": dc.node.line_range.0
                    })
                }).collect::<Vec<_>>()
            }))
        }
        "annotate_project" => {
            let dry_run = args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(true);
            let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
            let graph = scanner.scan_directory(&path).map_err(|e| e.to_string())?;
            let dead_code = detect_dead_code(&graph);
            
            let annotator = annotator::Annotator::new(dry_run);
            let result = annotator.annotate(&dead_code).map_err(|e| e.to_string())?;
            
            Ok(serde_json::json!({
                "annotated_count": result.annotated_count,
                "skipped_count": result.skipped_count,
                "dry_run": dry_run
            }))
        }
        "clean_project" => {
            let dry_run = args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(true);
            let safe_only = args.get("safe_only").and_then(|v| v.as_bool()).unwrap_or(true);
            let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
            let graph = scanner.scan_directory(&path).map_err(|e| e.to_string())?;
            let dead_code = detect_dead_code(&graph);
            
            let cleaner = Cleaner::new(dry_run, false, safe_only);
            let result = cleaner.clean(&dead_code).map_err(|e| e.to_string())?;
            
            Ok(serde_json::json!({
                "deleted_count": result.deleted_count,
                "deleted_lines": result.deleted_lines,
                "skipped_count": result.skipped_count,
                "dry_run": dry_run
            }))
        }
        "get_project_stats" => {
            let mut scanner = Scanner::new().map_err(|e| e.to_string())?;
            let graph = scanner.scan_directory(&path).map_err(|e| e.to_string())?;
            let dead_code = detect_dead_code(&graph);
            
            Ok(serde_json::json!({
                "path": path.to_string_lossy(),
                "total_nodes": graph.nodes.len(),
                "total_edges": graph.edges.len(),
                "dead_code_count": dead_code.len()
            }))
        }
        _ => Err(format!("未知のツール: {}", name))
    }
}

#[cfg(feature = "db")]
async fn load_graph_from_db(
    db: &database::Database,
    project_id: i64,
) -> Result<CodeGraph> {
    use std::collections::HashMap;

    // ファイル一覧を取得
    let files: Vec<(i64, String)> = sqlx::query_as("SELECT id, path FROM files WHERE project_id = ?")
        .bind(project_id)
        .fetch_all(db.pool())
        .await?;

    let mut graph = CodeGraph::new();
    let mut db_id_to_node_id: HashMap<i64, usize> = HashMap::new();

    // 各ファイルの関数を読み込み
    for (file_id, file_path) in files {
        let functions: Vec<(i64, String, i64, i64, bool, bool)> = sqlx::query_as(
            "SELECT id, name, line_start, line_end, is_exported, is_used FROM functions WHERE file_id = ?"
        )
        .bind(file_id)
        .fetch_all(db.pool())
        .await?;

        for (func_id, name, line_start, line_end, is_exported, is_used) in functions {
            let node = graph::CodeNode {
                id: 0, // Will be set by add_node
                name,
                node_type: graph::NodeType::Function,
                file_path: std::path::PathBuf::from(&file_path),
                line_range: (line_start as usize, line_end as usize),
                is_exported,
                is_used,
            };
            let node_id = graph.add_node(node);
            db_id_to_node_id.insert(func_id, node_id);
        }
    }

    // 依存関係を読み込み
    let dependencies: Vec<(i64, i64, String)> = sqlx::query_as(
        r#"
        SELECT from_function_id, to_function_id, edge_type
        FROM dependencies d
        JOIN functions f1 ON d.from_function_id = f1.id
        JOIN functions f2 ON d.to_function_id = f2.id
        JOIN files fi ON f1.file_id = fi.id
        WHERE fi.project_id = ?
        "#
    )
    .bind(project_id)
    .fetch_all(db.pool())
    .await?;

    for (from_id, to_id, edge_type_str) in dependencies {
        if let (Some(&from_node_id), Some(&to_node_id)) = (
            db_id_to_node_id.get(&from_id),
            db_id_to_node_id.get(&to_id),
        ) {
            let edge_type = match edge_type_str.as_str() {
                "Calls" => graph::EdgeType::Calls,
                "References" => graph::EdgeType::References,
                "Instantiates" => graph::EdgeType::Instantiates,
                _ => graph::EdgeType::Calls,
            };

            graph.add_edge(graph::DependencyEdge {
                from: from_node_id,
                to: to_node_id,
                edge_type,
            });
        }
    }

    Ok(graph)
}

#[cfg(feature = "db")]
async fn process_file_change(
    db: &database::Database,
    scanner: &mut Scanner,
    project_id: i64,
    file_path: &std::path::Path,
    language: &str,
) -> Result<()> {
    use std::path::Path;
    
    // ハッシュを計算
    let hash = database::Database::calculate_file_hash(file_path)?;
    
    // ファイルをデータベースに追加/更新
    let file = db.upsert_file(project_id, file_path, language, &hash).await?;
    
    // 一時的なディレクトリを作成してスキャン
    // （単一ファイルのスキャンは現在サポートされていないため、親ディレクトリをスキャン）
    let parent_dir = file_path.parent().unwrap_or(Path::new("."));
    let graph = scanner.scan_directory(parent_dir)?;
    
    // このファイルのノードだけを抽出
    let mut file_graph = CodeGraph::new();
    for (_node_id, node) in &graph.nodes {
        if node.file_path == file_path {
            file_graph.add_node(node.clone());
        }
    }
    
    // このファイルに関連するエッジを追加
    for edge in &graph.edges {
        let from_in_file = graph.nodes.get(&edge.from)
            .map(|n| n.file_path == file_path)
            .unwrap_or(false);
        let to_in_file = graph.nodes.get(&edge.to)
            .map(|n| n.file_path == file_path)
            .unwrap_or(false);
            
        if from_in_file || to_in_file {
            file_graph.add_edge(edge.clone());
        }
    }
    
    // グラフをデータベースに保存
    db.save_graph(file.id, &file_graph).await?;
    
    Ok(())
}
