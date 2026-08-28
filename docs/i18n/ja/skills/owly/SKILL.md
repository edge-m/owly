---
name: owly
description: 構造化 Markdown Wiki を調査・検索・検証するために Owly を使う。
---

# Owly Wiki Skills

Owly は、構造化 Markdown Wiki のための決定論的なバリデーターです。このディレクトリにある、設定、概要確認、取り込み、保守、検索の各専門 Skill を使い分けます。

## ワークフロー

1. `owly.yml` を探し、`owly schema --format json` でスキーマを確認する。
2. `owly overview directories` と `owly overview types` で Wiki の構造を把握する。
3. 依頼された Wiki の変更を行う。
4. `owly validate --details` を実行し、報告されたエラーを修正する。
5. 関係や構造化 frontmatter を調べるときは `owly graph` と `owly search` を使う。

検証の正本は `owly` とする。既存の設定ファイルを黙って上書きしない。
