# Owly

![Owly logo](../../img/wide.webp)

Owlyは、構造化されたMarkdown Wiki向けの設定駆動型リンターです。

AI Agentとともに運用するWikiを対象に、Wikiの構造を調査し、記事を
決定的なルールで検証するためのツールです。

YAML frontmatter、文書種別、必須フィールド、フィールド型、許可値、
リンク、孤立ページ、追跡対象ソースを検証します。Markdown本文は不透明な
コンテンツとして扱い、意味や文章品質の評価は行いません。

このリポジトリには次のものが含まれます。

- `owly`: コマンドラインアプリケーション
- `owly_core`: 検証とWikiモデルを提供する再利用可能なライブラリ
- Wikiの設定・調査・検索・保守に使うAgent Skill
- CLIの統合テストとリリース自動化

OwlyはRustで実装され、スクリプトやCI向けに安定した終了コードを提供します。
現在のバージョンは`0.1.0`で、MITライセンスです。

## 使い始める

- [はじめに](getting-started.md): インストールから最初の検証まで
- [CLI リファレンス](cli.md): 現在利用できるコマンドとオプション
- [設定リファレンス](configuration.md): `owly.yml` の項目と YAML 例
- [仕様](spec.md): Owly の責務、データモデル、将来仕様
- [哲学](philosophy.md): 設計上の判断と境界

## Agent Skill

英語版 Skill は `owly skill <directory>` で Agent 用ディレクトリへ配置されます。日本語版は利用者が内容を確認するための翻訳です。

- [using-wiki](skills/owly/using-wiki/SKILL.md): Wiki と操作内容を解決する入口
- [wiki-config](skills/owly/wiki-config/SKILL.md): `owly.yml` を設計・検証する
- [wiki-ingest](skills/owly/wiki-ingest/SKILL.md): ローカルソースと Git リポジトリを取り込む
- [wiki-overview](skills/owly/wiki-overview/SKILL.md): Wiki の構造と現在の状態を調べる
- [wiki-query](skills/owly/wiki-query/SKILL.md): Wiki の根拠を使って質問に答える
- [wiki-search](skills/owly/wiki-search/SKILL.md): 構造化メタデータと関係を検索する
- [wiki-maintenance](skills/owly/wiki-maintenance/SKILL.md): 既存 Wiki の構造を監査・保守する

各 Skill の [references](skills/owly/) には、ソース取り込みやグラフ保守の補助手順も含まれています。

開発コマンドとリポジトリの規約は[AGENTS.md](../../../AGENTS.md)を参照してください。
