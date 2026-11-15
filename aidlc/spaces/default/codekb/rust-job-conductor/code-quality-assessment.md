# Code Quality Assessment

## 評価(good / fair / poor)

| 項目 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | good | Rust 標準の命名、日本語の `//!` 文書、BR・NFR の ID を参照 |
| 関数の大きさ | fair | `walk.rs` の `Walker` は小さな関数に分かれる。`ctx.rs` の `report_execution`、`run.rs`・`pipeline.rs` の `run_*` はやや長い |
| エラー処理 | good | 失敗は戻り値、外部入力でパニックしない(NFR)。誤りを集めて返す。表示は固定文+エスケープ |
| テストの有無 | good | `#[test]` 約 567 件(core 115、runtime 154、converter 113、cli 185)。単体、統合、バイナリ実行、NFR(性能・安全・ソース規約) |
| 重複 | fair | `escape` 相当の関数が 4 クレートに重複(cli の文書が明記、表で揃えを検査) |
| 不要コード | good | TODO/FIXME/HACK は 0。`#[allow]` はテスト共通の `dead_code` と runtime の `unsafe_code` 1 か所 |

## CI/CD・品質ゲート

- lint(`cargo fmt --check`、`clippy -D warnings`)、build-check(Linux・Windows、MSRV 1.85)、test(3 OS)、msrv-test、coverage(行 80% 未満で失敗)、docs-check(README の必須語)、startup-bench(警告のみ)、audit(警告のみ)。
- release はタグで、4 ターゲットのバイナリを作る。

## ドキュメント

`README.md` は利用者向けで、定義の書式、command の書き方、終了コードの表、ステートの場所を網羅する。`demo/` がある。コード内の文書は充実。

## 今回の変更(command-array)に対する所見

1. **型の前提が散っている**: `command: String` を前提とする箇所は 2 つの型(`JobDefinition`、`JobExecution`)、分割(`split.rs`)、事前検査(`ctx.rs` `prepare`)、起動失敗の表示の代替経路(`ctx.rs` の `split_whitespace().next()`)、`Runtime::check_command(&str)` の trait。
2. **配列の検査の先例がある**: `walk.rs` の `exit_codes` が、配列の取り出しと要素ごとの `typeMismatch` の報告を実装している。`command` の配列版は同じ形で書ける見込み。`string()` は文字列以外を `typeMismatch` にするため、現在は配列の `command` が定義の誤りになる。
3. **空の判定**: 文字列は `trim().is_empty()` で `emptyField`、分割後にコマンド名が無ければ `CommandErrorKind::Empty`。配列(空配列、先頭要素が空)の扱いは新しく決める必要がある。
4. **ステート・watch・`-v`**: command を保存・表示しない。変更の対象外の見込み(表示は起動失敗のプログラム名のみ)。配列のときは先頭要素がプログラム名になる。
5. **converter**: 汎用の木のため、配列を往復できる見込み。ただし本スキャンではテストを実行していない。
6. **テストの量**: `command` を含むテストファイルは約 30。`JobDefinition { ... command: ... }` を作る箇所は、型の変更で全部コンパイルエラーになる(core `flow/*`、runtime テストの `common/mod.rs` など)。
7. **README と demo**: 「command の書き方」節と、demo の定義の更新が要る。docs-check の必須語に `command` は無い。

## 技術的負債

- 上記 1 の散在と、`escape` の重複。ほかに目立つ負債は検出していない(ビルド・テストは実行していない)。
