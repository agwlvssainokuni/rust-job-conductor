# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-16
- Commit: 5bdcbbfc66a4af65db045373b01d134b91655fc9
- 種別: 全体の再走査(既存の 9 つの成果物を、この実行の内容で置き換えた)。深さは Minimal。
- 深く読んだもの・軽く見たものは、`inception/reverse-engineering/developer-scan.md` の Scan Coverage を参照。テスト(`crates/*/tests/`)と、converter・runtime のステート読み取りなどの内部は、ファイル名と公開の API の水準で見た。

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: readme-reorg
fingerprint: 099463d7cdf033d70532e561608f9643b7416214
analyzed:
  paths:
    - ./
  components:
    - conductor-core
    - conductor-runtime
    - conductor-converter
    - conductor-cli
    - CI とリリースのワークフロー
    - デモ
    - ドキュメント(README.md)
shallow:
  paths:
    - crates/core/tests/
    - crates/runtime/tests/
    - crates/converter/tests/
    - crates/cli/tests/
    - crates/converter/src/
    - crates/runtime/src/state/
```
