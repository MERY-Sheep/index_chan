#![cfg(feature = "db")]

use crate::graph::{CodeGraph, CodeNode, DependencyEdge, EdgeType, NodeType};
use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::path::Path;
use std::str::FromStr;

pub struct GraphDB {
    pool: Pool<Sqlite>,
}

impl GraphDB {
    pub async fn new(db_path: &Path) -> Result<Self> {
        let db_url = format!("sqlite://{}", db_path.display());

        // Ensure database file exists
        if !db_path.exists() {
            File::create(db_path)?;
        }
        use std::fs::File;

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .context("Failed to connect to database")?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS nodes (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                is_exported BOOLEAN NOT NULL,
                is_used BOOLEAN NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                target_id INTEGER NOT NULL,
                type TEXT NOT NULL,
                FOREIGN KEY(source_id) REFERENCES nodes(id),
                FOREIGN KEY(target_id) REFERENCES nodes(id)
            );
            
            CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);
            CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(file_path);
            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to initialize database schema")?;

        Ok(())
    }

    pub async fn save_graph(&self, graph: &CodeGraph) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Clear existing data
        sqlx::query("DELETE FROM edges").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM nodes").execute(&mut *tx).await?;

        // Insert nodes
        for node in graph.nodes.values() {
            let node_type_str = format!("{:?}", node.node_type);
            sqlx::query(
                r#"
                INSERT INTO nodes (id, name, type, file_path, start_line, end_line, is_exported, is_used)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(node.id as i64)
            .bind(&node.name)
            .bind(node_type_str)
            .bind(node.file_path.display().to_string())
            .bind(node.line_range.0 as i64)
            .bind(node.line_range.1 as i64)
            .bind(node.is_exported)
            .bind(node.is_used)
            .execute(&mut *tx)
            .await?;
        }

        // Insert edges
        for edge in &graph.edges {
            let edge_type_str = format!("{:?}", edge.edge_type);
            sqlx::query(
                r#"
                INSERT INTO edges (source_id, target_id, type)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(edge.from as i64)
            .bind(edge.to as i64)
            .bind(edge_type_str)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load_graph(&self) -> Result<CodeGraph> {
        let mut graph = CodeGraph::new();

        // Load nodes
        let nodes = sqlx::query("SELECT * FROM nodes")
            .fetch_all(&self.pool)
            .await?;

        for row in nodes {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            let type_str: String = row.get("type");
            let file_path_str: String = row.get("file_path");
            let start_line: i64 = row.get("start_line");
            let end_line: i64 = row.get("end_line");
            let is_exported: bool = row.get("is_exported");
            let is_used: bool = row.get("is_used");

            // Parse enums (simplified, assumes generated strings match)
            let node_type = match type_str.as_str() {
                "Function" => NodeType::Function,
                "Class" => NodeType::Class,
                "Method" => NodeType::Method,
                "Variable" => NodeType::Variable,
                _ => NodeType::Function, // Default fallback
            };

            let node = CodeNode {
                id: id as usize,
                name,
                node_type,
                file_path: std::path::PathBuf::from(file_path_str),
                line_range: (start_line as usize, end_line as usize),
                is_exported,
                is_used,
                signature: String::new(), // DB doesn't store signatures yet
            };

            // 手動で挿入して next_id を適切に更新する必要があるが、
            // graph.add_node は ID を自動生成してしまう。
            // 既存の ID を維持するために nodes マップに直接挿入する。
            graph.nodes.insert(node.id, node);
        }

        // next_id の更新 (最大ID + 1)
        let max_id = graph.nodes.keys().max().copied().unwrap_or(0);
        graph.next_id = max_id + 1;

        // Load edges
        let edges = sqlx::query("SELECT * FROM edges")
            .fetch_all(&self.pool)
            .await?;

        for row in edges {
            let source_id: i64 = row.get("source_id");
            let target_id: i64 = row.get("target_id");
            let type_str: String = row.get("type");

            let edge_type = match type_str.as_str() {
                "Calls" => EdgeType::Calls,
                "References" => EdgeType::References,
                "Instantiates" => EdgeType::Instantiates,
                "Imports" => EdgeType::Imports,
                _ => EdgeType::Calls,
            };

            graph.edges.push(DependencyEdge {
                from: source_id as usize,
                to: target_id as usize,
                edge_type,
            });
        }

        Ok(graph)
    }
}
