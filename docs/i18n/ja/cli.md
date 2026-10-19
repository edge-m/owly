# CLI リファレンス

基本形は次のとおりです。

```sh
owly <command> [options]
```

設定ファイルを省略した場合、カレントディレクトリの `owly.yml` を読み込みます。別の設定を使うコマンドでは `--config <path>` を指定します。

## コマンド一覧

| コマンド | 用途 |
|---|---|
| `validate` | Wiki 全体または 1 ファイルを検証する |
| `overview directories` | 設定されたディレクトリ構造を表示する |
| `overview types` | 設定された文書タイプを表示する |
| `init` | 初期設定と入口ページを作成する |
| `index` | SQLite 検索インデックスを作成・更新する |
| `inspect` | Wiki、Graph、検証、インデックスの状態をまとめて表示する |
| `config validate` | `owly.yml` だけを検証する |
| `graph` | Wiki の Graph を JSON で出力する |
| `schema` | 設定されたスキーマを表示する |
| `onboard` | Agent への導入手順を表示する |
| `skill <directory>` | Agent Skill 一式を書き出す |
| `search` | frontmatter の構造化検索を行う |

各コマンドの完全なヘルプは `owly <command> --help` で確認できます。

## 初期化

```sh
owly init --list-templates
owly init --template blank
owly init --template personal
owly init --template research
```

`--list-templates` はファイルを変更しません。`--template` を指定した実行は `owly.yml` と `entrance.md` を作成し、初回インデックスも更新します。既存ファイルは上書きしません。

## 設定の検証と表示

```sh
owly config validate --config path/to/owly.yml
owly schema --config path/to/owly.yml --format text
owly schema --config path/to/owly.yml --format json
```

`schema --format json` は Wiki ルート、source tracking、ディレクトリ、タイプ、診断を含む Agent 向けの出力です。

## 概要と状態

```sh
owly overview directories --statistics
owly overview types --statistics
owly overview types --type note --format human
owly inspect --format json
```

`overview` の既定出力は JSON です。`inspect` の JSON にはページ数、タイプ別集計、Graph の入口・壊れた参照・孤立ページ、検証結果、インデックス状態が含まれます。

## インデックス

```sh
owly index
owly index --force
owly index --format json
owly index --database .owly/other.sqlite
```

既定のインデックスは Wiki ルートの `.owly/index.sqlite` です。`--force` は完全再構築を行います。インデックスは派生データであり、Markdown、frontmatter、設定が正本です。

## 検証

```sh
owly validate
owly validate --details
owly validate --file path/to/page.md
owly validate --format json
owly validate --details --format json
```

`--file` は Wiki ルートからの相対パスを指定します。`--details` を付けない場合は概要のみ、付けた場合は個別診断も出力します。

検証対象には設定、frontmatter、必須フィールド、型、許可値、リンク、Graph の入口、ソース参照、孤立ページが含まれます。

## Graph

```sh
owly graph
owly graph --config path/to/owly.yml > graph.json
```

現在の `graph` はサブコマンドを持たず、Graph を JSON として標準出力へ出します。`graph export` や `graph analyze` は現行バージョンでは利用できません。

## 構造化検索

```sh
owly search --query 'field:value'
owly search --query 'field:value' --format human
owly search --config path/to/owly.yml --query 'type:note' --format json
```

現在のクエリ構文は 1 つの `field:value` 条件だけです。Boolean 演算子、本文検索、意味検索、自然言語の質問には対応していません。値は型に応じて boolean、数値、日付、日時として解釈され、それ以外は文字列として扱われます。

## 終了コード

| コード | 意味 |
|---:|---|
| `0` | 成功。検証対象にエラーがない |
| `1` | コマンドは実行できたが、検証エラーがある |
| `2` | 設定を読めない、引数が不正、ファイル操作に失敗したなど、コマンドを完了できない |

CI では、通常 `owly validate --format json` を実行し、終了コードと JSON の `summary` を利用します。
