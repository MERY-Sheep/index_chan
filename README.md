# index-chan

TypeScriptプロジェクトのデッドコード検出CLI

## 概要

index-chanは、TypeScriptプロジェクト内の未使用コード（デッドコード）を検出し、安全に削除するためのCLIツールです。

## 機能

- TypeScriptのAST解析
- 関数呼び出しの依存グラフ構築
- 未使用関数・クラスの検出
- 安全性レベル評価（確実に安全/おそらく安全/要確認）
- 削除機能（対話的/自動）
- アノテーション機能（警告抑制コメント自動追加）
- **🆕 LLM統合**（Phase 1.5 ✅ 完了）
  - Qwen2.5-Coder-1.5Bによる高精度分析
  - 「将来使う予定」の自動検出
  - 実験的機能・WIPの識別
  - 完全ローカル実行（プライバシー保護）
  - 意味のある日本語応答を生成

## インストール

```bash
cargo install --path .
```

## 使い方

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

## LLMモード（Phase 1.5）

### 概要

LLMモードでは、Qwen2.5-Coder-1.5Bを使用して高精度な分析を行います。

**特徴**
- 完全ローカル実行（コードが外部に送信されない）
- 「将来使う予定」「実験的機能」「WIP」の自動検出
- Git履歴を考慮した判断
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

### LLMアノテーション

```bash
# LLMで分析して自動アノテーション
index-chan annotate test_project --llm

# 結果（TypeScriptの例）
// @ts-ignore - index-chan: 実験的機能、issue #123で議論中
function experimentalFeature() {
    // ...
}
```

### システム要件

**LLMモード使用時**
- メモリ: 3GB以上推奨
- ディスク: 3GB以上（モデルキャッシュ）
- 初回ダウンロード: 約3GB
- 推論速度: 約2秒/関数（CPU）

**通常モード**
- メモリ: 数十MB
- ディスク: 数MB

## 開発状況

現在Phase 1（デッドコード検出CLI）を開発中です。

### 完了
- [x] プロジェクト構造
- [x] 基本的なデータ構造
- [x] CLIインターフェース
- [x] tree-sitterによるTypeScript解析
- [x] ファイル走査とグラフ構築
- [x] 依存関係の追跡
- [x] デッドコード検出アルゴリズム
- [x] レポート生成機能（コンソール/JSON）
- [x] 削除機能（対話的/自動/ドライラン）
- [x] アノテーション機能（警告抑制）

### Phase 1.5完了 ✅
- [x] LLMモジュール完全実装
- [x] Candle + Qwen2.5統合完了
- [x] モデルダウンロード機構
- [x] コンテキスト収集（Git履歴）
- [x] LLM推論パイプライン完成
- [x] プロンプトエンジニアリング
- [x] JSON解析の堅牢化
- [x] CLI統合完了（scan --llm, annotate --llm）
- [x] 自動アノテーション機能（LLM統合）
- [x] LLM推論の実機テスト（Codexの支援により成功）
- [x] test-llmコマンド実装

### 今後の改善
- [ ] 実プロジェクトでの精度検証
- [ ] プロンプトの最適化
- [ ] エラーハンドリングの改善
- [ ] 設定ファイル対応

## テスト

```bash
# サンプルプロジェクトでテスト
cargo run -- scan test_project

# JSON出力
cargo run -- scan test_project --output report.json
```

## ライセンス

MIT License

## 注意事項

⚠️ **このプロジェクトは開発中です**

- Phase 1.5（LLM統合）が完了したばかりです
- 実プロジェクトでの精度検証は進行中です
- バグ報告や機能要望は大歓迎です
- 本番環境での使用は慎重に行ってください

## 貢献

現在は個人開発中ですが、Issue・PRは歓迎します。

