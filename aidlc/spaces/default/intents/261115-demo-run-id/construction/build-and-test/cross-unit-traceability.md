# Cross-Unit Traceability

`aidlc engine sensor-traceability --stage build-and-test` の結果(2026-11-15)。

## 判定: 合格(未対応 0 件)

| ID | 状態 | 担当 | 対応先 |
|----|------|------|--------|
| FR1.1〜FR1.4 | 対応済み | code-generation | `demo/run-demo.sh`(場面 7〜9)と `crates/cli/tests/unix_demo.rs` |
| FR2 | 対応済み(Windows の実機での実行は人の確認) | code-generation | `demo/windows/run-demo.cmd` |
| FR3 | 対応済み | code-generation | `demo/conductor.toml`、`demo/windows/conductor.toml`(`run_id_env`) |
| FR4 | 対応済み | code-generation | `demo/run-demo.sh`、`demo/windows/run-demo.cmd`(冒頭の整理) |
| FR5 | 対応済み | code-generation | `demo/README.md` |
| FR6.1 | 対応済み(実行は CI の Windows) | code-generation | `crates/cli/tests/windows_demo.rs` |
| FR6.2 | 対応済み | code-generation | `crates/cli/tests/unix_demo.rs` |
| NFR1〜NFR4 | 対応済み | code-generation | 各行の `traceability.json`(code-generation)のとおり |

未対応の要素は、ない。
