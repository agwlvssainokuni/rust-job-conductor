# セキュリティの設計: u2-optional-env-file

出典: セキュリティの要件(`security-requirements.md` の NFR6.1〜NFR6.3、NFR5.1、NFR4.1)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3、4、6)、契約(`contract-summary.md` の C1、C3)。

## 1. 入力の検証(境界は定義の読み取り)

- `env_file` の表は、書けるキーの許可リスト(`path`、`required`)で検査する。今の `check_keys` を使う。
- `path` は今の文字列の項目と同じ検査(空なら EmptyField)を通す。表と文字列で検査を 2 つ持たないよう、パスの検査は 1 つの関数にまとめる。

## 2. 誤りの表示(NFR6.1)

| 場面 | 種類(既存) | 理由の文(固定) |
|---|---|---|
| `env_file` が文字列・表・その配列のどれでもない | TypeMismatch | `env_file` は、文字列かテーブル、またはその配列で書きます |
| 配列の要素が文字列でも表でもない | TypeMismatch | `env_file` の配列の要素は、文字列かテーブルで書きます |
| 表に `path` が無い・文字列でない | TypeMismatch | 必須の値 `path` がありません / `path` は文字列で書きます(今の `string` の文) |
| `path` が空 | EmptyField | `env_file` に空の文字列は書けません(今の文) |
| `required` が真偽でない | TypeMismatch | `required` は真偽値(`true` または `false`)で書きます(今の `boolean` の文) |
| 知らないキー | UnknownKey | キー {キー名} は定義にありません(今の文。キー名はエスケープと切り詰め) |

- 確かめ方: core の `crates/core/tests/nfr_security.rs` の `cases` に、表の型の誤り、知らないキー(キー名ではなく値の位置に目印)、`required` の型の誤りを足し、表示に目印が出ないことを確かめる。

## 3. 読み取り(NFR6.2、NFR6.3)

- runtime の `load_env_files` は、解決したパスごとに required を論理和でまとめてから読む(BR2.2)。
- 読み取りの失敗の見分けは、今の `read_regular_file` の結果だけで行う: `Missing` かつ required = false のときだけ飛ばす。`Unreadable`(通常のファイルでない、開けない)と `TooLarge`、UTF-8 でない、書式の誤りは required によらず今の誤り。
- 飛ばしたことは、誤りの並びにも警告にも入れない。表示の層には何も渡らない(NFR6.2)。
- 確かめ方: runtime の `crates/runtime/tests/env_loader.rs` で、無い・ディレクトリ・大きすぎる・UTF-8 でない、をそれぞれ required の真偽で確かめる。

## 4. 依存とコードの規則(NFR5.1、NFR4.1)

- 変更は core の `types.rs`、`definition/walk/env.rs`、`envfile/mod.rs`(`env_file_refs`)、`envfile/merge.rs`(引き当ての型)、runtime の `envfile.rs`、cli の `deps.rs`・`ctx.rs`(引数の型)に閉じる。新しい依存、`unsafe`、`cfg` を書かない。`nfr_source.rs`(runtime、cli)と CI の `build-check` が確かめる。
