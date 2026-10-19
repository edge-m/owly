# はじめに

このページでは、Owly をインストールして小さな Wiki を検証するまでの手順を説明します。

## インストール

### リリースバイナリを使う

GitHub の [Releases](https://github.com/edge-m/owly/releases) から、利用する OS と CPU に合うアーカイブをダウンロードし、`owly`（Windows では `owly.exe`）を PATH の通ったディレクトリへ配置します。

現在のリリースには次のターゲットがあります。

- Linux x86_64
- macOS x86_64
- macOS arm64
- Windows x86_64

### Rust からインストールする

Rust と Cargo がある場合は、crates.io からインストールできます。

```sh
cargo install owly
```

インストールできたことを確認します。

```sh
owly --help
```

## Wiki を初期化する

空のディレクトリを Wiki のルートとして使います。

```sh
mkdir my-wiki
cd my-wiki
owly init --template personal
```

`init` は次のファイルを作成し、初回の SQLite インデックスも更新します。

- `owly.yml`: Wiki の設定
- `entrance.md`: Wiki の入口ページ
- `.owly/index.sqlite`: 構造化検索用の派生インデックス

テンプレートを確認するだけなら、ファイルは作成されません。

```sh
owly init --list-templates
```

利用できるテンプレートは `blank`、`personal`、`research` です。既存の `owly.yml` または `entrance.md` がある場合、`init` は上書きせずエラーになります。

## 設定と Wiki の状態を確認する

設定だけを検証します。

```sh
owly config validate
```

Agent が読むための設定全体は JSON で取得できます。

```sh
owly schema --format json
```

Wiki、Graph、検証、インデックスの状態をまとめて確認できます。

```sh
owly inspect --format json
```

## Wiki を検証する

```sh
owly validate --details
```

特定のページだけを確認するときは、Wiki ルートからの相対パスを指定します。

```sh
owly validate --file entrance.md --details
```

検証はファイルを変更しません。エラーがある場合は診断コード、対象パス、メッセージを確認してから Markdown または `owly.yml` を編集します。

## 検索する

構造化された frontmatter は `field:value` 形式で検索します。

```sh
owly search --query 'type:note'
owly search --query 'title:Wiki entrance' --format human
```

検索は必要に応じて `.owly/index.sqlite` を作成・更新します。本文の全文検索や自然言語検索は行いません。

## Agent と使う

Agent Skill を配置する場合は、Agent が管理する Skills ディレクトリを指定します。

```sh
owly skill <agent-skills-directory>
```

また、導入手順は次のコマンドでも確認できます。

```sh
owly onboard
```

生成された `owly/` ディレクトリと Wiki ディレクトリを Agent から利用できるようにし、最初に `using-wiki` Skill を使わせます。

## 次に読むページ

- [CLI リファレンス](cli.md)
- [設定リファレンス](configuration.md)
- [Wiki の使い方 Skill](skills/owly/using-wiki/SKILL.md)
