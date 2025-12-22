use anyhow::Result;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

use super::models::*;
use super::schema::SCHEMA;
use crate::graph::CodeGraph;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// プールへの参照を取得
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// データベースを開く（存在しない場合は作成）
    pub async fn open(db_path: &Path) -> Result<Self> {
        // 親ディレクトリを作成
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };

        // スキーマを初期化
        db.init_schema().await?;

        Ok(db)
    }

    /// スキーマを初期化
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(SCHEMA).execute(&self.pool).await?;
        Ok(())
    }

    /// プロジェクトを作成または取得
    pub async fn get_or_create_project(&self, path: &Path, name: &str) -> Result<Project> {
        let path_str = path.display().to_string();

        // 既存のプロジェクトを検索
        let existing: Option<(i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, path, name, created_at, updated_at FROM projects WHERE path = ?"
        )
        .bind(&path_str)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, path, name, created_at, updated_at)) = existing {
            return Ok(Project {
                id,
                path,
                name,
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            });
        }

        // 新規作成
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let result = sqlx::query(
            "INSERT INTO projects (path, name, created_at, updated_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&path_str)
        .bind(name)
        .bind(&now_str)
        .bind(&now_str)
        .execute(&self.pool)
        .await?;

        Ok(Project {
            id: result.last_insert_rowid(),
            path: path_str,
            name: name.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// ファイルハッシュを計算
    pub fn calculate_file_hash(path: &Path) -> Result<String> {
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// ファイルを追加または更新
    pub async fn upsert_file(
        &self,
        project_id: i64,
        path: &Path,
        language: &str,
        hash: &str,
    ) -> Result<File> {
        let path_str = path.display().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        // 既存のファイルを検索
        let existing: Option<(i64, i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, project_id, path, language, hash, last_modified FROM files WHERE project_id = ? AND path = ?"
        )
        .bind(project_id)
        .bind(&path_str)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, project_id, path, lang, file_hash, last_modified)) = existing {
            // ハッシュが同じなら更新不要
            if file_hash == hash {
                return Ok(File {
                    id,
                    project_id,
                    path,
                    language: lang,
                    hash: file_hash,
                    last_modified: DateTime::parse_from_rfc3339(&last_modified)?.with_timezone(&Utc),
                });
            }

            // 更新
            sqlx::query("UPDATE files SET language = ?, hash = ?, last_modified = ? WHERE id = ?")
                .bind(language)
                .bind(hash)
                .bind(&now_str)
                .bind(id)
                .execute(&self.pool)
                .await?;

            Ok(File {
                id,
                project_id,
                path,
                language: language.to_string(),
                hash: hash.to_string(),
                last_modified: now,
            })
        } else {
            // 新規作成
            let result = sqlx::query(
                "INSERT INTO files (project_id, path, language, hash, last_modified) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(project_id)
            .bind(&path_str)
            .bind(language)
            .bind(hash)
            .bind(&now_str)
            .execute(&self.pool)
            .await?;

            Ok(File {
                id: result.last_insert_rowid(),
                project_id,
                path: path_str,
                language: language.to_string(),
                hash: hash.to_string(),
                last_modified: now,
            })
        }
    }

    /// ファイルのハッシュを取得
    pub async fn get_file_hash(&self, project_id: i64, path: &Path) -> Result<Option<String>> {
        let path_str = path.display().to_string();

        let result: Option<(String,)> = sqlx::query_as(
            "SELECT hash FROM files WHERE project_id = ? AND path = ?"
        )
        .bind(project_id)
        .bind(&path_str)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.0))
    }

    /// ファイルを削除
    pub async fn delete_file(&self, project_id: i64, path: &Path) -> Result<()> {
        let path_str = path.display().to_string();

        sqlx::query("DELETE FROM files WHERE project_id = ? AND path = ?")
            .bind(project_id)
            .bind(&path_str)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// グラフをデータベースに保存
    pub async fn save_graph(
        &self,
        file_id: i64,
        graph: &CodeGraph,
    ) -> Result<()> {
        // 既存の関数を削除
        sqlx::query("DELETE FROM functions WHERE file_id = ?")
            .bind(file_id)
            .execute(&self.pool)
            .await?;

        // ノードIDとDBのIDのマッピング
        let mut node_to_db_id = std::collections::HashMap::new();

        // 関数を挿入
        for (node_id, node) in &graph.nodes {
            let result = sqlx::query(
                "INSERT INTO functions (file_id, name, line_start, line_end, is_exported, is_used) VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(file_id)
            .bind(&node.name)
            .bind(node.line_range.0 as i64)
            .bind(node.line_range.1 as i64)
            .bind(node.is_exported)
            .bind(node.is_used)
            .execute(&self.pool)
            .await?;

            node_to_db_id.insert(*node_id, result.last_insert_rowid());
        }

        // 依存関係を挿入
        for edge in &graph.edges {
            if let (Some(&from_id), Some(&to_id)) = (
                node_to_db_id.get(&edge.from),
                node_to_db_id.get(&edge.to),
            ) {
                let edge_type = format!("{:?}", edge.edge_type);

                sqlx::query(
                    "INSERT OR IGNORE INTO dependencies (from_function_id, to_function_id, edge_type) VALUES (?, ?, ?)"
                )
                .bind(from_id)
                .bind(to_id)
                .bind(&edge_type)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// デッドコードを取得
    pub async fn get_dead_code(&self, project_id: i64) -> Result<Vec<Function>> {
        let rows: Vec<(i64, i64, String, i64, i64, bool, bool, Option<String>)> = sqlx::query_as(
            r#"
            SELECT f.id, f.file_id, f.name, f.line_start, f.line_end, 
                   f.is_exported, f.is_used, f.signature
            FROM functions f
            JOIN files fi ON f.file_id = fi.id
            WHERE fi.project_id = ? AND f.is_used = 0
            "#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, file_id, name, line_start, line_end, is_exported, is_used, signature)| {
            Function {
                id,
                file_id,
                name,
                line_start,
                line_end,
                is_exported,
                is_used,
                signature,
            }
        }).collect())
    }

    /// プロジェクトの統計を取得
    pub async fn get_project_stats(&self, project_id: i64) -> Result<ProjectStats> {
        let row: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(DISTINCT fi.id) as file_count,
                COUNT(DISTINCT f.id) as function_count,
                COUNT(DISTINCT CASE WHEN f.is_used = 0 THEN f.id END) as dead_code_count,
                COUNT(DISTINCT d.id) as dependency_count
            FROM files fi
            LEFT JOIN functions f ON fi.id = f.file_id
            LEFT JOIN dependencies d ON f.id = d.from_function_id
            WHERE fi.project_id = ?
            "#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ProjectStats {
            file_count: row.0 as usize,
            function_count: row.1 as usize,
            dead_code_count: row.2 as usize,
            dependency_count: row.3 as usize,
        })
    }
}

#[derive(Debug)]
pub struct ProjectStats {
    pub file_count: usize,
    pub function_count: usize,
    pub dead_code_count: usize,
    pub dependency_count: usize,
}
