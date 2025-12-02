# Phase 1.5: LLM統合

## 概要

**期間**: 2週間  
**目標**: LLMによる高精度な判断と多言語対応の自動化  
**成果物**: index-chan CLI v0.1.5

---

## 実装スコープ

### 含めるもの

**LLMによるコード分析**
├─ コンテキスト収集（コード、コメント、git履歴）
├─ 削除可否の自動判定
├─ 判断理由の自然言語説明
├─ 確信度スコア（0-100%）
└─ 自動アノテーション機能

**多言語対応の自動化**
├─ LLMによる言語文法の学習
├─ 設定ファイルの自動生成
├─ `index-chan add-language <言語名>` コマンド
└─ ユーザーが自分で言語追加可能

**判断の高度化**
├─ 「将来使う予定」の検出
├─ 実験的機能の識別
├─ 古いコード vs 新しいコードの判別
├─ プロジェクトコンテキストの理解
└─ 自動アノテーション（#[allow(dead_code)]等）

---

## 技術設計

### アーキテクチャ

```
CLI (--llm フラグ)
 ↓
モデルダウンロード（初回のみ）
 ↓
コンテキスト収集
 ├─ コード内容
 ├─ コメント
 ├─ Git履歴
 └─ プロジェクト構造
 ↓
LLM推論（Candle + Qwen2.5-Coder）
 ↓
判定結果
 ├─ 削除推奨 / 保持推奨
 ├─ 確信度スコア
 └─ 理由の説明
 ↓
レポート生成
```

### データ構造

**LLM設定**
```rust
struct LLMConfig {
    model_name: String,
    model_path: Option<PathBuf>,
    temperature: f32,
    max_tokens: usize,
    confidence_threshold: f32,
}
```

**分析結果**
```rust
struct LLMAnalysis {
    should_delete: bool,
    confidence: f32,
    reason: String,
}
```

### プロンプト設計

```
あなたはコード分析の専門家です。以下の関数を削除すべきか分析してください。

関数: {name}
ファイル: {file_path}
行: {line_range}
エクスポート: {is_exported}

コンテキスト:
{context}

分析:
1. この関数は実際に使用されていますか？
2. 将来使用される可能性はありますか？
3. 歴史的な意義はありますか？
4. 削除すべきですか？

JSON形式で回答してください:
{
  "should_delete": true/false,
  "confidence": 0.0-1.0,
  "reason": "説明"
}
```

---

## 実装状況（2024-12-02更新）

### Week 1: Candle + Qwen統合 ✅

**完了 ✅**
├─ Cargo.tomlに依存関係追加
│  ├─ candle-core 0.8
│  ├─ candle-nn 0.8
│  ├─ candle-transformers 0.8
│  ├─ tokenizers 0.19
│  └─ hf-hub 0.3
├─ LLMモジュール構造作成
│  ├─ src/llm/mod.rs
│  ├─ src/llm/config.rs
│  ├─ src/llm/model.rs
│  ├─ src/llm/analyzer.rs
│  ├─ src/llm/downloader.rs
│  └─ src/llm/context.rs
├─ LLMConfig実装
│  ├─ デフォルト設定
│  └─ キャッシュディレクトリ管理
├─ ModelDownloader実装
│  ├─ HuggingFace Hubからのダウンロード
│  └─ キャッシュチェック機能
├─ ContextCollector実装
│  ├─ ファイル情報収集
│  └─ Git履歴取得
├─ LLMModel実装
│  ├─ Qwen2.5-Coder-1.5Bの統合
│  ├─ 推論パイプライン
│  └─ トークン生成ロジック
├─ LLMAnalyzer実装
│  ├─ プロンプト構築
│  ├─ LLM呼び出し
│  └─ レスポンス解析
└─ CLI統合（--llmフラグ追加）

### Week 2: 分析機能とCLI統合 🚧

**完了 ✅**
├─ プロンプトエンジニアリング
│  ├─ 効果的なプロンプト設計
│  ├─ カテゴリ分類（SafeToDelete, KeepForFuture等）
│  └─ JSON出力フォーマット改善
├─ JSON解析の強化
│  ├─ serde統合
│  ├─ フォールバック解析
│  └─ エラーハンドリング
├─ 削除可否判定ロジック
│  ├─ 確信度スコアの計算
│  ├─ カテゴリベースの判定
│  └─ 理由の自然言語生成
├─ CLI統合
│  ├─ scan --llm の完全実装
│  ├─ annotate --llm の完全実装
│  └─ LLM分析結果の反映
├─ 自動アノテーション機能
│  ├─ LLM分析結果との統合
│  ├─ #[allow(dead_code)]の自動追加（Rust）
│  ├─ @ts-ignore等の言語別対応（TypeScript）
│  ├─ # noqa対応（Python）
│  └─ コメント付与（理由説明）
└─ 基本動作確認
   ├─ ビルド成功
   ├─ 基本スキャン動作確認
   └─ アノテーション機能動作確認

**進行中 🚧**
├─ LLM推論の実機テスト
│  ├─ モデルダウンロードテスト
│  ├─ 推論速度の測定
│  └─ メモリ使用量の確認
└─ プロンプトの最適化
   ├─ Few-shot例の追加
   └─ 出力品質の改善

**残タスク 📋**
├─ コンテキスト収集の強化
│  ├─ コメント解析
│  ├─ Git履歴の詳細分析
│  └─ プロジェクト構造の理解
├─ 設定ファイル対応
│  ├─ .index-chan/config.json
│  ├─ モデル選択機能
│  └─ 設定コマンド実装
├─ エラーハンドリング改善
│  ├─ ネットワークエラー対応
│  ├─ モデルロードエラー対応
│  └─ ユーザーフレンドリーなメッセージ
├─ パフォーマンス最適化
│  ├─ バッチ処理
│  ├─ キャッシュ活用
│  └─ 並列処理
└─ テストとドキュメント
   ├─ 精度評価
   ├─ 使用例の作成
   └─ README更新

---

## 自動アノテーション機能

### 概要

LLMが「将来使う予定」「実験的機能」「WIP」と判断したコードに対して、自動的にアノテーションを追加する機能。コンパイラ警告を抑制しつつ、理由をコメントで残す。

### 使用例

```bash
# 自動アノテーション
$ index-chan annotate ./src --llm

🤖 LLMで分析中...
✅ 12個の未使用関数を検出

[削除推奨] 8個 → 削除候補としてマーク
[保持推奨] 4個 → アノテーション追加

📝 4個の関数にアノテーションを追加しました:
├─ futureFeature: #[allow(dead_code)] // WIP: 1週間前に追加
├─ experimentalAI: #[allow(dead_code)] // 実験的機能
└─ ...

# ドライラン
$ index-chan annotate ./src --llm --dry-run
```

### 言語別対応

**Rust**
```rust
// Before
fn future_feature() {
    // ...
}

// After
#[allow(dead_code)] // index-chan: WIP機能、1週間前に追加
fn future_feature() {
    // ...
}
```

**TypeScript**
```typescript
// Before
function experimentalFeature() {
    // ...
}

// After
// @ts-ignore - index-chan: 実験的機能、issue #123で議論中
function experimentalFeature() {
    // ...
}
```

**Python**
```python
# Before
def future_feature():
    pass

# After
def future_feature():  # noqa: F841 - index-chan: WIP機能
    pass
```

### 判定基準

**アノテーション追加の条件**
├─ LLMが「保持推奨」と判定
├─ 確信度が85%以上
├─ 理由が明確（WIP、実験的、将来使用予定等）
└─ ユーザーが承認（対話モード）

**アノテーションの内容**
├─ 言語別の警告抑制構文
├─ index-chanが追加したことを明記
├─ 判断理由を簡潔に記載
└─ 必要に応じてissue番号やコミットハッシュ

### CLIコマンド

```bash
# 基本
index-chan annotate <directory> --llm

# 自動モード（確信度90%以上のみ）
index-chan annotate <directory> --llm --auto --confidence 0.9

# ドライラン
index-chan annotate <directory> --llm --dry-run

# 特定のファイルのみ
index-chan annotate src/llm/ --llm
```

### 実装方針

**Phase 1.5 Week 2で実装**
├─ Rust対応のみ（#[allow(dead_code)]）
├─ 対話的モード
├─ ドライラン
└─ 理由コメント付与

**Phase 2以降で拡張**
├─ TypeScript対応（@ts-ignore）
├─ Python対応（# noqa）
├─ 自動モード
├─ 設定ファイルでカスタマイズ
└─ Git統合（コミット前に自動実行）

---

## 技術的な課題

### 課題1: Qwen2モデルのAPI変更

**問題**
- Candle 0.8のQwen2モデルは`forward`メソッドが3引数必要
- `forward(&input, seq_len, cache)`の形式

**対応**
├─ シーケンス長を明示的に渡す
├─ キャッシュは現時点ではNone
└─ 将来的にKVキャッシュを実装してパフォーマンス向上

### 課題2: トークン生成の最適化

**問題**
- 現在はグリーディサンプリングのみ
- 生成品質が低い可能性

**対応案**
├─ Top-kサンプリング実装
├─ Top-pサンプリング実装
├─ Temperature調整
└─ ビームサーチ（オプション）

### 課題3: メモリ使用量

**問題**
- 1.5Bモデルでも2-3GB必要
- 大規模プロジェクトでは負荷が高い

**対応案**
├─ バッチ処理の最適化
├─ モデルの量子化（将来）
├─ より小さいモデルの選択肢
└─ オンデマンドロード/アンロード

---

## 使用例（完成時）

### 基本的な使用

```bash
# 初回起動（モデルダウンロード）
$ index-chan scan ./src --llm

📥 初回起動: Qwen2.5-Coder-1.5Bをダウンロード中...
  モデルファイルをダウンロード中...
  トークナイザーをダウンロード中...
  設定ファイルをダウンロード中...
✅ ダウンロード完了

🤖 LLMで分析中...
  [1/12] oldAuthMethod を分析中...
  [2/12] deprecatedHelper を分析中...
  ...
✅ 12個の未使用関数を検出

[削除推奨] 8個（確信度 95%以上）
├─ oldAuthMethod
│  └─ 理由: 2年前に作成、新実装に置き換え済み
├─ deprecatedHelper
│  └─ 理由: コミットログに「deprecated」と記載
└─ ...

[保持推奨] 4個（確信度 85%以上）
├─ futureFeature
│  └─ 理由: 1週間前に追加、WIP状態
├─ experimentalAI
│  └─ 理由: 実験的機能、issue #123で議論中
└─ ...
```

### モデル設定

```bash
# 設定確認
$ index-chan config show

LLM設定
├─ モデル: Qwen/Qwen2.5-Coder-1.5B-Instruct
├─ Temperature: 0.7
├─ 最大トークン数: 512
└─ 確信度閾値: 0.85

# モデル変更
$ index-chan config set llm.model "Qwen/Qwen2.5-Coder-7B-Instruct"
✅ モデルを変更しました（次回起動時に適用）

# 確信度閾値変更
$ index-chan config set llm.confidence_threshold 0.9
✅ 確信度閾値を0.9に設定しました
```

---

## 成功指標

### 定量指標

**精度向上**
├─ 誤検出率: 5% → 1%以下
├─ 見落とし率: 10% → 5%以下
├─ 「将来使う予定」の検出精度: 80%以上
└─ ユーザー満足度: 大幅向上

**パフォーマンス**
├─ 初回ダウンロード時間: 5分以内
├─ モデルロード時間: 30秒以内
├─ 1関数あたりの分析時間: 2秒以内
└─ メモリ使用量: 3GB以下

**コスト**
├─ API料金: ゼロ（完全ローカル）
├─ プライバシー: コードが外部に送信されない
├─ 初回ダウンロード: 約1GB
└─ ディスク使用量: 約2GB

### 定性指標

**使いやすさ**
├─ 初回セットアップが簡単
├─ 判断理由が分かりやすい
├─ 確信度が信頼できる
└─ エラーメッセージが親切

**信頼性**
├─ 誤削除のリスクが極めて低い
├─ 判断が一貫している
├─ エッジケースに対応
└─ ユーザーが安心して使える

---

## 次のステップ

### Week 1完了後
├─ LLMモデルの動作確認
├─ 簡単なテストケースで精度検証
└─ Week 2のタスク詳細化

### Week 2完了後
├─ Phase 1.5完了
├─ v0.1.5リリース
├─ ユーザーフィードバック収集
└─ Phase 2準備開始

---

## 更新履歴

- 2024-12-02: 初版作成、Week 1進捗反映
