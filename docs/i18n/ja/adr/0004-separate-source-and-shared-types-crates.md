# ADR 0004: ソース処理と共有データ型をクレート分離する

- 状態: 採用
- 日付: 2026-08-30

## 文脈

ファイル探索・ファイル読み込み・Markdown解析は、SQLiteキャッシュの責務ではない。一方、解析後のデータはソース処理とキャッシュ処理の間で受け渡す必要がある。

`owly_core` にデータ型を置き、`owly_cache` やソース処理から参照すると、依存方向によっては循環参照になる。また、SQLiteのテーブル構造と同じ型を共有すると、保存形式がクレート間の公開契約になってしまう。

## 決定

共有データ型を `owly_types` クレートへ切り出す。

依存方向は次の通りとする。

```text
owly_source → owly_types
owly_cache  → owly_types
owly_core   → owly_source
owly_core   → owly_cache
```

`owly_types` には、解析済みWikiデータを表すドメイン型だけを置く。

- `DocumentIndex`
- `IndexedDocument`
- `IndexedFrontmatterValue`
- `IndexedLink`
- `IndexedSourceReference`
- `FrontmatterValue`

ファイル探索・読み込み・解析は `owly_source` が担当する。

```rust
pub fn discover(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, SourceError>;
pub fn load(root: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<DocumentIndex, SourceError>;
```

SQLiteのカラム構造、行変換型、CRUD、検索条件型は `owly_cache` 内部または公開APIに閉じ込める。

コンフィグ構造体は `owly_types` に含めない。設定はアプリケーションの実行構成であり、Wikiドメインデータではないためである。

## 理由

- `owly_source` と `owly_cache` が `owly_core` に依存せずに済む
- SQLiteの保存形式を変更しても、ソース処理側へ影響しにくい
- ファイルIO、解析、永続化、アプリケーションの組み立てを分離できる
- `DocumentIndex` をsourceの出力かつcacheの入力として明確に扱える
- 共有型にキャッシュ実装固有のnullableカラムやSQL都合を持ち込まずに済む

## 結果

共有型にはドメイン上の意味を持つメソッドだけを置く。SQLite変換やキャッシュ判定のような処理は、`owly_cache` の `Cache` / `CacheDb` のメソッドまたは内部関数として実装する。

既存の `owly_core` の解析処理を `owly_source` へ接続する作業は、別の変更として行う。
