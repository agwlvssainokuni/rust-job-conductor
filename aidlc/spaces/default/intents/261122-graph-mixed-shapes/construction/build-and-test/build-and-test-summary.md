# ビルドとテストのまとめ(Build and Test Summary): graph の text 表示の改善 その 4

## 全体の状況

- ビルド: 成功。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: 全体で **1,091 件が成功、失敗 0 件**(この intent の前は 1,075 件。以前からの `#[ignore]` が 1 件)。
- 行カバレッジ: **97.94%**(下限 80%)。`graph/layout.rs` は 98.22%。
- 前提: Rust 1.100 以上(`rustc 1.100.0`、MSRV と同じ)、`cargo-llvm-cov`。新しい依存なし。
- 詳細: `build-instructions.md`、`integration-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類(Minimal)

| 種類 | 内容 | 場所 |
|------|------|------|
| 単体(core) | 正解データ 537、1,517、871、940、487 例の照合。図にできるかの判定。M2〜M4 と辺の一覧に残る形。性能(1,000 ジョブ) | `graph_layout_oracle.rs`、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_crossing_lanes.rs`、`graph_mixed_shapes.rs`、`graph_perf.rs` |
| 整形(cli) | M2、M4(`_` を含む)の text の全文、`-v`、`--state`、辺の一覧の例 | `graph_report.rs` |
| 実バイナリ | M2 の定義で `conductor graph`、辺の一覧に残る形 | `binary_graph.rs` |
| 文書 | README の M2 の例が実出力と一致 | `docs_graph.rs` |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract、`team.md`、`project.md` の ALWAYS | 行カバレッジ 80% 以上(プロジェクト全体) | 97.94%(9,583 行のうち未通過 197) | `cargo llvm-cov ... --fail-under-lines 80 --summary-only` が終了コード 0 | build-and-test | Met |
| T2-quality | `project.md` の ALWAYS(`fmt`、`clippy -D warnings`、MSRV 1.100) | 差分なし、警告なし、MSRV で通る | 差分なし、警告なし。rustc 1.100.0(MSRV と同じ)で全テスト成功 | `cargo fmt --check`、`cargo clippy -D warnings`、`rustc --version` | build-and-test | Met |
| T3-no-new-deps | `project.md` の NEVER(新しい依存を足さない) | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行(`e01525c` 以降) | `git diff e01525c HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` | build-and-test | Met |
| T4-other-unchanged | BR3.1、SM2 | 前回までの出力は変わらない | 既存の正解データ 537、1,517、871、940 例が、変更なしで 1 文字も違わず一致 | `graph_layout_oracle.rs`(全体のテスト) | build-and-test | Met |
| T5-mixed-oracle | BR4.1、SM1 | 交差と同じ行き先が混ざる 487 例(ジョブ 4〜6 個)と一字一句一致 | 一致 | `graph_layout_oracle.rs` | build-and-test | Met |
| T6-oneoff | BR4.1(質問 Q2: A) | ジョブ 7 個の 3,861 例も一致(リポジトリには入れず、1 回だけ照合) | 混ざる形の全 4,348 例(ジョブ 4〜7 個)と、交差の全 4,757 例(ジョブ 3〜8 個)が全例一致。ただし、照合は、開発担当の一時のテスト(削除済み)で、再現できない。設計のときに、ジョブ 8〜11 個のランダム 1,517 例も、git の実際の出力と一致を確認済み | code-summary.md、開発担当の報告 | build-and-test | Met(再現できない点は、既知の限界) |
| T7-license | BR5.1(git の GPL v2 のソースは読まない) | git のソースを読まず、コードに写していない | 読んでいない。確認でも、git の内部の名前やコードの写しはないと確認 | 設計と実装の確認 | build-and-test | Met |
| T8-docs | BR4.2、SM3、CI の `docs-check` | README と仕様書が実装に合う。必須語句が残る | README の M2 の例が実出力と一致。必須語句 7 つが残る。仕様書(rust、language-agnostic)は更新済み | `docs_graph.rs`、`grep` | build-and-test | Met |
| T9-ci-os | `team.md`(他の OS は CI で確認。マージ前に CI を通す) | ubuntu、macos、windows の `cargo test` が通る | 途中のコミット(`965c972`。テストと README まで)の CI は全ジョブ成功。最終のコミットは未実行 | 既存の `ci.yml`(push で動く) | (既存の CI) | N/A(最終のコミットの CI は、push のあとで確かめる。この変更は OS に依存するコードを足していない) |

`Pending` の行は残っていない。

## 準備の評価

- ビルドとテストの準備: できている(全体 1,091 件、カバレッジ 97.94%)。
- 配信の準備: この作業の範囲の外(リリースは別の依頼)。

## 既知の限界

- 最終のコミットの CI(Windows と ubuntu、MSRV)は未実行(T9)。push 後に確認する。
- ジョブ 7 個、8 個の全例の照合は、開発担当の一時のテストで、再現できない。リポジトリに残るのは、ジョブ 6 個以下の 487 例など。
- 線が非常に多い形(1,000 ジョブなど)の図の正しさは、git の出力と照合していない(動作と性能だけ確認)。
- 設計の確認(アーキテクトの読み直し)は、承認の時点で終わっておらず、結果は、計画の承認のあとに届いた(その指摘は、開発担当への指示に加えて反映した)。
- 「両方の飛び越し」と分岐を含むグラフで、辺の一覧になる形は、残っている(別の intent の候補)。
- 開発担当が、一部のファイルにシェルを使った(`sed -i`、`perl -pi`。内容は fmt、clippy、全テストで確かめた)。
