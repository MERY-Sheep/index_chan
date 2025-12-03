// index-chanライブラリのエントリポイント
// Tauriアプリから使用するためのモジュールを公開

pub mod scanner;
pub mod parser;
pub mod graph;
pub mod detector;
pub mod annotator;
pub mod cleaner;
pub mod reporter;
pub mod filter;
pub mod exporter;
pub mod backup;
pub mod error_helper;

// データベース機能（オプション）
#[cfg(feature = "db")]
pub mod database;

// 会話分析機能
pub mod conversation;

// LLM機能（オプション）
pub mod llm;

// 検索機能
pub mod search;

// チャットグラフWebサーバー（オプション）
pub mod chat_server;

// MCP Server (Phase 6)
pub mod mcp;

// 再エクスポート
pub use scanner::Scanner;
pub use detector::{detect_dead_code, DeadCode, SafetyLevel};
pub use annotator::{Annotator, AnnotationResult};
pub use cleaner::{Cleaner, CleanResult};
pub use graph::{CodeGraph, CodeNode};
