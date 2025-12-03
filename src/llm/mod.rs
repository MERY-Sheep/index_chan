// Phase 1.5: LLM統合モジュール（Gemini API使用）
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod analyzer;
pub mod config;
pub mod context;
pub mod function_calling;
pub mod gemini;

pub use analyzer::LLMAnalyzer;
pub use config::LLMConfig;
pub use context::ContextCollector;
pub use function_calling::{Tool, FunctionCall, FunctionResponse, IndexChanTool, create_index_chan_tools};
pub use gemini::{GeminiClient, GeminiResult, Content, Part};
