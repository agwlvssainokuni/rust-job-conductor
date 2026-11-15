# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-15
- Commit: 1d8a46119c9ad5e5cc03726cb38ab23718b3e7ea

## Scope of Analysis

```yaml
scope_version: 1
kind: partial
intent: demo-run-id
fingerprint: b267aded77ee979025c5d8a4070ff88801fefcf2
analyzed:
  paths:
    - crates/cli/src/
    - crates/cli/tests/windows_demo.rs
    - crates/cli/tests/bin_support/
    - crates/runtime/src/
    - crates/core/src/
    - demo/
    - README.md
  components:
    - conductor-cli
    - conductor-core
    - conductor-runtime
    - demo
shallow:
  paths:
    - crates/
    - crates/converter/src/
    - crates/cli/tests/
    - crates/core/tests/
    - crates/runtime/tests/
    - crates/converter/tests/
```
