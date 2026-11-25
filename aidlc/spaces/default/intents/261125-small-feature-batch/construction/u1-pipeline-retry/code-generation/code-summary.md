# コードのまとめ: u1-pipeline-retry

## 変えたファイル

新しく作ったファイルと消したファイルはない。

| ファイル | 変更 |
|---|---|
| `crates/core/src/definition/walk/retry.rs` | `JobRetrySetting`(無し・false・表)、`PipelineRetryDefault`、表 1 つを読む共有の `retry_table`、`job_retry`、`pipeline_retry`、`check_retry_interval`、`overlaps_job_codes`。理由の文 3 つは固定の定数 |
| `crates/core/src/definition/walk/mod.rs` | `PIPELINE_KEYS` に `retry`。`RawJob` に `start`。既定は `name` の判定より前に読み、`pipeline_section` とは別に返す。`cross_check` と `build_jobs` が既定を受け取る |
| `crates/core/src/types.rs` | `JobDefinition.retry` のドキュメントコメントだけ(型の形は変えない) |
| `crates/core/tests/definition_retry.rs` | 19 件追加 |
| `crates/core/tests/definition_valid.rs` | 1 件追加(既定を書かない定義は今と同じ) |
| `crates/core/tests/nfr_security.rs` | `cases` に 3 件、テスト 1 件追加 |
| `crates/core/tests/definition_perf.rs` | P3(1 万のジョブに既定を当てはめる。上限 700 ms) |
| `crates/cli/tests/flow_retry.rs` | 3 件追加(既定によるリトライ、`retry = false`) |
| `crates/cli/tests/flow_validate.rs` | 1 件追加(重なりが 64) |
| `crates/cli/tests/binary_basic.rs` | 1 件追加(実バイナリの `validate` が 64、値が出ない) |
| `crates/cli/src/tests/graph_report.rs`、`plan_report.rs` | 各 1 件追加(既定由来と `[job.retry]` で表示が同じ) |
| `README.md` | `[pipeline.retry]`、実効のリトライの決まり方、重なりの誤り、`retry = false`、終了コード 64 の説明 |
| `demo/retry.toml`、`demo/windows/retry.toml`、`demo/README.md` | `flaky` のリトライを `[pipeline.retry]` の既定に移し、`retry = false` をコメントで示す |

## 主な判断

- 既定の組み立ては 1 度だけ。書いていなければ無し、書いたのに値が欠けていれば定義を組み立てない(`build_retry` と同じ扱い)。`retry` を書かないジョブには既定の複製を入れる(BR2.2)。
- 既定の `max_interval < interval` は読み取りのときに 1 回だけ誤りにする(BR3.5)。
- ジョブの `retry = true` などの誤りは、誤りを積んだうえで「無し」として扱う。誤りがあれば定義は組み立てないため、結果に影響しない。
- 性能のテスト P3 の上限は 700 ms(debug の実測: 既定あり約 140 ms、既定なし約 137 ms。今のテストと同じく約 5 倍の余裕)。
- 既存のテストは書き換えていない。`br1_2_retry_が表でない` は種類と行だけを見るため、直す必要が無かった。README の、テストが求める一文はそのまま残し、括弧の中に既定のことを足した。

## 計画からのずれ

- コミットの順: `RawJob.start` は Step 4 まで読まれず、Step 2 だけでは clippy の `-D warnings` で落ちる。このため `start` は Step 4 のコミットに入れ、Step 3 と Step 5 の当てはめは 1 つのコミットにまとめた。計画の内容はすべて実装した。
- `flow_retry.rs` の `retry = false` のテストは、前のジョブを先に済ませる形にした(先に済ませないと、リトライとは関係なく、前のジョブが済んでいないことで 66 になる)。
- `unit-test-instructions.md` の 3 つ目のコマンド(`cargo test -p conductor-cli --lib --locked tests::graph_report tests::plan_report tests::docs`)は、cargo が `--` の前にフィルタを 1 つしか受け付けないため、そのままでは `unexpected argument` で止まる。実際には `cargo test -p conductor-cli --lib --locked -- tests::graph_report tests::plan_report tests::docs` で走らせた。承認済みの手順の文書は変えていない。

## コミット(`main`、push はしていない)

3e2f629、6ab4a6e、131fd27、32acac1(core の実装と文書)、abdf8be、2c7b5e5(core のテスト)、40a9cd3(cli のテスト)、5ad33dd、a73693d(デモと README)。

## テスト

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-core --test definition_retry --test definition_valid --test nfr_security --test definition_perf --locked` | 変更前 36/9/5/2、変更後 55/10/6/3、すべて通過 |
| `cargo test -p conductor-cli --test flow_retry --test flow_validate --test binary_basic --test unix_demo --locked` | 変更前 19/13/19/3、変更後 22/14/20/3、すべて通過 |
| `cargo test -p conductor-cli --lib --locked -- tests::graph_report tests::plan_report tests::docs` | 57 件通過 |
| `cargo test --workspace --locked` | 1,178 件通過、失敗 0 |
| `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | 通過 |

## 書いたが、ここでは走らせていないテスト

- `crates/cli/tests/windows_demo.rs`: Windows でだけ動くため、このマシン(macOS)では走らせていない。CI の windows の `test` で確かめる。Windows 版のデモの定義が読めることは `conductor validate --config demo/windows/retry.toml`(0 で終了)で確かめた。
- 行カバレッジ 80% の下限: プロジェクト全体の下限なので、Build and Test で全体のテストで測る。
