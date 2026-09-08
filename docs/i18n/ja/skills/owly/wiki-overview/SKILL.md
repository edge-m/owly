---
name: wiki-overview
description: Owly Wiki の構造、文書タイプ、ページ数、関係、現在の検証状態を調べる。
---

# Owly Wiki の概要

Wiki の構造、文書タイプ、ページ数、関係、現在の検証状態を把握するときに使う。

## ワークフロー

1. グラフ入口として `<wiki-root>/entrance.md` を読み、その後 `owly schema --format json` で Wiki ルート、設定されたタイプ、フィールド、ディレクトリの役割、設定診断を取得する。
2. `owly overview directories --statistics` と `owly overview types --statistics` で実際のページ数を確認する。
3. `owly graph` でリンク、壊れた参照、孤立ページ、到達不能ページを調べる。
4. `owly validate` を実行し、現在の検証状態を報告する。

ディレクトリ名やページ内容から、欠けている設定を推測しない。
