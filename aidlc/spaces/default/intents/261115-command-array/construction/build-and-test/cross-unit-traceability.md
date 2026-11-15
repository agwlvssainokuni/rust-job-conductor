# Cross-Unit Traceability

`aidlc engine sensor-traceability --stage build-and-test` の結果(2026-11-15)。

## 判定: 合格(未対応 0 件)

| ID | 状態 | 担当 | 対応先 |
|----|------|------|--------|
| FR1、FR1.1 | 対応済み | code-generation | `crates/core/src/types.rs`(`Command`) |
| FR1.2〜FR1.5 | 対応済み | code-generation | `crates/core/src/definition/walk.rs`(`Walker::command`) |
| FR2、FR2.1 | 対応済み | code-generation | `crates/runtime/src/split.rs`(`command_line`、`check_command`) |
| FR2.2 | 対応済み | code-generation | `crates/cli/src/ctx.rs`(`program_name()`) |
| FR2.3 | 対応済み(Windows の実行は CI) | code-generation | `crates/runtime/tests/runner_basic.rs` |
| FR3 | 対応済み | code-generation | `crates/converter/tests/convert_roundtrip.rs` |
| FR4 | 対応済み | code-generation | `README.md` |
| FR5 | 対応済み | code-generation | `demo/conductor.toml` |
| NFR1 | 対応済み | code-generation | `crates/cli/tests/binary_basic.rs`(既存のテストが成功のまま) |
| NFR2 | 対応済み | code-generation | `crates/core/src/definition/walk.rs`(値を含まない固定の文) |
| NFR3 | 対応済み | code-generation | `.github/workflows/ci.yml`(1.85、80%、3 OS) |
| NFR4 | 対応済み | code-generation | `crates/cli/Cargo.toml`(0.2.0) |

未対応の要素は、ない。
