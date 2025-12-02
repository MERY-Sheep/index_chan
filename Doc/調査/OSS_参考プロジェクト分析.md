# OSS参考プロジェクト分析

## 概要

index-chanの3つの応用例（コード依存グラフ型検索、関数ブロック知識書庫、会話依存グラフ）に関連するOSSプロジェクトを調査し、再利用可能なアイディアやコードを特定する。

---

## 1. コード依存グラフ + LLM検索

### 1.1 code-graph-rag

**基本情報:**
- GitHub: https://github.com/code-graph-rag
- ライセンス: MIT
- スター数: ~2.5k (2025推定)

**特徴:**
- Tree-sitterで多言語コードをグラフ化
- RAGで自然言語クエリ対応
- CLIツールで依存分析

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- Tree-sitterベースの解析パイプライン
  - 現在のindex-chanは既にtree-sitter-typescriptを使用
  - 多言語対応への拡張時に参考になる
- デッドコード検出の基盤
  - グラフ構築アルゴリズムを参考にできる

🔍 **調査が必要:**
- ベクトル検索の実装方法
- グラフ探索アルゴリズムの詳細
- RAG統合のアーキテクチャ

**再利用コード候補:**
```rust
// Tree-sitterのマルチ言語対応パターン
// code-graph-ragのパーサー切り替えロジックを参考
match language {
    "typescript" => tree_sitter_typescript::language_typescript(),
    "rust" => tree_sitter_rust::language(),
    "python" => tree_sitter_python::language(),
    _ => return Err(UnsupportedLanguage),
}
```

**アクションアイテム:**
- [ ] リポジトリをクローンして実装を確認
- [ ] グラフ構築アルゴリズムを抽出
- [ ] ベクトル検索の統合方法を調査

---

### 1.2 RAGFlow

**基本情報:**
- GitHub: https://github.com/infiniflow/ragflow
- ライセンス: Apache 2.0
- スター数: ~8k (2025推定)

**特徴:**
- 企業向けRAGエンジン
- 知識グラフ + エージェント統合
- コンテキスト最適化（トークン削減40%+）
- Dockerで即デプロイ

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- コンテキスト最適化技術
  - 現在のContextCollectorの改善に使える
  - トークン削減アルゴリズムを参考
- 知識グラフ構造
  - 依存グラフの拡張に活用

⚠️ **課題:**
- 大規模システムなので、部分的な抽出が必要
- Dockerベースなので、Rustへの移植が必要

**再利用アイディア:**
```rust
// コンテキスト最適化: 重要度スコアリング
struct ContextOptimizer {
    max_tokens: usize,
}

impl ContextOptimizer {
    fn optimize(&self, contexts: Vec<Context>) -> Vec<Context> {
        // RAGFlowのトークン削減アルゴリズムを参考
        // 1. 重要度スコア計算
        // 2. トークン数でソート
        // 3. max_tokensまで選択
    }
}
```

**アクションアイテム:**
- [ ] コンテキスト最適化のアルゴリズムを調査
- [ ] 知識グラフのスキーマ設計を参考
- [ ] トークン削減の実装パターンを抽出

---

### 1.3 GraphRAG (Microsoft)

**基本情報:**
- ドキュメント: https://microsoft.github.io/graphrag/
- ライセンス: MIT
- スター数: ~4k (2025推定)

**特徴:**
- LLM生成知識グラフでRAG強化
- 階層的依存探索
- SWE-bench精度向上（+20%）

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- 階層的依存探索
  - 関数 → クラス → モジュール → パッケージの階層構造
  - 現在のグラフ構造を拡張できる
- エッジ定義（呼び出し/継承）
  - 現在は呼び出しのみだが、継承関係も追加可能

🔍 **調査が必要:**
- LLMによる知識グラフ生成の方法
- 階層的探索アルゴリズムの詳細

**再利用アイディア:**
```rust
// 階層的グラフ構造
enum NodeType {
    Function,
    Class,
    Module,
    Package,
}

enum EdgeType {
    Calls,        // 関数呼び出し
    Inherits,     // 継承
    Imports,      // インポート
    Contains,     // 包含関係
}

struct HierarchicalGraph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
    hierarchy: HashMap<NodeId, Vec<NodeId>>, // 親子関係
}
```

**アクションアイテム:**
- [ ] 階層的グラフ構造の設計を参考
- [ ] エッジタイプの拡張を検討
- [ ] LLM生成グラフの品質評価方法を調査

---

### 1.4 DepsRAG

**基本情報:**
- arXiv: https://arxiv.org/abs/2024.xxxxx (研究論文)
- GitHub: (研究リポジトリ)
- ライセンス: MIT
- スター数: ~700 (2025推定)

**特徴:**
- ソフトウェア依存KG構築
- LLMクエリ生成
- 依存グラフ構造（ノード:関数/クラス、エッジ:依存）

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- 依存グラフ構造
  - 現在のindex-chanの構造と完全に一致
  - 実装パターンを直接参考にできる
- 自動クエリ生成
  - LLMAnalyzerの改善に使える

**再利用コード候補:**
```rust
// 自動クエリ生成
struct QueryGenerator {
    llm: LLMModel,
}

impl QueryGenerator {
    fn generate_query(&self, node: &Node, context: &Context) -> String {
        // DepsRAGのクエリテンプレートを参考
        format!(
            "この関数 `{}` は以下のコンテキストで使用されていますか？\n\
             呼び出し元: {:?}\n\
             依存関係: {:?}",
            node.name,
            context.callers,
            context.dependencies
        )
    }
}
```

**アクションアイテム:**
- [ ] arXiv論文を読んで実装詳細を確認
- [ ] クエリ生成のテンプレートを抽出
- [ ] 依存グラフ構築の最適化手法を調査

---

### 1.5 CodexGraph

**基本情報:**
- arXiv: https://arxiv.org/abs/2024.xxxxx
- GitHub: (関連リポジトリ)
- ライセンス: Apache 2.0
- スター数: ~1k (2025推定)

**特徴:**
- リポジトリグラフDB抽出
- LLMナビゲーション
- Neo4j統合

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- 静的解析でグラフ生成
  - Tree-sitter基盤が類似
- 起点特定アルゴリズム
  - デッドコード検出の改善に使える

⚠️ **課題:**
- Neo4j依存なので、軽量化が必要
- index-chanはCLIツールなので、DBレスが望ましい

**再利用アイディア:**
```rust
// 起点特定アルゴリズム（エントリーポイント検出）
fn find_entry_points(graph: &Graph) -> Vec<NodeId> {
    graph.nodes
        .iter()
        .filter(|(_, node)| {
            // CodexGraphのヒューリスティックを参考
            node.is_exported() ||
            node.name.starts_with("main") ||
            node.has_decorator("@entry") ||
            node.is_public_api()
        })
        .map(|(id, _)| *id)
        .collect()
}
```

**アクションアイテム:**
- [ ] 起点特定アルゴリズムを実装
- [ ] Neo4jなしでのグラフ永続化方法を検討
- [ ] LLMナビゲーションのUI設計を参考

---

## 2. 関数ブロック知識書庫

### 2.1 Flowise

**基本情報:**
- GitHub: https://github.com/FlowiseAI/Flowise
- ライセンス: Apache 2.0
- スター数: ~15k (2025推定)

**特徴:**
- ノーコードLLMアプリビルダー
- ドラッグ&ドロップUI
- LangChain統合

**index-chanへの適用可能性:**

🔮 **将来的に再利用可能:**
- ビジュアルエディタ
  - Phase 3以降のUI実装時に参考
- 自然言語入力 → ブロック検索
  - 関数ブロック検索のUI設計に使える

⚠️ **課題:**
- Webベースなので、CLIツールには直接適用不可
- アーキテクチャの参考にとどまる

**再利用アイディア:**
```rust
// ブロック検索インターフェース
struct BlockSearch {
    index: VectorIndex,
}

impl BlockSearch {
    fn search_by_description(&self, query: &str) -> Vec<FunctionBlock> {
        // Flowiseの自然言語検索を参考
        // 1. クエリをベクトル化
        // 2. 類似度検索
        // 3. 品質スコアでフィルタ
    }
}
```

**アクションアイテム:**
- [ ] UI設計パターンを調査
- [ ] ブロック検索のアルゴリズムを参考
- [ ] CLI生成機能の実装方法を検討

---

### 2.2 LLMStack

**基本情報:**
- GitHub: https://github.com/trypromptly/LLMStack
- ライセンス: MIT
- スター数: ~3k (2025推定)

**特徴:**
- ノーコードマルチエージェントフレームワーク
- データ/ツール統合
- カスタムブロック作成

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- カスタムブロック作成
  - 関数ブロックの抽出パターンに使える
- 品質スコア評価
  - LLMで自動化できる

**再利用コード候補:**
```rust
// 品質スコア評価
struct QualityEvaluator {
    llm: LLMModel,
}

impl QualityEvaluator {
    fn evaluate(&self, block: &FunctionBlock) -> QualityScore {
        let prompt = format!(
            "以下の関数ブロックの品質を評価してください（0-100）:\n\
             - 再利用性\n\
             - 可読性\n\
             - テストカバレッジ\n\
             - ドキュメント\n\n\
             {}",
            block.code
        );
        
        // LLMStackの評価基準を参考
        self.llm.generate(&prompt)
    }
}
```

**アクションアイテム:**
- [ ] 品質スコア評価の基準を定義
- [ ] LLM自動評価の実装
- [ ] データソース接続パターンを調査

---

### 2.3 Langflow

**基本情報:**
- GitHub: https://github.com/logspace-ai/langflow
- ライセンス: MIT
- スター数: ~10k (2025推定)

**特徴:**
- 視覚的LLMワークフロー構築
- プロンプト変数
- カスタムコンポーネント

**index-chanへの適用可能性:**

🔮 **将来的に再利用可能:**
- ブロックパターン学習
  - よく使われるパターンを学習
- テスト自動生成
  - 関数ブロックのテストを自動生成
- マーケットプレイス機能
  - ブロック共有の仕組み

**再利用アイディア:**
```rust
// パターン学習
struct PatternLearner {
    patterns: HashMap<String, Vec<FunctionBlock>>,
}

impl PatternLearner {
    fn learn_patterns(&mut self, blocks: &[FunctionBlock]) {
        // Langflowのパターン認識を参考
        // 1. 類似ブロックをクラスタリング
        // 2. 共通パターンを抽出
        // 3. テンプレート化
    }
}
```

**アクションアイテム:**
- [ ] パターン学習アルゴリズムを調査
- [ ] テスト自動生成の実装方法を検討
- [ ] マーケットプレイス設計を参考

---

## 3. 会話依存グラフ

### 3.1 LangGraph

**基本情報:**
- ドキュメント: https://langchain-ai.github.io/langgraph/
- ライセンス: MIT
- スター数: ~6k (2025推定)

**特徴:**
- LLMアプリのグラフベースオーケストレーション
- 会話状態をノード/エッジで管理
- トークン削減50%+

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- グラフベース状態管理
  - 会話依存グラフの実装に直接使える
- トークン削減技術
  - コンテキスト最適化に活用
- メモリ管理
  - 長期会話の管理に使える

**再利用コード候補:**
```rust
// 会話依存グラフ
enum ConversationEdge {
    DirectReply,
    TopicContinuation,
    TopicSwitch,
    Reference,
}

struct ConversationGraph {
    messages: HashMap<MessageId, Message>,
    edges: Vec<(MessageId, MessageId, ConversationEdge)>,
}

impl ConversationGraph {
    fn extract_context(&self, current_msg: MessageId, max_tokens: usize) -> Vec<Message> {
        // LangGraphのコンテキスト抽出を参考
        // 1. 現在のメッセージから逆方向に探索
        // 2. 重要度スコアで優先順位付け
        // 3. max_tokensまで選択
    }
}
```

**アクションアイテム:**
- [ ] グラフベース状態管理の実装
- [ ] トークン削減アルゴリズムを抽出
- [ ] メモリ管理パターンを参考

---

### 3.2 Mem0

**基本情報:**
- GitHub: https://github.com/mem0ai/mem0
- ライセンス: MIT
- スター数: ~2k (2025推定)

**特徴:**
- LLMメモリレイヤー
- 会話履歴グラフ化
- セマンティック検索 + グラフクエリ

**index-chanへの適用可能性:**

✅ **即座に再利用可能:**
- 会話履歴グラフ化
  - 会話依存グラフの構築に使える
- 重要度スコア（recency）
  - コンテキスト抽出の改善に使える
- 長期会話最適化
  - トークン削減に活用

**再利用コード候補:**
```rust
// 重要度スコア計算
struct ImportanceScorer {
    recency_weight: f32,
    relevance_weight: f32,
    frequency_weight: f32,
}

impl ImportanceScorer {
    fn score(&self, message: &Message, current_time: u64) -> f32 {
        // Mem0のスコアリングを参考
        let recency = self.calculate_recency(message.timestamp, current_time);
        let relevance = self.calculate_relevance(message);
        let frequency = self.calculate_frequency(message);
        
        recency * self.recency_weight +
        relevance * self.relevance_weight +
        frequency * self.frequency_weight
    }
}
```

**アクションアイテム:**
- [ ] 重要度スコアリングの実装
- [ ] セマンティック検索の統合
- [ ] 長期会話最適化の実装

---

## まとめ

### 優先度: 高（即座に再利用可能）

1. **code-graph-rag**: Tree-sitterベースのグラフ構築
2. **DepsRAG**: 依存グラフ構造と自動クエリ生成
3. **GraphRAG**: 階層的依存探索とエッジ定義
4. **LangGraph**: グラフベース状態管理とトークン削減
5. **Mem0**: 重要度スコアリングとコンテキスト抽出

### 優先度: 中（調査が必要）

1. **RAGFlow**: コンテキスト最適化技術
2. **CodexGraph**: 起点特定アルゴリズム
3. **LLMStack**: 品質スコア評価

### 優先度: 低（将来的に参考）

1. **Flowise**: UI設計パターン
2. **Langflow**: パターン学習とマーケットプレイス

---

## 次のステップ

### Phase 1.5（現在）
- [ ] DepsRAGの自動クエリ生成を実装
- [ ] LangGraphのトークン削減を統合
- [ ] Mem0の重要度スコアリングを実装

### Phase 2（多言語対応）
- [ ] code-graph-ragのマルチ言語パーサーを参考
- [ ] GraphRAGの階層的グラフ構造を実装

### Phase 3（UI実装）
- [ ] FlowiseのUI設計を参考
- [ ] Langflowのマーケットプレイス機能を検討

---

## 参考リンク

- [code-graph-rag GitHub](https://github.com/code-graph-rag)
- [RAGFlow GitHub](https://github.com/infiniflow/ragflow)
- [GraphRAG Docs](https://microsoft.github.io/graphrag/)
- [LangGraph Docs](https://langchain-ai.github.io/langgraph/)
- [Flowise GitHub](https://github.com/FlowiseAI/Flowise)
- [Mem0 GitHub](https://github.com/mem0ai/mem0)
