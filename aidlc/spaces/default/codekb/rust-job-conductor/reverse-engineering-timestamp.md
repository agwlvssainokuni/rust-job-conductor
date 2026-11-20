# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-21
- Commit: 1a77179273aed7763b89a69fed1b383c9b4ba8b2

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: env-graph-dryrun-init
fingerprint: 7a3d59e630550b80d292eb468713a43d0ba8131c
analyzed:
  paths:
    - ./
  components:
    - conductor-core
    - core::definition
    - core::flow
    - core::types
    - core::error
    - core::run_id
    - conductor-runtime
    - runtime::runner
    - runtime::split
    - runtime::platform
    - runtime::state
    - runtime::wait
    - conductor-converter
    - conductor-cli
    - cli::args
    - cli::ctx
    - cli::flows
    - cli::report
    - cli::deps
    - cli::platform
    - cli::exit
    - cli::panic_hook
    - test-helpers
shallow:
  paths:
    - docs/spec/
    - demo/
    - .idea/
```
