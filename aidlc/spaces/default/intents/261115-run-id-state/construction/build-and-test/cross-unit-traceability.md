# Cross-Unit Traceability

`aidlc engine sensor-traceability --stage build-and-test` の結果(2026-11-15)。

## 判定: 合格(未対応 0 件)

| ID | 状態 | 担当 | 対応先 |
|----|------|------|--------|
| FR1.1〜FR1.4 | 対応済み | code-generation | `crates/cli/src/args.rs`、`crates/core/src/run_id.rs`、`crates/cli/src/ctx.rs` とそのテスト |
| FR2.1〜FR2.3 | 対応済み | code-generation | `crates/core/src/definition/walk.rs`(`run_id_env`) |
| FR3.1〜FR3.4 | 対応済み | code-generation | `crates/runtime/src/state/`(`reader.rs`、`writer.rs`、`record.rs`)、`crates/runtime/src/types.rs` |
| FR4.1〜FR4.3 | 対応済み | code-generation | `crates/cli/src/aux_commands.rs`、`crates/cli/src/report/watch.rs` |
| FR5.1〜FR5.4 | 対応済み | code-generation | `crates/runtime/src/state/reset_all.rs`、`crates/cli/src/aux_commands.rs` |
| FR6 | 対応済み | code-generation | `crates/core/src/definition/walk.rs`(`validate` も同じ検査) |
| FR7 | 対応済み | code-generation | `README.md`、`crates/cli/src/report/messages.rs`(`--help`) |
| NFR1〜NFR4 | 対応済み | code-generation | 各行の `traceability.json`(code-generation)のとおり |

未対応の要素は、ない。
