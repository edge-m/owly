# 設定リファレンス

Owly の設定ファイル名は通常 `owly.yml` です。相対パスは設定ファイルがあるディレクトリを基準に解決されます。

最小構成は次のとおりです。

```yaml
name: my-wiki
owly_version: 0.1.1
wiki_root: .
```

## トップレベルの項目

| 項目 | 型 | 説明 |
|---|---|---|
| `name` | 文字列 | Wiki Graph の論理名 |
| `owly_version` | 文字列 | 設定を生成した Owly のバージョン。`config validate` で互換性を検証する |
| `wiki_root` | パス | Wiki ルート。相対パスは設定ファイルからの相対 |
| `directories` | オブジェクト | ディレクトリの役割と説明 |
| `mandatory_fields` | オブジェクト | すべての文書に必要な frontmatter フィールド |
| `types` | オブジェクト | 文書タイプごとのフィールド |
| `wiki_lint` | オブジェクト | Wiki の走査・検証設定 |
| `source_tracking` | オブジェクト | raw ソース参照の検証設定 |
| `config_lint` | オブジェクト | 設定自体の検証設定 |

未知のキーはエラーになります。設定を変更したら、まず `owly config validate` を実行してください。

## ディレクトリ

`directories` は Wiki ルートからの相対パスを階層として記述します。

```yaml
directories:
  notes:
    description: 一般的なノート
    directories:
      projects:
        description: プロジェクト別のノート
  raw:
    description: 取り込み元のファイル
```

各ディレクトリには次を指定できます。

- `description`: Agent が読む説明
- `directories`: 子ディレクトリの設定

ディレクトリ設定は、ディレクトリ名から自動推測されません。設定に書かれた構造だけが契約になります。

## 必須フィールド

`mandatory_fields` に書いたフィールドは、タイプに関係なくすべての対象文書で検証されます。

```yaml
mandatory_fields:
  title:
    type: string
    description: 表示名
  tags:
    type: array
    items:
      type: string
  generated:
    type: object
    fields:
      at:
        type: datetime
      by:
        type: string
```

必須フィールドでは `optional: true` は使えません。

## 文書タイプ

`types` は frontmatter の `type` の値ごとに追加フィールドを定義します。

```yaml
types:
  note:
    description: 一般的なノート
    fields:
      status:
        type: string
        values: [draft, published]
        optional: true
      related:
        type: array
        items:
          type: string
```

タイプが設定されている場合、文書の `type` が未指定または未知だとエラーになります。`fields` には次の項目を指定できます。

- `type`: `string`、`date`、`datetime`、`boolean`、`number`、`array`、`object`
- `description`: フィールドの説明
- `optional`: `true` の場合はフィールドを省略可能にする
- `values`: 許可する値の一覧
- `items`: 配列要素のルール
- `fields`: オブジェクト内のフィールドルール

## Wiki の検証設定

```yaml
wiki_lint:
  exclude:
    - '**/.*'
    - '**/README.md'
    - 'raw/**'
```

`exclude` は Wiki ルートからの相対 Markdown パスに対する glob パターンです。除外されたファイルは走査・検証・インデックス作成の対象になりません。

## ソース追跡

```yaml
source_tracking:
  enabled: true
```

`enabled: true` の場合、ページの `sources` 参照に対して、指定されたパスと SHA-256 ハッシュを検証します。ソースファイルは通常 `raw/` 配下に置きます。空の Wiki やソース追跡を使わない Wiki では `false` にできます。

## 設定自体の検証

```yaml
config_lint:
  max_nesting_depth: 1
```

`max_nesting_depth` は、設定されたオブジェクトや配列のネスト深度に対する検査設定です。不要に深い frontmatter を避けるために使います。

## 完全な例

```yaml
name: research
owly_version: 0.1.1
wiki_root: .

directories:
  raw:
    description: 取り込み元のファイル
    directories:
      inbox:
        description: 未整理の入力
  references:
    description: 研究上の参考資料

mandatory_fields:
  type:
    type: string
    description: 文書の種類
  title:
    type: string
    description: 表示名
  description:
    type: string
    description: 短い概要
  tags:
    type: array
    items:
      type: string
  generated:
    type: object
    fields:
      at:
        type: datetime
      by:
        type: string
  stale_after:
    type: date

types:
  note:
    description: 一般的な研究ノート
    fields: {}

wiki_lint:
  exclude:
    - '**/.*'
    - '**/README.md'
    - 'raw/**'

source_tracking:
  enabled: true

config_lint:
  max_nesting_depth: 1
```

設定を作成・変更した後は、次の順で確認します。

```sh
owly config validate --config owly.yml
owly schema --config owly.yml --format json
owly validate --config owly.yml
```
