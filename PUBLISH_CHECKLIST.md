# GitHub公開前チェックリスト

## ✅ 完了した項目

- [x] LICENSEファイル作成（MIT License）
- [x] CONTRIBUTING.md作成
- [x] SECURITY.md作成
- [x] README.en.md作成（英語版）
- [x] .github/workflows/ci.yml作成（CI/CD）
- [x] .github/ISSUE_TEMPLATE作成
- [x] Cargo.tomlのedition修正（2024→2021）
- [x] Cargo.tomlにrepository, keywords追加
- [x] .gitignoreの見直し

## ⚠️ 公開前に確認すべき項目

### 1. Cargo.tomlの更新

```toml
repository = "https://github.com/YOUR_USERNAME/index-chan"
```

↑ YOUR_USERNAMEを実際のGitHubユーザー名に変更してください

### 2. 個人情報の確認

以下のファイルに個人情報が含まれていないか確認：

- [ ] Docフォルダ内のドキュメント
- [ ] Gitコミット履歴
- [ ] test_projectのサンプルコード

### 3. 機密情報の確認

- [ ] APIキーやトークンが含まれていない
- [ ] パスワードが含まれていない
- [ ] 内部URLやIPアドレスが含まれていない

### 4. ドキュメントの最終確認

- [ ] README.mdの内容が最新
- [ ] インストール手順が正確
- [ ] 使用例が動作する
- [ ] ライセンス表記が正確

### 5. コードの最終確認

```bash
# フォーマット確認
cargo fmt --check

# Clippy警告確認
cargo clippy

# ビルド確認
cargo build --release

# テスト実行
cargo run -- scan test_project
```

### 6. Gitの準備

```bash
# 不要なファイルが追跡されていないか確認
git status

# コミット履歴の確認
git log --oneline

# リモートリポジトリの設定
git remote add origin https://github.com/YOUR_USERNAME/index-chan.git
```

### 7. GitHubリポジトリの設定

- [ ] リポジトリの説明文を設定
- [ ] トピック（タグ）を追加：typescript, dead-code, cli, rust, llm
- [ ] Aboutセクションを記入
- [ ] Issuesを有効化
- [ ] Discussionsを有効化（オプション）

### 8. 公開後の作業

- [ ] README.mdのバッジ追加（CI status, license, etc.）
- [ ] crates.ioへの公開検討
- [ ] ソーシャルメディアでの告知
- [ ] ブログ記事の執筆（オプション）

## MITライセンスについて

### ✅ あなたの権利（保持されます）

- 著作権は「Copyright (c) 2024 index-chan contributors」として明記
- あなたが作者であることは変わらない
- 商用利用されても著作権は保持される

### ✅ 他者に許可すること

- 自由に使用・改変・再配布できる
- 商用利用も可能
- 条件：ライセンス文と著作権表示を含めること

### ❌ 権利放棄ではない

- パブリックドメインとは異なる
- 著作権は完全に保持される
- 「MIT License」という条件付きで許可しているだけ

### 📝 実例

```
あなた: 著作権保持、作者として認められる
企業A: index-chanを商用製品に組み込む → OK（ライセンス表示必須）
企業B: index-chanを改変して販売 → OK（ライセンス表示必須）
個人C: index-chanをフォークして開発 → OK（ライセンス表示必須）
```

すべてのケースで「Copyright (c) 2025 index-chan contributors」は残ります。

## 公開前の最終確認

### 心構え

- [ ] 完璧である必要はない（実験的プロジェクトとして公開）
- [ ] 質問に答えられなくても大丈夫（DISCLAIMER.mdで明記済み）
- [ ] プロフェッショナルである必要はない（学習目的と明記済み）
- [ ] サポートできなくても大丈夫（限定的サポートと明記済み）

### 免責事項の確認

- [ ] DISCLAIMER.mdを作成済み
- [ ] README.mdに免責事項を追加済み
- [ ] CONTRIBUTING.mdに注意事項を追加済み

### GitHubリポジトリの設定推奨

公開後、以下の設定を推奨します：

1. **Issueテンプレートの活用**
   - 自動的に「実験的プロジェクト」の注意書きが表示される

2. **Discussionsの無効化（オプション）**
   - 質問対応の負担を減らしたい場合

3. **リポジトリの説明に明記**
   - "⚠️ Experimental project - Not for production use"

4. **トピックタグの追加**
   - "experimental", "learning-project", "work-in-progress"

## 公開コマンド

すべて確認したら：

```bash
# 最終コミット
git add .
git commit -m "chore: GitHub公開準備完了"

# プッシュ
git push -u origin main

# タグ作成（オプション）
git tag -a v0.1.0-alpha -m "Initial experimental release"
git push origin v0.1.0-alpha
```

**注意:** バージョンに`-alpha`を付けることで、実験的であることを明示できます。

## 注意事項

⚠️ 一度公開したコードは完全には削除できません
⚠️ 機密情報が含まれていないか必ず確認してください
⚠️ Gitの履歴にも機密情報が含まれていないか確認してください

## 参考リンク

- [MITライセンス日本語訳](https://licenses.opensource.jp/MIT/MIT.html)
- [GitHub公開ガイド](https://docs.github.com/ja/repositories/creating-and-managing-repositories/about-repositories)
- [Cargo.toml設定](https://doc.rust-lang.org/cargo/reference/manifest.html)
