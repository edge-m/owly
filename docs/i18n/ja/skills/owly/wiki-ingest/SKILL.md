---
name: wiki-ingest
description: 外部またはローカルのソースを raw/ に保存し、正式な sources メタデータを持つ Owly Wiki ページを作成し、ファイルを黙って上書きせずに検証する。
---

# Owly Wiki への取り込み

ソースファイルから新しい Wiki ページを作るときに使う。Owly はソースとの関係を検証し、Agent はソースの意味と要約方法を判断する。

## ワークフロー

1. `owly schema --format json` でルールを確認する。
2. 設定された `raw/` ディレクトリにソースを保存する。明示的な許可なく既存ソースを削除・上書きしない。
3. ソースを読み、新規ページと既存ページの更新のどちらが適切か判断する。
4. ページの frontmatter に `sources` レコードを追加する。

   ```yaml
   sources:
     - path: raw/inbox/example.md
       sha256: <sha256 of the saved source file>
       origin: https://example.com/optional-origin
   ```

5. `owly validate --file <page>`、続けて `owly validate` を実行する。
6. 追跡対象のソースを後から確認するときは `owly validate --details --format json` を実行する。

ソースにない内容を推測しない。ページや raw ソースを黙って編集、移動、削除しない。本文検索には `rg` を使い、構造化されたソースの確認には `owly validate` を使う。
