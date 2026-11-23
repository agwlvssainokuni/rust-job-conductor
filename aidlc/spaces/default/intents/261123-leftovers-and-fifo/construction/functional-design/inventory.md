# 変える場所の一覧(Inventory): 残りの整理と読み込みの順序のそろえ

HEAD 025067a(v0.9.7 のあとの意図の把握の記録のコミット)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。直前の調査の候補と実物が違ったところは「(調査との違い)」と書く。各段の設計の要点、テストの変更の一覧(T…)、消すテストの対の表、足すテスト(N…)、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

## 0. 基準

### 0.1 テストの件数(`cargo test --locked --workspace`、macOS(Apple M2)、rustc 1.100.0、debug、HEAD 025067a)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 115 | 268 | 383 |
| runtime | 68 | 127 | 195 |
| converter | 54 | 64 通過 + 1 無視 | 118 通過 + 1 無視 |
| cli | 0 | 461 | 461 |
| 全体 | | | **1,157 通過、1 無視、0 失敗** |

前回(types-perf-visible)の見込み 1,157 と一致する。この intent に関わるファイルの件数(基準): core `definition_errors.rs` 28、`env_merge.rs` 8、`retry_flow.rs` 17、`flow_sequences.rs` 9、`graph_types.rs` 5。runtime `runner_basic.rs` 12、`runner_env.rs` 5、`runner_signal.rs` 6、`nfr_robust.rs` 6、`state_read.rs` 14、`state_read_raw.rs` 4、`state_write.rs` 10、`run_id_state.rs` 15。converter `read_resolve.rs` 29、`convert_ac.rs` 9。cli `report_messages.rs` 36、`flow_init.rs` 9、`flow_validate.rs` 12、`escape_table.rs` 4、`graph_long_id.rs` 2、`binary_basic.rs` 19、`flow_aux.rs` 33。

### 0.2 計算量の直す前の測定(Q6)

- 測り方: macOS(Apple M2)、rustc 1.100.0、HEAD 025067a、**debug**(CI の `test` ジョブと同じ `cargo test`)。一時的なテストのファイル `crates/core/tests/tmp_measure_lf.rs` と `crates/cli/tests/tmp_measure_lf.rs` を作って `cargo test -p <crate> --locked --test tmp_measure_lf -- --nocapture --test-threads=1` で測り、**測り終えたあと消した**(`git status` で残っていないことを確かめた。コミットしていない)。
- 各場面を **5 回測った中央値**(と最小・最大)。前回(types-perf-visible の §0.2)と同じ方法。
- 「直した後の見積もり」は、今の線形の探索の部分だけを同じ入力で取り出して測った時間(L)と、直した後の形(`HashSet` など)を同じ入力で測った時間(H)から、「直す前 − L + H」で見積もった。

#### 入力の大きさの決め方

| 場面 | 大きさ | 根拠 |
|------|--------|------|
| M1: 定義の `env` の表の大文字小文字の検査 | `[pipeline.env]` に別々の名前 10,000 個(118,977 バイト) | 定義の `env` の項目の数には上限が無い(定義ファイルの上限 1 MiB(`MAX_DEFINITION_BYTES`)で約 8 万個書ける)。`.env` の項目の上限 `MAX_ENV_ENTRIES = 10,000` と同じ大きさにそろえる。直す前でも 1 秒以内に測れ、二乗なので大きくすると差が広がるだけ(20,000 個で 1.9 秒、40,000 個で 8.4 秒を確かめた) |
| M2: 遷移の重複の確認 | 1 つのジョブに `success` の遷移 20,000 個、続けて `warning` の遷移 20,000 個(インラインの配列。960,120 バイト) | 最悪の並び(前半が 1 種類、後半が別の種類。後半の各確認が前半をすべて読む)を、定義ファイルの上限 1 MiB に収まる最大に近い大きさで。重複の誤り(`duplicateTransition`)が 39,998 件出る |
| M3: `env_file_refs` の重複の確認 | `[pipeline].env_file` に別々の記述 10,000 個と、ジョブの `env_file` に同じ 10,000 個 | M1 と同じく `MAX_ENV_ENTRIES` にそろえる(1 MiB で約 8 万個書ける)。20,000 個で 2.2 秒、40,000 個で 9.3 秒を確かめた |
| M4: `graph -v` の詳細の辺の絞り込み | ジョブ 1,000 個(各ジョブが成功で次、警告で 2 つ先へ遷移。辺 1,998 本。レーンの図) | 仕様書 11 章の「1,000 ジョブ」と前回の P4 と同じ大きさ |
| M5: `record_skip` の複製と比較 | ジョブ 1,000 個で、`j0` のあと残り 999 個を `AfterEnd` でスキップ(ファイルの書き込みを除き、`base.cloned()` と `apply_skip` と比較の部分だけ) | M4 と同じ |

#### 結果

| 場面 | 直す前 debug 中央値(最小〜最大) | 線形の探索の部分(L) | 直した後の形(H) | 直した後の見積もり |
|------|----------------------------------|--------------------|------------------|--------------------|
| M1: `parse_definition`(10,000 個) | **618 ms**(616〜631) | 621 ms(`seen: Vec` の `eq_ignore_ascii_case` を 10,000 回) | 5.5 ms(小文字の名前の `HashSet`) | **≒ 72 ms**(注 1) |
| M2: `parse_definition`(20,000 + 20,000 個) | **2,115 ms**(2,100〜2,128) | 1,791 ms(`out.iter().any(..)` を 40,000 回) | 0(種類ごとの真偽値 2 つ) | **≒ 324 ms** |
| M3: `env_file_refs(def, None)`(10,000 + 10,000 個) | **630 ms**(623〜639) | (全体が線形の探索) | 7.0 ms(`HashSet<&str>`) | **≒ 7 ms** |
| M4: `render_text(.., verbose = true)` を 1 回 | **21.2 ms**(20.8〜24.5)。`-v` なしは 1.6 ms | 15.0 ms(ジョブごとに全辺を `filter`) | 1 ms 未満(辺を出どころのジョブで 1 回まとめる) | **≒ 7 ms** |
| M5: 999 回のスキップの記録 | **165 ms**(164〜167) | — | 91 ms(`base.cloned()` をやめ、土台を借りる) | **≒ 91 ms**(実測) |

- 注 1: M1 の L は、`parse_definition` の全体とほぼ同じ時間になった(部品だけを取り出した測り方では、名前の `to_string` の割り当てを含むため)。残りの時間は、同じ形の入力で項目を 1,000 個にしたとき(13.0 ms − L 6.4 ms ≒ 6.6 ms)の 10 倍(入力の大きさに比例)の ≒ 66 ms と見て、H を足した ≒ 72 ms を見積もりとする。参考: 二乗の部分の無いジョブ 3,000 個の定義(103,931 バイト)の `parse_definition` は 24.8 ms。
- 直した後の見積もりと性能のテストの上限は `functional-spec.md` §5.2(D9。決定(ユーザー、2026-11-23))。M4・M5 は改善の絶対量が小さいので、性能のテストは足さず、前後の時間を記録する(D8。決定(ユーザー、2026-11-23))。
- **実物で分かったこと(調査との違い)**:
  - M1 の 10,000 個で 0.6 秒、40,000 個で 8.4 秒(debug)。定義ファイル 1 MiB に収まる約 8 万個では 30 秒を超える見込みで、利用者の入力で目に見える遅さになりうる。M3 も同じ(`pipeline`・`validate` の前の `.env` の記述の一覧)。
  - M2 は重複の誤りがある入力だけで起きる(重複の無い正しい定義では、遷移は `success` と `warning` の 2 つまでで、確認は 2 回の比較で終わる)。それでも 1 MiB の入力で 2 秒かかる。
  - runtime の `load_env_files`(`crates/runtime/src/envfile.rs:43-47`)の `seen: Vec<(PathBuf, Option<usize>)>` の線形の探索も、M3 と同じ記述の数で二乗になる。前回(types-perf-visible §3)で範囲の外とした項目で、初期の記述にも無い。異なる記述ごとにファイルの読み取り(システムコール)が要るので、今回は範囲の外に置く(D11。決定(ユーザー、2026-11-23))。

---

## 段 1(動きを変えない整理)

### 1a. 古くなった説明(コメントと仕様書の文だけを直す)

| # | 今の場所 | 今の文(要旨) | 目標 |
|---|----------|----------------|------|
| 1a-1 | `crates/runtime/src/lib.rs:26` | `run_job` の行の入出力が「`JobDefinition`、分割済みの `CommandLine`、…」 | 「分割済みの [`CommandLine`]([`check_command`] の結果)、足す環境変数、[`InterruptSignal`] → `JobExecution`」(`JobDefinition` を消す。前の intent(types-perf-visible)で `job` を外した。版の番号を書くと、版の更新のときの `grep "0\.9\.7"` の確かめに当たるので、コメントには版の番号を書かない) |
| 1a-2 | `crates/runtime/src/runner.rs:86-88` | 「`job` は、結果の `job_id` と `command` のために使う」 | この 1 文を消し、「v0.9.6 までは `run_job(job, line, env, signal)` で、`job` は使われなかった」を足す(仕様書 rust 02:108 と同じ書き方) |
| 1a-3 | `crates/runtime/src/state/reader.rs:57` | `read_state_raw` は「`watch` が使う」 | 「`watch` と `graph --state` が使う」(lib.rs:31・:86 と同じ) |
| 1a-4 | `crates/runtime/src/state/writer.rs:43-44` | `write_state` の入力が「定義ファイルのパス(…`<定義ファイル名>.state`)、ステート、耐久性の指定」 | 「定義ファイルのパスと run ID(…`<定義ファイル名>.state`、run ID があれば `<定義ファイル名>.<run-id>.state`)、ステート、耐久性の指定」 |
| 1a-5 | `crates/runtime/src/lib.rs:46` | 「`read_state`・`write_state`・`reset_state` は … run ID を取る」 | `read_state_raw` を足す(4 つ) |
| 1a-6 | `crates/cli/src/report/messages.rs:207` | 「最初の文言(最終の確定は U3)」 | 消す(文言は確定済み。前の intent で仕様書 08 に載っている) |
| 1a-7 | `crates/cli/src/step.rs:27` | 「…は、まとめる前の `run.rs`・`pipeline.rs` と同じ」(経緯の文) | 「書く順序、`Durability`、表示の文脈(`StateContext`)は、各関数の説明のとおり」に。経緯は書かない |
| 1a-8 | `crates/cli/src/run.rs:20-21` | 「`run_job` の呼び出しは、リトライを含む共通の処理(…)に置き換わっている」(経緯の文) | 順序の列の `run_job` を `retry::run_job_with_retry`(リトライを含む)に書き換え、経緯の文を消す |
| 1a-9 | `crates/runtime/src/types.rs:37` | 「この Unit は [`InterruptSignal::is_raised`] などで読むだけ」 | 「この Unit は [`InterruptSignal::signal_number`] で読むだけ」(runtime の製品のコードは `signal_number` だけを読む。`is_raised` は cli のテストと `Debug` が使うので残す) |
| 1a-10 | `crates/cli/src/report/mod.rs:23-24` | 「(名前は 64 文字で切り、パスは切らない)」 | 「名前(使い方の誤りの対象、遷移の表示のジョブID など)は `escape_name` で 64 文字で切る。パスと、図・計画のジョブID・コマンドの語は `escape_full`(1g-1)で切らない」 |
| 1a-11 | `crates/cli/src/dry_run.rs:32-33`(守りは `:41-44`) | 「公開の型 `CliCommand` を直接組み立てて呼ばれたときの守り」 | 「`parse_args` を通る限り起こらない、内部の不変条件の守り(位置引数の無い `run`・`pipeline` は `parse_args` が 68 で止める)」。(調査との違いではないが確かめた: `CliCommand` を受け取る関数はすべて `pub(crate)` で、公開の入口は `run_cli`・`run_cli_with` だけ。今の理由の文は誤り) |
| 1a-12 | `crates/cli/src/aux_commands.rs:97-100`(守りの `let … else`) | 理由の文が無い | `run_convert` の説明に同じ「`parse_args` を通る限り起こらない、内部の不変条件の守り」を足す |
| 1a-13 | `docs/spec/rust/05-execution-flow.md:172` | 「`apply_outcome_in`・`apply_skip_in`・`apply_interrupt_in` を使い」 | `apply_interrupt_in` を消す(1b-1) |

- 守りの文(1a-11・1a-12)の守りのコードは残す(初期の記述のとおり)。
- 完了の判定: `grep -n "JobDefinition" crates/runtime/src/lib.rs` が 0 件、`grep -n "結果の \`job_id\`" crates/runtime/src/runner.rs` が 0 件、`grep -n "U3" crates/cli/src/report/messages.rs` が 0 件、`grep -n "まとめる前\|置き換わっている" crates/cli/src/step.rs crates/cli/src/run.rs` が 0 件、`grep -n "直接組み立てて" crates/cli/src` が 0 件、`grep -rn "apply_interrupt_in" crates docs/spec` が 0 件。

### 1b. 使われないもの

#### 1b-1. `apply_interrupt_in`

- 今: `crates/core/src/flow/record.rs:221-225`(`apply_interrupt` が複製して `_in` を呼ぶ)、`:227-232`(`apply_interrupt_in`)。`_in` を呼ぶのは `apply_interrupt` だけ(模擬は使わない。`flow/mod.rs:34` の再公開にも無い)。
- 目標: `apply_interrupt` の本体に戻す(`let mut out = state.clone(); out.resume_job_id = …; out`)。`apply_interrupt_in` を消す。
- 仕様書: rust 05:172(1a-13)。
- 影響するテスト: なし。
- 完了の判定: `grep -rn "apply_interrupt_in" crates docs/spec` が 0 件。

#### 1b-2. `EffectiveEnv.job_id` と `EffectiveEnv::names()`

- 今: `crates/core/src/envfile/mod.rs:346-351`(`job_id: String`)、`:361-367`(`names()`)。作る所 `envfile/merge.rs:45-46`。読む所は製品のコードに無い(計画の名前は `merge_env_names` を使う。前回の 2e)。読むのはテスト `crates/core/tests/env_merge.rs:85`、`:150`、`:167`、`:169`、`:170` だけ。
- 目標: フィールド `job_id` とメソッド `names()` を消す。`EffectiveEnv { entries }` と `additions()` だけ。
- 仕様書: rust `05-execution-flow.md:646`(`{ job_id, entries }` と `names()`)、`:677`(擬似コード)。language-agnostic `05-execution-flow.md:657`、`:688`(論理の形の `job_id` を消し、「名前と出どころの並び」は「項目から取り出せる」に)。
- 影響するテスト: T1(`env_merge.rs`)。
- 完了の判定: `grep -rn "\.names()\|job_id: job.id.clone()" crates/core` が 0 件、`grep -n "pub job_id" crates/core/src/envfile/mod.rs` が 0 件。

#### 1b-3. core のテストの補助の使わない引数(D2。決定(ユーザー、2026-11-23))

| 場所 | 補助 | 呼ぶ所 | 目標 |
|------|------|--------|------|
| `crates/core/src/flow/next_step.rs:227` | `exec(_job_id: &str, code)` | `:239`、`:410` の 2 か所 | `exec(code)` に(引数を外し、呼ぶ所 2 か所を直す) |
| `crates/core/src/flow/record.rs:278` | `exec(_job: &str, code)` | 12 か所 | `exec(code)` |
| `crates/core/src/flow/record.rs:286` | `outcome(_job: &str, c)` | 12 か所 | `outcome(c)` |
| `crates/core/src/flow/record.rs:294` | `step(_job: &str)` | 11 か所 | `step()` |
| `crates/core/tests/retry_flow.rs:39` | `exec(_j: &JobDefinition)` | 1 か所 | `exec()` |
| `crates/core/tests/retry_flow.rs:47` | `exited(_j: &JobDefinition, code)` | 14 か所 | `exited(code)` |
| `crates/core/tests/common/mod.rs:63` | `exited(_job_id: &str, code)` | `flow_sequences.rs` の 17 か所 | **残す**(前回の決定で、意図して残した。説明に「ジョブ ID は読みやすさのため。値は使わない」とある。`flow_sequences.rs` は複数のジョブの並びを読むテストで、どのジョブの結果かが読めることに意味がある) |

- 影響するテスト: T2(src の単体テスト 2 つ)、T3(`retry_flow.rs`)。確かめる中身は変わらない(引数は値を使っていない)。
- 完了の判定: `grep -n "_job\b\|_job_id\b\|_j\b" crates/core/src/flow/record.rs crates/core/src/flow/next_step.rs crates/core/tests/retry_flow.rs` が 0 件。

#### 1b-4. 模擬の分類の二重の計算と、`classify_exit_code`・`ExitCodeClass` の公開範囲

- 今: `crates/core/src/plan/simulate.rs:283-284`(`classify_and_decide` で分類したあと、`classify_exit_code(job, exit_code).classification` でもう一度分類する)。`crates/core/src/flow/classify.rs:92-98`(`pub(crate) struct ExitCodeClass`)、`:105`(`pub(crate) fn classify_exit_code`)。`simulate.rs:31` の import。
- 目標: 模擬は `outcome.classification` を使う。合成した実行は `Exited { exit_code: Some(code), terminated_by_signal: None, interrupted_by: None }` なので、`classify` の手順 5 を通り、分類は必ず `Some`(`None` は `NotStarted` だけ)。`let classification = outcome.classification.unwrap_or(Classification::Error);`(`None` の枝は到達しない。説明に理由を書く)。`classify_exit_code` と `ExitCodeClass` は `classify.rs` の中だけの非公開に。
- 動きが変わらない根拠: `classify` は手順 5 で `classify_exit_code(job, code)` の `classification` をそのまま `Some(..)` にする(`classify.rs:82-87`)。同じ値になる。
- 仕様書: rust `05-execution-flow.md:897`(「結果の語は `classify_exit_code` で決める」があれば「`classify_and_decide` の分類で決める」に)、`simulate.rs:268-271` の説明。
- 影響するテスト: なし(計画のテストがそのまま確かめる)。
- 完了の判定: `grep -rn "classify_exit_code" crates/core/src` が `classify.rs` の中だけ、`grep -n "pub(crate) fn classify_exit_code\|pub(crate) struct ExitCodeClass" crates/core/src` が 0 件。

#### 1b-5. `decide_before_run` の常に真の条件

- 今: `crates/core/src/flow/decide.rs:81`(`if status == JobStatus::Pending && predecessors_settled`)。ここに来る `status` は必ず `Pending`(`Skipped` は 1・2 で、`Success`・`Warning` は 3 で、`Error` は 4 で返している。`JobStatus` は 5 種。`types.rs:266-273`)。
- 目標: `if predecessors_settled`。コメント「5 は 6 に含まれる(ここまでで `status` は pending)」は残す。
- 影響するテスト: なし。

#### 1b-6. cli の `interrupted_signal`

- 今: `crates/cli/src/step.rs:263-273`(`interrupted_signal`)と、`:236-261` の `result_detail`(`Interrupted { signal, .. }` を同じ条件で作る)。呼ぶ所 `:201`(`result_detail`)と `:230`(`interrupted_signal`)。
- 目標: `interrupted_signal` を消し、`:230` は `if let ResultDetail::Interrupted { signal, .. } = detail`(`:201` で作った値。`ResultDetail` は `Copy`)。
- 影響するテスト: なし(表示は同じ。`flow_run.rs`・`flow_pipeline.rs` の中断の表示のテストがそのまま確かめる)。
- 完了の判定: `grep -n "fn interrupted_signal" crates/cli/src` が 0 件。

#### 1b-7. runtime の `command_line` を `check_command` に畳む

- 今: `crates/runtime/src/split.rs:131-150`(`pub(crate) fn command_line`)と `:152-159`(`pub fn check_command` は `command_line(command)` を呼ぶだけ)。`command_line` を呼ぶのは `check_command` だけ。
- 目標: `check_command` の本体を `command_line` の中身にし、説明を 1 つにまとめる。`command_line` を消す。
- 仕様書: rust `07-process-and-signals.md:152`(「`Command::Argv(argv)`(`command_line` 関数)」を「(`check_command` の配列の枝)」に)。
- 影響するテスト: なし。
- 完了の判定: `grep -rn "fn command_line" crates/runtime` が 0 件。

### 1c. 二重の処理

#### 1c-1. 「一時ファイルに書いて置き換える」処理(runtime `temp_file` に共通化)

- 今: `crates/runtime/src/state/writer.rs:76-90`(`create_temp_beside` → `persist` → `drop` → `rename` → 失敗なら一時ファイルを消す)と `crates/runtime/src/file_create.rs:133-156`(同じ順。書き込みは `write_all` と `flush`)。
- 目標: `temp_file.rs` に `pub(crate) fn replace_via_temp(dir: &Path, target_name: &str, target: &Path, write: impl FnOnce(&mut File) -> io::Result<()>) -> Result<(), ReplaceError>` を置く。順は「一時ファイルを新規作成 → `write` → 閉じる(Windows のため)→ `rename` → 失敗なら一時ファイルを消す」。`ReplaceError` は `CreateTemp(io::ErrorKind)`(一時ファイルを作れない)と `WriteOrRename(io::ErrorKind)`(書き込み・同期・置き換えの失敗)の 2 つ。ステートは両方を `WriteFailed` に、`init --force` は今と同じく前者を `classify_kind(kind)`、後者を `PermissionDenied`/`OtherIo` に写す。
- 動きが変わらない根拠: 手順の順と、失敗のときに一時ファイルを消すことは同じ。誤りの種類の写し方は各呼び出し側に残る。
- 仕様書: rust `02-architecture.md:322`(`temp_file.rs` の行に `replace_via_temp` を足す)、`06-state-file.md:455` 付近(`init` の置き換えが同じ関数を使うこと)。
- 影響するテスト: なし(`state_write.rs`、`file_create.rs` がそのまま確かめる)。
- 完了の判定: `grep -n "fs::rename" crates/runtime/src` が `temp_file.rs` の 1 件だけ。

#### 1c-2. ステートの読み込みの段と、上限つきの読み取りの写し

- 今:
  - `crates/runtime/src/state/reader.rs:62-67`(`read_state_raw`)は `read_state_file_raw`(`:69-93`)を呼ぶだけの段。
  - `:200-206`(`parse_state_for`)は `parse_state_raw_for` の結果から完了を空にするだけで、使うのは `inspect_run_state`(`:142`)と単体テストの補助 `parse_state`(`:236-240`)だけ。`inspect_run_state` では、完了のステートは `Ok(None)` でも `Ok(Some)` でも `Inspected::Ours` になる(空のファイルは先に `bytes.is_empty()` で分ける)ので、完了を空にする処理は結果に効かない。
  - `:154-161`(`read_limited`)と `crates/runtime/src/envfile.rs:123-130`(`take(上限 + 1).read_to_end`)が同じ処理の写し。
- 目標:
  - `read_state_file_raw` の中身を `read_state_raw` に入れ、段を 1 つにする。
  - `parse_state_for` を消し、`inspect_run_state` は `parse_state_raw_for` を使う。単体テストの補助 `parse_state` は `parse_state_raw_for(bytes, None).map(|s| s.filter(|s| !s.completed))`(テストの本体は変えない。T4)。
  - 新しいモジュール `crates/runtime/src/bounded_read.rs` に、`read_limited`(`reader.rs` から移す)と、通常のファイルを形 B で上限つきで読む `pub(crate) fn read_regular_file(path: &Path, limit: u64) -> Result<Vec<u8>, ReadFailure>`(`ReadFailure { Missing, Unreadable, TooLarge }`)を置く。4 の段(上限 + 1 バイトまで読み、超えれば `TooLarge`)は、読み手と上限を受け取る `fn read_opened<R: Read>(reader: R, limit: u64) -> Result<Vec<u8>, ReadFailure>` に分け、ファイルを使わない単体テスト N8 で確かめられるようにする。段 1 では `.env`(`envfile.rs` の `read_one` の 1〜4)だけがこれを使う(`.env` はすでに形 B なので動きは変わらない)。ステートが使うのは段 2(2b)。`read_limited` は、ステートの `read_bounded` が段 2 まで呼ぶので、段 1 では `pub(crate)` のまま置き、段 2 のコミット 40 で非公開にする(1f)。
- 守ること: runtime の製品のコードなので、`bounded_read.rs` に `.unwrap()`・`.expect(`・`panic!(` などを書かない(失敗はすべて `ReadFailure`)。
- 仕様書: rust `02-architecture.md:325`(`state/reader.rs` の行の「`read_bounded` の 1 か所」)と、新しい行 `bounded_read.rs`。`06-state-file.md:218` の「`read_bounded` の 1 か所」は段 2 で書き換える。
- 影響するテスト: T4(`reader.rs` の単体テストの補助)。単体テスト `上限_1_つぶんだけ読む`(`reader.rs:461-467`)は `read_limited` と一緒に `bounded_read.rs` の単体テストへ移す(名前と中身は変えない。件数は変わらない。T4)。
- 完了の判定: `grep -rn "fn read_state_file_raw\|fn parse_state_for\b" crates/runtime/src` が 0 件、`grep -rn "read_to_end" crates/runtime/src` が `bounded_read.rs` の 1 件だけ。

#### 1c-3. `build_retry` の所有権と、欠けたときの扱い(D3。決定(ユーザー、2026-11-23))

- 今: `crates/core/src/definition/walk/retry.rs:251-263`(`build_retry(raw: Option<&RawRetry>)`。`exit_codes` を `clone`。値が欠けていれば黙って `None`)。呼ぶ所 `walk/mod.rs:603`(`build_retry(job.retry.as_ref())`)。同じ `build_jobs`(`walk/mod.rs:574-609`)の中で、ほかの項目は値を移し(`job.id?.0`、`job.command?`)、欠けていればジョブを組み立てない(`t.on?`、`t.next?`。全体が `None` になり、`walk` の「定義を組み立てられません」の防御に落ちる)。
- 目標: `build_retry(raw: RawRetry) -> Option<RetryPolicy>`(値を移す。`exit_codes?.0` は複製しない)。呼ぶ所は `retry: match job.retry { None => None, Some(raw) => Some(build_retry(raw)?) }`(書いたのに欠けていれば、遷移と同じくジョブを組み立てない)。
- 動きが変わらない根拠: 値が欠けるのは誤りを記録したときだけで、そのときは `w.errors` が空でないので `build_jobs` まで来ない(`walk/mod.rs:546-548`)。欠けた組は到達しない。
- 仕様書: なし(`02:299` の行はそのまま)。
- 影響するテスト: なし。

#### 1c-4. ごく小さい整理

| # | 今の場所 | 目標 |
|---|----------|------|
| 1c-4a | `crates/core/src/definition/walk/mod.rs:525`、`:533`(同じ理由の文「ジョブ(`job`)が 1 つもありません」が 2 回) | 定数 `NO_JOBS_REASON` にし、2 か所から使う |
| 1c-4b | `crates/core/src/definition/intake.rs:88-91`、`:96-99`(パスのメタデータと `File::open` の失敗の `map_err` の写し) | `let open_error = \|e: std::io::Error\| match e.kind() { … };` を 1 つ作り、2 か所から使う |
| 1c-4c | `crates/core/src/flow/record.rs:120`(`<JobStatus as From<Classification>>::from(classification)`) | `JobStatus::from(classification)` |
| 1c-4d | `crates/core/src/flow/record.rs:108-115`(`apply_outcome_in` の `hint: Option<usize>`)と `:172-178`(`apply_skip_in` の `hint: usize`。公開の関数は 0 を渡す) | 両方を `hint: Option<usize>` にそろえる(`None` は「先頭から探す。公開の関数の今の動き」、`Some(位置)` は模擬)。`apply_skip` は `None`、模擬(`simulate.rs:93`)は `Some(index)`(D4。決定(ユーザー、2026-11-23))。仕様書 rust 05:884 の擬似コードを `apply_skip_in(&mut S, Some(index), …)` に |
| 1c-4e | `crates/converter/src/yaml_write.rs:51-69`(`scalar_or_empty` が空の集まりを `"[]"`・`"{}"` にし、`inline` も同じ枝を持つ) | `scalar_or_empty` は「空でない集まりは `None`、ほかは `Some(inline(value))`」にし、空の集まりの文字列は `inline` の 1 か所に |

- 影響するテスト: なし。

### 1d. cli の `Runtime::run_job` から `job` を外す(Q3: A)

- 判断の背景(ユーザー、2026-11-23): 「リファクタリングで trait `Runtime` の `run_job` の `job` 引数は不要になったが、テストコードの都合(偽物が `job.id` を参照)で残っていた。テストコードの都合で本番コードが曲げられるのは本来あるべきではないので A を選んだ」。
- 今: `crates/cli/src/deps.rs:93-100`(トレイトの `run_job(&self, job: &JobDefinition, line, env, signal)`)、`:175-185`(`SystemRuntime` は `_job` で受け、runtime の `run_job(line, env, signal)` に渡さない)、`crates/cli/src/retry.rs:63`(`ctx.runtime.run_job(job, line, &env.additions(), signal)`)。偽物: `crates/cli/tests/common/mod.rs:315-366`(`ScriptedRuntime`。`job.id` で `run_envs`・`run_lines` の記録、`log` の `run:<id>`・`skip-run:<id>`、台本 `scripts` の引き当て、`during_job`・`after_job` の呼び出しをする)、`:716-728`(`MemRuntime`。`_job` は使わない)。
- 目標:
  - トレイトを `fn run_job(&self, line: &CommandLine, env: &[(&str, &str)], signal: &InterruptSignal) -> JobExecution;`(runtime の `run_job` と同じ形)。`SystemRuntime` は `conductor_runtime::run_job(line, env, signal)` をそのまま呼ぶ(`_job` と説明の「偽物は…」を消す)。`retry.rs:63` は `ctx.runtime.run_job(line, &env.additions(), signal)`。
  - **消えるのはトレイトの `run_job` に渡す引数だけ**。`retry.rs` の中で `job` を使う所(`merge_env` の環境変数の合成(`:52-61`)、`retry_decision(job, &execution, …)` のリトライの判断(`:65-69`)、知らせ `RetryScheduled` のジョブ ID)は今のまま残る。`run_job_with_retry(ctx, session, job, line)` の引数も変わらない。
  - 偽物は、渡された `env` の `CONDUCTOR_JOB_ID` からジョブ ID を取る(テストの側の事情。本番の形は曲げない)。core の `merge_env` は、自動の変数の層(最後の層。`envfile/merge.rs:101`)で `CONDUCTOR_JOB_ID` をいつも足す(`ctx.job_id` = `job.id`。`retry.rs:55`)。予約の接頭辞 `CONDUCTOR_` は定義でも `.env` でも書けない(`ReservedEnvName`)ので、ほかの値で上書きされない。偽物の補助 `fn job_id_of(env: &[(&str, &str)]) -> String` を `common/mod.rs` に置き、`ScriptedRuntime` はこれで取った ID で、今と同じ記録(`run_envs`、`run_lines`、`run:<id>`、`skip-run:<id>`)、台本の引き当て、`during_job`・`after_job` を行う。`MemRuntime` は署名だけを変える。
  - `CONDUCTOR_JOB_ID` が無いとき(本番の流れでは起こらない): 偽物はパニックで止める(`expect("CONDUCTOR_JOB_ID がありません(core の merge_env がいつも足す)")`)。テストの補助なので、契約が崩れたことにすぐ気付けるようにする(D1。決定(ユーザー、2026-11-23))。
- 仕様書: rust `02-architecture.md:206`(流れの図の `Runtime::run_job(job, command, 足す環境, signal)` を `Runtime::run_job(command, 足す環境, signal)` に)、`:255`(`run_job(line, env, signal)`。「v0.9.7 までは `run_job(job, line, env, signal)` で、`job` は偽物がジョブ ID を記録するためだけに受け取った」)、`:269`(`ScriptedRuntime` の行に「ジョブ ID は `env` の `CONDUCTOR_JOB_ID` から取る」)。
- 影響するテスト: T15(`common/mod.rs` の偽物 2 つ)。偽物を使うテスト(`flow_*.rs`、`binary_*` は本物)の期待は変わらない。
- 完了の判定: `grep -rn "_job: &JobDefinition\|job: &JobDefinition" crates/cli/src/deps.rs crates/cli/tests/common/mod.rs` が 0 件、`grep -n "run_job(job" crates/cli/src` が 0 件、`grep -n "merge_env\|retry_decision(job" crates/cli/src/retry.rs` が今と同じ件数。

### 1e. core の公開範囲を狭める

調べ方: 各名前について、`grep -rn "<名前>" crates | grep -v "^crates/core/src"` で、ほかのクレートの src と、すべてのクレートの `tests/` から、そのパスでの参照が無いことを確かめた(§8)。テストは別のクレートなので、テストが使う `pub` は狭められない。

| 今 | 目標 | 外からの参照 |
|----|------|--------------|
| `flow/mod.rs:22-27` の `pub mod classify`、`pub mod decide`、`pub mod next_step`、`pub mod resume`、`pub mod retry` | `mod`(関数と型は `flow/mod.rs:29-36` の `pub use` で今と同じ名前で出る。`flow::classify`(関数)、`flow::RetryDecision` など) | `conductor_core::flow::<下位モジュール>::` は 0 件 |
| `flow/classify.rs:20`、`:23`、`:26` の `pub const EXIT_START_FAILURE`・`EXIT_SIGNAL_BASE`・`EXIT_UNLISTED_ZERO` | `pub(crate)`(`classify.rs` の中だけで使う) | 0 件(cli は自分の終了コードの定数を持つ) |
| `plan/mod.rs:23` の `pub mod simulate` | `mod`(`PlanInput`・`plan_pipeline`・`plan_run` は `:28` の `pub use`) | 0 件 |
| `plan/mod.rs:31`、`:33` の `pub const ASSUME_EXIT_CODE_MIN`・`ASSUME_EXIT_CODE_MAX` | `pub(crate)`(`:54` の `exit_code_in_range` が使う) | 0 件 |
| `envfile/mod.rs:23-24` の `pub mod merge`、`pub mod parse` | `mod`(`merge_env`・`parse_env_file` は `:35`、`:37` の `pub use`) | 0 件 |
| `envfile/mod.rs:58` の `pub const RESERVED_ENV_PREFIX` | `pub(crate)`(`:62-64` の `is_reserved_env_name` が使う) | 0 件 |
| `envfile/mod.rs:44` の `pub const MAX_REPORTED_ERRORS` | `pub(crate)`(`:208` の `summarize_errors` が使う) | 0 件 |
| `envfile/mod.rs:40`、`:42` の `MAX_ENV_FILE_BYTES`、`MAX_ENV_ENTRIES` | **狭めない**(runtime `envfile.rs:25`、cli `messages.rs:318-319`、テスト `env_merge_perf.rs`・`env_parse_perf.rs` が使う) | 使われる |
| `graph/mod.rs:28-30` の `pub mod edges`、`pub mod layout`、`pub mod overlay` | `mod`(関数は `:32-34` の `pub use`) | **1 件: `crates/core/tests/graph_types.rs:92` の `conductor_core::graph::overlay::node_state_of`** → `conductor_core::graph::node_state_of`(再公開の道)に変える(T5) |

- 狭めたあとの確かめ方: (1) `cargo build --workspace --all-targets --locked` と `cargo test --locked --workspace` が通る。(2) `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る(使われなくなった項目があれば `dead_code` で止まる。止まったら、その項目を記録して消す)。(3) §8 の検索をもう一度行い、表の「0 件」が変わらないこと。
- `lib.rs` と各 `mod.rs` の説明の中の、非公開になるモジュールへの内部リンク(`[`flow::classify`]` など)は、`` `flow::classify` `` の書き方にする(前回の 1k と同じ)。
- 仕様書: rust `02-architecture.md:42`(公開モジュールの説明「`flow`(`classify` を含む。定数 `EXIT_START_FAILURE = 127`、… は `flow::classify` にある)」を「`flow`(下位モジュールは crate の中。関数は `flow::` の再公開。終了コードの定数は crate の中)」に。`plan`・`envfile`・`graph` の下位モジュールも同じ。「v0.9.7 までは `flow` の下位モジュール、`plan::simulate`、`envfile::{merge, parse}`、`graph::{edges, layout, overlay}` と定数 `EXIT_*`・`ASSUME_EXIT_CODE_*`・`RESERVED_ENV_PREFIX`・`MAX_REPORTED_ERRORS` も公開だった」)、`:287`(`envfile/mod.rs` の行の定数)、`05:857`(`ASSUME_EXIT_CODE_*` は crate の中の定数)、`10:19`(`EXIT_START_FAILURE` の名前は crate の中の定数として残す)、`03:411`(`MAX_REPORTED_ERRORS` は crate の中の定数)。

### 1f. runtime の `pub(crate)` のうち、モジュールの中でしか使わないもの

| 今 | 目標 | 使う所 |
|----|------|--------|
| `state/writer.rs:66` `pub(crate) fn write_state_file` | 非公開の `fn` | `writer.rs:59` だけ |
| `state/writer.rs:123` `pub(crate) fn persist` | 非公開 | `writer.rs` の中と、その単体テスト |
| `state/writer.rs:138` `pub(crate) fn remove_stale_temps` | 非公開 | 同上 |
| `state/writer.rs:183` `pub(crate) fn serialize` | 非公開 | 同上 |
| `temp_file.rs:47` `pub(crate) fn temp_name` | 非公開 | `temp_file.rs` の中と単体テスト |
| `temp_file.rs:53` `pub(crate) fn create_temp` | 非公開 | 同上(外は `create_temp_beside` を使う。1c-1 のあとは `replace_via_temp` の中だけ) |
| `split.rs:52` `pub(crate) fn split` | 非公開 | `split.rs` の中と単体テスト |
| `state/reader.rs:155` `pub(crate) fn read_limited` | 1c-2 で `bounded_read.rs` に移す(段 1 は `pub(crate)` のまま。ステートの `read_bounded` が段 2 まで呼ぶ)。段 2 で `read_bounded` が無くなったあと(コミット 40)、非公開の `fn` にする | 段 2 のあとは `bounded_read.rs` の中と単体テスト |
| (調査との違い)`runner.rs:35` `pub(crate) trait Waitable`、`:50` `pub(crate) enum Waited`、`:61` `pub(crate) fn wait_loop` | 非公開 | `runner.rs` の中と単体テスト |

- 狭めないもの: `temp_file.rs` の `dir_or_current`(`platform/unix.rs`、`reset_all.rs`、`file_create.rs`、`writer.rs`)、`temp_prefix`(`writer.rs`)、`create_temp_beside`(1c-1 のあとは `temp_file.rs` の中だけになるので、そのとき非公開にする)、`state/reader.rs` の `MAX_STATE_BYTES`(`error.rs:106`)、`FORMAT_VERSION`(`writer.rs`)、`Inspected`・`inspect_run_state`(`reset_all.rs`)、`writer.rs` の `remove_state_file`(`reset_all.rs`)、`error.rs` の `new`・`reason`、`platform` の関数。
- 確かめ方: 1e と同じ(ビルド・テスト・clippy の `dead_code`)。runtime の単体テストは同じモジュールの中なので、非公開にしても使える。
- 仕様書: なし(02 の表は関数の名前だけで、公開範囲を書いていない)。

### 1g. 型と名前

#### 1g-1. cli の `escape_path` の名前(D6。決定(ユーザー、2026-11-23): `escape_full`)

- 今: `crates/cli/src/report/escape.rs:32-35`(`escape_path`。「パス(切らない)」)。使う所 27 か所: `report/mod.rs` 8(パス・理由)、`plan_text.rs` 11(パイプライン名・ジョブID・コマンドの語・環境変数の名前)、`graph_text.rs` 8(ジョブID・コマンド・行き先・スキップのID)。パス以外にも使う。
- 目標: `escape_full`(「切らずに、エスケープだけ」)。説明を「64 文字で切らない(パス、図と計画のジョブID・コマンドの語など)」に。
- 影響するテスト: T16(`tests/escape_table.rs:21` の import と `:69` の呼び出し)、T17(`tests/graph_long_id.rs:17` の説明の文の名前)。テスト名は変えない。
- 仕様書: rust `02-architecture.md:374`、`08-output-and-i18n.md:96`、`:300`、`:322`、`:362`、`:582`、`:583`、`:605`、`:624`、`11-quality-and-build.md:255`(「v0.9.7 までは `escape_path`」を 08:582 に 1 回だけ書く)。language-agnostic には無い。
- 完了の判定: `grep -rn "escape_path" crates docs/spec/rust` が「v0.9.7 までは」の注記の 1 件だけ。

#### 1g-2. run ID の有無の真偽値を `StateContext` に(D5。決定(ユーザー、2026-11-23))

- 今: `crates/cli/src/report/mod.rs:112-117`(`DisplayEvent::ConsistencyMismatch { state_path, kind, with_run_id: bool }`)、`:70-79`(`StateContext::for_read(with_run_id: bool)`)、`:352-365`(`ConsistencyMismatch` の表示は `messages::reset_hint(*with_run_id)`)、`:593-597`(`state_line` は `reset_hint(context == StateContext::ReadRunId)`)、`crates/cli/src/report/messages.rs:408-416`(`reset_hint(with_run_id: bool)`)。作る所 `crates/cli/src/ctx.rs:296`(`StateContext::for_read(run_id.is_some())`)、`:306`(`with_run_id: run_id.is_some()`)、`crates/cli/src/aux_commands.rs:36`(`for_read(run_id.is_some())`)。
- 目標:
  - `StateContext::for_read(run_id: Option<&RunId>) -> Self`(run ID があれば `ReadRunId`、無ければ `Read`)。
  - `DisplayEvent::ConsistencyMismatch { state_path, kind, context: StateContext }`(説明「読んだときの文脈。`Read` か `ReadRunId`」)。
  - `messages::reset_hint(context: StateContext) -> Text`(`ReadRunId` なら `RESET_HINT_RUN_ID`、ほかは `RESET_HINT`)。`state_line` と `ConsistencyMismatch` の表示は `reset_hint(context)`。
  - `ctx.rs:296`・`:306`、`aux_commands.rs:36` は `StateContext::for_read(run_id)`。
- 動きが変わらない根拠: 今の `true`/`false` は、`run_id.is_some()` から `ReadRunId`/`Read` に 1 対 1 で写る。表示の文は同じ。
- 影響するテスト: T18(`report_messages.rs:178` の `with_run_id: false` → `context: StateContext::Read`)。
- 仕様書: rust `02-architecture.md:368`(`StateContext` の行)。08 に `with_run_id` の記述は無い(検索で確かめた)。
- 完了の判定: `grep -rn "with_run_id\|for_read(run_id.is_some())" crates/cli` が 0 件。

#### 1g-3. converter の行番号の 0 の番兵を `Option` に

- 今: `crates/converter/src/raw.rs:94-98`(`RawNode { kind, line: usize }`。「TOML の木では 0」)、`:107-112`(`line_of` が 0 を `None` にする)。作る所: `toml_read.rs:76`、`:100`、`:123`(`line: 0`)、`yaml_read.rs:164-166`(`push_node(kind, line)`)。
- 目標: `RawNode { kind, line: Option<usize> }`。TOML は `None`、YAML は `Some(line)`。`line_of` は `self.nodes[node].line`。
- 影響するテスト: なし(src の単体テストは `RawNode.line` を直接読まない。`grep -n "\.line\b" crates/converter/src` で確かめた。読むのは `ConversionError.line`)。
- 完了の判定: `grep -n "line: 0\|0 => None" crates/converter/src` が 0 件。

#### 1g-4. converter の `ConversionError.reason` を `&'static str` に

- 今: `crates/converter/src/error.rs:105-117`(`pub reason: String`。`new(…, reason: &str)` が `to_string`)。理由はすべて `error.rs` の `reason` モジュールの定数(`:142-178`)か、`resolve.rs:215` の `&'static str`。作る所の引数: `intake.rs:36`、`toml_read.rs:24`、`yaml_read.rs:87`、`resolve.rs:69`、`:180` の `why: &str`。
- 目標: `pub reason: &'static str`。`new(…, reason: &'static str)`、作る所の `why: &'static str`。`DefinitionError`・`EnvFileError`(core)・`StateError`(runtime。すでに `&'static str`)とそろえる。core の `DefinitionError.reason: String` は、`format!` で名前を埋める理由があるので変えない(範囲の外)。
- cli への影響: `report/mod.rs:612` の `escape_path(&e.reason)` は `&&str` から `&str` への自動の参照外しでそのまま通る(1g-1 で `escape_full` に)。テストの組み立て `reason: "reason".into()` は `&'static str` から `&'static str` への `into` で型は通るが、`clippy::useless_conversion`(`-D warnings` で止まる)に当たるので、`.into()` を外す(T19)。`e.reason.contains(..)`、`assert_eq!(e.reason, "…")`、`shown.contains(&e.reason)`(converter の tests)はそのまま通る。
- 仕様書: rust `02-architecture.md:135`、`10-errors-and-exit-codes.md:178`(`reason: &'static str`(固定の文)。「v0.9.7 までは `String`」)。
- 影響するテスト: T19(cli の 4 か所)。
- 完了の判定: `grep -n "pub reason: String" crates/converter/src` が 0 件。

### 1h. 計算量(Q6)

#### 1h-1. 定義の `env` の表の大文字小文字の検査

- 今: `crates/core/src/definition/walk/env.rs:42`(`seen: Vec<String>`)、`:72`(`seen.iter().any(|s| s.eq_ignore_ascii_case(name))`)、`:83`(`seen.push`)。項目の数に上限が無く、二乗(§0.2 M1)。
- 目標: `seen: HashSet<String>`(ASCII の小文字にした名前)。`if !seen.insert(name.to_ascii_lowercase()) { 誤り }`。今と同じく、誤りの項目は `seen` に入れない(`continue` の前に入れない)。
- 動きが変わらない根拠: `eq_ignore_ascii_case` は ASCII の小文字への写しが等しいことと同じ。入れる順と、入れる条件(名前が正しく、予約でないとき)は今と同じ。誤りの位置と文は変わらない。
- 新しいテスト: N1(性能 P1)。

#### 1h-2. 遷移の重複の確認

- 今: `crates/core/src/definition/walk/mod.rs:344`(`on.is_some_and(|on| out.iter().any(|r| r.on == Some(on)))`)。最悪で二乗(§0.2 M2)。
- 目標: 見た種類を、種類ごとの真偽値で持つ(`TransitionOn` は `Success`・`Warning` の 2 種。`types.rs:158-162`。`Hash` を持たないので、公開の型に derive を足さず、`let mut seen = [false; 2];` を `match on { Success => 0, Warning => 1 }` で引く)。`if let Some(on) = on { let slot = &mut seen[..]; if 既に真 { 誤り } 真にする }`。`out.push` は今のまま。
- 動きが変わらない根拠: 今の確認は「前に同じ `on` の遷移があったか」で、`on` が無い(誤り)要素は比べない。集合も、`on` があるときだけ入れて確かめる。誤りの数と位置は同じ。
- 新しいテスト: N2(性能 P2)。

#### 1h-3. `env_file_refs` の重複の確認

- 今: `crates/core/src/envfile/mod.rs:374-390`(`out.contains(r)`。`:378`)。二乗(§0.2 M3)。
- 目標: `let mut seen: HashSet<&str>` で、初めての記述だけを `out` に足す。書いた順を保つ。
- 動きが変わらない根拠: 「同じ記述は 1 つにまとめ、書いた順を保つ」は同じ。
- 新しいテスト: N3(性能 P3)。

#### 1h-4. cli の `-v` の図の辺の絞り込み

- 今: `crates/cli/src/report/graph_text.rs:123-145`(`detail_sentences` がジョブごとに `graph.edges.iter().filter(|e| e.from_job == job.id)`。`:137-141`)。ジョブ数 × 辺の数(§0.2 M4)。
- 目標: 呼ぶ側(レーンの図と辺の一覧の `-v` の詳細)で、`-v` のときだけ、辺を出どころのジョブ ID で 1 回まとめる(`HashMap<&str, Vec<&FlowEdge>>`。各ジョブの中の並びは `graph.edges` の順のまま)。`detail_sentences(lang, job, edges: &[&FlowEdge])` は、まとめた表から引いた辺を受け取る。
- 動きが変わらない根拠: 各ジョブの辺の集まりと並びは、`filter` と同じ(`graph.edges` の順に足す)。
- 性能のテストは足さない(D8。決定(ユーザー、2026-11-23)。直す前でも 21 ms)。前後の時間を記録する。

#### 1h-5. cli の `record_skip` の複製

- 今: `crates/cli/src/step.rs:94-100`(`base.cloned().unwrap_or_else(|| initial_state(..))` で土台を複製し、`apply_skip`(中で、もう一度複製)、`recorded != base`(全体の比較))。
- 目標: 土台を借りる(`let initial; let base = match base { Some(b) => b, None => { initial = initial_state(..); &initial } };`)。`apply_skip(base, …)` と `recorded != *base` は今のまま(公開の `apply_skip` の複製と比較は残る。比較は、書いたかを決めるので要る)。
- 動きが変わらない根拠: 同じ土台に同じ関数を呼び、同じ比較をする。
- 改善は 1 回の複製の分だけ(§0.2 M5。165 → 91 ms。ファイルの書き込みを除く部分)。性能のテストは足さない(D8。決定(ユーザー、2026-11-23))。

### 1i. テスト(補助の重複と、重複したテスト)

#### 1i-1. core の src の単体テストの定義の手組み

- 今: 定義やジョブの構造体を、フィールドを全部書いて組む所(`#[cfg(test)]` より後の `JobDefinition {`・`PipelineDefinition {` の行): `crates/core/src/types.rs:486`(補助 `job(id)`)・`:595`(テスト `ジョブの位置は記述順` の中の定義)、`consistency.rs:71-72`・`:80`(補助 `definition(name, ids)`)、`flow/resume.rs:58-59`・`:66`(`def(n)`)、`flow/decide.rs:93-94`・`:101`(`def(n)`)、`flow/next_step.rs:196`(`job(id, transitions)`。警告コード `[3]`)と `:209-210`(`def(j0_transitions)`)、`flow/classify.rs:128`(`job(success, warning)`)、`flow/record.rs:254`・`:262`(`definition(ids)`)。
- 目標: `crates/core/src/test_support.rs`(`lib.rs` に `#[cfg(test)] mod test_support;`)に、`pub(crate) fn job(id: &str) -> JobDefinition`(コマンド `c`、ほかは空)と `pub(crate) fn definition(name: &str, jobs: Vec<JobDefinition>) -> PipelineDefinition`(版 1、ほかは空)を置く。各モジュールの補助は、名前と引数を今のままにして、本体を `test_support` と構造体の更新の書き方(`JobDefinition { warning_exit_codes: vec![3], transitions, ..job(id) }`)で書く。テストの本体は変えない。
- 確かめたこと: 今の補助のコマンドの語は `echo` と `c` が混ざるが、これらの単体テストはコマンドの中身を読まない(分類・判断・照合・記録の関数はコマンドを使わない)。実装で `grep -n "command" <各モジュールの tests>` をもう一度確かめる。
- 影響するテスト: T6(src の単体テストの補助 7 か所)。件数は変わらない。

#### 1i-2. cli の tests の補助の重複

| # | 今 | 目標 |
|---|----|------|
| 1i-2a | `FakeEnv` の言語分け: `crates/cli/tests/report_messages.rs:34-39`(`env_for(lang)`)と `flow_init.rs:43-46`(同じ `match`) | `common/mod.rs` に `impl FakeEnv { pub fn for_language(lang: Language) -> Self }`。2 か所はそれを使う(`report_messages.rs` の `env_for` は消し、呼ぶ所(33 か所)を `FakeEnv::for_language` に置き換えるか、`env_for` を 1 行の包みで残す。差分の小さい後者にする) |
| 1i-2b | `Deps` を組んで `run_cli_with` を呼ぶ形: `common/mod.rs:633-651`(`Harness::run_os`)、`:816-830`(`run_mem`)、`flow_init.rs:48-57`(`run_with`)、`nfr_robust.rs:136-142`(`utf8_でない引数でパニックしない` の中。確認担当の指摘で見つけた 4 か所目) | `common/mod.rs` に `pub fn run_with_fakes(runtime: &dyn Runtime, sink: &mut FakeSink, env: &FakeEnv, signals: &FakeSignals, args: Vec<OsString>) -> i32` を置き、4 か所はそれを呼ぶ(`nfr_robust.rs` は同じ `sink` を使い回すので、そのまま渡せる。T24)。検索は `grep -rn "Deps {\|run_cli_with(" crates/cli/tests` |
| 1i-2c | `state_error` の補助の二重: `report_messages.rs:41-47`(パス `dir/c.toml.state`、理由 `"reason"`)と `common/mod.rs:257-263`(パスを受け取り、理由 `"テスト用の固定の理由"`) | `common::state_error` の理由を ASCII の `"reason"` にし(`report_messages.rs` の「英語の表示は英語の文だけで日本語の文字を含まない」が、すべてのイベントの表示を ASCII と確かめるため。ほかに理由の文を確かめるテストは無い: `grep -rn "テスト用の固定の理由" crates/cli/tests` は定義の 1 件だけ)、`report_messages.rs` の `state_error(kind)` は `common::state_error(Path::new("dir/c.toml.state"), kind)` の 1 行の包みにする(D12。決定(ユーザー、2026-11-23)) |

- 影響するテスト: T20(`report_messages.rs`)、T21(`flow_init.rs`)、T15(`common/mod.rs`)。件数と期待の文字列は変わらない。

#### 1i-3. runtime の tests の `def`・`put`・`paths` の重複

- 今: `crates/runtime/tests/state_read_raw.rs:24-31`(`def(dir)` = `x.toml`、`put(dir, name, content)`)、`run_id_state.rs:55-61`(同じ `def`・`put`)、`state_read.rs:45-47`(`paths(dir)` = `conductor.toml` と `conductor.toml.state`)、`state_write.rs:28-30`(同じ `paths`)。
- 目標: `crates/runtime/tests/common/mod.rs` に `pub fn def(dir: &Path) -> PathBuf`、`pub fn put(dir: &Path, name: &str, content: &str)`、`pub fn paths(dir: &Path) -> (PathBuf, PathBuf)` を置き、4 つのファイルの写しを消す。`state_read_raw.rs` と `run_id_state.rs` に `mod common;` を足す(今は無い)。`use common::*` をするファイル(`flow_with_state.rs`、`runner_basic.rs` など)の名前と衝突しないことを、実装で `grep -n "fn def\b\|fn put\b\|fn paths\b\|let def\b\|let put\b" crates/runtime/tests/*.rs` で確かめる(衝突すれば `common::def` と書く)。
- 影響するテスト: T9〜T12。件数は変わらない。

#### 1i-4. 深さの境界のテストの重複(Q2。消す方針は D10 で決定(ユーザー、2026-11-23)。個々のテストの削除は `functional-spec.md` §4.6 の対の表で、設計の承認で認めてもらう(承認待ち))

- core: `crates/core/src/definition/depth.rs` の単体テスト `配列の深さ_32_は受理し_33_は拒否`(`:100-106`)と `インラインテーブルの深さ_32_は受理し_33_は拒否`(`:108-113`)は、統合テスト `crates/core/tests/definition_errors.rs:332-348` の `深さ_32_は受け入れ_33_と_1000_は_syntax`(配列とインラインテーブルの 32・33・1000 を、公開の入口で確かめる)と重なる。**単体テストの 2 つを消す**(行番号の確認は、残る単体テスト `深い入力でも行番号を返す` が確かめる)。`テーブルの見出しの深さも数える`(32・33 の見出しは統合テストに無い)、`通常の定義の深さは浅い`、`深い入力でも行番号を返す` は残す。統合テストの `深さの上限と巨大な入れ子`(`:312-330`)は、1000・100,000 の大きな入れ子と見出しを確かめるので残す。
- converter:
  - 単体テスト `crates/converter/src/yaml_read.rs:344-370` の `とても深い入力は_異常終了せずに失敗する`(時間制限の手書き。YAML の流れの配列・流れのマップ・ブロックの配列の 1000・100,000 と、インデントの 1,300 段)は、統合テスト `crates/converter/tests/read_resolve.rs:251-288` の `極端に深い入力は_異常終了せずに文法の誤りで失敗する`(同じ YAML の入力に加え TOML。共通の `with_timeout` を使い、種類が `syntax` であることまで確かめる)に含まれる。**単体テストを消す**。
  - 統合テスト `read_resolve.rs:243-249` の `読み取りの深さの上限_32_と_33`(YAML の流れの配列 31 段で受理、32 段で `syntax`)は、単体テスト `yaml_read.rs:330-342` の `深さ32は受理し_33は文法の誤り`(同じ入力に加え、理由の文とブロックの配列も確かめる)に含まれる。**統合テストを消す**。
  - `toml_read.rs:207-230` の `とても深い入力は_異常終了せずに失敗する`(TOML の閉じない配列、ドットのキー、見出しの 1000・100,000)は、統合テストに無い入力を確かめるので**残す**(時間制限の手書きは、この 1 か所だけになる)。
- 件数: core −2(単体)、converter −2(単体 1、統合 1)。

#### 1i-5. runtime のテストの `run_job` の名残(`job(..).command`)

- 今: 前回 `run_job` から `job` を外したあと、テストがまだジョブの定義を組んでからコマンドを取り出している: `crates/runtime/src/runner.rs:228-245`(単体テストの `job(command)` と `line(&job)`)、`tests/runner_basic.rs:27-28`、`:108`(`let _j = job(..)`。使わない)、`:175-176`、`:221-222`、`:265-266`、`tests/runner_signal.rs:38-39`、`:119-120`、`:156-157`、`:199-200`、`tests/runner_env.rs:29-30`、`tests/nfr_robust.rs:180-181`、`:188-189`。`flow_with_state.rs:205` は本当のジョブの定義からコマンドを取るので対象外。
- 目標: `tests/common/mod.rs` に `pub fn command_line(command: impl Into<Command>) -> CommandLine`(`check_command(&command.into()).expect("分けられる")`)を置き、2 行の組 `let j = job("j", command); let line = check_command(&j.command)…` を `let line = command_line(command);` にする。`runner_basic.rs:108` の使わない `_j` は消す。`:175-176` は `assert!(check_command(&Command::from("'")).is_err())`。単体テストは `fn line(command: &str) -> CommandLine { check_command(&command.into()).expect(..) }` にし、`job` の補助と `JobDefinition` の import を消す。
- 影響するテスト: T7(`runner.rs` の単体テスト)、T8(統合テスト 4 ファイル)。件数と確かめる中身は変わらない。

---

## 段 2(利用者から見える変更と版。Q5: A)

### 2a. converter の読み込みを形 B に

- 今: `crates/converter/src/intake.rs:48-97`(`read_input`): `File::open` を先に行い(`:52`)、失敗したときだけパスのメタデータで通常でないかを見る(`:58-65`)→ 開いたハンドルのメタデータで種別と大きさ(`:68-80`)→ 上限 + 1 バイトまで(`:83-93`)→ UTF-8。書き手のいない名前付きパイプは `File::open` が戻らない(止まる)。
- 目標(定義の `read_definition`(`crates/core/src/definition/intake.rs:71-129`)と同じ形 B):
  1. パスのメタデータ: 無い → `fileMissing`(`指定の場所にファイルがありません`)、ほかの失敗 → `unreadable`(権限なら `読み取りの権限がありません`、ほかは `読み取りに失敗しました`)。通常のファイルでない → `unreadable`(`通常のファイルではありません`)。開かない。
  2. 開く: 無い(1 のあとに消えた)→ `fileMissing`、ほか → `unreadable`(権限・その他の理由)。
  3. 開いたハンドルのメタデータ: 取れない → `unreadable`、通常でない → `unreadable`(`通常のファイルではありません`)、1 MiB 超 → `tooLarge`。
  4. 上限 + 1 バイトまで読む → 5. UTF-8。(今のまま)
- 変わること: 書き手のいない名前付きパイプ(など、開くと止まりうる通常でないもの)は、開かずに `unreadable`(`通常のファイルではありません`)。今は止まる。
- 変わらないこと(種類と理由): 無い(`fileMissing`)、ディレクトリ(`unreadable`、`通常のファイルではありません`。UNIX は今 3' で、Windows は今 2 の「開けないときにメタデータで見分ける」で同じ結果。形 B では 1 で同じ結果)、読む権限のないファイル(大きさにかかわらず `unreadable`、`読み取りの権限がありません`。パスのメタデータは読む権限が無くても取れるので 1 を通り、2 で今と同じ理由)、1 MiB 超(`tooLarge`)、UTF-8 でない(`unreadable`)。宙に浮いたシンボリックリンクは、パスのメタデータが `NotFound` で `fileMissing`(今も `File::open` が `NotFound` で同じ)。
- 確かめ方: 既存の単体テスト(`intake.rs` の `#[cfg(test)]`。無い・ディレクトリ・権限・上限・UTF-8)と統合テスト(`convert_ac.rs`、`nfr_security.rs`)がそのまま通ること(ほかの場合の種類が変わらないことの確かめ)。新しいテスト N4(`tests/` に名前付きパイプ。src のテストに `cfg(unix)` を増やさない。Q4: C)。
- 仕様書(「v0.9.7 までは、先に開いたため、書き手のいない名前付きパイプを渡すと止まった」と注記):
  - rust `09-converter.md:47-62`(第 3 節の表を形 B の順に。1 にパスのメタデータの行を足し、2 の「Windows でディレクトリを開いたときなど」の行を 1 に寄せる)、`:30-34`(第 2 節の順の表の Intake の行)。
  - language-agnostic `09-converter.md:49-66`(同じ表と、「先にファイルを開き、開いたものの情報で…」の段落を「先にパスで種別だけを見て、通常のファイルだけを開き、開いたものの情報で種別と大きさを見る」に)、`:34-37`。
  - language-agnostic `11-quality-and-build.md` の converter の取り込みの受け入れ基準の表に、名前付きパイプの行(「止まらずに `unreadable`」)を足す(実装で AC の番号を確かめる)。

### 2b. ステートの読み込みを形 B に

- 今: `crates/runtime/src/state/reader.rs:108-123`(`read_bounded`): `File::open`(`NotFound` → 空)→ 開いたハンドルの大きさ → 上限 + 1 バイトまで。使う所: `read_state`・`read_state_raw`(`:74`)、`reset --all` の `inspect_run_state`(`:139`)。ステートのパスが書き手のいない名前付きパイプなら、`File::open` が戻らない(`run`・`pipeline`・`watch`・`graph --state` が止まる)。ディレクトリは、UNIX では開けて読み取りで失敗し `unreadable`、Windows では開けずに `unreadable`。
- 目標: `read_bounded` を、1c-2 の `bounded_read::read_regular_file(path, MAX_STATE_BYTES)` に置き換える(形 B: パスのメタデータ → 通常でなければ開かない → 開く → 開いたハンドルの種別と大きさ → 上限 + 1 バイトまで)。写し方: `Missing` → 空(`Ok(None)`)、`Unreadable` → `unreadable`、`TooLarge` → `tooLarge`。`inspect_run_state` も同じ関数(`Missing`・`Unreadable`・`TooLarge` はどれも `Broken`。今と同じ)。
- 名前付きパイプ・ディレクトリのステートの扱い(D7。決定(ユーザー、2026-11-23)): どちらも `unreadable`、理由は今と同じ `開く、または読み取ることができません`(`reason::UNREADABLE`)。ディレクトリの表示は今と変わらない。名前付きパイプは、止まる代わりに、ディレクトリと同じ表示で終了コード 65。`reset`(`reset_state`)は読まずに消すので変わらない。`reset --all` は、候補を `symlink_metadata` で通常のファイルに絞ってから中身を読む(`reset_all.rs:73-81`)ので、名前付きパイプは今も対象外で、変わらない。
- 変わること: ステートのパスが通常のファイルでない(名前付きパイプ、`/dev/zero` などの文字デバイスへのシンボリックリンクなど)とき、開かずに `unreadable`。今は、名前付きパイプは止まり、`/dev/zero` へのリンクは開いて終わりのない読み取りを上限 + 1 バイトで止め `tooLarge`。変わらないこと: 無い・長さ 0(空)、ディレクトリ(`unreadable`、同じ理由)、読む権限のないファイル(`unreadable`。1 MiB 超でも今と同じく `unreadable`)、通常のファイルの 1 MiB 超(`tooLarge`)、ちょうど 1 MiB(受理)、読み取りの途中の失敗(`unreadable`)、以降の検査(5〜14)。
- 期待が変わる既存のテスト: ★ T23(`crates/runtime/tests/state_read.rs:112-121` の `終わりのない入力でも上限で止まる`。`/dev/zero` へのリンクで `TooLarge` を期待する。形 B では `Unreadable`)。推奨: 期待を `Unreadable` に、テスト名を `文字デバイスへのリンクのステートは開かずに_unreadable` に変える(承認待ち。`functional-spec.md` §4.7 に代案)。「終わりのない入力でも上限 + 1 バイトで読み止める」性質は、N8(`bounded_read.rs` の `read_opened` の単体テスト)で保つ。同種のテストの検索: `grep -rn "/dev/\|mkfifo\|create_dir\|symlink(" crates/*/tests crates/*/src`(`/dev/` はこの 1 つだけ。`.env`・converter・定義ファイルのテストには無い)。
- 守ること: runtime の製品のコードなので、パニックしうる書き方を使わない(`nfr_source.rs` が検査する)。
- 確かめ方: 既存の `state_read.rs`(上限ちょうど・超え、読めないファイル(ディレクトリ)、権限のないファイル)、`state_read_raw.rs`、`run_id_state.rs`(`reset --all`)、`nfr_robust.rs` がそのまま通ること。★ T23 は期待を変えて通ること。新しいテスト N5(名前付きパイプと、読む権限のない 1 MiB 超のステート)、N8。
- 仕様書(「v0.9.7 までは、先に開いたため、書き手のいない名前付きパイプのステートで止まった」と注記):
  - rust `06-state-file.md:218-224`(§5.1 の表の 1〜3 を、「1 パスの種別(無い → 空、通常のファイルでない → `unreadable`。開かない)、2 開く、3 開いたハンドルの種別と大きさ、4 上限 + 1 バイトまで」に。以降の番号を 1 つずつ送る)と、`:218` の「`read_bounded` の 1 か所」を「`bounded_read::read_regular_file`(`.env` の読み込みと共有)」に。`02-architecture.md:325` と新しい行。
  - language-agnostic `06-state-file.md:288-293`(同じ表)。
  - `.env` との共有: rust `03-definition-format.md` の `.env` の読み込みの順の表(前回 3b で形 B にした所)に「ステートと同じ関数」を 1 文。

### 2c. 空白だけのコマンド名(文字列)を定義の誤りに

- 今: `crates/runtime/src/split.rs:121-128`(`Some(program) if !program.is_empty()`)。`command = "'  '"`(引用符の中が空白だけ)は、分けると `program = "  "` で `Ok` になり、`run_job` の守り(`runner.rs:102`。`trim().is_empty()`)で `StartFailed`(終了コード 127)。配列の `["  "]` は `check_command` の配列の枝(`:142`。`trim().is_empty()`)で `Empty`(定義の誤り。cli は 64)。core の定義の検査は、文字列の全体が空白だけ(`"  "`)のときだけ `emptyField`(rust 03:202)で、引用符の中の空白は見ない。
- 目標: `Some(program) if !program.trim().is_empty()`(配列の枝と同じ判定)。`'  '`、`"\t"`、`'  ' x` は `Empty`(`コマンド名がありません`)。cli の実行の前の検査が定義の誤りとして 64 で報告する(`validate`、`run`、`pipeline`、`--dry-run`、`graph`)。
- 変わらないこと: 空白を含むが空白だけでない先頭の語(`' my prog'`)はそのまま `program`(配列の `"my prog"` と同じ)。`''`・空・区切りだけは今も `Empty`。誤りの表示に入力の値を含めない(種類の固定の文だけ)。
- 仕様書(「v0.9.7 までは、文字列の先頭の語が空白だけ(`'  '`)なら分けられ、起動の失敗(127)になった」と注記):
  - rust `07-process-and-signals.md:98`(手順 3 の「先頭の語が空文字列なら」を「先頭の語が空文字列、または空白だけ(`trim` して空)なら」)、`:104`、`:144-148`(拒否の例に `'  '` と `'  ' x` の行)、`:445` と `12-rebuild-guide.md:253`(配列と文字列で同じになったこと)。
  - language-agnostic `07-process-and-signals.md:112`、`:118`、`:148`(擬似コード `words[0] == ""` を「`words[0]` を trim して空」)、`:183-186`(例)、`11-quality-and-build.md:374` 付近(AC-097 の入力に `'  '` を足すか、新しい AC の行)。
- 新しいテスト: N6(runtime の分け方)、N7(cli の終了コード 64)。
- 期待が変わる既存のテスト: なし(`grep -rn "'  '" crates` で、空白だけの先頭の語で 127 を確かめるテストが無いことを確かめた)。

### 2d. 版を 0.9.8 に(最後のコミット)

- `git show --stat 7239aa2`(0.9.6 → 0.9.7)と同じ範囲(22 ファイル): `Cargo.lock`(4 クレートの版)、`crates/{cli,converter,core,runtime}/Cargo.toml`、★ T22 `crates/cli/tests/binary_basic.rs`(`"conductor 0.9.7\n"` → `"conductor 0.9.8\n"`)、仕様書の版の表記(rust `00-README.md`、`03`、`04`、`09`、`11`、`12`。language-agnostic `00`、`01`、`02`、`04`、`05`、`06`、`07`、`08`、`11`、`12`)。今の木の `0.9.7` には「v0.9.7 以降」の注記(rust 02・05・08・09・11、language-agnostic 08・09・11 など)が混ざるので、`7239aa2` の差分と同じ行だけを上げる(実装で `git show 7239aa2` の行と突き合わせる)。
- 段 1・2 で足す「v0.9.7 までは…」の注記と、今ある「v0.9.7 以降」「v0.9.6 までは…」は上げない。
- タグとリリースはしない [memory:M6]。

---

## 8. 使った検索のコマンド(テストの変更の一覧の網羅のため)

リポジトリのルートで実行した。結果は §0〜§2 と `functional-spec.md` §4 に反映した。

```sh
# 基準の件数(クレートごとの合計)
cargo test --locked --workspace 2>&1 | awk '/Running |Doc-tests/{n=$0} /^test result/{print n " | " $0}'
# 1a: 古くなった説明
grep -rn "is_raised" crates docs/spec/rust docs/spec/language-agnostic
grep -n "守り\|到達しない\|parse_args" crates/cli/src/dry_run.rs crates/cli/src/aux_commands.rs
grep -rn "pub fn .*CliCommand\|pub(crate) fn .*CliCommand" crates/cli/src
# 1b: 使われないもの
grep -rn "apply_interrupt_in\|apply_skip_in\|apply_outcome_in" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "\.names()\|EffectiveEnv\b" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "e3\.job_id\|env\.job_id" crates
grep -n "exec(\|outcome(\|step(" crates/core/src/flow/record.rs crates/core/src/flow/next_step.rs
grep -n "exec(\|exited(" crates/core/tests/retry_flow.rs
grep -c "exited(\"" crates/core/tests/flow_sequences.rs
grep -rn "classify_exit_code\|ExitCodeClass" crates docs/spec
grep -rn "interrupted_signal\|result_detail" crates/cli/src
grep -rn "command_line\b" crates/runtime docs/spec/rust
# 1c: 二重の処理
grep -rn "pub(crate)" crates/runtime/src
grep -rn "fs::rename\|read_to_end\|read_limited\|read_bounded\|parse_state_for\|read_state_file_raw" crates/runtime/src
grep -rn "build_retry" crates
# 1d: cli の Runtime::run_job
grep -rn "impl Runtime for\|fn run_job\|\.run_job(" crates/cli
grep -rn "job\.id" crates/cli/tests/common/mod.rs
grep -n "CONDUCTOR_JOB_ID" crates/core/src/envfile/merge.rs
grep -rn "run_job" docs/spec/rust/02-architecture.md
# 1e: core の公開範囲(外からの参照)
for p in "flow::classify" "flow::decide" "flow::next_step" "flow::resume" "flow::retry" EXIT_START_FAILURE EXIT_SIGNAL_BASE EXIT_UNLISTED_ZERO "plan::simulate" ASSUME_EXIT_CODE_MIN ASSUME_EXIT_CODE_MAX "envfile::merge" "envfile::parse" RESERVED_ENV_PREFIX MAX_REPORTED_ERRORS MAX_ENV_FILE_BYTES MAX_ENV_ENTRIES "graph::edges" "graph::layout" "graph::overlay"; do echo "== $p"; grep -rn "$p" crates | grep -v "^crates/core/src"; done
grep -rn "conductor_core::\(flow\|plan\|envfile\|graph\)" crates | grep -v "^crates/core/src"
# 1f: runtime の pub(crate) の使う所
for n in write_state_file persist remove_stale_temps serialize temp_name create_temp read_limited "split(" wait_loop Waitable Waited dir_or_current temp_prefix create_temp_beside; do echo "== $n"; grep -rn "$n" crates/runtime/src; done
# 1g: 型と名前
grep -rn "escape_path" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "with_run_id\|for_read\|reset_hint\|StateContext::" crates/cli/src crates/cli/tests docs/spec/rust
grep -n "\.line\b\|line:" crates/converter/src/*.rs
grep -rn "\.reason\b" crates/cli/src crates/cli/tests crates/converter
grep -rn "ConversionError::new\|ConversionError {" crates
# 1h: 計算量の場所
grep -n "seen\|eq_ignore_ascii_case" crates/core/src/definition/walk/env.rs
grep -n "out.iter().any" crates/core/src/definition/walk/mod.rs
grep -n "contains" crates/core/src/envfile/mod.rs
grep -n "filter(|e| e.from_job" crates/cli/src/report/graph_text.rs
grep -n "cloned()" crates/cli/src/step.rs
# 1i: テストの補助
grep -n "fn job\|fn def\b\|fn definition" crates/core/src/*.rs crates/core/src/flow/*.rs
grep -rn "FakeEnv::with(&\[(\"LANG\"\|fn env_for\|fn state_error\|run_cli_with\|Deps {" crates/cli/tests
grep -rn "テスト用の固定の理由" crates/cli/tests
grep -n "^fn \|mod common" crates/runtime/tests/state_read_raw.rs crates/runtime/tests/run_id_state.rs crates/runtime/tests/state_read.rs crates/runtime/tests/state_write.rs
grep -n "job(\|\.command" crates/runtime/tests/runner_basic.rs crates/runtime/tests/runner_env.rs crates/runtime/tests/runner_signal.rs crates/runtime/tests/nfr_robust.rs crates/runtime/tests/flow_with_state.rs
grep -n "fn \|深\|depth\|with_timeout" crates/converter/tests/read_resolve.rs
# 段 2
grep -n "空白だけ\|空白のみ\|先頭の語が空\|Empty" docs/spec/rust/*.md docs/spec/language-agnostic/*.md
grep -rn "'  '\|空白だけ" crates/*/tests crates/runtime/src/split.rs
grep -n "上限 + 1\|名前付きパイプ\|通常のファイル\|開いたハンドル" docs/spec/rust/06-state-file.md docs/spec/language-agnostic/06-state-file.md docs/spec/rust/09-converter.md docs/spec/language-agnostic/09-converter.md
grep -rn "is_dir\|create_dir\|mkfifo" crates/runtime/tests crates/converter/tests crates/cli/tests
grep -rn "/dev/\|mkfifo\|create_dir\|symlink(" crates/*/tests crates/*/src
grep -rn "Endless\|io::repeat\|repeat(b'" crates/*/src crates/*/tests
grep -rn "Deps {\|run_cli_with(" crates/cli/tests
git show --stat 7239aa2
grep -rno "0\.9\.7" crates Cargo.lock docs/spec/rust docs/spec/language-agnostic README.md
```
