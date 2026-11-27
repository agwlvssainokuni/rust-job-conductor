# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-28
- Commit: 8c5c21e5e2d8ad08768891a8381ec4594a044d14

全体の再スキャン(Full rescan、深さ Minimal)。前回(v0102-spec-docs の intent、2026-11-28、`55958fd`)の 9 資料をすべて置き換えた。リポジトリの根(`./`)を対象に、`55958fd..8c5c21e` の変更(仕様書 `docs/spec/rust/`・`language-agnostic/` の全 26 章、`crates/runtime/src/lib.rs` の呼び出しの順序の文書、版 0.10.2)を確かめ、4 クレートの本体のコードは前回の知識ベースと突き合わせた(コードの動きの変更は無い)。この intent の対象のため、文書と実装の一致を確かめるテスト(`crates/cli/src/tests/docs.rs`、`crates/cli/tests/msrv_consistency.rs`、デモのテスト、`crates/core/tests/common/kinds.rs`)、数の取り出し口(誤りの種類の列挙、定義とステートのキーの表、`--interval` の定数、`CommandKind::ALL`)、仕様書の 2 版の数の記述と表の行の数、language-agnostic の 11 章の性能の表と受け入れテストの根拠の列、性能のテストの上限を、行の単位で読んだ。テストの数は `find` と `grep` で、履歴は `git rev-list` と `git tag` で数えた。ほかの 5 つの言語版の仕様書はファイルの数と版の記述だけ、converter(前回から変わっていない)、テストの fixtures とデモのジョブのスクリプトは構成だけを見た。ビルド出力(`target/`)、無視される個人の文書(`local-docs/`)、IDE の設定(`.idea/`)、AI-DLC の導入物は開いていない(`aidlc/spaces/default/memory/team.md` だけは、古い記述を並べるために読んだ。直していない)。テスト・ビルド・プログラムは実行していない。

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: v0103-spec-checks
fingerprint: 2cbb02cbe41f9df5be056d41d8038bc802d45c2f
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
