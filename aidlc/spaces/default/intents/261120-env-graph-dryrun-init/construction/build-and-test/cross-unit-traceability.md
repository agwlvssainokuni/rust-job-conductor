# ユニットをまたぐ追跡(Cross-Unit Traceability): v0.9.0 の機能拡張

出典: `aidlc engine sensor-traceability --stage build-and-test` の出力(要件の ID ごとに、作ったユニットとファイルを列挙)。

## 判定

**機能要件(FR): 合格**(FR1.1〜FR1.13、FR2.1〜FR2.7、FR3.1〜FR3.4、FR4.1〜FR4.5、FR5.1、FR5.2 の 31 のサブ要件が、すべて、いずれかのユニットで作られている)。

**非機能要件(NFR1〜NFR7): 追跡の対象外(自動の検査は「不合格」と表示するが、理由は下)**。自動の検査は、NFR1〜NFR7 を「uncovered」と報告した。理由: この作業では、各ユニットの「非機能要件」と「非機能の設計」の段階を、(要件で決まっていて、機能の設計に具体化済みのため)省略したので、NFR の ID は、ユニットの追跡(`traceability.json`)の対象にならなかった。NFR は、`build-and-test-summary.md` の Target Verification Matrix で、1 件ずつ、実行した確認に結びつけて確かめた(T3〜T9。T5 だけは CI Pipeline が受け持つ N/A)。承認のゲートで、人に知らせる。

## 機能要件の対応(ID ごと)

| 要件 | ユニット | 実装のファイル |
|------|----------|----------------|
| FR1.1、FR1.2、FR1.4、FR1.11、FR1.13 | U1 u1-env-injection | `crates/core/src/definition/walk.rs` |
| FR1.3、FR1.10 | U1 | `crates/core/src/envfile/merge.rs` |
| FR1.5 | U1 | `crates/runtime/src/envfile.rs` |
| FR1.6、FR1.9 | U1 | `crates/core/src/envfile/parse.rs` |
| FR1.7 | U1 | `crates/cli/src/aux_commands.rs` |
| FR1.8 | U1 | `crates/cli/src/ctx.rs` |
| FR1.12 | U1、U2、U3 | `crates/core/src/types.rs`、`crates/cli/tests/binary_graph.rs`、`crates/cli/tests/binary_dry_run.rs` |
| FR2.1 | U2 u2-graph | `crates/cli/src/graph.rs` |
| FR2.2 | U2 | `crates/core/src/graph/layout.rs` |
| FR2.3 | U2 | `crates/cli/src/report/graph_text.rs` |
| FR2.4 | U2 | `crates/core/src/flow/next_step.rs` |
| FR2.5 | U2 | `crates/cli/src/report/graph_mermaid.rs` |
| FR2.6 | U2 | `crates/cli/src/aux_commands.rs` |
| FR2.7 | U2 | `crates/core/src/graph/overlay.rs` |
| FR3.1 | U3 u3-dry-run | `crates/cli/src/dry_run.rs` |
| FR3.2、FR3.4 | U3 | `crates/core/src/plan/simulate.rs` |
| FR3.3 | U3 | `crates/cli/src/ctx.rs` |
| FR4.1、FR4.2、FR4.4 | U4 u4-init | `crates/cli/src/init.rs` |
| FR4.3 | U4 | `crates/runtime/src/file_create.rs` |
| FR4.5 | U4 | `crates/cli/src/report/init_template.rs` |
| FR5.1 | U1、U2、U3、U4 | `crates/cli/src/report/messages.rs` |
| FR5.2 | U1、U2、U3、U4 | `README.md` |

覆われていない機能要件(FR): なし。

## 覆われていない要素(承認のゲートで知らせる)

- NFR1〜NFR7: 上の「判定」のとおり。追跡の対象外で、マトリクスで確かめた(NFR3 は、CI Pipeline が受け持つ)。
