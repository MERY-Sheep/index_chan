# Contributing to index-chan

[日本語](CONTRIBUTING.ja.md) | [English](CONTRIBUTING.md)

index-chanへの貢献に興味を持っていただきありがとうございます！

## 開発状況

現在Phase 1.5（LLM統合）が完了し、Phase 2（多言語対応）の準備中です。

## 貢献方法

### バグ報告

GitHubのIssueで以下の情報を含めて報告してください：

- 実行環境（OS、Rustバージョン）
- 再現手順
- 期待される動作と実際の動作
- エラーメッセージ（あれば）

### 機能提案

新機能のアイデアがあれば、Issueで提案してください。
以下の点を含めると議論がスムーズです：

- ユースケース
- 期待される動作
- 実装の難易度（わかれば）

### プルリクエスト

1. このリポジトリをフォーク
2. 機能ブランチを作成（`git checkout -b feature/amazing-feature`）
3. 変更をコミット（`git commit -m 'feat: 素晴らしい機能を追加'`）
4. ブランチにプッシュ（`git push origin feature/amazing-feature`）
5. プルリクエストを作成

### コーディング規約

- Rustの標準的なスタイルに従う（`cargo fmt`を実行）
- `cargo clippy`の警告を解消
- 可能な限りテストを追加

### コミットメッセージ

Conventional Commitsに従ってください：

```
feat: 新機能
fix: バグ修正
docs: ドキュメント変更
refactor: リファクタリング
test: テスト追加
chore: その他の変更
```

## 開発環境のセットアップ

```bash
# リポジトリをクローン
git clone https://github.com/YOUR_USERNAME/index-chan.git
cd index-chan

# ビルド
cargo build

# テスト実行
cargo run -- scan test_project

# LLMモードのテスト
cargo run --release -- test-llm
```

## 質問

質問がある場合は、遠慮なくIssueで聞いてください。

## 重要な注意事項

**このプロジェクトは個人の実験的プロジェクトです。**

- 作者はプロフェッショナルなプログラマではありません
- 本番環境での使用は推奨しません
- バグや問題が含まれている可能性があります
- サポートは限定的です（ベストエフォート）
- 質問への回答は保証できません

**貢献について:**
- バグ報告は歓迎しますが、即座の対応は期待しないでください
- プルリクエストは歓迎しますが、レビューに時間がかかる場合があります
- このプロジェクトは学習目的で作成されています

## ライセンス

貢献したコードはMITライセンスの下で公開されます。
