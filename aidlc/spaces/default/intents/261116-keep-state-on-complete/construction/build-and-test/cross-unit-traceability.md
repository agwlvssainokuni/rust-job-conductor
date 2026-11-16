# Cross-Unit Traceability

`aidlc engine sensor-traceability --stage build-and-test` の結果(2026-11-17)。

## 判定: 合格(未対応 0 件)

| ID | 状態 | 担当 | 対応先 |
|----|------|------|--------|
| FR1.1 | 対応済み | code-generation | `crates/cli/src/pipeline.rs` |
| FR1.2・FR6.1 | 対応済み | code-generation | `crates/cli/tests/binary_basic.rs` |
| FR1.3・FR1.4 | 対応済み | code-generation | `crates/cli/tests/flow_pipeline.rs` |
| FR2.1 | 対応済み | code-generation | `crates/cli/src/run.rs` |
| FR2.2 | 対応済み | code-generation | `crates/core/src/definition/walk.rs` |
| FR2.3・FR2.6 | 対応済み | code-generation | `crates/cli/src/ctx.rs` |
| FR2.4 | 対応済み | code-generation | `crates/core/tests/definition_errors.rs` |
| FR2.5 | 対応済み | code-generation | `crates/cli/src/args.rs` |
| FR3.1 | 対応済み | code-generation | `crates/cli/src/report/watch.rs` |
| FR3.2 | 対応済み | code-generation | `crates/cli/src/aux_commands.rs` |
| FR3.3 | 対応済み | code-generation | `crates/cli/tests/flow_run_id.rs` |
| FR3.4・NFR5 | 対応済み | code-generation | `crates/runtime/tests/state_read_raw.rs` |
| FR4.1 | 対応済み | code-generation | `crates/cli/tests/flow_aux.rs` |
| FR5.1 | 対応済み | code-generation | `README.md` |
| FR5.2 | 対応済み | code-generation | `demo/run-demo.sh` |
| FR5.3 | 対応済み | code-generation | `crates/cli/tests/unix_demo.rs` |
| NFR1 | 対応済み | code-generation | `crates/runtime/src/state/reader.rs` |
| NFR2 | 対応済み | code-generation | `crates/cli/src/pipeline.rs` |
| NFR3 | 対応済み | code-generation | `.github/workflows/ci.yml` |
| NFR4 | 対応済み | code-generation | `crates/cli/src/lib.rs` |

未対応の要素は、ない。
