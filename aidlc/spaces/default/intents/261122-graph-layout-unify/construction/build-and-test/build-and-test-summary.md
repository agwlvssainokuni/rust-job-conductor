# ビルドとテストのまとめ(Build and Test Summary): graph の配置のリファクタリング

## 全体の状況

- ビルド: 成功。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: 全体で **1,118 件が成功、失敗 0 件**(この intent の前は 1,091 件)。
- 行カバレッジ: **97.99%**(下限 80%)。`graph/layout.rs` は 99.65%。
- 前提: Rust 1.100 以上(`rustc 1.100.0`、MSRV と同じ)、`cargo-llvm-cov`。新しい依存なし。
- 詳細: `build-instructions.md`、`integration-test-instructions.md`、`performance-test-instructions.md`、`security-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類(Minimal)

| 種類 | 内容 | 場所 |
|------|------|------|
| 単体(core) | 正解データ 7 つ(計 7,400 例)の照合。判定。N1〜N5、B1、D1。性能(1,000 ジョブ) | `graph_layout_oracle.rs`、`graph_layout_unify.rs`、`graph_drawable.rs`、`graph_perf.rs` ほか |
| 整形(cli) | text の全文、`-v` の詳細の行の列 0、`--state`、辺の一覧の整形 | `graph_report.rs` |
| 実バイナリ | N1 の定義で `conductor graph` | `binary_graph.rs` |
| 文書 | README の図の例が実出力と一致 | `docs_graph.rs` |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract、`team.md`、`project.md` の ALWAYS | 行カバレッジ 80% 以上(プロジェクト全体) | 97.99%(9,482 行のうち未通過 191) | `cargo llvm-cov ... --fail-under-lines 80 --summary-only` が終了コード 0 | build-and-test | Met |
| T2-quality | `project.md` の ALWAYS(`fmt`、`clippy -D warnings`、MSRV 1.100) | 差分なし、警告なし、MSRV で通る | 差分なし、警告なし。rustc 1.100.0(MSRV と同じ)で全テスト成功 | `cargo fmt --check`、`cargo clippy -D warnings`、`rustc --version` | build-and-test | Met |
| T3-no-new-deps | `project.md` の NEVER(新しい依存を足さない) | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行(`510382e` 以降) | `git diff 510382e HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` | build-and-test | Met |
| T4-git-oracles | BR3.1、SM2 | git の出力の正解データは、変えずに一致 | 537、1,517、940、487 例が、ファイルの変更なしで 1 文字も違わず一致 | `graph_layout_oracle.rs`、`git diff`(差分 0 行) | build-and-test | Met |
| T5-derived-oracles | BR4.1、SM3 | 規則から導いた 871、2,748、300 例と一致 | 一致(参照実装でも全例一致) | `graph_layout_oracle.rs` | build-and-test | Met |
| T6-one-engine | BR1.2、SM1 | 古い経路がなくなる | `lane_layout`、`open_all_off_main`、`is_ordinary_skip_graph` などは、コードとテストに残らない | `grep`、確認(アーキテクト) | build-and-test | Met |
| T7-license | BR5.1(git の GPL v2 のソースは読まない) | git のソースを読まず、コードに写していない | 読んでいない | 設計と実装の確認 | build-and-test | Met |
| T8-docs | BR4.2、SM4、CI の `docs-check` | README と仕様書が実装に合う。必須語句が残る | README の図の例が実出力と一致。必須語句 7 つが残る。仕様書(rust、language-agnostic の 05、08、11、12)を更新 | `docs_graph.rs`、`grep` | build-and-test | Met |
| T9-ci-os | `team.md`(他の OS は CI で確認。マージ前に CI を通す) | ubuntu、macos、windows の `cargo test` が通る | 未実行(push の前) | 既存の `ci.yml`(push で動く) | (既存の CI) | N/A(push のあとで確かめる。この変更は OS に依存するコードを足していない) |

`Pending` の行は残っていない。

## 準備の評価

- ビルドとテストの準備: できている(全体 1,118 件、カバレッジ 97.99%)。
- 配信の準備: この作業の範囲の外(リリースは別の依頼)。

## 既知の限界

- CI(Windows と ubuntu、MSRV)は未実行(T9)。push 後に確認する。
- 規則から導いた正解データ(871、2,748、300 例)は、git では確かめられない。照合できるのは、Rust の移植が参照実装と一致することまで。ジョブ 8 個以上の「両方の飛び越し」・分岐の図は、性能と動作だけを確かめた。
- 辺の一覧は、どの定義からも出なくなった(防御の経路として残す。注記の文言は、今の理由に合わせた)。
- 途中のコミット(`37cbbe5`〜`344843a` の手前、`docs_graph` は `fb5a08e` の手前まで)では、一部のテストが失敗する(`git bisect` に注意。履歴は書き換えない)。
- 版は 0.9.3 のまま(リリースは別の依頼)。
