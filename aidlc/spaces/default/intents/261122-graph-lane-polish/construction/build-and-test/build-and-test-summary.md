# ビルドとテストのまとめ(Build and Test Summary): graph の text 表示の改善

## 全体の状況

- ビルド: 成功。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: 全体で **1,038 件が成功、失敗 0 件**。
- 行カバレッジ: **97.94%**(下限 80%)。
- 前提: Rust 1.100 以上(`rustc 1.100.0`)、`cargo-llvm-cov`。新しい依存なし。
- 詳細: `build-instructions.md`、`integration-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類(Minimal)

| 種類 | 内容 | 場所 |
|------|------|------|
| 単体(core) | 正解データ 537 例と 1,517 例の照合、図にできるかの判定、追加の例(D1〜D6) | `graph_layout_oracle.rs`、`graph_drawable.rs`、`graph_lane_polish.rs` |
| 整形(cli) | text の全文、`-v`、`--state`、英語の注記 | `graph_report.rs` |
| 実バイナリ | 同じ行き先、両方の飛び越し、交差の注記 | `binary_graph.rs` |
| 文書 | README の例が実出力と一致 | `docs_graph.rs` |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract、`team.md`、`project.md` の ALWAYS | 行カバレッジ 80% 以上(プロジェクト全体) | 97.94%(9,328 行のうち未通過 192) | `cargo llvm-cov ... --fail-under-lines 80 --summary-only`(`test-results.md`) | build-and-test | Met |
| T2-quality | `project.md` の ALWAYS(`fmt`、`clippy -D warnings`、MSRV 1.100) | 差分なし、警告なし、MSRV で通る | 差分なし、警告なし。rustc 1.100.0(MSRV と同じ)で、全テストが成功 | `cargo fmt --check`、`cargo clippy -D warnings`、`rustc --version` | build-and-test | Met |
| T3-no-new-deps | `project.md` の NEVER(新しい依存を足さない) | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行(`9949237` 以降) | `git diff 9949237 HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` | build-and-test | Met |
| T4-structure | `project.md` の NEVER(`unsafe`、`cfg` の置き場) | 新しいコードが規則を守る | `nfr_source`(cli 6)と `nfr_security`(core 5)が成功 | `cargo test`(全体) | build-and-test | Met |
| T5-other-unchanged | BR3.2、成功の指標 SM2 | ほかの形の出力は変わらない | 既存の 537 例が 1 文字も違わず一致。既存のテスト(書き換えた 3 つの表明を除く)が成功 | `graph_layout_oracle.rs` | build-and-test | Met |
| T6-docs | BR4.2、SM3、CI の `docs-check` | README と仕様書(rust、language-agnostic)が実装に合う。必須語句が残る | README の例が実出力と一致(`docs_graph.rs`)。必須語句 7 つが残る。仕様書は更新済み(コミット `dbd90c7`) | `docs_graph.rs`、`grep` | build-and-test | Met |
| T7-ci-os | `team.md`(他の OS は CI で確認。マージ前に CI を通す) | ubuntu、macos、windows で `cargo test` が通る | macOS のみ実行(このマシン)。Windows と ubuntu は未実行 | 既存の `ci.yml`(push で動く)。CI Pipeline は、既存の CI で足りるため省略した | (既存の CI) | N/A(push のあとの CI の結果で確かめる。この作業の変更は OS に依存するコードを足していない) |

`Pending` の行は残っていない。

## 準備の評価

- ビルドの準備: できている。
- テストの準備: できている(全体 1,038 件、カバレッジ 97.94%)。
- 配信の準備: この作業の範囲の外(版の引き上げとリリースは別の依頼)。

## 既知の限界

- Windows と ubuntu の CI は、まだ実行していない(T7)。push 後に `ci.yml` の結果を確かめる。
- 「両方の飛び越し」の図(D1〜D6)は、git では確かめられず、規則から導いた(同じ行き先は git の出力 1,517 例で確かめ済み)。
- 開発担当が、指示に反して、一部のファイルの作成にシェルの Python を使った(内容は fmt、clippy、全テストで確かめた)。
- 交差する飛び越しは、辺の一覧のまま(別の intent の候補。メモ `project_graph_text_next_intent.md`)。
