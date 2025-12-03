# Documentation

English | [日本語](#日本語版)

This directory contains design and vision documents for the index-chan project.

## Documents

### [VISION.md](VISION.md) | [日本語版](VISION.ja.md)
Project vision and concept document. Describes the overall goals, target users, and future direction of the code dependency graph search system.

**Key Topics:**
- System concept and architecture
- Hybrid search approach (vector + graph)
- Unified context for batch editing
- Target users and use cases
- Development roadmap

**Status:** 🎉 **MVP Achieved!** (Phase 6 Complete)
- 9 MCP tools for LLM agents
- Context generation with dependencies
- Batch changes with validation
- Import validation (prevents hallucinations)

### [DESIGN.md](DESIGN.md) | [日本語版](DESIGN.ja.md)
Technical design document. Details the system architecture, data models, and implementation approach.

**Key Topics:**
- System architecture and technology stack
- Data models (nodes, edges, graphs)
- Core features (analysis, search, LLM interface)
- Dead code detection algorithms
- API design (MCP and CLI)
- Performance optimization strategies

**Latest Updates:**
- MCP server implementation (JSON-RPC 2.0, stdio)
- Context generation with dependency traversal
- Batch change validation and application
- Automatic backup with timestamps

## Reading Order

**For New Contributors:**
1. Start with [VISION.md](VISION.md) to understand the project goals
2. Read [DESIGN.md](DESIGN.md) for technical details
3. Check the main [README.md](../README.md) for current implementation status

**For Users:**
1. Main [README.md](../README.md) for installation and usage
2. [VISION.md](VISION.md) for understanding the project's direction

## Language

These documents are available in both English and Japanese:
- English versions are for international collaboration (published)
- Japanese versions are for internal review and confirmation (not published)

Development notes and research documents are in the `Doc/` directory (Japanese only, not published).

## Contributing

Improvements to documentation are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

MIT License - See [LICENSE](../LICENSE) for details.

---

# 日本語版

[English](#documentation) | 日本語

このディレクトリには、index-chanプロジェクトの設計書とビジョンドキュメントが含まれています。

## ドキュメント

### [VISION.ja.md](VISION.ja.md) | [English](VISION.md)
プロジェクトのビジョンとコンセプトドキュメント。コード依存グラフ型検索システムの全体的な目標、ターゲットユーザー、将来の方向性を説明します。

**主なトピック:**
- システムのコンセプトとアーキテクチャ
- ハイブリッド検索アプローチ（ベクトル + グラフ）
- 統合コンテキストによる一括編集
- ターゲットユーザーとユースケース
- 開発ロードマップ

**状況:** 🎉 **MVP達成！**（Phase 6完了）
- LLMエージェント向け9個のMCPツール
- 依存関係を含むコンテキスト生成
- 検証付き一括変更
- Import検証（ハルシネーション防止）

### [DESIGN.ja.md](DESIGN.ja.md) | [English](DESIGN.md)
技術設計ドキュメント。システムアーキテクチャ、データモデル、実装アプローチの詳細を説明します。

**主なトピック:**
- システムアーキテクチャと技術スタック
- データモデル（ノード、エッジ、グラフ）
- コア機能（解析、検索、LLMインターフェース）
- デッドコード検出アルゴリズム
- API設計（MCPとCLI）
- パフォーマンス最適化戦略

**最新の更新:**
- MCPサーバー実装（JSON-RPC 2.0、stdio）
- 依存関係トラバーサルを含むコンテキスト生成
- 一括変更の検証と適用
- タイムスタンプ付き自動バックアップ

## 読む順序

**新しい貢献者向け:**
1. [VISION.ja.md](VISION.ja.md)でプロジェクトの目標を理解
2. [DESIGN.ja.md](DESIGN.ja.md)で技術的な詳細を確認
3. メインの[README.md](../README.md)で現在の実装状況を確認

**ユーザー向け:**
1. メインの[README.md](../README.md)でインストールと使い方を確認
2. [VISION.ja.md](VISION.ja.md)でプロジェクトの方向性を理解

## 言語について

これらのドキュメントは英語版と日本語版の両方があります:
- 英語版は国際的なコラボレーション用（公開）
- 日本語版は内部レビューと確認用（非公開）

開発メモと調査ドキュメントは`Doc/`ディレクトリにあります（日本語のみ、非公開）。

## 貢献

ドキュメントの改善は歓迎します！ガイドラインについては[CONTRIBUTING.md](../CONTRIBUTING.md)を参照してください。

## ライセンス

MIT License - 詳細は[LICENSE](../LICENSE)を参照してください。
