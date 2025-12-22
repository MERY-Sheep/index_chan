<div align="center">
  <img src="ascii_image.png" alt="index-chan" width="600">
  
  # index-chan
  
  [日本語](README.ja.md) | [English](README.md)
  
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
  
  LLMエージェント向けコード解析・変更ツール（Phase 7.2完了 - Concept Transformer統合！）
</div>

## 概要

**🎉 Phase 7.2 完了！（Concept Transformer 統合 - v0.4.0）**

index-chan は、LLM エージェント（Kiro、Cursor 等）向けに設計されたコード解析・変更ツールです。9 個の MCP（Model Context Protocol）ツールを提供し、LLM が安全にコードを理解・修正できるようにします。

**主な機能:**

- **デッドコード検出**: 未使用コードを自動検出
- **コンテキスト生成**: 関数と依存関係を自動収集（S/N比スコア付き）
- **セマンティック検索**: Concept Transformer 統合によるグラフ探索
- **一括変更**: 変更の検証・プレビュー・安全な適用
- **Import 検証**: 依存グラフを使用して LLM のハルシネーションを防止
- **自動バックアップ**: タイムスタンプ付きで安全性を確保

**アーキテクチャ:**

```
LLMエージェント（Kiro/Cursor）
    ↓ MCPプロトコル
index-chan MCPサーバー
    ↓ 依存グラフ
TypeScriptプロジェクト
```

## 機能

### MCP ツール（Phase 6 ✅ 完了）

**LLM エージェント向けの 9 個の MCP ツール:**

**基本機能:**

1. **scan**: デッドコード検出
2. **search**: コード検索（要インデックス作成）
3. **stats**: プロジェクト統計

**コンテキスト生成:**
4. **gather_context**: 関数と依存関係を含むコンテキスト生成（S/N比スコア付き）
5. **get_dependencies**: 指定関数の依存先を取得（修飾名検索対応）
6. **get_dependents**: 指定関数の依存元を取得（重複除去済み）

**一括変更:** 7. **validate_changes**: 変更の妥当性を検証 8. **preview_changes**: 変更内容を差分表示 9. **apply_changes**: 検証済み変更を安全に適用

**グラフ検索（Phase 7）:**
10. **search_with_graph**: グラフ探索 + セマンティック検索（デフォルト有効）

### Phase 7.2 新機能 🆕

**S/N比スコア（Concept Transformer 統合）:**

`gather_context` が返すコンテキストに品質メトリクスが付与されます：

```
// ===== QUALITY METRICS =====
// Estimated tokens: 1200
// S/N ratio: 2.35
// Quality: high
// =============================
```

- **S/N比**: 意味のある識別子（3文字以上）と短い変数名（1-2文字）の比率
- **Quality**: high (S/N > 2.0) / medium (1.0-2.0) / low (< 1.0)
- **Recommendation**: 品質が低い場合の改善提案

**修飾名検索:**

`get_dependencies` / `get_dependents` で絞り込み検索が可能：

```
# ファイル名で絞り込み
get_dependencies("graph_search.rs::new")

# 型名で絞り込み（シグネチャ内のimpl Typeにマッチ）
get_dependencies("GraphSearcher::new")
```

**skeleton モードのシグネチャ表示:**

```
// Before
search_semantic Function // ...

// After
pub fn search_semantic(&self, query: &str, ...) -> Vec<GraphSearchResult>
```

### コア機能

- **TypeScript AST 解析**: tree-sitter による高速解析
- **依存グラフ**: 構築と解析
- **デッドコード検出**: 未使用関数・クラスの検出
- **安全性レベル評価**: 確実に安全/おそらく安全/要確認
- **対話的・自動削除**: 柔軟な削除モード
- **アノテーション機能**: 警告抑制コメント自動追加
- **Import 検証**: 依存グラフを使用（LLM のハルシネーション防止）
- **自動バックアップ**: タイムスタンプ付き
- **Undo 機能**: マニフェスト方式で安全な復元（Phase 7.1 ✅）
- **.indexchanignore**: スキャン対象の柔軟な除外（Phase 7.1 ✅）

## インストール

```bash
cargo install --path .
```

## クイックスタート

### LLM エージェント（Kiro/Cursor）向け

**1. index-chan をビルド:**

```bash
cargo build --release
```

**2. Kiro で MCP を設定:**

`~/.kiro/settings/mcp.json`を編集:

```json
{
  "mcpServers": {
    "index-chan": {
      "command": "/path/to/index-chan/target/release/index-chan",
      "args": ["mcp-server"],
      "disabled": false,
      "autoApprove": [
        "scan",
        "stats",
        "search",
        "gather_context",
        "get_dependencies",
        "get_dependents"
      ]
    }
  }
}
```

**3. LLM から使用:**

```
ユーザー: 「認証機能にレート制限を追加して」

LLM: index-chan.gather_context({
       directory: ".",
       entry_point: "authenticateUser",
       depth: 2
     })
     → 関連コードを取得

LLM: コードを修正

LLM: index-chan.validate_changes({...})
     → 変更を検証

LLM: index-chan.preview_changes({...})
     → 差分を表示

ユーザー: 「適用して」

LLM: index-chan.apply_changes({...})
     → バックアップ付きで適用完了
```

## CLI 使用方法

### スキャン（検出のみ）

```bash
# 基本的なスキャン
index-chan scan <directory>

# JSON出力
index-chan scan <directory> --output report.json

# LLM分析モード（Phase 1.5 ✅）
index-chan scan <directory> --llm
```

### 削除（対話的）

```bash
# 対話的に確認しながら削除
index-chan clean <directory>
```

### 削除（自動、確実に安全なもののみ）

```bash
# 確実に安全なもののみ自動削除
index-chan clean <directory> --auto --safe-only
```

### ドライラン

```bash
# 実際には削除せず、動作確認のみ
index-chan clean <directory> --dry-run
```

### アノテーション追加

```bash
# 「将来使う予定」のコードに警告抑制アノテーションを追加
index-chan annotate <directory>

# ドライラン
index-chan annotate <directory> --dry-run

# LLM分析モード（高精度）
index-chan annotate <directory> --llm
```

### グラフエクスポート（Phase 3.1 ✅）

```bash
# GraphML形式（Gephi、yEd、Cytoscapeで開ける）
index-chan export <directory> -o graph.graphml -f graphml

# DOT形式（Graphvizで可視化）
index-chan export <directory> -o graph.dot -f dot

# JSON形式（カスタム可視化用）
index-chan export <directory> -o graph.json -f json
```

**Graphviz での可視化:**

```bash
# SVG出力
dot -Tsvg graph.dot -o graph.svg

# PNG出力（3Dレイアウト）
neato -Tpng graph.dot -o graph.png
```

### 3D Web 可視化（Phase 3.2 ✅）

```bash
# Web機能を有効にしてビルド
cargo build --features web --release

# Webサーバー起動
cargo run --features web --release -- visualize <directory> --port 8080

# ブラウザ自動起動
cargo run --features web --release -- visualize <directory> --port 8080 --open
```

**機能:**

- Three.js + force-graph-3d によるインタラクティブ 3D グラフ
- リアルタイム統計（ノード数、エッジ数、未使用数）
- ノードクリックで詳細表示
- カメラ操作（回転、ズーム、パン）
- ダークテーマ UI

**ブラウザで開く:** http://localhost:8080

### データベース層（Phase 4 ✅）

**Phase 4.0: 基礎機能**

```bash
# DB機能を有効にしてビルド
cargo build --features db --release

# プロジェクト初期化
cargo run --features db --release -- init <directory>

# 統計表示
cargo run --features db --release -- stats <directory>

# ファイル監視（リアルタイム更新）
cargo run --features db --release -- watch <directory>
```

**Phase 4.1: 既存コマンドの DB 統合**

```bash
# DBから高速スキャン
cargo run --features db --release -- scan <directory> --use-db

# DBから高速エクスポート
cargo run --features db --release -- export <directory> -o graph.json -f json --use-db

# DBから高速可視化
cargo run --features db,web --release -- visualize <directory> --use-db
```

**機能:**

- SQLite による永続化
- ファイルハッシュベースの変更検知
- リアルタイムファイル監視
- 自動データベース更新
- プロジェクト統計
- 既存コマンドの DB 対応（再スキャン不要）

**理想的なワークフロー:**

```bash
# 1. プロジェクト初期化（一度だけ）
$ cargo run --features db --release -- init test_project

🔧 プロジェクトを初期化中: test_project
✅ セットアップ完了！

📊 プロジェクト統計:
  ファイル数: 2
  関数数: 13
  依存関係: 1
  デッドコード: 13 個 (100.0%)

# 2. ファイル監視開始（バックグラウンド）
$ cargo run --features db --release -- watch test_project

👀 ファイル監視を開始: test_project
✅ 監視開始（Ctrl+Cで終了）

[23:38:20] 🔄 変更: sample.ts
   ✅ データベースを更新

[23:38:34] 📄 追加: new_file.ts
   ✅ データベースを更新

# 3. 高速スキャン（DBから、再スキャン不要）
$ cargo run --features db --release -- scan test_project --use-db

💾 Using database
📂 データベースから読み込み中...
🗑️  Unused Functions: 11 (38 lines)

# 4. 統計確認
$ cargo run --features db --release -- stats test_project

📊 プロジェクト統計: test_project
  ファイル数: 3
  関数数: 15
  デッドコード: 15 個 (100.0%)

# 5. 高速エクスポート（DBから）
$ cargo run --features db --release -- export test_project -o graph.json -f json --use-db

💾 Using database
✅ JSON形式でエクスポート完了
```

**特徴:**

- 一度 init すれば、あとは自動で追跡
- watch が変更を検知して自動更新
- すべてのコマンドが--use-db で高速化
- 再スキャン不要

## LLM モード（Phase 1.5）

### 概要

LLM モードでは、Qwen2.5-Coder-1.5B を使用して高精度な分析を行います。

**特徴**

- 完全ローカル実行（コードが外部に送信されない）
- 「将来使う予定」「実験的機能」「WIP」の自動検出
- Git 履歴を考慮した判断
- 確信度スコア付き

### 推論テスト

```bash
# トークナイザーのみテスト
cargo run --release -- test-llm --tokenizer-only

# 推論テスト（デフォルトプロンプト）
cargo run --release -- test-llm

# カスタムプロンプトでテスト
cargo run --release -- test-llm --prompt "このコードにバグはありますか？"

# 実際の出力例
🤖 LLM推論テスト開始

📝 プロンプト:
この関数は削除しても安全ですか？

function unusedHelper() {
  return 42;
}

✅ 推論成功！

📤 応答:
この関数は削除しても安全です。JavaScript では、関数はスコープ内に存在し、
関数の定義はそのスコープの外に存在します。つまり、関数はスコープ外
```

### 実際のプロジェクトでの使用

```bash
# 初回起動（モデルダウンロード、約3GB）
cargo run -- scan test_project --llm

# 出力例
🔍 Scanning directory: test_project
🤖 LLM分析モード有効

📥 初回起動: Qwen2.5-Coder-1.5Bをダウンロード中...
✅ モデル読み込み完了

🤖 LLMで分析中...
✅ LLM分析完了

[削除推奨] 8個（確信度 95%以上）
├─ oldAuthMethod: 2年前に作成、新実装に置き換え済み (確信度: 95%)
└─ deprecatedHelper: コミットログに「deprecated」と記載 (確信度: 98%)

[保持推奨] 4個（確信度 85%以上）
├─ futureFeature: 1週間前に追加、WIP状態 (確信度: 90%)
└─ experimentalAI: 実験的機能、issue #123で議論中 (確信度: 88%)
```

### LLM アノテーション

```bash
# LLMで分析して自動アノテーション
index-chan annotate test_project --llm

# 結果（TypeScriptの例）
// @ts-ignore - index-chan: 実験的機能、issue #123で議論中
function experimentalFeature() {
    // ...
}
```

## Phase 2 の新機能

### コード検索

```bash
# インデックス作成
index-chan index ./src

# コード検索
index-chan search "authentication" --context -k 5

# 出力例
🔍 Searching: authentication
📊 Found 5 results:

1. authenticateUser (score: 0.92)
   📄 src/auth.ts:45:78
   📝 Code:
      function authenticateUser(username, password) {
        return checkCredentials(username, password);
      }
```

### 会話履歴の分析

```bash
# トピック抽出（キーワードベース）
index-chan topics chat_history.json

# 出力例
📚 トピック抽出: chat_history.json

📊 5個のトピックを検出:

1. デッドコード検出 (4 messages)
   キーワード: デッドコード, 検出, 削除

2. データベース接続エラー (4 messages)
   キーワード: データベース, 接続, エラー

# LLMによる高精度トピック検出
index-chan topics chat_history.json --llm

# 出力例
🤖 LLM分析モード有効
📊 3個のトピックを検出:

1. TypeScriptデッドコード検出とクリーンアップ
   メッセージ数: 4
   キーワード: デッドコード, 検出, 削除, LLM分析
```

### 関連メッセージ検索

```bash
# 関連メッセージを検索
index-chan related chat_history.json "エラー" -k 3 --context

# 出力例
🔍 関連メッセージ検索: chat_history.json
📝 クエリ: エラー

📊 3件の関連メッセージを発見:

1. [user] 2024-12-02T10:15:00Z (類似度: 0.850)
   💬 データベース接続エラーが出ています
   📖 コンテキスト:
      [assistant] 接続文字列を確認してください

🎯 トークン削減効果:
  全体トークン数: 233
  関連トークン数: 141
  削減率: 39.5%
```

### 会話グラフ UI & プロンプト可視化（Phase 2.5 🚧）

```bash
# Web機能を有効にしてビルド
cargo build --features web --release

# 会話グラフUIを起動
cargo run --features web --release -- visualize-chat test_project/chat_history.json --prompt-file test_project/prompt_history.json --port 8081

# ブラウザ自動起動
cargo run --features web --release -- visualize-chat test_project/chat_history.json --prompt-file test_project/prompt_history.json --port 8081 --open
```

**機能:**

- Cytoscape.js によるインタラクティブな会話グラフ
- 削減されたメッセージの視覚化（透明度、色分け）
- プロンプト履歴の表示（シンタックスハイライト）
- グラフとプロンプトの連携（クリックで相互ジャンプ）
- トークン削減率の表示
- リアルタイム統計

**ブラウザで開く:** http://localhost:8081

**プロンプト履歴の表示:**

```bash
# プロンプト統計のみ表示
index-chan show-prompts test_project/prompt_history.json --stats

# 出力例
📊 プロンプト統計:
  総プロンプト数: 3
  総トークン数: 1368
  平均トークン数: 456

# 全プロンプトを表示
index-chan show-prompts test_project/prompt_history.json

# 特定のノードIDを含むプロンプトを検索
index-chan show-prompts test_project/prompt_history.json --node-id "0"
```

### Embedding モデルのテスト

```bash
# 基本テスト
index-chan test-embedding

# 類似度比較テスト
index-chan test-embedding --compare

# 出力例
🧪 Embeddingモデルのテスト

📊 類似度比較テスト:

テキスト1: function authenticate(user, password) { return true; }
テキスト2: function login(username, pwd) { return checkCredentials(username, pwd); }
テキスト3: function calculateTotal(items) { return items.reduce(...); }

📈 類似度スコア:
  テキスト1 vs テキスト2 (認証関連): 0.8542
  テキスト1 vs テキスト3 (異なる機能): 0.3214
  テキスト2 vs テキスト3 (異なる機能): 0.2987

💡 期待される結果:
  - 認証関連の関数同士（1 vs 2）の類似度が高い
  - 異なる機能の関数（1 vs 3, 2 vs 3）の類似度が低い
```

### システム要件

**LLM モード使用時**

- メモリ: 3GB 以上推奨
- ディスク: 3GB 以上（モデルキャッシュ）
- 初回ダウンロード: 約 3GB
- 推論速度: 約 2 秒/関数（CPU）

**通常モード**

- メモリ: 数十 MB
- ディスク: 数 MB

## 開発状況とロードマップ

### 🎉 MVP 達成！（Phase 6 完了）

**Phase 1: デッドコード検出 CLI** ✅ 完了

- TypeScript 解析と依存グラフ構築
- 未使用コードの検出と削除

**Phase 1.5: LLM 統合** ✅ 完了

- ローカル LLM による高精度分析
- 「将来使う予定」のコード識別

**Phase 2: 検索 + 会話グラフ** ✅ 完了

- ベクトル検索によるコード検索
- 会話グラフによるチャット履歴分析
- トークン削減（39.5〜60%達成）

**Phase 3: グラフ可視化** ✅ 完了

- GraphML/DOT/JSON エクスポート
- 3D Web 可視化

**Phase 4: データベース層** ✅ 完了

- SQLite 永続化
- ファイル監視と自動更新

**Phase 6: MCP 統合** ✅ 完了（MVP！）

- LLM エージェント向け 9 個の MCP ツール
- 依存関係を含むコンテキスト生成
- 検証付き一括変更
- Import 検証（ハルシネーション防止）
- 自動バックアップ

**Phase 7: GraphRAG 統合** ✅ 完了

- Rust パーサー強化（メソッドチェーン、マクロ、scoped_identifier 検出）
- セマンティック検索（embedding 類似度 + グラフ探索）
- スコア計算改善（探索先ノードも類似度再計算）
- search ツールを search_with_graph に統合

**Phase 7.2: Concept Transformer 統合** ✅ 完了 🆕

- **S/N比スコア**: gather_context の品質メトリクス（Concept Transformer 理論に基づく）
- **シグネチャ表示**: skeleton モードで関数シグネチャを出力
- **修飾名検索**: `file.rs::func` や `Type::func` 形式での絞り込み検索
- **重複除去**: get_dependencies/get_dependents の結果から重複を排除
- **semantic 検索デフォルト化**: embedding 検索がデフォルトで有効

**Phase 5: Tauri デスクトップアプリ** 🗑️ 削除済み

- MCP/CLI に集中するためコード削除

詳細なビジョンは[docs/VISION.ja.md](docs/VISION.ja.md)、ロードマップは[Doc/MVP/MVP\_ロードマップ.md](Doc/MVP/MVP_ロードマップ.md)を参照してください。

### 完了した Phase ✅

**Phase 1: デッドコード検出**

- [x] TypeScript 解析（tree-sitter）
- [x] 依存グラフ構築
- [x] デッドコード検出
- [x] 削除機能（対話的/自動）
- [x] アノテーション機能

**Phase 1.5: LLM 統合**

- [x] LLM 統合（Qwen2.5-Coder-1.5B）
- [x] ローカル推論
- [x] コンテキスト収集（Git 履歴）
- [x] 高精度分析

**Phase 2: 検索 + 会話グラフ**

- [x] ベクトル検索基盤
- [x] 会話グラフ基盤
- [x] CLI 統合
- [x] Embedding モデル統合（Candle による BERT）
- [x] トピック検出
- [x] 関連メッセージ検索
- [x] トークン削減（39.5〜60%達成）

**Phase 3: グラフ可視化**

- [x] GraphML/DOT/JSON エクスポート
- [x] 3D Web 可視化（Three.js + force-graph-3d）

**Phase 4: データベース層**

- [x] SQLite 永続化
- [x] ファイルハッシュベースの変更検知
- [x] ファイル監視と自動更新
- [x] 既存コマンドの DB 統合

**Phase 6: MCP 統合（MVP！）**

- [x] MCP サーバー実装（JSON-RPC 2.0、stdio）
- [x] 9 個の MCP ツール（scan、search、stats、gather_context 等）
- [x] 依存関係を含むコンテキスト生成
- [x] 一括変更（validate、preview、apply）
- [x] 依存グラフを使用した Import 検証
- [x] タイムスタンプ付き自動バックアップ
- [x] 統合テスト

**Phase 7: GraphRAG 統合**

- [x] Rust パーサー強化（メソッドチェーン、マクロ検出）
- [x] セマンティック検索（embedding + グラフ探索）
- [x] スコア計算改善
- [x] search_with_graph ツール

**Phase 7.2: Concept Transformer 統合** 🆕

- [x] S/N比スコア（コンテキスト品質メトリクス）
- [x] シグネチャ表示（skeleton モード改善）
- [x] 修飾名検索（file.rs::func, Type::func）
- [x] 重複除去（get_dependencies/get_dependents）
- [x] semantic 検索デフォルト有効化

### 次のステップ

**短期:**

- 実運用でのフィードバック収集
- エラーハンドリングの改善
- パフォーマンス最適化

**中期:**

- TypeScript 型チェック統合
- ESLint 統合
- テスト自動実行

**長期:**

- 多言語対応（JavaScript、Python、Rust）
- 変更履歴の Web UI
- 他の LLM エージェント対応（Claude、ChatGPT）

## テスト

```bash
# サンプルプロジェクトでテスト
cargo run -- scan test_project

# JSON出力
cargo run -- scan test_project --output report.json
```

## 免責事項

**このプロジェクトを使用する前に[DISCLAIMER.md](DISCLAIMER.md)を必ずお読みください。**

これは個人の実験的プロジェクトです。作者はプロフェッショナルなプログラマではなく、プロフェッショナルなサポートを提供できません。

## ライセンス

MIT License

## 注意事項

⚠️ **重要な免責事項**

**このプロジェクトは個人の実験的プロジェクトです。**

- 作者はプロフェッショナルなプログラマではありません
- Phase 1.5（LLM 統合）が完了したばかりで、まだ不安定です
- 本番環境での使用は推奨しません
- バグや問題が含まれている可能性が高いです
- サポートは限定的です（質問に答えられない場合があります）
- 使用は自己責任でお願いします

**貢献について:**

- バグ報告は歓迎しますが、即座の対応は保証できません
- このプロジェクトは学習・実験目的で作成されています

## ドキュメント

- [docs/](docs/): 設計書・企画書（英語）
- [Doc/](Doc/): 開発メモ・調査資料（日本語、非公開）

## 貢献

現在は個人開発中ですが、Issue・PR は歓迎します。
