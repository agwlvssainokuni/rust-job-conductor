# 要件の追跡の最終の確かめ: v0.10.1

出典: `aidlc engine sensor-traceability --stage build-and-test` の出力、`construction/code-generation/traceability.json`、`build-and-test-summary.md`、`test-results.md`。refactor の作業で Units は無い(コード生成は 1 つの段階で行った)。

## 判定: 合格(下の 2 つは、コード生成の外で、この段階が満たした)

道具の出力は `pass: false`、`uncovered: FR4.1, NFR3`。この 2 つは、コード生成で `Deferred`(Build and Test と CI に回す)としたもので、コードのファイルを指さないため道具には「未」と出る。この段階で次のとおり満たした。

| ID | 満たした所 | 根拠 |
|---|---|---|
| FR4.1 | `build-and-test-summary.md` の節「原因不明の失敗の記録の手順」(5 つの残すもの、書く場所、再実行、最初の手がかり) | 同じ文書の T-F4(Met)。`test-results.md` で、この手順に従って CI 45908344996 の失敗を記録した |
| NFR3 | CI 45912018052 の 12 のジョブ(3 つの OS の `test`、`msrv-test`、`build-check` など)がすべて成功 | `build-and-test-summary.md` の T-CI(Met)、`test-results.md` の「やり直し 1 の後の実行」 |

## ID ごとの追跡

| ID | 段階 | 実装・確かめの所 | 状態 |
|---|---|---|---|
| FR1.1 | code-generation | `README.md` | 済 |
| FR1.2 | code-generation | `README.md` | 済 |
| FR1.3 | code-generation | `crates/core/src/flow/activity.rs`(変えていない) | 済 |
| FR2.1 | code-generation | `crates/runtime/src/log.rs` | 済 |
| FR2.2 | code-generation | `crates/runtime/tests/log_file.rs` | 済 |
| FR2.3 | code-generation | `crates/runtime/src/runner.rs` | 済 |
| FR3.1 | code-generation | `crates/core/tests/plan_simulate_perf.rs`(変えていない) | 済 |
| FR3.2 | code-generation | `code-summary.md` の Step 5 | 済 |
| FR3.3 | code-generation(やり直し 1) | `crates/core/src/flow/`、`crates/core/src/plan/simulate.rs`、`code-summary.md` | 済 |
| FR4.1 | build-and-test | `build-and-test-summary.md` | 済(上の表) |
| FR5.1 | code-generation | `demo/run-demo.sh` | 済 |
| FR5.2 | code-generation | `crates/cli/tests/unix_demo.rs` | 済 |
| FR5.3 | code-generation | `demo/windows/run-demo.cmd`(変えていない) | 済 |
| FR5.4 | code-generation | `demo/README.md` | 済 |
| FR5.5 | code-generation | `crates/cli/tests/unix_demo.rs` | 済 |
| FR6.1 | code-generation | `crates/runtime/src/lib.rs` | 済 |
| FR6.2 | code-generation | `docs/spec/rust/` の 6 章、2 つのテストの注釈 | 済 |
| FR7.1 | code-generation | `crates/cli/Cargo.toml` ほか 22 ファイル | 済 |
| NFR1 | code-generation | 既存のテストがすべて通る(1,437 passed) | 済 |
| NFR2 | code-generation | 足した単体テスト、行カバレッジ 98.22% | 済 |
| NFR3 | build-and-test | CI 45912018052 | 済(上の表) |
| NFR4 | code-generation | `Cargo.lock`(package 46) | 済 |
| NFR5 | code-generation | `crates/runtime/tests/nfr_source.rs` | 済 |
| NFR6 | code-generation | `crates/runtime/tests/nfr_content.rs` | 済 |
| NFR7 | code-generation | `cargo fmt`・`cargo clippy`(手元と CI の `lint`) | 済 |
| NFR8 | code-generation | `plan_simulate_perf.rs` の上限を変えずに通る(手元と CI の `coverage`) | 済 |

## 覆われていない要素

無し。
