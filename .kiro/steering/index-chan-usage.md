---
inclusion: always
---

# index-chan 使用ガイド

このプロジェクトでは、開発中のindex-chanツールを活用できます。

## 利用可能なコマンド

### TypeScriptファイルのスキャン

```bash
# test_projectのスキャン
cargo run -- scan test_project

# JSON出力
cargo run -- scan test_project --output report.json
```

### アノテーション追加

```bash
# ドライラン（確認のみ）
cargo run -- annotate test_project --dry-run

# 実際に追加
cargo run -- annotate test_project
```

### クリーニング

```bash
# ドライラン
cargo run -- clean test_project --dry-run

# 実際に削除
cargo run -- clean test_project
```

## 自動判断の基準

以下の場合、自動的にindex-chanを使用してコード品質を確認してください：

### スキャンを実行すべき状況

- TypeScriptファイルを大量に追加・変更した時
- リファクタリング後
- デッドコードの疑いがある時
- コードレビュー前

### アノテーションを追加すべき状況

- 「将来使う予定」のコードがある時
- 実験的機能を実装中の時
- WIP（Work In Progress）のコードがある時
- コンパイラ警告を一時的に抑制したい時

### 実行例

```bash
# ユーザーがTypeScriptファイルを編集した後
$ cargo run -- scan test_project
🔍 デッドコード検出結果
🗑️  未使用関数: 2個 (6行)

# 結果を確認して、必要に応じてアノテーション
$ cargo run -- annotate test_project --dry-run
📝 アノテーション追加: 2個

# 問題なければ実行
$ cargo run -- annotate test_project
✅ アノテーションを追加しました
```

## 注意事項

### 現在の制限

- **TypeScriptのみ対応**: Rustファイル（src/）は未対応
- **Phase 1.5開発中**: LLM機能は準備中
- **test_projectで試す**: 本番コードへの適用は慎重に

### 安全な使い方

1. 必ず`--dry-run`で確認
2. Gitでコミット後に実行
3. 変更内容を確認してからコミット
4. 重要なファイルはバックアップ

## 自動化の提案

将来的には以下のような自動化が可能です：

### Git Hookとの連携

```bash
# pre-commit hook
cargo run -- scan . --output .index-chan-report.json
```

### CI/CDでの活用

```yaml
# GitHub Actions例
- name: Dead Code Check
  run: |
    cargo run -- scan test_project --output report.json
    # 閾値を超えたら警告
```

### Kiro Hookの活用

- ファイル保存時に自動スキャン
- デッドコード検出時に通知
- 定期的なレポート生成

## メタ的な活用

index-chan自身の開発にindex-chanを使う：

```bash
# Phase 2で多言語対応完了後
cargo run -- scan src --language rust
cargo run -- annotate src --language rust --llm

# 結果
#[allow(dead_code)] // index-chan: Phase 1.5で使用予定
fn future_feature() { ... }
```

## トラブルシューティング

### ビルドエラー

```bash
# クリーンビルド
cargo clean
cargo build
```

### 予期しない結果

```bash
# 詳細ログ（将来実装予定）
cargo run -- scan test_project --verbose
```

### 元に戻す

```bash
# Gitで元に戻す
git checkout test_project/sample.ts
```

## 開発フロー例

```bash
# 1. 機能開発
# ... TypeScriptコードを編集 ...

# 2. デッドコードチェック
cargo run -- scan test_project

# 3. 必要に応じてアノテーション
cargo run -- annotate test_project --dry-run
cargo run -- annotate test_project

# 4. コミット
git add .
git commit -m "feat: 新機能追加"

# 5. 定期的なクリーンアップ
cargo run -- clean test_project --dry-run
# 確認後
cargo run -- clean test_project --auto --safe-only
```

## まとめ

index-chanは開発中のツールですが、すでに実用的な機能を提供しています。
積極的に活用して、コード品質の向上とメンテナンス性の改善に役立ててください。
