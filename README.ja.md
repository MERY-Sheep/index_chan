# index-chan

[日本語](README.ja.md) | [English](README.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

TypeScriptプロジェクトのデッドコード検出CLI（Phase 1）

## 概要

**現在の機能（Phase 1）:**
index-chanは、TypeScriptプロジェクト内の未使用コード（デッドコード）を検出し、安全に削除するためのCLIツールです。

**将来のビジョン:**
最終的には、依存グラフとベクトル検索を組み合わせた「コード依存グラフ型検索システム」を目指しています。LLMが正確なコンテキストでコードを理解・編集できるようにする次世代の開発支援ツールです。詳細は[docs/VISION.ja.md](docs/VISION.ja.md)を参照してください。

**現在はPhase 1（デッドコード検出）の段階です。**

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

## 開発状況とロードマップ

### 現在の位置: Phase 1.5（LLM統合）完了 ✅

このプロジェクトは段階的に開発されています：

**Phase 1: デッドコード検出CLI** ✅ 完了
- TypeScript解析と依存グラフ構築
- 未使用コードの検出と削除

**Phase 1.5: LLM統合** ✅ 完了
- ローカルLLMによる高精度分析
- 「将来使う予定」のコード識別

**Phase 2: 多言語対応**（計画中）
- Rust, Python, Go, Javaなどへの対応
- より高度な依存関係解析

**Phase 3: コード依存グラフ型検索システム**（将来）
- ベクトル検索 + グラフ探索
- LLM向け最適化コンテキスト提供
- 統合コンテキスト編集

詳細なビジョンは[docs/VISION.ja.md](docs/VISION.ja.md)を参照してください。

### Phase 1 完了項目 ✅
- [x] TypeScript解析（tree-sitter）
- [x] 依存グラフ構築
- [x] デッドコード検出
- [x] 削除機能（対話的/自動）
- [x] アノテーション機能

### Phase 1.5 完了項目 ✅
- [x] LLM統合（Qwen2.5-Coder-1.5B）
- [x] ローカル推論
- [x] コンテキスト収集（Git履歴）
- [x] 高精度分析

### Phase 1.5 改善予定
- [ ] 実プロジェクトでの精度検証
- [ ] プロンプトの最適化
- [ ] エラーハンドリングの改善

### Phase 2 計画（多言語対応）
- [ ] Rust, Python, Go, Java対応
- [ ] 高度な依存関係解析
- [ ] インクリメンタル更新

### Phase 3 計画（検索システム）
- [ ] ベクトル検索統合
- [ ] ハイブリッド検索（ベクトル + グラフ）
- [ ] LLM向けコンテキスト最適化
- [ ] 統合コンテキスト編集

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
- Phase 1.5（LLM統合）が完了したばかりで、まだ不安定です
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

現在は個人開発中ですが、Issue・PRは歓迎します。

