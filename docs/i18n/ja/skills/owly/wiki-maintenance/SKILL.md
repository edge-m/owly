---
name: wiki-maintenance
description: Owly Wikiの既存構造、リンク、重複、グラフ境界を監査・変更するときに使う。先に監査し、書き込みには明示的な許可を得る。
---

# Owly Wiki の保守

読み取り専用の診断と、許可された保守作業を分けて扱います。

## ワークフロー

1. `<wiki-root>/entrance.md`とグラフの`owly.yml`を読む。
2. `owly validate --details --format json`で診断を集める。期間指定時は`--stale-before YYYY-MM-DD`を加える。
3. `owly graph`で壊れた参照、孤立ページ、到達不能ページの根拠を確認する。ルート外へのリンクは存在する外部グラフの`entrance.md`だけを指すことも確認し、外部グラフの中身は調べない。
4. `owly validate`でフロントマター、型、値、重複識別子を確認する。
5. 単一グラフ内のリファクタか、グラフ境界の移行かを分類し、対応するreferenceを読む。
6. 候補ごとに理由、パス、影響、変更案を報告する。
7. 変更、移動、アーカイブ、削除は明示的な許可を待ち、許可後に再検証する。

## References

- `references/internal-refactor.md`は単一グラフ内のリンク、孤立、重複などを扱う。
- `references/graph-migration.md`は別グラフの`entrance.md`を指す境界を扱う。
