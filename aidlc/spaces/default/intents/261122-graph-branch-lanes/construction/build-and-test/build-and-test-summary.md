# ビルドとテストのまとめ(Build and Test Summary): graph の text 表示の改善 その 2

## 全体の状況

- ビルド: 成功。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: 全体で **1,057 件が成功、失敗 0 件**(この intent の前は 1,042 件)。
- 行カバレッジ: **97.90%**(下限 80%)。`graph/layout.rs` は 96.24%。
- 前提: Rust 1.100 以上(`rustc 1.100.0`、MSRV と同じ)、`cargo-llvm-cov`。新しい依存なし。
- 詳細: `build-instructions.md`、`integration-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類(Minimal)

| 種類 | 内容 | 場所 |
|------|------|------|
| 単体(core) | 正解データ 537 例、1,517 例、871 例の照合。図にできるかの判定。分岐の例(B1〜B4)と辺の一覧に残る形 | `graph_layout_oracle.rs`、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs` |
| 整形(cli) | 分岐の text の全文、`-v`、`--state` | `graph_report.rs` |
| 実バイナリ | 分岐の定義で `conductor graph` | `binary_graph.rs` |
| 文書 | README の分岐の例が実出力と一致 | `docs_graph.rs` |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract、`team.md`、`project.md` の ALWAYS | 行カバレッジ 80% 以上(プロジェクト全体) | 97.90%(9,375 行のうち未通過 197) | `cargo llvm-cov ... --fail-under-lines 80 --summary-only` が終了コード 0 | build-and-test | Met |
| T2-quality | `project.md` の ALWAYS(`fmt`、`clippy -D warnings`、MSRV 1.100) | 差分なし、警告なし、MSRV で通る | 差分なし、警告なし。rustc 1.100.0(MSRV と同じ)で全テスト成功 | `cargo fmt --check`、`cargo clippy -D warnings`、`rustc --version` | build-and-test | Met |
| T3-no-new-deps | `project.md` の NEVER(新しい依存を足さない) | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行(`3ea7054` 以降) | `git diff 3ea7054 HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` | build-and-test | Met |
| T4-other-unchanged | BR3.1、SM2 | 分岐以外の形の出力は変わらない | 既存の正解データ 537 例と 1,517 例が、変更なしで 1 文字も違わず一致。既存のテスト(書き換えた表明を除く)が成功 | `graph_layout_oracle.rs`(全体のテスト) | build-and-test | Met |
| T5-branch-oracle | BR4.1、SM1 | 分岐の 871 例と一字一句一致 | 一致(6 本のテスト) | `graph_layout_oracle.rs` | build-and-test | Met(ただし、正解データは規則から導いた図で、git の出力ではない) |
| T6-docs | BR4.2、SM3、CI の `docs-check` | README と仕様書が実装に合う。必須語句が残る | README の例が実出力と一致。必須語句 7 つが残る。仕様書(rust、language-agnostic)は更新済み | `docs_graph.rs`、`grep` | build-and-test | Met |
| T7-ci-os | `team.md`(他の OS は CI で確認。マージ前に CI を通す) | ubuntu、macos、windows の `cargo test` が通る | macOS のみ実行。Windows と ubuntu は未実行 | 既存の `ci.yml`(push で動く) | (既存の CI) | N/A(push のあとの CI の結果で確かめる。この変更は OS に依存するコードを足していない) |

`Pending` の行は残っていない。

## 準備の評価

- ビルドとテストの準備: できている(全体 1,057 件、カバレッジ 97.90%)。
- 配信の準備: この作業の範囲の外(リリースは別の依頼)。

## 既知の限界

- Windows と ubuntu、MSRV の CI は未実行(T7)。push 後に確認する。
- 分岐の図(871 例)は、`git log --graph` では確かめられず、規則から導いた。ユーザーが B1、B2 の形を承認し、合流の記号は「既存の図と同じ規則」を選んだ。
- 交差する飛び越しと、同じ行き先を含む分岐は、辺の一覧のまま(別の intent の候補)。
- 開発担当が、最初の実装で、一部のファイルにシェルの Python を使った(内容は fmt、clippy、全テストで確かめた)。
