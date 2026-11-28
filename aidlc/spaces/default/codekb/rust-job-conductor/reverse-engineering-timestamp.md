# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-28
- Commit: 9376cb7b3791a48bd584dab803c43ecbff6b976d

全体の再スキャン(Full rescan、深さ Minimal)。前回(v0103-spec-checks の intent、2026-11-28、`8c5c21e`)の 9 資料をすべて置き換えた。リポジトリの根(`./`)を対象に、`8c5c21e..9376cb7` の変更(`crates/cli/tests/spec_numbers.rs` の追加、版 0.10.3、テスト名 2 つ、仕様書 `docs/spec/rust/`・`language-agnostic/` の直し)を確かめ、4 クレートの本体のコードは前回の知識ベースと突き合わせた(本体のコードの動きの変更は無い)。この intent の対象のため、`crates/cli/src/report/mod.rs`、`crates/core/src/definition/walk/mod.rs`(と子の `env.rs`・`retry.rs` が親を使う箇所)、`crates/cli/src/args.rs` を全行、パスに依存するテスト(`spec_numbers.rs`、`src/tests/docs.rs`、`nfr_source.rs` 2 つ、`msrv_consistency.rs`)と、外からの参照(`use` の場所)を行の単位で読んだ。テストの数は `find` と `grep` で、履歴は `git rev-list`・`git tag`・`git log` で数えた。ほかの 5 つの言語版の仕様書はファイルの数と版の記述だけ、converter(前回から変わっていない)、テストの fixtures とデモのジョブのスクリプトは構成だけを見た。ビルド出力(`target/`)、無視される個人の文書(`local-docs/`)、IDE の設定(`.idea/`)、AI-DLC の導入物は開いていない(前回の知識ベースとこの intent の記録だけを読んだ)。テスト・ビルド・プログラムは実行していない。走査の結果の引き渡しは、intent の記録 `inception/reverse-engineering/developer-scan.md`。

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: v0104-split-files
fingerprint: be56b5dcc861ec45d3e6b28f9d831c79dabaf0d1
analyzed:
  paths:
    - ./
  components:
    - conductor-core
    - core::types
    - core::time
    - core::error
    - core::run_id
    - core::definition
    - core::envfile
    - core::flow
    - core::consistency
    - core::graph
    - core::plan
    - conductor-runtime
    - runtime::types
    - runtime::error
    - runtime::split
    - runtime::runner
    - runtime::log
    - runtime::clock
    - runtime::wait
    - runtime::envfile
    - runtime::bounded_read
    - runtime::temp_file
    - runtime::file_create
    - runtime::state
    - runtime::platform
    - conductor-converter
    - conductor-cli
    - cli::types
    - cli::args
    - cli::ctx
    - cli::step
    - cli::flows
    - cli::run_id
    - cli::report
    - cli::deps
    - cli::platform
    - cli::exit
    - cli::panic_hook
    - test-helpers
    - repo-file-tests
    - demo
    - spec-docs
    - ci-workflows
shallow:
  paths:
    - docs/spec/csharp/
    - docs/spec/go/
    - docs/spec/powershell/
    - docs/spec/python/
    - docs/spec/typescript/
    - crates/converter/src/
    - crates/core/tests/fixtures/
    - crates/cli/tests/fixtures/
    - crates/converter/tests/fixtures/
    - demo/scripts/
    - demo/windows/scripts/
```
