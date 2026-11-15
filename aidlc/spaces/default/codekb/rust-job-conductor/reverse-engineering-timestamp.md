# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-15
- Commit: 7ee62bfa6af2308cd625415753e8587ebe60c6f7

## Scope of Analysis

```yaml
scope_version: 1
kind: partial
intent: validate-command
fingerprint: 4300839661669bb3e045de2d806b47cf976cd952
analyzed:
  paths:
    - crates/
    - README.md
    - demo/
  components:
    - conductor-cli
    - conductor-core
    - conductor-runtime
    - demo
shallow:
  paths:
    - ./
    - crates/converter/src/
    - crates/cli/tests/
    - crates/core/tests/
    - crates/runtime/tests/
    - crates/converter/tests/
```
