# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-28
- Commit: 55958fd0454647339d353ef04e576fc4204a7291

全体の再スキャン(Full rescan、深さ Minimal)。前回(v0101-followups の intent、2026-11-27、`4ebd8e7`)の 9 資料をすべて置き換えた。リポジトリの根(`./`)を対象に、4 クレートの本体のコード、ビルドの設定、CI、README、デモのスクリプトを読み、前回から変わった箇所(v0.10.1 の F1〜F5 の直し、`flow` の内部の入口、版)を確かめた。この intent の主対象の仕様書 `docs/spec/rust/` は 13 文書を、v0.10.0〜v0.10.1 の機能ごとに行の単位で実装と照合した。テストの数は `find` と `grep` で数えた。`docs/spec/language-agnostic/` はキーワードと版の記述だけ、ほかの 5 つの言語版はファイルの数と版の記述だけ、converter と図の整形(前回から変わっていない)、Windows のデモ、テストの fixtures は構成だけを見た。ビルド出力(`target/`)、無視される個人の文書(`local-docs/`)、IDE の設定(`.idea/`)、AI-DLC の導入物は開いていない(`aidlc/spaces/default/memory/team.md` だけは、テストの数を並べるために読んだ)。テスト・ビルド・プログラムは実行していない。

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: v0102-spec-docs
fingerprint: 69c490046248622897692cb598a83bed0eba0b2e
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
    - docs/spec/language-agnostic/
    - docs/spec/csharp/
    - docs/spec/go/
    - docs/spec/powershell/
    - docs/spec/python/
    - docs/spec/typescript/
    - demo/windows/
    - crates/core/tests/fixtures/
    - crates/cli/tests/fixtures/
    - crates/converter/tests/fixtures/
```
