# Cross-Unit Traceability — job-retry

出典: `aidlc engine sensor-traceability --stage build-and-test` の出力(要件ごとに、作った Unit とファイル)。

**判定: FAIL**(各 Unit の記録で、作った・対象外としていない要件が 2 つ: NFR4、NFR5)

- 33 のうち、31 の ID が、Unit の記録で、作った、または対象外と示されている
- 記録がない 2 つの ID は、各 Unit が「Build and Test に任せる」とした横断の目標である。このステージで確認した結果は、次のとおり

| ID | 判定 | このステージでの確認 |
|---|---|---|
| NFR4(行カバレッジ 80%) | 記録なし → このステージで確認済み | 全体の行カバレッジ 97.80%(`test-results.md`)。`build-and-test-summary.md` の行は Met |
| NFR5(3 つの OS) | 記録なし → **未確認** | ローカル(macOS)のみ。Windows はコンパイルと静的解析のみ。Linux は未実行。`build-and-test-summary.md` の行は Unverified |

## 要件ごとの対応

| ID | 作った Unit | ファイル |
|---|---|---|
| FR1.1 | u1 | `crates/core/src/definition/walk.rs` |
| FR1.2 | u1、u2 | `crates/core/src/types.rs`、`crates/core/src/flow/retry.rs` |
| FR1.3 | u1 | `crates/core/src/definition/walk.rs` |
| FR1.4 | u1 | `crates/core/src/definition/duration.rs` |
| FR1.5 | u1 | `crates/core/src/definition/walk.rs` |
| FR1.6 | u1、u2 | `crates/core/tests/definition_retry.rs`、`crates/cli/tests/flow_retry.rs` |
| FR1.7 | u1 | `crates/core/src/definition/walk.rs` |
| FR1.8 | u1 | `crates/core/tests/definition_retry.rs` |
| FR1.9 | u1 | `crates/core/src/definition/walk.rs` |
| FR2.1 | u2 | `crates/core/src/flow/retry.rs` |
| FR2.2 | u2 | `crates/core/src/flow/retry.rs` |
| FR3.1 | u2 | `crates/core/src/flow/retry.rs` |
| FR3.2 | u2 | `crates/core/src/flow/retry.rs` |
| FR3.3 | u2 | `crates/cli/src/retry.rs` |
| FR3.4 | u2 | `crates/cli/tests/flow_retry.rs` |
| FR3.5 | u2 | `crates/cli/src/pipeline.rs` |
| FR3.6 | u2 | `crates/core/src/flow/retry.rs` |
| FR4.1 | u2 | `crates/runtime/src/wait.rs` |
| FR4.2 | u2 | `crates/cli/tests/signals_order.rs`(u2 の記録による。待機中のシグナルの動きは、`crates/cli/tests/flow_retry.rs` でも確かめている) |
| FR5.1 | u2 | `crates/cli/tests/flow_retry.rs` |
| FR5.2 | u2 | `crates/cli/src/retry.rs` |
| FR6.1 | u2 | `crates/cli/src/report/mod.rs` |
| FR6.2 | u2、u3 | `crates/cli/src/report/messages.rs`(u3 は、文言を確認して、変更なしと記録) |
| FR7.1 | u1 | `crates/cli/tests/flow_validate.rs` |
| FR7.2 | u1 | `crates/converter/tests/convert_roundtrip.rs` |
| FR8.1 | u3 | `README.md` |
| NFR1 | u1、u2、u3 | `crates/core/tests/definition_retry.rs`、`crates/cli/tests/flow_retry.rs`、`crates/cli/tests/docs_retry.rs`(例外 1 件。`build-and-test-summary.md`) |
| NFR2 | u1、u2 | `crates/cli/tests/nfr_source.rs` |
| NFR3 | u2 | `crates/cli/tests/common/mod.rs` |
| NFR4 | (横断) | `build-and-test-summary.md` |
| NFR5 | (横断) | **未確認** |
| NFR6 | u1、u2、u3(対象外: 新しい外部クレートなし) | `Cargo.toml`、`Cargo.lock` に変更なし |
| NFR7 | u2 | `crates/runtime/tests/wait_interruptible.rs` |
