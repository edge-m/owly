---
name: wiki-maintenance
description: ファイルを変更せずに、壊れたリンク、孤立ページ、到達不能ページ、古い内容など Owly Wiki の保守候補を見つけて評価する。
---

# Owly Wiki の保守

Wiki を変更せずに、保守が必要な候補を見つけて評価するときに使う。

## ワークフロー

1. `owly validate --details --format json` で診断を集める。期間のしきい値が必要なら `--stale-before YYYY-MM-DD` を追加する。
2. `owly graph` で壊れた参照、孤立ページ、到達不能ページの根拠を確認する。
3. `owly validate` で frontmatter、タイプ、値、重複 ID を確認する。
4. 各候補について、理由、パス、想定される影響を報告する。
5. ファイルの変更、移動、アーカイブ、削除は、明示的な許可を得るまで待つ。現在の CLI は検出のみを行う。
