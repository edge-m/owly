---
name: wiki-search
description: Owly Wiki の構造化メタデータと関係を検索し、必要な場合は Markdown 本文に rg を使う。
---

# Owly Wiki の検索

通常の本文検索に戻る前に、構造化された Wiki メタデータと関係を検索するときに使う。

## ワークフロー

- `owly search --query 'field:value'` で frontmatter を検索する。
- `owly graph` のノードとエッジでページの関係を調べる。
- 構造化検索だけでは足りない本文検索には `rg` を使う。
- クエリを組み立てる前に `owly schema --format json` で有効なフィールドとタイプを確認する。
- 検索結果をファイル変更の根拠にする場合は、変更後に `owly validate` を実行する。

現在のクエリ構文は 1 つの `field:value` 条件だけをサポートする。`owly search` が Boolean 演算子や本文検索をサポートすると仮定しない。
