---
name: wiki-query
description: Owly Wikiから情報を検索し、構造・メタデータ・リンク・本文を調べるときに使う。読み取り専用で、変更はwiki-ingestまたはwiki-maintenanceに振り分ける。
---

# Owly Wikiの検索

1つのグラフから根拠を取得するための読み取り専用スキルです。ファイルは
変更しません。

## Workflow

1. Wikiルートを特定し、`<wiki-root>/entrance.md`を読む。
2. `owly schema --config <config> --format json`で`owly.yml`を確認する。
3. 構造化メタデータは`owly search --config <config> --query '<field>:<value>' --format json`で検索する。
4. 関係は`owly graph --config <config>`で確認し、該当するパスをたどる。
5. 本文や完全一致の語句は、候補パスを絞ってからWikiルート内で`rg`を使う。
6. 根拠となるパスを示し、Wikiに書かれていることと推論を区別する。

検索中に編集、インデックス作成、正規化、黙った修復をしてはいけません。
インデックスが使えなければその制約を報告し、ソースファイルやグラフ出力を
使います。知識の保存は`wiki-ingest`、既存知識の修復や再編成は
`wiki-maintenance`へ振り分けます。

## Safety rules

- 選択したグラフの入口は`entrance.md`として扱う。
- 壊れたリンクや外部リンクから、対象が存在すると推測しない。
- 単一グラフの検索で、明示的に求められない限り外部グラフの中身を調べない。
- 根拠の不足を埋めるためにフィールド、型、関係を作らない。
