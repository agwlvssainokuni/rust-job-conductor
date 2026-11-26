# ユニットをまたぐ対応: v0.10.0

出典: `aidlc engine sensor-traceability --stage build-and-test` の出力(2026-11-26)、各ユニットの `code-generation/traceability.json`、`test-results.md`。

## 判定

**合格(条件つき)**: 機能の要件 FR1.1〜FR8.2 はすべて、いずれかのユニットのファイルで作られている。非機能の要件のうち NFR2・NFR3・NFR7 は、ユニットの段階で対応先が「この段階か CI に任せる」になっていたため、道具の判定では未対応と出る。NFR2 と NFR7 はこの段階の実行で満たした。NFR3 は CI Pipeline の段階が持つ。

## 機能の要件

| ID | ユニット | ファイル |
|---|---|---|
| FR1.1〜FR1.6 | u1-pipeline-retry | `crates/core/src/definition/walk/retry.rs`、`walk/mod.rs`、`crates/core/tests/definition_valid.rs` |
| FR2.1〜FR2.5 | u2-optional-env-file | `crates/core/src/definition/walk/env.rs`、`crates/runtime/src/envfile.rs`、`crates/cli/tests/binary_env.rs` |
| FR3.1〜FR3.5 | u3-unset-env | `crates/core/src/definition/walk/env.rs`、`crates/core/src/envfile/mod.rs`、`crates/runtime/src/runner.rs`、`crates/cli/tests/flow_env.rs` |
| FR4.1〜FR4.4 | u4-graph-dot | `crates/cli/src/report/graph_dot.rs`、`crates/cli/src/args.rs` |
| FR5.1〜FR5.5 | u5-log-dir | `crates/core/src/definition/walk/mod.rs`、`crates/runtime/src/log.rs`、`crates/runtime/src/runner.rs` |
| FR6.1〜FR6.9 | u6-attempt-times(FR6.6 は u5 と共有) | `crates/cli/src/step.rs`、`crates/runtime/src/state/reader.rs`・`writer.rs`、`crates/runtime/src/error.rs`、`crates/cli/src/report/watch.rs`、`crates/core/src/time.rs` |
| FR7.1〜FR7.5 | u7-watch-follow | `crates/cli/src/aux_commands.rs`、`args.rs`、`platform/mod.rs`、`crates/core/src/flow/activity.rs` |
| FR8.1 | u1〜u7 | `README.md` |
| FR8.2 | u7-watch-follow | `crates/cli/Cargo.toml`(「版を 0.10.0 にする」) |

## 非機能の要件

| ID | 状態 | 対応 |
|---|---|---|
| NFR1(後方互換) | 対応 | u1〜u7 の NFR1.x。全体のテストで確かめた(`build-and-test-summary.md` の T-COMPAT) |
| NFR2(テスト、80%) | この段階で満たした | u1〜u4 は NFR2.3(行カバレッジ)をこの段階に任せていた。全体のカバレッジ 98.18%(T-COV)と全体のテスト(T-TESTS) |
| NFR3(3 つの OS と MSRV) | CI Pipeline が持つ | u1〜u7 の NFR3.1 を CI に任せている。手元はデモのテスト(NFR3.2)まで(T-DEMO)。`ci-pipeline` で確かめる(T-CI) |
| NFR4(依存を足さない) | 対応 | u1〜u5 の NFR4.1。package の数は変わらない(T-DEPS) |
| NFR5(`unsafe`・`cfg`) | 対応 | u1〜u7 の NFR5.1。`nfr_source.rs`(T-SRC) |
| NFR6(秘密の保護) | 対応 | u1〜u7 の NFR6.x。`nfr_security.rs`、`nfr_content.rs`(T-SECRET) |
| NFR7(fmt と clippy) | この段階で満たした | ユニットの設計の対応先(NFR7.1)は道具の対応に載っていない。この段階で fmt と clippy が通った(T-FMT、T-CLIPPY) |

## 未対応のまま残るもの

- NFR3: Windows・ubuntu・最小の版での実行。CI Pipeline の段階で確かめるまで残る。
