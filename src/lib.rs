// index-chanライブラリのエントリポイント
// Tauriアプリから使用するためのモジュールを公開

pub mod annotator;
pub mod backup;
pub mod cleaner;
pub mod detector;
pub mod error_helper;
pub mod exporter;
pub mod filter;
pub mod graph;
pub mod parser;
pub mod reporter;
pub mod scanner;

// データベース機能（オプション）
#[cfg(feature = "db")]
pub mod database;

// 会話分析機能
pub mod conversation;

// LLM機能（オプション）
pub mod llm;

// 検索機能
pub mod search;

// 埋め込み機能 (Phase 7 GraphRAG)
pub mod embedding;
pub mod embedding_cache;

// チャットグラフWebサーバー（オプション）
pub mod chat_server;

// MCP Server (Phase 6)
pub mod mcp;

// 再エクスポート
pub use annotator::{AnnotationResult, Annotator};
pub use cleaner::{CleanResult, Cleaner};
pub use detector::{detect_dead_code, DeadCode, SafetyLevel};
pub use graph::{CodeGraph, CodeNode, EdgeType, SemanticRelationType};
pub use scanner::Scanner;
