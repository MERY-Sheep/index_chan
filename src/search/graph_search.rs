// Graph-based semantic search
// Phase 7 GraphRAG integration

use crate::graph::{CodeGraph, NodeId, TraversalResult};
use crate::search::index::CodeMetadata;
use serde::{Deserialize, Serialize};

#[cfg(feature = "semantic-search")]
use crate::embedding::EmbeddingGenerator;

/// Generic function names that should be filtered from search results
/// These are common trait implementations and utility functions that add noise
const GENERIC_NAMES: &[&str] = &[
    // Rust standard trait implementations
    "new", "default", "init", "from", "into", "clone", "drop",
    "fmt", "eq", "ne", "cmp", "hash", "serialize", "deserialize",
    // Common conversion methods
    "as_ref", "as_mut", "borrow", "borrow_mut", "deref", "deref_mut",
    // Iterator methods
    "index", "index_mut", "next", "size_hint", "len", "is_empty",
    // Collection methods
    "get", "set", "push", "pop", "insert", "remove", "contains",
    "iter", "iter_mut", "into_iter",
    // Error handling
    "unwrap", "expect", "ok", "err", "map", "and_then",
    // Builder pattern
    "build", "builder", "with",
];

/// Check if a function name is a generic/noise name
pub fn is_generic_name(name: &str) -> bool {
    GENERIC_NAMES.contains(&name)
}

/// マッチタイプ: 直接ヒット or グラフ経由
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    DirectHit,     // セマンティック検索で直接ヒット
    GraphNeighbor, // グラフ探索で発見
}

/// トレースステップ: ノード or エッジの1ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub node: String,
    pub node_type: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

/// スコア内訳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub base_score: f32,
    pub decay_factor: f32,
    pub depth: usize,
}

/// マッチ理由の説明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchExplanation {
    pub trace: Vec<TraceStep>,
    pub score_details: ScoreBreakdown,
}

/// グラフ探索検索結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSearchResult {
    pub metadata: CodeMetadata,
    pub score: f32,
    pub depth: usize,
    pub path: Vec<NodeId>,
    pub match_type: MatchType,
    pub explanation: MatchExplanation,
}

/// グラフベースのセマンティック検索
pub struct GraphSearcher {
    graph: CodeGraph,
}

impl GraphSearcher {
    /// 新しい GraphSearcher を作成
    pub fn new(graph: CodeGraph) -> Self {
        Self { graph }
    }

    pub fn search(&self, query: &str, top_k: usize, depth: usize) -> Vec<GraphSearchResult> {
        self.search_with_graph(query, top_k, depth)
    }

    /// テキストマッチング + グラフ探索で検索
    ///
    /// # Arguments
    /// * `query` - 検索クエリ
    /// * `top_k` - 初期検索で取得する件数
    /// * `graph_depth` - グラフ探索の最大深さ
    pub fn search_with_graph(
        &self,
        query: &str,
        top_k: usize,
        graph_depth: usize,
    ) -> Vec<GraphSearchResult> {
        // Default: filter generic names
        self.search_with_graph_filtered(query, top_k, graph_depth, true)
    }

    /// テキストマッチング + グラフ探索で検索（フィルタリングオプション付き）
    ///
    /// # Arguments
    /// * `query` - 検索クエリ
    /// * `top_k` - 初期検索で取得する件数
    /// * `graph_depth` - グラフ探索の最大深さ
    /// * `filter_generic` - 汎用名（new, default等）をフィルタリングするか
    pub fn search_with_graph_filtered(
        &self,
        query: &str,
        top_k: usize,
        graph_depth: usize,
        filter_generic: bool,
    ) -> Vec<GraphSearchResult> {
        // 1. 名前でマッチするノードを取得
        let matched_ids = self.graph.find_nodes_by_name(query);

        if matched_ids.is_empty() {
            return Vec::new();
        }

        // 上位 top_k に制限
        let start_ids: Vec<NodeId> = matched_ids.into_iter().take(top_k).collect();

        // 2. グラフを探索
        let traversal_results = self.graph.traverse_from(&start_ids, graph_depth);

        // 3. 結果を変換（フィルタリング適用）
        let mut results: Vec<GraphSearchResult> = traversal_results
            .into_iter()
            .filter_map(|tr| {
                self.graph.get_node(tr.node_id).and_then(|node| {
                    // フィルタリング: 汎用名をスキップ（depth > 0 の場合のみ）
                    // DirectHit（depth == 0）は検索クエリそのものなので除外しない
                    if filter_generic && tr.depth > 0 && is_generic_name(&node.name) {
                        return None;
                    }

                    // スコアは深さに応じて減衰
                    let base_score = 1.0;
                    let decay_factor = 1.0 / (1.0 + tr.depth as f32 * 0.3);
                    let score = base_score * decay_factor;

                    // マッチタイプ判定
                    let match_type = if tr.depth == 0 {
                        MatchType::DirectHit
                    } else {
                        MatchType::GraphNeighbor
                    };

                    // トレース構築
                    let trace = self.build_trace(&tr.path, tr.depth, "name_match");

                    Some(GraphSearchResult {
                        metadata: CodeMetadata {
                            file_path: node.file_path.clone(),
                            function_name: node.name.clone(),
                            start_line: node.line_range.0,
                            end_line: node.line_range.1,
                            code_snippet: String::new(),
                            dependencies: Vec::new(),
                        },
                        score,
                        depth: tr.depth,
                        path: tr.path,
                        match_type,
                        explanation: MatchExplanation {
                            trace,
                            score_details: ScoreBreakdown {
                                base_score,
                                decay_factor,
                                depth: tr.depth,
                            },
                        },
                    })
                })
            })
            .collect();

        // スコア順にソート
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// グラフ探索のみ（ベクトル検索なし）
    pub fn traverse_from_names(&self, names: &[&str], max_depth: usize) -> Vec<TraversalResult> {
        let start_ids: Vec<NodeId> = names
            .iter()
            .flat_map(|name| self.graph.find_nodes_by_name(name))
            .collect();

        self.graph.traverse_from(&start_ids, max_depth)
    }

    /// トレースステップを構築
    fn build_trace(&self, path: &[NodeId], _depth: usize, reason: &str) -> Vec<TraceStep> {
        let mut trace = Vec::new();

        for (i, &node_id) in path.iter().enumerate() {
            if let Some(node) = self.graph.get_node(node_id) {
                let step_reason = if i == 0 {
                    reason.to_string()
                } else {
                    "graph_traversal".to_string()
                };

                // エッジ情報を取得
                let (edge, direction) = if i > 0 {
                    let prev_id = path[i - 1];
                    // 出力エッジを探す
                    let edge_info = self.graph.edges.iter().find(|e| {
                        (e.from == prev_id && e.to == node_id)
                            || (e.to == prev_id && e.from == node_id)
                    });

                    if let Some(e) = edge_info {
                        let dir = if e.from == prev_id {
                            "forward"
                        } else {
                            "backward"
                        };
                        (Some(format!("{:?}", e.edge_type)), Some(dir.to_string()))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                trace.push(TraceStep {
                    node: node.name.clone(),
                    node_type: format!("{:?}", node.node_type),
                    reason: step_reason,
                    edge,
                    direction,
                });
            }
        }

        trace
    }

    /// セマンティック検索 + グラフ探索 (Concept Transformer 統合)
    #[cfg(feature = "semantic-search")]
    pub fn search_semantic(
        &self,
        query: &str,
        node_embeddings: &std::collections::HashMap<NodeId, Vec<f32>>,
        top_k: usize,
        graph_depth: usize,
    ) -> Vec<GraphSearchResult> {
        // クエリの埋め込みを生成
        let generator = match EmbeddingGenerator::new() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Failed to load embedding model: {}", e);
                return self.search_with_graph(query, top_k, graph_depth);
            }
        };

        let query_embedding = match generator.embed(query) {
            Ok(emb) => emb,
            Err(e) => {
                eprintln!("Failed to embed query: {}", e);
                return self.search_with_graph(query, top_k, graph_depth);
            }
        };

        // 各ノードとの類似度を計算
        let mut similarities: Vec<(NodeId, f32)> = node_embeddings
            .iter()
            .map(|(&node_id, embedding)| {
                let sim = EmbeddingGenerator::cosine_similarity(&query_embedding, embedding);
                (node_id, sim)
            })
            .collect();

        // 類似度でソート
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 上位 top_k を取得
        let start_ids: Vec<NodeId> = similarities.iter().take(top_k).map(|(id, _)| *id).collect();

        let initial_scores: std::collections::HashMap<NodeId, f32> = similarities
            .iter()
            .take(top_k)
            .map(|(id, score)| (*id, *score))
            .collect();

        if start_ids.is_empty() {
            return Vec::new();
        }

        // グラフ探索
        let traversal_results = self.graph.traverse_from(&start_ids, graph_depth);

        // 結果を変換
        let mut results: Vec<GraphSearchResult> = traversal_results
            .into_iter()
            .filter_map(|tr| {
                self.graph.get_node(tr.node_id).map(|node| {
                    // マッチタイプ判定
                    let is_direct_hit = initial_scores.contains_key(&tr.node_id);
                    let match_type = if is_direct_hit {
                        MatchType::DirectHit
                    } else {
                        MatchType::GraphNeighbor
                    };

                    // スコア計算を改善
                    let (score, base_score, decay_factor) = if let Some(&s) =
                        initial_scores.get(&tr.node_id)
                    {
                        (s, s, 1.0)
                    } else {
                        let semantic_sim = node_embeddings
                            .get(&tr.node_id)
                            .map(|emb| EmbeddingGenerator::cosine_similarity(&query_embedding, emb))
                            .unwrap_or(0.0);

                        let depth_decay = 0.8f32.powi(tr.depth as i32);
                        let base = if semantic_sim > 0.0 {
                            semantic_sim
                        } else {
                            similarities.first().map(|(_, s)| *s * 0.5).unwrap_or(0.3)
                        };
                        (base * depth_decay, base, depth_decay)
                    };

                    // トレース構築
                    let reason = if is_direct_hit {
                        format!("semantic_match ({:.2})", base_score)
                    } else {
                        "graph_traversal".to_string()
                    };
                    let trace = self.build_trace(&tr.path, tr.depth, &reason);

                    GraphSearchResult {
                        metadata: CodeMetadata {
                            file_path: node.file_path.clone(),
                            function_name: node.name.clone(),
                            start_line: node.line_range.0,
                            end_line: node.line_range.1,
                            code_snippet: String::new(),
                            dependencies: self
                                .graph
                                .edges
                                .iter()
                                .filter(|e| e.from == tr.node_id)
                                .filter_map(|e| self.graph.get_node(e.to))
                                .map(|n| n.name.clone())
                                .collect(),
                        },
                        score,
                        depth: tr.depth,
                        path: tr.path.clone(),
                        match_type,
                        explanation: MatchExplanation {
                            trace,
                            score_details: ScoreBreakdown {
                                base_score,
                                decay_factor,
                                depth: tr.depth,
                            },
                        },
                    }
                })
            })
            .collect();

        // スコア順にソート
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CodeNode, DependencyEdge, EdgeType, NodeType};
    use std::path::PathBuf;

    fn create_test_graph() -> CodeGraph {
        let mut graph = CodeGraph::new();

        // ノードを追加
        let auth_id = graph.add_node(CodeNode {
            id: 0,
            name: "auth".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("auth.ts"),
            line_range: (1, 50),
            is_exported: true,
            is_used: true,
            signature: "function auth()".to_string(),
        });

        let user_db_id = graph.add_node(CodeNode {
            id: 1,
            name: "user_db".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("user_db.ts"),
            line_range: (1, 100),
            is_exported: true,
            is_used: true,
            signature: "function user_db()".to_string(),
        });

        let config_id = graph.add_node(CodeNode {
            id: 2,
            name: "config".to_string(),
            node_type: NodeType::Variable,
            file_path: PathBuf::from("config.ts"),
            line_range: (1, 20),
            is_exported: true,
            is_used: true,
            signature: "const config".to_string(),
        });

        // エッジを追加
        graph.add_edge(DependencyEdge {
            from: auth_id,
            to: user_db_id,
            edge_type: EdgeType::Calls,
        });

        graph.add_edge(DependencyEdge {
            from: auth_id,
            to: config_id,
            edge_type: EdgeType::References,
        });

        graph
    }

    #[test]
    fn test_search_with_graph() {
        let graph = create_test_graph();
        let searcher = GraphSearcher::new(graph);

        let results = searcher.search_with_graph("auth", 3, 2);

        // auth + 関連ノード (user_db, config) が取得されること
        assert!(results.len() >= 1);
        assert_eq!(results[0].metadata.function_name, "auth");
        assert_eq!(results[0].depth, 0);
    }

    #[test]
    fn test_graph_traversal() {
        let graph = create_test_graph();
        let searcher = GraphSearcher::new(graph);

        let results = searcher.traverse_from_names(&["auth"], 2);

        // auth から user_db と config に到達できること
        assert!(results.len() >= 3);
    }
}
