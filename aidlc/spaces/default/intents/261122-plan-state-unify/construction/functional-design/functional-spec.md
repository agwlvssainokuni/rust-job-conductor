# 機能の仕様(Functional Spec): `--dry-run` の模擬の重複をなくすリファクタリング

対象: `crates/runtime/src/state/record.rs`(本番の更新)と `crates/core/src/plan/simulate.rs`(模擬)。意図: `ideation/intent-capture/intent-statement.md`(Q1: A、Q2: B、Q3: A)。

## 1. 今の二重(読んで確かめた)

| 更新 | 本番(runtime の `record.rs`) | 模擬(core の `simulate.rs`) |
|------|-------------------------------|-------------------------------|
| 結果の記録 | `apply_outcome`(77〜115 行) | `apply_result`(298〜325 行)。分類を状態に変える部分と、`skipped_job_ids` の書き方が同じ文の写し。再開位置と `end_reached` は、別の変数 `resume`、`end_reached` で持つ |
| スキップの記録 | `apply_skip`(125〜167 行) | 何もしない(「模擬の状態は変えない(BR1.8)」) |
| 空のステート | `initial_state` | すべて pending の状態の一覧を自分で作る |

- core は runtime を呼べない(依存の向きは runtime → core)ので、写しになった。`record.rs` は、入出力のない純粋な関数だけで、core の型にしか依存しない(`std::path::Path` と `conductor_core` の型だけ)。
- 模擬が状態を更新するのは `plan_pipeline` だけ。`plan_run` は 1 つのジョブの判断と分類だけで、状態を更新しない(この変更の対象外)。

## 2. 比べる確認の結果(Q1: A、Q3: A。設計の段階で 1 回)

今の模擬(`plan_pipeline`)と、スキップを `apply_skip`、結果を `apply_outcome` で記録する模擬を、ランダムな定義とステートで比べた。比べたのは、ジョブごとの扱い(実行する、遷移でスキップ、終点の後でスキップ、実行済み、届かない)、終わり方(完了、終点に着く、エラーで止まる)、拒否(順序外)。

- 定義: ジョブ 2〜6 個。各ジョブの成功と警告に、確率 1/3 で、後ろのジョブか `@end` への遷移。警告の終了コードは 3。
- ステート: 本番の更新の関数(`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`、`resume_point_on_interrupt`)で、本番の流れ(`pipeline` の 1 回か `run` の 1 回。終了コードは 0、3、1 をランダム。`pipeline` ではジョブのあいだの中断も入れる)を、定義ごとに 6 回まで進めて作った。完了したステートは、本番と同じく空として渡す。
- 仮定(`--assume`): 各ジョブ、確率 1/3 で、0、3、1 のどれか。
- 乱数の種を 3 つ(20261122、7、123456789)で、各 30 万例、計 90 万例。

| 種 | 例の数 | 違い | 遷移でスキップ | 終点の後でスキップ | 実行済み | エラーを含むステート | 再開位置のあるステート |
|----|--------|------|----------------|--------------------|----------|----------------------|------------------------|
| 20261122 | 300,000 | 0 | 112,886 | 125,313 | 175,625 | 85,592 | 117,838 |
| 7 | 300,000 | 0 | 113,641 | 123,359 | 176,168 | 85,393 | 117,638 |
| 123456789 | 300,000 | 0 | 113,866 | 122,788 | 176,345 | 85,570 | 117,707 |

**結論: 本番が作るステートでは、違いは 0 件。** 意図の記述の「まれな形で判断が違う恐れ」(未確認の見込み)は、この範囲では起きなかった。理由(読んで確かめた):

- 新しい模擬では、`apply_skip` が再開位置を「最初の pending」に書き換えるが、今の模擬は古い値のまま。後のジョブの判断で再開位置が効くのは、判定表(05 §4.2)の規則 4 だけ。
- ループが後のジョブ `j` に着くとき、前のジョブは、すべて終わっている(success、warning、skipped)か、終点の後で pending のまま(このときは `end_reached` なので、規則 2 が規則 4 より先に当たる)か、`Reject` で計画が終わっている。前がすべて終わっていれば、規則 6 で `Run` になり、規則 4 の結果と同じ。したがって、再開位置の違いは、判断を変えない。

確認のコード: `plan-compare.rs.txt`(この記録。リポジトリのテストには入れない)。定義に、エラーの遷移、`success_exit_codes`、リトライ、run ID は入れていない(どれも、判断と記録の規則に効かない)。

**範囲の外の形で、出力が 1 つ変わる(確認で見つかった。ユーザーの判断が要る)**: 手で書き換えたステートで、あるジョブが `skipped(byEnd)` なのに `end_reached = false` のもの(本番の流れでは作られないが、定義との照合 `check_consistency` は通る)。今の模擬は、終わり方を「完了」と出すが、新しい模擬は、`apply_skip`(終点の後)が `end_reached` を立てるので「終点に着く」と出す。本番の実行も、同じく `end_reached` を立てるので、**新しい模擬の出力のほうが、本番と一致する**。**ユーザーは、この形の変化を認めて承認した**(設計の承認のとき)。

## 3. 変更(WF1)

1. **更新の関数を core へ移す**: `crates/runtime/src/state/record.rs` を、中身(単体のテストを含む)ごと `crates/core/src/flow/record.rs` へ移す(`git mv`)。`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` を、core の公開の名前にする(`conductor_core::apply_outcome` など)。
2. **runtime は再エクスポート**: `crates/runtime/src/lib.rs` の `pub use state::record::{...}` を、`pub use conductor_core::{apply_interrupt, apply_outcome, apply_skip, initial_state};` にする。cli の `use conductor_runtime::{...}` は、変えずにそのまま動く。
3. **模擬は本番の更新を使う**: `plan_pipeline` は、`PipelineState` を 1 つ持つ(ステートが無ければ `initial_state`。パスは模擬の中だけで使う値)。スキップは `apply_skip`、結果は `apply_outcome`(仮定の終了コードの `JobExecution` で)で記録する。判断には、その状態の一覧、再開位置、`end_reached` を渡す。終わり方の「終点に着く」は、その状態の `end_reached` で決める。`apply_result` と、別に持っていた `resume`、`end_reached` の変数はなくす。`simulate_job` は、記録に使う `JobExecution` と `JobOutcome` も返す形にする(分類を 2 回しない)。
4. ほかの模擬の処理(計画の項目、仮定、`plan_run`)は変えない。
5. **`apply_skip` の計算量を下げる(性能)**: 今の `apply_skip` は、最初の pending を探すとき、ジョブごとに状態の一覧を頭から探すので、1 回が O(n²)。終点の後に 1,000 個のジョブをスキップする形では、本番の更新の関数で 1.13 秒かかった(デバッグのビルド。設計の段階で計測)。模擬がこれを使うと、`--dry-run` の 1 秒の目安(`plan_perf.rs`)を超える。そこで、状態を、定義の位置で引く形(`decide.rs` の `status_at` と同じ考え。位置が一致すれば O(1)、一致しなければ探す)にして、1 回を O(n) にする。結果は変えない(`record.rs` の単体のテストと、本番の流れのテストで確かめる)。本番の `pipeline` も、同じだけ速くなる。

## 4. 仕様書(WF2)

`docs/spec/rust` を直す。

- 05: 冒頭の「実装の所在」(記録は core の `flow/record.rs`)。§3.3〜§3.5 の見出しの所在。§16(`--dry-run`)の擬似アルゴリズムを、`apply_skip` と `apply_outcome` を呼ぶ形にし、`apply_result` の説明を、§3.3〜§3.5 を指す 1 文にする(更新の規則を 1 回だけ書く)。
- 02: core のモジュールの表に `flow/record.rs`、runtime の `state/record.rs` の行を削除、runtime の再公開の一覧、データの流れの図の所在。
- 12: 再構築の手順の、`state/record.rs` の行(58 行あたり)と、`plan/simulate.rs` の行の `simulate_job`、`apply_result`(126 行あたり)。
- 05 §16 の `simulate_job` の形(記録に使う実行と結果も返す)。
- コードの中の説明: `crates/runtime/src/lib.rs` のモジュールの表、`crates/runtime/src/state/mod.rs`(`record` のモジュールと説明)、`crates/core/src/plan/simulate.rs` の冒頭の説明、`crates/core/src/lib.rs` の公開の一覧。

`docs/spec/language-agnostic` は、変えない(模擬の出力は変わらず、「模擬の状態は変えない」という記述も、出力として同じ結果を与える。§2 で確かめた)。

完了の基準: rust の仕様書で、ステートの更新の規則は §3.3〜§3.5 に 1 回だけ書かれ、模擬の節はそれを指す。

## 5. テストの方針(Minimal の test-after)

- 既存のテストは、変えずに、すべて通る(runtime の `record.rs` の単体のテストは、ファイルごと core へ移るので、中身は変えない。`plan_pipeline.rs`、`plan_run.rs`、`flow_sequences.rs`、runtime と cli の流れのテスト)。
- 実装の後にも、比べる確認を 1 回行う: 書き換える前の `plan_pipeline` を、一時のテストに写して「今の模擬」とし、書き換えた本物の `plan_pipeline` と、§2 と同じ条件(3 つの種、各 30 万例)で比べ、違いが 0 件であることを確かめる。確かめたら、その一時のテストは削除する(リポジトリには入れない)。
- 新規: 性能のテストを 1 本(`plan_perf.rs`。終点の後に 1,000 個近いジョブをスキップする `--dry-run` の計画が 1 秒以内)。§2 の手で書き換えた形をユーザーが承認したときは、その形の計画のテストを 1 本(終わり方が「終点に着く」)。行カバレッジが下がるときだけ、通っていない枝のテストを足す。
- 合格の条件: `cargo fmt --check`、`clippy -D warnings`、全テスト、行カバレッジ 80% 以上、MSRV 1.100。新しい依存なし。

## 6. 成功の指標との対応(traceability.json の `SM1`〜`SM5`)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 更新の関数が core の 1 か所にあり、本番と模擬が呼ぶ。写しはなくなる。runtime の名前は再エクスポート | BR1.1、BR1.2 |
| SM2 | 模擬も、スキップを本番と同じ記録で扱う | BR1.3 |
| SM3 | 既存のテストが変えずに通る。比べる確認(違い 0 件、または承認) | BR2.1、BR2.2 |
| SM4 | 仕様書が、共通の更新として書き直される | BR3.1 |
| SM5 | 合格の条件 | BR4.1 |
