---
name: wiki-overview
description: Owly Wiki の構造、文書タイプ、ページ数、関係、現在の検証状態を調べる。
---

# Owly Wiki の概要

Wiki の構造、文書タイプ、ページ数、関係、現在の検証状態を把握するときに使う。

## ワークフロー

1. グラフ入口として `<wiki-root>/entrance.md` を読み、その後 `owly schema --format json` で Wiki ルート、設定されたタイプ、フィールド、ディレクトリの役割、設定診断を取得する。
2. `owly inspect --format json` でページ数、設定されたディレクトリとタイプ、Graph の状態、検証状態、SQLite インデックスの状態をまとめて取得する。
3. 一部だけを詳しく見るときは `owly overview directories --statistics` または `owly overview types --statistics` を使い、個別のリンクやエッジが必要なら `owly graph` を使う。
4. `owly validate` を実行し、現在の検証状態を報告する。

ディレクトリ名やページ内容から、欠けている設定を推測しない。
