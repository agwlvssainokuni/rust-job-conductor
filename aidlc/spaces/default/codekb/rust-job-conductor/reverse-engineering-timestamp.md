# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-25
- Commit: 1f88e59b2453ed80ea4d53b4275c85dcdb310550

全体の再スキャン(Standard)。リポジトリの根(`./`)を対象に、4 クレートの本体のコード、ビルドの設定、CI、README を深く読み、仕様書(`docs/spec/`)、デモ(`demo/`)、IDE の設定(`.idea/`)、図の参照出力(`crates/core/tests/fixtures/`)は構成だけを見た。ビルド出力(`target/`)、無視される個人の文書(`local-docs/`)、AI-DLC の導入物は開いていない。テスト・ビルド・プログラムは実行していない。

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: small-feature-batch
fingerprint: 3158a00db2f1cbbb86d46fda977e2a19fedc2f20
analyzed:
  paths:
    - ./
  components:
    - conductor-core
    - core::types
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
    - ci-workflows
shallow:
  paths:
    - docs/spec/
    - demo/
    - .idea/
    - crates/core/tests/fixtures/
```
