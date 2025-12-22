use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

pub type NodeId = usize;

/// 汎用名: これらはグラフ探索の終端として扱う（探索はここで止まる）
/// 理由: new(), default(), from() 等は多くの場所から呼ばれるため、
/// 探索を続けるとコンテキストが爆発する
const TERMINAL_NAMES: &[&str] = &[
    // Rust common
    "new", "default", "init", "from", "into", "clone", "drop",
    "fmt", "eq", "ne", "cmp", "hash", "serialize", "deserialize",
    "to_string", "as_ref", "as_mut", "borrow", "borrow_mut",
    "deref", "deref_mut", "index", "index_mut",
    "next", "size_hint", "count", "last", "nth",
    "map", "filter", "fold", "collect", "iter", "into_iter",
    "unwrap", "expect", "ok", "err", "is_some", "is_none", "is_ok", "is_err",
    "len", "is_empty", "capacity", "reserve", "push", "pop", "insert", "remove",
    "get", "set", "contains", "contains_key",
    "read", "write", "flush", "close",
    "lock", "unlock", "try_lock",
    "spawn", "join", "await",
    // JavaScript/TypeScript common
    "constructor", "toString", "valueOf", "hasOwnProperty",
    "reduce", "forEach", "find", "some", "every",
    "shift", "unshift", "slice", "splice",
    "then", "catch", "finally",
    // Python common
    "__init__", "__new__", "__str__", "__repr__", "__eq__", "__hash__",
    "__len__", "__iter__", "__next__", "__getitem__", "__setitem__",
];

/// Depth別の最大結果数制限
pub const DEPTH_LIMITS: &[usize] = &[
    50,  // depth 0: 直接マッチ
    30,  // depth 1: 1ホップ
    15,  // depth 2: 2ホップ
    5,   // depth 3+: それ以上
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: HashMap<NodeId, CodeNode>,
    pub edges: Vec<DependencyEdge>,
    pub next_id: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub file_path: PathBuf,
    pub line_range: (usize, usize),
    pub is_exported: bool,
    pub is_used: bool,
    #[serde(default)]
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Function,
    Class,
    Method,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Calls,
    References,
    Instantiates,
    Imports,
}

/// Semantic relation type based on Concept Transformer Phase 2 insights
/// These are higher-level classifications of relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRelationType {
    /// is-a: Inheritance, type hierarchy (class extends, implements)
    IsA,
    /// has: Ownership, composition (field access, contains)
    Has,
    /// transforms: Input -> Output transformation (function calls that change state)
    Transforms,
    /// uses: Read-only usage (references without modification)
    Uses,
    /// creates: Object instantiation
    Creates,
}

impl EdgeType {
    /// Convert to semantic relation type
    /// Based on Concept Transformer's discovery that relation types cluster semantically
    pub fn to_semantic(&self) -> SemanticRelationType {
        match self {
            EdgeType::Calls => SemanticRelationType::Transforms,
            EdgeType::References => SemanticRelationType::Uses,
            EdgeType::Instantiates => SemanticRelationType::Creates,
            EdgeType::Imports => SemanticRelationType::Uses,
        }
    }
}

impl SemanticRelationType {
    /// Get description for this relation type
    pub fn description(&self) -> &'static str {
        match self {
            SemanticRelationType::IsA => "inheritance/type hierarchy",
            SemanticRelationType::Has => "ownership/composition",
            SemanticRelationType::Transforms => "input→output transformation",
            SemanticRelationType::Uses => "read-only reference",
            SemanticRelationType::Creates => "object instantiation",
        }
    }

    /// Get weight for graph traversal (higher = more important for understanding)
    pub fn traversal_weight(&self) -> f32 {
        match self {
            SemanticRelationType::IsA => 1.0,       // Most important for understanding structure
            SemanticRelationType::Transforms => 0.9, // Important for data flow
            SemanticRelationType::Has => 0.8,        // Important for composition
            SemanticRelationType::Creates => 0.7,    // Relevant for lifecycle
            SemanticRelationType::Uses => 0.5,       // Less important (read-only)
        }
    }
}

/// グラフ探索結果
#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub node_id: NodeId,
    pub depth: usize,
    pub path: Vec<NodeId>,
}

/// 探索オプション
#[derive(Debug, Clone)]
pub struct TraversalOptions {
    pub max_depth: usize,
    /// depth別の最大結果数（Noneの場合はDEPTH_LIMITSを使用）
    pub depth_limits: Option<Vec<usize>>,
    /// 終端ノードで探索を止めるか
    pub stop_at_terminals: bool,
}

impl Default for TraversalOptions {
    fn default() -> Self {
        Self {
            max_depth: 2,
            depth_limits: None,
            stop_at_terminals: true,
        }
    }
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, mut node: CodeNode) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        node.id = id;
        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, edge: DependencyEdge) {
        self.edges.push(edge);
    }

    #[allow(dead_code)]
    pub fn get_node(&self, id: NodeId) -> Option<&CodeNode> {
        self.nodes.get(&id)
    }

    /// ノードが終端（汎用名）かどうかチェック
    fn is_terminal_node(&self, node_id: NodeId) -> bool {
        if let Some(node) = self.nodes.get(&node_id) {
            let name_lower = node.name.to_lowercase();
            TERMINAL_NAMES.iter().any(|&t| name_lower == t)
        } else {
            false
        }
    }

    /// BFS でグラフを探索し、関連ノードを取得
    ///
    /// # Arguments
    /// * `start_ids` - 探索開始ノードのID
    /// * `max_depth` - 最大探索深さ
    ///
    /// # Returns
    /// 各ノードへの到達結果（深さとパス）
    pub fn traverse_from(&self, start_ids: &[NodeId], max_depth: usize) -> Vec<TraversalResult> {
        self.traverse_with_options(start_ids, TraversalOptions {
            max_depth,
            ..Default::default()
        })
    }

    /// オプション付きでグラフを探索
    pub fn traverse_with_options(
        &self,
        start_ids: &[NodeId],
        options: TraversalOptions,
    ) -> Vec<TraversalResult> {
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut results: Vec<TraversalResult> = Vec::new();
        let mut queue: VecDeque<(NodeId, usize, Vec<NodeId>)> = VecDeque::new();

        // depth別の結果カウント
        let mut depth_counts: HashMap<usize, usize> = HashMap::new();

        let depth_limits = options.depth_limits.as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(DEPTH_LIMITS);

        // 開始ノードを追加
        for &start_id in start_ids {
            if self.nodes.contains_key(&start_id) && !visited.contains(&start_id) {
                visited.insert(start_id);

                // depth 0 の制限チェック
                let limit = depth_limits.first().copied().unwrap_or(50);
                let count = depth_counts.entry(0).or_insert(0);
                if *count >= limit {
                    continue;
                }
                *count += 1;

                results.push(TraversalResult {
                    node_id: start_id,
                    depth: 0,
                    path: vec![start_id],
                });

                // 終端ノードでなければキューに追加
                if !options.stop_at_terminals || !self.is_terminal_node(start_id) {
                    queue.push_back((start_id, 0, vec![start_id]));
                }
            }
        }

        // BFS
        while let Some((current_id, depth, path)) = queue.pop_front() {
            if depth >= options.max_depth {
                continue;
            }

            let next_depth = depth + 1;
            let limit = depth_limits.get(next_depth).copied()
                .unwrap_or_else(|| depth_limits.last().copied().unwrap_or(5));

            // 出力エッジ (from -> to)
            for edge in &self.edges {
                if edge.from == current_id && !visited.contains(&edge.to) {
                    // depth制限チェック
                    let count = depth_counts.entry(next_depth).or_insert(0);
                    if *count >= limit {
                        continue;
                    }

                    visited.insert(edge.to);
                    *count += 1;

                    let mut new_path = path.clone();
                    new_path.push(edge.to);

                    results.push(TraversalResult {
                        node_id: edge.to,
                        depth: next_depth,
                        path: new_path.clone(),
                    });

                    // 終端ノードでなければ探索を続ける
                    if !options.stop_at_terminals || !self.is_terminal_node(edge.to) {
                        queue.push_back((edge.to, next_depth, new_path));
                    }
                }
            }

            // 入力エッジ (to <- from)
            for edge in &self.edges {
                if edge.to == current_id && !visited.contains(&edge.from) {
                    // depth制限チェック
                    let count = depth_counts.entry(next_depth).or_insert(0);
                    if *count >= limit {
                        continue;
                    }

                    visited.insert(edge.from);
                    *count += 1;

                    let mut new_path = path.clone();
                    new_path.push(edge.from);

                    results.push(TraversalResult {
                        node_id: edge.from,
                        depth: next_depth,
                        path: new_path.clone(),
                    });

                    // 終端ノードでなければ探索を続ける
                    if !options.stop_at_terminals || !self.is_terminal_node(edge.from) {
                        queue.push_back((edge.from, next_depth, new_path));
                    }
                }
            }
        }

        results
    }

    /// ノード名で検索
    pub fn find_nodes_by_name(&self, query: &str) -> Vec<NodeId> {
        let query_lower = query.to_lowercase();
        self.nodes
            .iter()
            .filter(|(_, node)| node.name.to_lowercase().contains(&query_lower))
            .map(|(&id, _)| id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_names() {
        let mut graph = CodeGraph::new();

        let new_id = graph.add_node(CodeNode {
            id: 0,
            name: "new".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("test.rs"),
            line_range: (1, 10),
            is_exported: true,
            is_used: true,
            signature: "fn new()".to_string(),
        });

        let custom_id = graph.add_node(CodeNode {
            id: 0,
            name: "my_custom_function".to_string(),
            node_type: NodeType::Function,
            file_path: PathBuf::from("test.rs"),
            line_range: (11, 20),
            is_exported: true,
            is_used: true,
            signature: "fn my_custom_function()".to_string(),
        });

        assert!(graph.is_terminal_node(new_id));
        assert!(!graph.is_terminal_node(custom_id));
    }

    #[test]
    fn test_depth_limits() {
        let mut graph = CodeGraph::new();

        // チェーン構造を作成: a -> b -> c -> d -> e
        let ids: Vec<NodeId> = (0..5).map(|i| {
            graph.add_node(CodeNode {
                id: 0,
                name: format!("func_{}", i),
                node_type: NodeType::Function,
                file_path: PathBuf::from("test.rs"),
                line_range: (i * 10 + 1, i * 10 + 10),
                is_exported: true,
                is_used: true,
                signature: format!("fn func_{}()", i),
            })
        }).collect();

        for i in 0..4 {
            graph.add_edge(DependencyEdge {
                from: ids[i],
                to: ids[i + 1],
                edge_type: EdgeType::Calls,
            });
        }

        // depth 2 で探索
        let results = graph.traverse_from(&[ids[0]], 2);

        // depth 0: func_0, depth 1: func_1, depth 2: func_2
        assert!(results.len() <= 3);
    }
}
