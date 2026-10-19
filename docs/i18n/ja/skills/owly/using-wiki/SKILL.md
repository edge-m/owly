---
name: using-wiki
description: Wiki タスクの前に Owly Wiki のワークフローを準備し、現在のスキーマと状態を確認して、適切な操作 Skill を選ぶ。
---

# Owly Wiki の使い方

Owly Wiki に関するタスクの開始時に一度使い、操作に特化した Skill を読み込む前に実行します。

## ワークフロー

1. Wiki のルートを特定し、`owly.yml` を探す。
2. 必須の入口である `<wiki-root>/entrance.md` を読む。存在しない場合は Wiki が不正な状態だと報告する。
3. `owly schema --format json` で現在の設定を確認する。
4. `owly inspect --format json` で現在の Wiki 状態を取得する。スキーマの一部だけを確認するときは、`owly overview directories` または `owly overview types` を使う。
5. 依頼に合う操作 Skill を選ぶ。
6. ファイルを変更したら `owly validate` を実行し、残っている診断を報告する。

## 操作の選択

- `owly.yml` を設計・修正するときは `wiki-config` を使う。
- 構造化メタデータ、リンク、本文、基本的な構造を調べるときは `wiki-query` を使う。
- ソースを `raw/` に保存してページを作るときは `wiki-ingest` を使う。
- 壊れたリンク、孤立ページ、古い内容を調べるときは `wiki-maintenance` を使う。

## 安全ルール

- タイプ、フィールド、関係、配置ルールを推測しない。
- 明示的な許可なく既存ファイルを上書き、移動、削除しない。
- `owly` を決定論的なバリデーターとして扱い、意味の判断と内容の編集は Agent が行う。
