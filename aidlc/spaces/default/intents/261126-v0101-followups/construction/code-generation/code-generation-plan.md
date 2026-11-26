# コード生成の計画: v0.10.1 の整理・修正(F1〜F5、文書のずれ、版)

## Summary

- Builds: ログの名前の予約名の検査で前後の空白を除く(F2)、読み取りの前にも「止める」印を見る(F2)、計画の模擬の測定と、二乗が原因なら flow の内部の入口での直し(F3)、デモの場面 4 の待ち(F5)、README の `watch --follow` の注意の書き直し(F1)、契約の表と「33 種」の直し、版を 0.10.1 にする
- Touches: runtime の `log.rs`・`runner.rs`・`lib.rs`・`tests/log_file.rs`、(F3 で直すときだけ)core の `flow/decide.rs`・`flow/next_step.rs`・`flow/record.rs`・`flow/mod.rs`・`plan/simulate.rs`、`demo/run-demo.sh`・`demo/README.md`、`crates/cli/tests/unix_demo.rs`、`README.md`、`docs/spec/rust/` の 6 章、2 つのテストの注釈、版の 22 ファイル
- Tests: 単体テスト 2〜3(名前の表の行、読む前の印、F3 で直すときは判断の一致)、デモのテスト 1、既存のテストすべて(流れ・計画・性能・文書)

出典: 機能の設計(`functional-spec.md` の W1〜W6、`rules.md` の BR1.1〜BR7.1、`entities.md`)、要件(`requirements.md` の FR1.1〜FR7.1、NFR1〜NFR8)、設計の承認で直すと決めたレビューの指摘(R-05、R-06)。Units とユーザーストーリーは無いので、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。層を作ってから、その層のテストを書いて走らせる。

## データの形の変更

- ステートの形、定義の書式、公開の型は変えない。
- F3 で直すときだけ、core の `flow` の中に `pub(crate)` の内部の入口と、目印を進める関数を足す(`entities.md` の `PlanSimulationLookup`。名前は実装で決め、`code-summary.md` に書く)。公開の API は変えない。

## 設計のレビューからの持ち越し(この段階で扱う)

- **R-05**: Step 5 の測定で、時間のかかる走査ごとに(`decide.rs` の `job_index`・`record_of`・規則 6 の `job[..index]`、`next_step.rs` の `job_index` と `pending_ids`、`record.rs` の `apply_skip_in` の `any(Error)` と `first_pending` など)、効き目を確かめる。走査ごとに、直すときに使う値(位置、「最初の済んでいないジョブの位置」、「最初の pending の位置」、「エラーがあるか」など)を決めて `code-summary.md` の表に書く。測定で効かない走査は直さない。値を進める関数は `flow` に置き、plan は持ち回るだけにする。
- **R-06**: F3 で直すときは、`flow` の `mod tests` に、内部の入口(位置と目印を渡す)と、今の公開の入口(位置を自分で求める)が、同じ入力に同じ判断と同じ記録になることを確かめるテストを足す。入力には、記録が定義の順でないステート、記録の欠け、再開位置(`resume_job_id`)、`end_reached`、エラーの記録を含める。

## ステップ

### Step 1: テストの準備を確かめる

- [ ] 変更の前に、`unit-test-instructions.md` のコマンドをそのまま走らせ、結果を `code-summary.md` に記録する(書き方の誤りを早く見つけるため。v0.10.0 の振り返り)。

### Step 2: F2 ログの名前(runtime)とその層のテスト

- [ ] `crates/runtime/src/log.rs` の `safe_log_stem` で、最初の `.` より前の部分を予約名と比べるときに、前後の ASCII の空白を除いた値で比べる。名前そのものは変えない。ドキュメントコメントの手順 3 を直す(BR2.1、FR2.1)。
- [ ] `crates/runtime/tests/log_file.rs` の名前の表に、要件の FR2.1 の表の行(`con`、`console`、`con `、` con`、` con `、`con .x`、`nul .tar.gz`)を足す(BR2.2、FR2.2)。
- [ ] `README.md` の「名前の置き換え」に、空白の例(`con ` → `_con_`)を足す(BR2.2、FR2.2)。
- [ ] runtime のコマンドを走らせる。

### Step 3: F2 読む前の「止める」印(runtime)とその層のテスト

- [ ] `crates/runtime/src/runner.rs` の `copy_stream` の繰り返しの初めに、印を見て立っていれば抜ける確かめを足す。読んだ後の確かめは変えない。ドキュメントコメントを直す(BR2.3、BR2.4、FR2.3)。
- [ ] `runner.rs` の `mod tests` に、印を立てた共有の状態と、`read` の回数を数える読み手で `copy_stream` を呼び、`read` が 0 回、端末とファイルに何も書かれないことを確かめるテストを足す。今の `Shared::new` と既存の補助を使い、本番にテスト専用の引数・`pub` を足さない(BR2.3)。
- [ ] runtime のコマンド(`runner_log.rs` を含む)を走らせる(BR2.4)。

### Step 4: 変更の前の全体の確かめ

- [ ] `cargo test --workspace --locked` を走らせ、Step 2・3 の後も全体が通ることを確かめる。

### Step 5: F3 の測定と判定(使い捨ての仕組み)

- [ ] 使い捨ての測定用のテストを `crates/core/tests/` に一時的に置き(コミットしない)、ジョブの数 250・500・1,000・2,000 で、`plan_simulate_perf.rs` と同じ形の計画を作る時間の中央値を、通常の `cargo test --release` ではなく debug(`cargo test`)と `cargo llvm-cov` の下の両方で測る(BR3.2、FR3.2)。
- [ ] 走査ごとの効き目を確かめる(R-05)。測定の表と、走査ごとの判断を `code-summary.md` に書く。
- [ ] BR3.3 で判定する(llvm-cov の下で、2 倍ごとの伸びがすべて 3 倍以上なら原因)。
- [ ] 測定用のファイルを消し、作業の木に残っていないことを `git status` で確かめる。

### Step 6: F3 の直し(Step 5 で原因と判定したときだけ)とその層のテスト

- [ ] `crates/core/src/flow/` に、位置と目印を受け取る `pub(crate)` の内部の入口と、目印を進める関数を足す。判断の本体(`decide_before_run` の規則 1〜7、`classify_and_decide`、記録の規則)は 1 か所のままにし、今の公開の入口は位置を自分で求めて同じ本体を呼ぶ形にする。`job_index`・`record_of` そのものは変えない(BR3.4、FR3.3)。
- [ ] `crates/core/src/plan/simulate.rs` が、定義の順に進むことを使って位置と目印を持ち回り、内部の入口を呼ぶ。判断や記録の規則を plan に書き写さない(BR3.4)。
- [ ] `flow` の `mod tests` に、R-06 の判断の一致のテストを足す。
- [ ] core のコマンド(流れ・計画・性能のテスト)を走らせ、すべてが変えずに通ることを確かめる。`plan_simulate_perf.rs` は変えない(BR3.1)。
- [ ] Step 5 と同じ測定を一時的に行い、llvm-cov の下で 1,000 ジョブが 75ms 以下、2 倍ごとの伸びが 2.5 倍以下かを確かめ、直す前と後の表を `code-summary.md` に書く。測定用のファイルは消す(BR3.5)。
- [ ] 届かなければ、この Step の変更を戻し(コミットしていなければ作業の木から外し、コミット済みなら戻すコミットを入れる)、Build and Test で BR3.6 の記録にする。直しを繰り返さない(BR3.5、BR3.6)。
- 判定が「原因でない」のときは、この Step を飛ばし、その旨と理由を `code-summary.md` に書く(Build and Test で `build-and-test-summary.md` にも書く。BR3.6)。

### Step 7: F5 デモの場面 4 とその層のテスト

- [ ] `demo/run-demo.sh` の場面 4 で、`watch --follow` の説明の 2 行の後、`cmd "$CONDUCTOR" watch --follow` の前に `pause` を足す(BR5.1、FR5.1)。`demo/windows/run-demo.cmd` は変えない(BR5.3)。
- [ ] `demo/README.md` の `DEMO_PAUSE` の説明に、UNIX 版では場面 4 の `watch --follow` の前でも待つことを書き足す(BR5.4、FR5.4)。
- [ ] `crates/cli/tests/unix_demo.rs` に、`DEMO_PAUSE=1` を足して動かし(標準入力は `run_limited` の既定の空のまま)、場面 4 の範囲で `(Enter で次へ)` が 1 回の `watch` の完了の行より後、`watch --follow` のコマンドの行より前に出ることを確かめるテストを足す(BR5.5、FR5.5)。既存のテストは変えない(BR5.2)。
- [ ] cli のデモのコマンドを走らせる。

### Step 8: F1 README の注意

- [ ] `README.md` の `watch --follow` の節で、「エラーの後に続くジョブがあるときの注意」を W5 のとおり書き直す(起きる使い方、起きること、端末 A・B の例、避け方)。「ステートが無いとき」の項と区別できるようにする(BR1.1、BR1.2、FR1.1、FR1.2)。`crates/core/src/flow/activity.rs` は変えない(BR1.3、FR1.3)。
- [ ] `src/tests/docs.rs` の必須の語句がこの節の語に依るときは、合わせて直す。cli の文書のテストを走らせる。

### Step 9: 文書のずれ

- [ ] `crates/runtime/src/lib.rs` の契約の表の `run_job` の行に、`unset` の引数を書き足す(BR6.1、FR6.1)。
- [ ] 「33 種」を「34 種」に直す: `docs/spec/rust/00-README.md` 23 行、`02-architecture.md` 103 行(v0.10.0 の `UnsetEnvConflict` の追加も書く)・297 行、`03-definition-format.md` 523 行の見出し、`10-errors-and-exit-codes.md` 120 行、`11-quality-and-build.md` 213 行、`12-rebuild-guide.md` 22 行・24 行・321 行、`crates/core/tests/env_types.rs` 56 行の注釈、`crates/cli/src/tests/report_messages.rs` 1225 行の注釈(R-07)。`12-rebuild-guide.md` の 105 行と 285 行は残す(BR6.2、FR6.2)。
- [ ] `grep -rn '33 種' docs/spec/rust/ crates/` の結果が `12-rebuild-guide.md` の 105 行だけであることを確かめる。

### Step 10: 全体の確かめ

- [ ] `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked` を走らせる(NFR7、NFR1)。

### Step 11: 版を 0.10.1 にする

- [ ] 4 クレートの `Cargo.toml` と `Cargo.lock`、`crates/cli/tests/binary_basic.rs` の `--version` の期待、`docs/spec/` の版の表記(e112426 と同じ 22 ファイル)を 0.10.1 にする。「v0.10.0 から」などの注記は上げない(BR7.1、FR7.1)。件名は「版を 0.10.1 にする」の 1 コミット。
- [ ] `cargo test --workspace --locked` を走らせる。

### Step 12: 記録

- [ ] `code-summary.md`(変更の一覧、測定の表、走査ごとの判断、テストの結果、コミット)と `traceability.json` を書く。

## コミット

各ステップの区切りで、日本語の件名(`runtime:`、`core:`、`cli:`、`docs:`、`aidlc:`)の小さなコミットを入れる。版の更新は「版を 0.10.1 にする」の 1 コミット。push はしない(人が行う)。

## 守ること

- ファイルの作成と編集はファイルの道具だけで行い、シェル(`sed -i`、`python3`、heredoc)で書き換えない(v0.10.0 の振り返り)。
- 新しい依存を足さない。`unsafe` と `cfg` の場所の決まりを守る。エラーの表示に入力の値を含めない。テストの都合で本番を曲げない。
- 性能のテストの上限、カバレッジの下限を下げない。
