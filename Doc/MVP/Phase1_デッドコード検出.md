# Phase 1: デッドコード検出CLI

## 概要

**期間**: 2ヶ月  
**目標**: 即座に価値を提供できる最小機能  
**成果物**: index-chan CLI v0.1.0

---

## 実装スコープ

### 含めるもの

**コア機能**
├─ TypeScript の AST 解析
├─ 関数呼び出しの依存グラフ構築
├─ 未使用関数・クラスの検出
├─ 安全性レベル評価
├─ CLI インターフェース
└─ 削除機能（オプション）

**安全性レベル**
├─ 確実に安全: export されていない、どこからも呼ばれていない
├─ おそらく安全: 内部でのみ使用、テストなし
└─ 要確認: export されている、動的呼び出しの可能性

### 含めないもの（Phase 2以降）

**除外する機能**
├─ 型情報の解析
├─ import/export の完全な追跡
├─ テストコードとの連携
├─ ベクトル検索
├─ 会話グラフ
├─ 他言語対応（Python等）
└─ IDE統合

---

## 技術設計

### アーキテクチャ

```
CLI
 ↓
Parser (tree-sitter)
 ↓
AST → 依存グラフ構築
 ↓
デッドコード検出
 ↓
安全性評価
 ↓
レポート生成
 ↓
削除実行（オプション）
```

### データ構造

**依存グラフ**
```rust
struct CodeGraph {
    nodes: HashMap<NodeId, CodeNode>,
    edges: Vec<DependencyEdge>,
}

struct CodeNode {
    id: NodeId,
    name: String,
    node_type: NodeType,  // Function, Class, Variable
    file_path: PathBuf,
    line_range: (usize, usize),
    is_exported: bool,
    is_used: bool,
}

enum NodeType {
    Function,
    Class,
    Method,
    Variable,
}

struct DependencyEdge {
    from: NodeId,
    to: NodeId,
    edge_type: EdgeType,
}

enum EdgeType {
    Calls,        // 関数呼び出し
    References,   // 参照
    Instantiates, // クラスのインスタンス化
}
```

### 検出アルゴリズム

```rust
fn detect_dead_code(graph: &CodeGraph) -> Vec<DeadCode> {
    let mut dead_code = Vec::new();
    
    for (id, node) in &graph.nodes {
        // エントリーポイントは除外
        if is_entry_point(node) {
            continue;
        }
        
        // 使用されているかチェック
        let is_used = graph.edges.iter()
            .any(|edge| edge.to == *id);
        
        if !is_used {
            let safety = evaluate_safety(node, graph);
            dead_code.push(DeadCode {
                node: node.clone(),
                safety_level: safety,
            });
        }
    }
    
    dead_code
}

fn evaluate_safety(node: &CodeNode, graph: &CodeGraph) -> SafetyLevel {
    // export されている場合は要確認
    if node.is_exported {
        return SafetyLevel::NeedsReview;
    }
    
    // ファイル名に "test" が含まれる場合は要確認
    if node.file_path.to_string_lossy().contains("test") {
        return SafetyLevel::NeedsReview;
    }
    
    // 動的呼び出しの可能性をチェック
    if has_dynamic_call_risk(node) {
        return SafetyLevel::ProbablySafe;
    }
    
    SafetyLevel::DefinitelySafe
}

enum SafetyLevel {
    DefinitelySafe,   // 確実に安全
    ProbablySafe,     // おそらく安全
    NeedsReview,      // 要確認
}
```

---

## CLI インターフェース

### コマンド

**基本コマンド**
```bash
# スキャン（検出のみ）
index-chan scan <directory>

# 削除（対話的）
index-chan clean <directory>

# 削除（自動、確実に安全なもののみ）
index-chan clean <directory> --auto --safe-only

# ドライラン
index-chan clean <directory> --dry-run

# レポート出力
index-chan scan <directory> --output report.json
```

### 出力フォーマット

**標準出力**
```
🔍 デッドコード検出結果
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📁 プロジェクト: ./src
📊 総ファイル数: 150
📊 総関数数: 1,234
🗑️  未使用関数: 12個 (234行)

[確実に安全] 8個
├─ src/old-auth.ts:45-78 (oldAuthMethod)
├─ src/utils.ts:123-145 (deprecatedHelper)
└─ ...

[おそらく安全] 3個
├─ src/logger.ts:34-38 (emptyLogger)
└─ ...

[要確認] 1個
└─ src/api.ts:89-102 (unusedEndpoint)

💾 削減可能な行数: 234行 (全体の 8%)
💰 削減可能なトークン数: 約 4,680トークン
```

**JSON出力**
```json
{
  "summary": {
    "total_files": 150,
    "total_functions": 1234,
    "dead_code_count": 12,
    "dead_code_lines": 234,
    "reduction_percent": 8.0
  },
  "dead_code": [
    {
      "file": "src/old-auth.ts",
      "name": "oldAuthMethod",
      "line_start": 45,
      "line_end": 78,
      "safety_level": "definitely_safe",
      "reason": "Not exported, no references"
    }
  ]
}
```

---

## 開発計画

### Week 1-2: 基盤構築
├─ プロジェクト構造
├─ tree-sitter 統合
├─ TypeScript パーサー
└─ 基本的な AST 解析

### Week 3-4: グラフ構築
├─ 依存グラフのデータ構造
├─ 関数呼び出しの抽出
├─ グラフ構築アルゴリズム
└─ テストケース作成

### Week 5-6: デッドコード検出
├─ 未使用コードの検出アルゴリズム
├─ 安全性評価
├─ エッジケースの処理
└─ 精度検証

### Week 7: CLI構築
├─ clap による CLI
├─ レポート生成
├─ 出力フォーマット
└─ エラーハンドリング

### Week 8: 削除機能
├─ ファイル編集
├─ 対話的削除
├─ ドライラン
└─ ロールバック機能

---

## テスト戦略

### ユニットテスト
├─ パーサーのテスト
├─ グラフ構築のテスト
├─ 検出アルゴリズムのテスト
└─ 安全性評価のテスト

### 統合テスト
├─ 実際のプロジェクトでのテスト
├─ エッジケースのテスト
└─ パフォーマンステスト

### 精度検証
├─ 既知のデッドコードを含むプロジェクト
├─ 誤検出率の測定
├─ 見落とし率の測定
└─ 安全性評価の妥当性

---

## リスクと対策

### リスク1: 動的呼び出しの見落とし

**例**
```typescript
// 動的呼び出し
const funcName = "myFunction";
window[funcName]();

// リフレクション
Reflect.get(obj, "myMethod")();
```

**対策**
├─ 保守的に判定（疑わしい場合は「要確認」）
├─ ユーザーに確認を促す
└─ Phase 2 で高度な解析を追加

### リスク2: テストコードの誤検出

**例**
```typescript
// テストでのみ使用される関数
export function testHelper() { ... }
```

**対策**
├─ テストファイルは別扱い
├─ export されている関数は「要確認」
└─ ユーザーが除外パターンを指定可能

### リスク3: パフォーマンス

**対策**
├─ 並列処理
├─ インクリメンタル解析（Phase 2）
└─ キャッシュ機構（Phase 2）

---

## 成功指標

### 定量指標

**精度**
├─ 検出精度: 90% 以上
├─ 誤検出率: 5% 以下
└─ 見落とし率: 10% 以下

**パフォーマンス**
├─ 1000ファイルで 10秒以内
├─ メモリ使用量: 500MB 以下
└─ CPU使用率: 適切

**削減効果**
├─ 平均削減率: 5〜10%
├─ 最大削減率: 20% 以上（レガシープロジェクト）
└─ トークン削減: 行数 × 20トークン

### 定性指標

**使いやすさ**
├─ インストールが簡単
├─ 使い方が直感的
└─ エラーメッセージが分かりやすい

**信頼性**
├─ 誤削除のリスクが低い
├─ 安全性評価が妥当
└─ ユーザーが安心して使える

---

## リリース計画

### v0.1.0-alpha（Week 6）
├─ 検出機能のみ
├─ 内部テスト
└─ フィードバック収集

### v0.1.0-beta（Week 7）
├─ CLI完成
├─ 限定公開
└─ ユーザーテスト

### v0.1.0（Week 8）
├─ 削除機能追加
├─ 正式リリース
└─ ドキュメント完備

---

## 次のステップ（Phase 2への準備）

### Phase 1で得られる知見
├─ tree-sitter の使い方
├─ 依存グラフの構築方法
├─ TypeScript の特性
├─ ユーザーのニーズ
└─ 技術的な課題

### Phase 2への橋渡し
├─ グラフ構造の拡張準備
├─ 多言語対応の基盤
├─ ベクトル検索の統合ポイント
└─ 永続化の検討

---

## 更新履歴

- 2025-12-02: 初版作成
