// Gemini Function Calling support for index-chan tools
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool definition for Gemini Function Calling
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: serde_json::Map<String, Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

/// Function call from Gemini
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: Value,
}

/// Function response to send back to Gemini
#[derive(Debug, Clone, Serialize)]
pub struct FunctionResponse {
    pub name: String,
    pub response: Value,
}

/// Available tools for index-chan
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexChanTool {
    ScanProject,
    AnnotateProject,
    CleanProject,
    GetProjectStats,
}

impl IndexChanTool {
    /// Get all available tools
    pub fn all() -> Vec<Self> {
        vec![
            Self::ScanProject,
            Self::AnnotateProject,
            Self::CleanProject,
            Self::GetProjectStats,
        ]
    }

    /// Convert to function declaration
    pub fn to_declaration(&self) -> FunctionDeclaration {
        match self {
            Self::ScanProject => FunctionDeclaration {
                name: "scan_project".to_string(),
                description: "プロジェクトをスキャンしてデッドコード（未使用の関数やクラス）を検出します".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: {
                        let mut props = serde_json::Map::new();
                        props.insert("path".to_string(), serde_json::json!({
                            "type": "string",
                            "description": "スキャン対象のプロジェクトパス"
                        }));
                        props
                    },
                    required: vec!["path".to_string()],
                },
            },
            Self::AnnotateProject => FunctionDeclaration {
                name: "annotate_project".to_string(),
                description: "検出されたデッドコードにアノテーション（コメント）を追加します。dry_run=trueで変更をプレビューできます".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: {
                        let mut props = serde_json::Map::new();
                        props.insert("path".to_string(), serde_json::json!({
                            "type": "string",
                            "description": "対象のプロジェクトパス"
                        }));
                        props.insert("dry_run".to_string(), serde_json::json!({
                            "type": "boolean",
                            "description": "trueの場合、実際には変更せずプレビューのみ"
                        }));
                        props
                    },
                    required: vec!["path".to_string()],
                },
            },
            Self::CleanProject => FunctionDeclaration {
                name: "clean_project".to_string(),
                description: "デッドコードを削除してプロジェクトをクリーンアップします。dry_run=trueで変更をプレビューできます".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: {
                        let mut props = serde_json::Map::new();
                        props.insert("path".to_string(), serde_json::json!({
                            "type": "string",
                            "description": "対象のプロジェクトパス"
                        }));
                        props.insert("dry_run".to_string(), serde_json::json!({
                            "type": "boolean",
                            "description": "trueの場合、実際には変更せずプレビューのみ"
                        }));
                        props.insert("safe_only".to_string(), serde_json::json!({
                            "type": "boolean",
                            "description": "trueの場合、安全なコードのみ削除"
                        }));
                        props
                    },
                    required: vec!["path".to_string()],
                },
            },
            Self::GetProjectStats => FunctionDeclaration {
                name: "get_project_stats".to_string(),
                description: "プロジェクトの統計情報（ファイル数、デッドコード数など）を取得します".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: {
                        let mut props = serde_json::Map::new();
                        props.insert("path".to_string(), serde_json::json!({
                            "type": "string",
                            "description": "対象のプロジェクトパス"
                        }));
                        props
                    },
                    required: vec!["path".to_string()],
                },
            },
        }
    }

    /// Parse tool name from string
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "scan_project" => Some(Self::ScanProject),
            "annotate_project" => Some(Self::AnnotateProject),
            "clean_project" => Some(Self::CleanProject),
            "get_project_stats" => Some(Self::GetProjectStats),
            _ => None,
        }
    }
}

/// Create Tool object with all index-chan functions
pub fn create_index_chan_tools() -> Tool {
    Tool {
        function_declarations: IndexChanTool::all()
            .iter()
            .map(|t| t.to_declaration())
            .collect(),
    }
}
