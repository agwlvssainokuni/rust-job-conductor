# コードの要約(Code Summary): `--dry-run` の模擬の重複をなくすリファクタリング

計画: `code-generation-plan.md`(Step 1〜8)。テスト: Minimal、test-after。

## コミット(`main` に直接。push はしていない)

| コミット | 内容 | 規則・指標 |
|----------|------|------------|
| `f8ccfe8` | ステートの更新の関数を core の `flow/record.rs` へ移す(`git mv`)。runtime は `conductor_core` から再公開 | BR1.1、BR1.2。SM1 |
| `362ed8c` | `apply_skip` の計算量を O(n) にする(状態を定義の位置で引く) | BR2.3。SM3 |
| `384890c` | 模擬(`plan_pipeline`)は `PipelineState` を 1 つ持ち、`apply_skip`・`apply_outcome` で記録する。`apply_result` をなくす | BR1.3。SM2 |
| `6899023` | テストを 2 本足す(性能、手で書き換えたステートの形) | BR2.3、SM2、SM3 |
| `b61d68a` | 仕様書(`docs/spec/rust` の 05、02、12)を直す | BR3.1。SM4 |

各コミットで、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ることを確かめた。

## 変えたファイル

- `crates/core/src/flow/record.rs`(`crates/runtime/src/state/record.rs` から移動): `use` を `crate::...` に直し、冒頭の説明に「本番と模擬が共に使う」を足した。単体のテストは、`use` の 1 行(`conductor_core::JobDefinition` → `crate::types::JobDefinition`)のほかは変えていない。`apply_skip` の `first_pending` の探索を、位置で引く形にした。
- `crates/core/src/flow/mod.rs`: `mod record;` と `pub use record::{apply_interrupt, apply_outcome, apply_skip, initial_state};`。冒頭の説明。
- `crates/core/src/lib.rs`: 公開の一覧に 4 つの関数を足した。公開する関数の表とモジュールの表に記録の行を足した。
- `crates/core/src/plan/simulate.rs`: `plan_pipeline` は、ステートが無ければ `initial_state`(パスは模擬の中だけの値 `<dry-run>`)から始め、スキップは `apply_skip`、結果は `apply_outcome` で記録する。判断には、そのステートの一覧、再開位置、`end_reached` を渡す。終わり方の「終点に着く」は、そのステートの `end_reached`。`apply_result` と、別の変数 `resume`、`end_reached` をなくした。`simulate_job` は `(SimulatedResult, JobExecution, JobOutcome, NextStep)` を返す。`plan_run` は、戻り値の受け取り方のほかは変えていない。冒頭の説明を直した。
- `crates/runtime/src/lib.rs`: `pub use conductor_core::{apply_interrupt, apply_outcome, apply_skip, initial_state};`。関数の表(崩れていた表の並びも直した)とモジュールの表の `state::record` の行を直した。
- `crates/runtime/src/state/mod.rs`: `record` を外し、説明を直した。
- `crates/core/tests/plan_perf.rs`: `終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる` を足した。
- `crates/core/tests/plan_pipeline.rs`: `手で書き換えた_end_のスキップがあり_end_reached_が偽のステートは_終点に着くで終わる` を足した。
- `docs/spec/rust/05-execution-flow.md`、`02-architecture.md`、`12-rebuild-guide.md`。

`Cargo.toml`、`Cargo.lock` に差分はない。`unsafe`、`cfg(unix)`/`cfg(windows)`、新しい依存は足していない。core に入出力は持ち込んでいない(`std::path::Path` は文字列の型として使うだけ)。cli のコードは変えていない。

## テストの件数

| 時点 | 通った | 失敗 | 無視 |
|------|--------|------|------|
| 変更の前(Step 1) | 1,121 | 0 | 1 |
| 変更の後(Step 7) | 1,123 | 0 | 1 |

増えた 2 件は、足した 2 本のテスト。既存のテストは変えていない(移した単体のテストの 17 件は、runtime の lib から core の lib に数えが移っただけ)。

## 比べる確認(一時のテスト。削除済み。コミットしていない)

`crates/runtime/tests/tmp_plan_compare.rs` に、書き換える前の `plan_pipeline` のループと `apply_result` を「今の模擬」として写し、書き換えた本物の `conductor_core::plan_pipeline` と比べた(扱い、終わり方、拒否)。定義とステートの作り方は設計の `plan-compare.rs.txt` と同じ。

| 種 | 例の数 | 違い | 遷移でスキップ | 終点の後でスキップ | 実行済み | エラーを含むステート | 再開位置のあるステート |
|----|--------|------|----------------|--------------------|----------|----------------------|------------------------|
| 20261122 | 300,000 | 0 | 112,886 | 125,313 | 175,625 | 85,592 | 117,838 |
| 7 | 300,000 | 0 | 113,641 | 123,359 | 176,168 | 85,393 | 117,638 |
| 123456789 | 300,000 | 0 | 113,866 | 122,788 | 176,345 | 85,570 | 117,707 |

数は、設計の段階の結果(`functional-spec.md` §2)と一致した。

## 性能

`終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる`(999 個のスキップ。デバッグのビルド、macOS): 約 109 ms。設計の段階で、書き換える前の `apply_skip` を同じ形で 999 回呼ぶと 1.13 秒だった。

## 承認した変化

手で書き換えたステートで、`skipped(byEnd)` のジョブがあり `end_reached = false` の形は、終わり方が「完了」から「終点に着く」に変わる(本番と一致)。`plan_pipeline.rs` の新しいテストで固定した。
