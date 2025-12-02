use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: i64,
    pub project_id: i64,
    pub path: String,
    pub language: String,
    pub hash: String,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub id: i64,
    pub file_id: i64,
    pub name: String,
    pub line_start: i64,
    pub line_end: i64,
    pub is_exported: bool,
    pub is_used: bool,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub id: i64,
    pub caller_id: i64,
    pub callee_id: i64,
    pub line: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: i64,
    pub from_function_id: i64,
    pub to_function_id: i64,
    pub edge_type: String,
}

impl File {
    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}
