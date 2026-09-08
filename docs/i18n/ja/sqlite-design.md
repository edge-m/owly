# SQLite インデックス設計

この文書は、Owly の Markdown Wiki 用 SQLite インデックスの実装設計を定義する。SQLite は正本ではない。Wiki を走査すればいつでも全再構築でき、古いインデックスを削除しても Markdown Wiki は失われない。

ORM は使わず、SQL と `rusqlite` などの薄いバインディングを `IndexStore` に閉じ込める。

## テーブル

### `documents`

Wiki 内の有効な Markdown 文書を一行ずつ保持する。パスをページの識別子とする。

| 列 | 型 | 内容 |
|---|---|---|
| `path` | `TEXT` | Wiki ルートからの正規化相対パス。主キー。 |
| `frontmatter_hash` | `TEXT` | 正規化した frontmatter のハッシュ。 |
| `links_hash` | `TEXT` | 正規化したリンク集合のハッシュ。 |

2つのハッシュはキャッシュであり、内容の正しさをハッシュだけで判断しない。`frontmatter_hash` は YAML の表記ではなく、型を保持した正規化値から計算する。`links_hash` は、保存するリンク情報を正規化・ソートした値から計算する。ファイル名をまたぐ永続 ID は持たず、リネームは旧パスの削除と新パスの追加として扱う。旧パスへのリンクは検証で検出する。

### `frontmatter_values`

可変な frontmatter を検索できるよう、値をリーフ単位で展開する。設定に存在しないフィールドも保存するが、検索結果や検証結果は正本を再読して確定する。

| 列 | 型 | 内容 |
|---|---|---|
| `document_path` | `TEXT` | `documents.path` への外部キー。 |
| `field_path` | `TEXT` | `tags`、`generated.at`、`authors[0]` などの YAML パス。 |
| `value_type` | `TEXT` | `string`、`number`、`boolean`、`date`、`datetime`、`null` など。 |
| `value_text` | `TEXT` | 文字列の検索値。 |
| `value_number` | `REAL` | 数値の場合の検索値。 |
| `value_boolean` | `INTEGER` | 真偽値の場合の検索値。`0` または `1`。 |
| `value_date` | `TEXT` | 日付の場合の ISO 8601 検索値。 |
| `value_datetime` | `TEXT` | 日時の場合の UTC ISO 8601 検索値。 |
| `value_json` | `TEXT` | 配列・オブジェクト・複合値を含む元の JSON 表現。 |
| `ordinal` | `INTEGER` | 同一パスの配列要素の順序。不要なら `0`。 |

主キーは `document_path, field_path, ordinal` とする。`value_type` に対応する値カラムだけを埋め、その他の値カラムは `NULL` にする。文字列の `"true"` と YAML の真偽値 `true` を同一視しない。検索用には、論理的な検索キーである `field_path + value_type + typed value` に複合インデックスを作る。これはキーと値を連結した文字列を保存するより、型の混同を防ぎ、数値・日付の範囲検索にも対応しやすい。

### `links`

Markdown 本文から抽出したリンクを保持する。リンク先の存在や Graph 上の意味は、正本を基にした検証で確定する。

| 列 | 型 | 内容 |
|---|---|---|
| `source_path` | `TEXT` | リンク元。`documents.path` への外部キー。 |
| `raw_target` | `TEXT` | Markdown に書かれたリンク先。 |
| `resolved_target` | `TEXT` | 正規化後のページ・リソース・URL。 |
| `target_kind` | `TEXT` | `page`、`resource`、`external` など。 |
| `occurrence_count` | `INTEGER` | 同じ `raw_target` を持つリンクが本文に出現する回数。 |

`source_path, raw_target, resolved_target, target_kind` を主キーとし、`resolved_target` と `target_kind` にインデックスを作る。外部 URL や画像など、Wiki ページでないリンクも保持する。

同じページから同じリンク先へのリンクは一行にまとめるが、`occurrence_count` で回数を保持する。これにより、ページの改名時に「どのページのどのリンク記法を何箇所修正するか」を確認できる。本文中の正確な行・列やリンク表示文字列は保持しない。将来、自動修正や行単位の編集を提供する場合に追加で検討する。

### `source_references`

ページの `sources` frontmatter を検索するための逆引きテーブルである。ソースそのものを Wiki に複製するテーブルではない。

| 列 | 型 | 内容 |
|---|---|---|
| `document_path` | `TEXT` | 出典を記録したページ。 |
| `ordinal` | `INTEGER` | `sources` 配列内の順序。 |
| `source_path` | `TEXT` | raw ソースのパスまたは識別子。 |
| `sha256` | `TEXT` | frontmatter に記録されたハッシュ。 |
| `origin` | `TEXT` | 任意の出典 URL。 |

`document_path, ordinal` を主キーとし、`source_path` と `sha256` にインデックスを作る。実際のソースの存在・ハッシュ検証は、必要な場合に Wiki ルートのファイルを直接読んで行う。

## 更新と並列処理

ファイルの読み込みと Markdown/frontmatter/link の解析は複数ワーカーで並列化してよい。ただし SQLite への書き込みは一つの Writer に集約する。

`mtime` とファイルサイズがキャッシュと異なる文書は再解析する。`frontmatter_hash` と `links_hash` はインデックスの内容を記録するために使う。どちらかのハッシュが変わった場合は、その関連レコードを置き換える。

`links_hash` は、リンクの順序を検索対象にしない場合は `(raw_target, resolved_target, target_kind, occurrence_count)` を正規化してソートした集合から計算する。`raw_target` の表記変更を無視したい場合は、ハッシュから `raw_target` を除いてよい。ただし、改名時に修正対象となる記法を確認するなら `raw_target` を含める。

1. Writer が `BEGIN` を開始する。
2. 対象文書の `documents`、`frontmatter_values`、`links`、`source_references` を置き換える。
3. 全文書の投入が成功したら `COMMIT` する。失敗したら `ROLLBACK` する。

全再構築では一時 DB を作成して投入し、コミット後に完成した DB を原子的に置き換える。インクリメンタル更新でも、文書単位の削除と挿入を一つのトランザクションで行う。複数 Writer による同時書き込みは前提にしない。
