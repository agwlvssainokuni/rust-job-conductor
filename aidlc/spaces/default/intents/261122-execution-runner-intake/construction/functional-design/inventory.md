# 変える場所の一覧(Inventory): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

HEAD d4e6b82(v0.9.5)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。

## 0. 基準のテスト件数(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug)

**1,137 通過、1 無視、0 失敗**(2026-11-23 に実行して確かめた)。

この intent に関わるファイルの件数: core の単体(src)108、`retry_flow.rs` 16、`flow_sequences.rs` 9、`flow_perf.rs` 1、`plan_perf.rs` 2、`graph_edges.rs` 9、`definition_errors.rs` 28、runtime の単体(src)71、`runner_basic.rs` 11、`runner_signal.rs` 6、`runner_env.rs` 5、`nfr_robust.rs` 6、`nfr_content.rs` 3、`flow_with_state.rs` 20、cli の `binary_basic.rs` 19、`flow_run.rs` 52、`flow_pipeline.rs` 33、`flow_retry.rs` 18。

---

## 段 1: `JobExecution` の型

### 1a. 今の型

- `crates/core/src/types.rs:211-228`: `JobExecution { job_id, command, started: bool, exit_code: Option<i32>, terminated_by_signal: Option<i32>, start_failure: bool, interrupted: bool, signal_number: Option<i32> }`。derive は `Debug, Clone, PartialEq, Eq` だけ(`serde` は無い)。
- `crates/core/src/types.rs:230-244`: `JobExecution::exited(job, exit_code)`。
- 3 つの真偽値と 3 つの Option で、64 × (Option の組み合わせ) 通りを表せるが、実際に作られるのは下の 1b の 3 つの形だけ。

### 1b. 今のコードが実際に作る値(すべて)

`○` は真、`-` は偽または `None`。`n` は合図の番号(`InterruptSignal::signal_number()` の値)、`c` は終了コード、`s` は終了のシグナル。

| ID | 作る場所 | 場面 | started | exit_code | terminated_by_signal | start_failure | interrupted | signal_number | 新しい型 |
|----|----------|------|---------|-----------|----------------------|---------------|-------------|---------------|----------|
| P1 | `crates/runtime/src/runner.rs:119-125` | `command` を分けられない(防御。cli が先に検査するので通常は来ない) | ○ | - | - | ○ | - | - | `StartFailed`(段 2 で、この場所ごと消える) |
| P2 | `runner.rs:127-132` | 起動の前に合図が立っていた | - | - | - | - | ○ | `Some(n)` | `NotStarted { signal: n }` |
| P3 | `runner.rs:140-146` | `spawn` の失敗 | ○ | - | - | ○ | - | - | `StartFailed` |
| P4 | `runner.rs:155-157` | 終了を観測し、合図は立っていない | ○ | `status.code()` | `termination_signal(status)` | - | - | - | `Exited { exit_code, terminated_by_signal, interrupted_by: None }` |
| P5 | `runner.rs:155-161` | 終了を観測し、その直後に合図が立っていた | ○ | `status.code()` | `termination_signal(status)` | - | ○ | `Some(n)` | `Exited { exit_code, terminated_by_signal, interrupted_by: Some(n) }` |
| P6 | `runner.rs:163-166` | `try_wait` の失敗(合図を転送していても `interrupted` は立てない) | ○ | - | - | ○ | - | - | `StartFailed` |
| P7 | `crates/core/src/types.rs:232-243`(`--dry-run` の `plan/simulate.rs:280` が使う) | 仮定の終了コードで終わった | ○ | `Some(c)` | - | - | - | - | `Exited { exit_code: Some(c), terminated_by_signal: None, interrupted_by: None }` |
| P8 | `crates/cli/src/retry.rs:81-91` | リトライの待機中に合図が立った | - | - | - | - | ○ | `Some(n)` | `NotStarted { signal: n }` |
| P9 | `crates/cli/tests/common/mod.rs:348-358`(`ScriptedRuntime`) | 偽物: 起動前の合図 | - | - | - | - | ○ | `Some(n)` | `NotStarted { signal: n }` |
| P10 | `common/mod.rs:370`、`:374-377` | 偽物: `Exec::Exit(c)`(実行中の合図があれば足す) | ○ | `Some(c)` | - | - | 合図があれば ○ | 合図があれば `Some(n)` | `Exited { exit_code: Some(c), terminated_by_signal: None, interrupted_by }` |
| P11 | `common/mod.rs:371`、`:374-377` | 偽物: `Exec::KilledBy(s)`(同上) | ○ | - | `Some(s)` | - | 同上 | 同上 | `Exited { exit_code: None, terminated_by_signal: Some(s), interrupted_by }` |
| P12 | `common/mod.rs:372` | 偽物: `Exec::StartFailure` | ○ | - | - | ○ | (下の注) | (下の注) | `StartFailed` |
| P13 | `common/mod.rs:739-749`(`MemRuntime`) | 偽物: 終了コード、または起動前の合図 | P7 か P2 と同じ | | | | | | `Exited {..}` か `NotStarted { signal: n }` |

- P12 の注: 今の偽物は、`Exec::StartFailure` のあとでも実行中の合図があれば `interrupted` を立てる(本物は立てない。P3、P6)。この組み合わせを使うテストは無い(`Exec::StartFailure` は `flow_run.rs:515`・`:528`、`flow_retry.rs:159`・`:198`、`flow_pipeline.rs:439` の 5 か所で、どれも `during_job` を使わない)。新しい偽物は本物と同じく、`StartFailed` に合図を足さない(T1)。
- 本物が作る値で、`signal_number` が `None` のまま `interrupted` が立つものは無い。`started = false` で `interrupted` が立たないものも無い。`start_failure` と `interrupted` が同時に立つものも無い。
- `exit_code` と `terminated_by_signal` は、UNIX では片方だけに値があり、Windows では `terminated_by_signal` が常に `None`(`platform/windows.rs:27-29`)。新しい型でも 2 つの Option のまま残す(OS の性質を型で約束しない。`classify` の「終了コードが無い」の分岐も今のまま残る)。
- テストの中の組み立て(1d)には、本物が作らない組み合わせがある(`classify.rs:211-219` など)。新しい型では作れなくなる。T3、T9 で扱う。

### 1c. 今の値を読む場所(製品のコード)

| 場所 | 読み方 | 目標の形 |
|------|--------|----------|
| `crates/core/src/flow/classify.rs:48-101` | `started`→`interrupted`→`start_failure`→`terminated_by_signal`→`exit_code` の順に `if` で判定。`:58-60` の「起動していないのに中断でもない → 127」はあり得ない組み合わせの扱い | `match execution.result` の 3 つの場合で判定する(functional-spec.md §2.1 の対応表)。あり得ない組み合わせの分岐と `:58-60` の説明は消える |
| `crates/core/src/flow/retry.rs:31-43`(`is_retry_target`) | `started && !interrupted && !start_failure && terminated_by_signal.is_none() && exit_code ∈ 一覧` | `Exited { exit_code: Some(c), terminated_by_signal: None, interrupted_by: None }` で、`c` が一覧に含まれる |
| `crates/core/src/flow/record.rs:74`、`:95`(`apply_outcome`) | `job.exit_code = execution.exit_code` | `Exited { exit_code, .. }` ならその値、ほかは `None`(今も `NotStarted`・`StartFailed` に当たる値の `exit_code` は必ず `None`) |
| `crates/cli/src/step.rs:148-153`(`run_step` の `after`) | `if execution.started` | `NotStarted` なら `JobNotStarted`、ほかは `JobExecuted` |
| `crates/cli/src/step.rs:219-223`(`report_execution` の中断の表示) | `if execution.interrupted` → `signal_number.unwrap_or(0)` | `NotStarted { signal }` か `Exited { interrupted_by: Some(signal), .. }` のときだけ、`signal` を表示する |
| `crates/cli/src/step.rs:226-251`(`result_detail`) | `classify` と同じ順の `if` | `match` の 3 つの場合(§2.1 の対応表) |
| `crates/cli/src/retry.rs:70`(`RetryScheduled.last_exit_code`) | `execution.exit_code.unwrap_or_default()` | リトライの判断が `Retry` になるのは `Exited { exit_code: Some(c), .. }` だけなので、その `c`(ほかは 0。今と同じ値) |

説明の文(コードは上と一緒に直す): `crates/runtime/src/lib.rs:26`(「起動失敗は `start_failure` の値」)、`:100`(「`JobExecution` の `start_failure`」)、`crates/runtime/src/runner.rs:98-111`、`crates/core/src/flow/classify.rs:48-61`、`crates/core/src/flow/retry.rs:31-34`、`crates/core/src/flow/record.rs:74`、`crates/cli/src/retry.rs:33-35`、`crates/cli/src/step.rs:148`、`:226`。

### 1d. 値を組み立てる・読むテスト(承認の要る変更は functional-spec.md §4 の T1〜T12)

| ファイル | 行 | 内容 |
|----------|----|------|
| `crates/cli/tests/common/mod.rs` | `:28`(import)、`:339-381`、`:735-749` | 偽物の `run_job`(P9〜P13) |
| `crates/core/tests/flow_sequences.rs` | `:21`、`:46-56`、`:291-300` | 補助 `exited` と、起動しなかった中断の組み立て |
| `crates/core/tests/retry_flow.rs` | `:18`、`:38-56`、`:101-143` | 補助 2 つと、4 つのテストの組み立て |
| `crates/core/tests/flow_perf.rs` | `:49-58` | 組み立て 1 つ |
| `crates/core/tests/plan_perf.rs` | `:91-100` | 同上 |
| `crates/core/tests/graph_edges.rs` | `:158-169` | 補助 `exec` |
| `crates/runtime/tests/nfr_content.rs` | `:138-147` | 組み立て 1 つ |
| `crates/runtime/tests/runner_basic.rs` | `:39-41`、`:73`、`:87-88`、`:98-99`、`:129-131`、`:140`、`:151`、`:172-174`、`:183-184`、`:215-216`、`:234`、`:254` | 結果の項目の読み方 |
| `crates/runtime/tests/runner_signal.rs` | `:67-71`、`:78-80`、`:89-92`、`:163`、`:182` | 同上 |
| `crates/runtime/tests/runner_env.rs` | `:29` | 同上 |
| `crates/runtime/tests/nfr_robust.rs` | `:179`、`:185-186` | 同上 |
| `crates/runtime/tests/flow_with_state.rs` | `:234` | `started: execution.started` |
| `crates/core/src/flow/classify.rs`(単体) | `:120-136`、`:185-189`、`:197-201`、`:211-219`、`:226-231`、`:237-243`、`:253-270` | 補助 2 つと 6 つのテスト |
| `crates/core/src/flow/next_step.rs`(単体) | `:226-236`、`:355-376` | 補助と、4 つの場合の組み立て |
| `crates/core/src/flow/record.rs`(単体) | `:223-233`、`:389-392` | 補助と、起動しなかった中断の組み立て |
| `crates/runtime/src/runner.rs`(単体) | `:277`、`:279`、`:287-289`、`:299-300` | 結果の項目の読み方 |

`signals_order.rs`・`signals_real.rs` の `signal_number()` と、`FakeSignals.started` は別の型の項目で、関係ない。

### 1e. ステートファイルの書式

- `JobExecution` は、ステートファイルに書かれない。derive に `serde` が無く、ステートの書き込み(`crates/runtime/src/state/writer.rs`)は `PipelineState` と `JobState` だけを書く。読み取り(`state/reader.rs:231` の `JOB_KEYS = ["id", "status", "exit_code", "skip_reason"]`)にも無い。
- `JobExecution` からステートに届くのは、`apply_outcome`(`record.rs:95`)が写す `exit_code` の値だけ。1c の表のとおり、写す値は今と同じ(`NotStarted`・`StartFailed` は `None`、`Exited` は観測した値)。よって書式も値も変わらない。

### 1f. 仕様書(段 1)

| 仕様書 | 行 | 今 | 目標 |
|--------|----|----|------|
| `docs/spec/rust/02-architecture.md` | `:79` | `JobExecution` の 8 つの項目 | `job_id`、`command`、`result: ExecutionResult`(3 つの場合)と、4 つの組み立ての関数 |
| `docs/spec/rust/05-execution-flow.md` | `:25-36`(§1.2)、`:98-103`(§2.3 の順 1〜3)、`:298`、`:466-476`(§7.2)、`:526-527`、`:534`、`:895` | 真偽値の言葉 | 3 つの場合の言葉。§2.3 の順 1'(あり得ない組み合わせ)を消す |
| `docs/spec/rust/07-process-and-signals.md` | `:179-193`(§4.1)、`:205`、`:209-219`(§4.3)、`:223-236`(§4.4。順 2 を消す)、`:277`、`:312-313`、`:323`、`:342`、`:358`、`:363` | 同上 | 同上 |
| `docs/spec/rust/11-quality-and-build.md` | `:136` | 「すべて `start_failure`」「`interrupted`」 | 「すべて起動失敗(`StartFailed`)」「起動しない(`NotStarted`)」(段 2 で、分割の誤りの扱いも直す) |
| `docs/spec/rust/12-rebuild-guide.md` | `:50` | 「`start_failure`」 | 「`StartFailed`」 |
| `docs/spec/language-agnostic/05-execution-flow.md`、`07-process-and-signals.md` | (§1.2、§4.1、§4.3、§4.4) | 真偽値の観測モデル | **変えない**(言語に依らない観測のモデルとして正しいまま。D2 の決定。ユーザー、2026-11-23) |

---

## 段 2: `run_job` に分割済みのコマンドを渡す(F)

### 2a. 今の形

- 分割は、実行の前の検査で 1 回: `crates/cli/src/ctx.rs:349-385`(`check_definition`)が全ジョブに `Runtime::check_command` を呼び、`CheckedDefinition.commands`(`:51-53`)、`Checked.commands`(`:59-61`)に、ジョブと同じ並びで持つ。誤りがあれば 64 で終わり、ジョブを 1 つも起動しない。
- 使う場所: `run.rs:83`(`&session.checked.commands[index]`)、`pipeline.rs:54`(`zip(&session.checked.commands)`)が `step::run_step`(`step.rs:117-124`)に渡し、`report_execution`(`:181-203`)が起動の失敗の表示のコマンド名にだけ使う。`dry_run.rs:63-64` も使う。
- それでも `run_job` は自分で分け直す: `crates/runtime/src/runner.rs:119-125`(`command_line(&job.command)`)。リトライのたびに `run_job_with_retry`(`crates/cli/src/retry.rs:36-95`)が `ctx.runtime.run_job(job, &env.additions(), signal)`(`:59`)を呼び、その中で毎回分ける。
- 署名: `crates/runtime/src/runner.rs:112-116` `pub fn run_job(job: &JobDefinition, env: &[(&str, &str)], signal: &InterruptSignal) -> JobExecution`。`crates/cli/src/deps.rs:93-99`(トレイト)、`:174-181`(`SystemRuntime`)。
- `crates/runtime/src/split.rs:136-150` の `command_line` は、`run_job` と `check_command`(`:157-159`)の 2 つが呼ぶ。

### 2b. 目標の形

- `pub fn run_job(job: &JobDefinition, line: &CommandLine, env: &[(&str, &str)], signal: &InterruptSignal) -> JobExecution`。`job` は、結果の `job_id` と `command` のために残す。
- 引数の名前は `line` にする。`Command::new(&line.program)` の書き方を変えない(`crates/runtime/tests/nfr_source.rs:153-156` が、この形以外の `Command::new` を拒む)。
- `run_job` の中に、分ける呼び出し(`command_line`、`split`、`check_command`)を残さない。`split.rs` の `command_line` は `check_command` だけが呼ぶ(関数は残すか、`check_command` に畳む)。
- cli: `Runtime::run_job(&self, job, line: &CommandLine, env, signal)`。`run_job_with_retry(ctx, session, job, line)` が受け取って毎回同じ `line` を渡す。`run_step` はすでに `command: &CommandLine` を持つので、それを渡す(`step.rs:143`)。
- 分けられないときの防御の行き先は functional-spec.md §2.2。

### 2c. 関わるテスト(承認の要る変更は functional-spec.md §4 の T13〜T17)

| ファイル | 行 | 内容 |
|----------|----|------|
| `crates/cli/tests/common/mod.rs` | `:334-339`、`:734-739` | 偽物の `run_job` の引数 |
| `crates/runtime/tests/runner_basic.rs` | `:26`、`:169`、`:208`(呼び出し)、`:84-90`(配列の先頭が空) | 呼び出しの引数と、分けられないコマンドのテスト |
| `crates/runtime/tests/runner_signal.rs` | `:38`、`:105`、`:140`、`:175` | 呼び出しの引数 |
| `crates/runtime/tests/runner_env.rs` | `:28` | 同上 |
| `crates/runtime/tests/flow_with_state.rs` | `:223` | 同上 |
| `crates/runtime/tests/nfr_robust.rs` | `:166-188` | 呼び出しの引数と、分けられないコマンドの場合 |
| `crates/runtime/src/runner.rs`(単体) | `:274-301` | 呼び出しの引数と、分けられないコマンドのテスト |
| `crates/cli/tests/init_template.rs` | `:21`、`:140` | `check_command` を直接呼ぶ(変えない) |

### 2d. 仕様書(段 2。「4 つの章」は rust の 02、05、07、12)

| 仕様書 | 行 | 目標 |
|--------|----|------|
| `docs/spec/rust/02-architecture.md` | `:107`(`run_job` の署名)、`:205`(流れの図)、`:254`(`Runtime` トレイト)、`:317`(`runner.rs`)、`:359`(`retry.rs`) | `run_job(job, line, env, signal)`。分割は `check_definition` の 1 回だけ |
| `docs/spec/rust/05-execution-flow.md` | `:36`(起動手順の「command を分ける」)、`:519`(§7.5 の擬似コード)、`:697`(§14.4 の見出し) | 分割済みの `line` を受け取る。擬似コードは `run_job(job, line, env.additions(), signal)`。「v0.9.5 までは `run_job(job, env, signal)` で、中で分け直した」の注記 |
| `docs/spec/rust/07-process-and-signals.md` | `:167`(§3.6 検査の時点)、`:169`(§3.6 防御)、`:177`(§4.1 の署名)、`:181`(§4.1 の手順 2)、`:217`(§4.3 の `start_failure` の中身)、`:445`(配列の先頭の二重の検査) | 防御は「`run_job` は分けない。`program` が空(空白だけ)なら起動失敗」に。手順 2 を消す |
| `docs/spec/rust/12-rebuild-guide.md` | `:49`(`run_job: 分割 → …`)、`:106`、`:107` | 「分割 →」を消し、引数に `line` を足す |
| `docs/spec/rust/11-quality-and-build.md` | `:136`(`nfr_robust.rs` の説明)、`:214`(`runner_basic.rs` の説明) | 分けられない `command` は `check_command` の誤り、それ以外は起動失敗 |
| `docs/spec/language-agnostic/05-execution-flow.md` | `:42`(手順 1)、`:524`(§7.5) | (D2 の決定で最小限を直す)分割済みのコマンドを受け取る |
| `docs/spec/language-agnostic/07-process-and-signals.md` | `:206`(§3.6 防御)、`:213`(§4.1 の入力)、`:217`(手順 2)、`:253`(§4.3) | 同上 |

---

## 段 3: 定義ファイルの読み込みの順序(B3)

### 3a. 今の順序(core)

`crates/core/src/definition/intake.rs:62-106`(`read_definition`):

1. `std::fs::metadata(path)`(`:65-72`): `NotFound` → `FileMissing`、ほか → `Unreadable`(権限なら「読み取りの権限がありません」、ほかは「読み取りに失敗しました」)。
2. 通常のファイルでない(`:74-80`)→ `Unreadable`「通常のファイルではありません」。
3. **パスのメタデータの大きさ**が 1 MiB を超える(`:81-83`)→ `TooLarge`。
4. `File::open`(`:85-86`)→ 失敗は `Unreadable`(権限など)。
5. 上限 + 1 バイトまで読む(`:87-97`)→ 超えれば `TooLarge`、読み取りの失敗は `Unreadable`。
6. UTF-8 でない(`:99-105`)→ `Unreadable`。

呼ぶ場所: `crates/core/src/definition/mod.rs:44`(`load_definition`)と `crates/cli/src/deps.rs:225-229`(`SystemRuntime::run_id_env`。失敗はすべて `None` に丸める)。

### 3b. converter の順序(揃える先。変えない)

`crates/converter/src/intake.rs:48-98`(`read_input`):

1. `File::open(path)`(`:52-68`): `NotFound` → `FileMissing`。ほかは `Unreadable`。ただし、パスのメタデータで通常のファイルでないと分かれば「通常のファイルではありません」(Windows でディレクトリを開くと権限の誤りになるため。`:59-65`)。
2. **開いたハンドルのメタデータ**(`:70-72`)。取れなければ `Unreadable`。
3. 通常のファイルでない(`:73-79`)→ `Unreadable`。
4. 大きさが 1 MiB を超える(`:80-82`)→ `TooLarge`。
5. 上限 + 1 バイトまで読む(`:85-95`)、UTF-8 の確認(`:96-97`)。

### 3c. 2 つで食い違う場合

| 場合 | core(今) | converter | core(目標) |
|------|-----------|-----------|--------------|
| 読む権限のない、1 MiB を超えるファイル | `TooLarge` | `Unreadable`(権限) | `Unreadable`(権限)【利用者から見える変更】 |
| 書き手のいない名前付きパイプ(FIFO) | `Unreadable`(通常でない。開かない) | 開く時点で待ち続ける | 今と同じ `Unreadable`(開かない。D1 で形 B に決定。functional-spec.md §2.3) |
| そのほか(無い、ディレクトリ、読めて上限超過、UTF-8 でない、権限のない小さなファイル、親のディレクトリに入れない) | 3a のとおり | 同じ種類・同じ理由 | 変わらない |

### 3d. 関わるテスト(今のまま通ることを確かめる。変えない)

- `crates/core/src/definition/intake.rs:193-198`(`存在しないと_file_missing`)、`:200-206`(`ディレクトリは_unreadable`。理由に「通常のファイル」。Windows の CI でも通る必要がある)、`:208-223`(`ちょうど上限は受理し_上限超過は_too_large`)、`:225-233`(`utf8_でない内容は_unreadable`)、`:235-241`(`空のファイルは空の文字列`)、`:243-256`(`権限がないと_unreadable`。UNIX)。
- `crates/core/tests/definition_errors.rs:199-235`(`br1_9_取り込みの誤りはファイルを使って確かめる`。無い・上限超過・ちょうど 1 MiB・UTF-8 でない・ディレクトリ)。
- `crates/core/tests/nfr_security.rs:212-227`(種類の網羅と、表示に値を含めない)。
- 期待が変わる既存のテストは**無い**(読む権限のない大きなファイルを確かめるテストが無いため)。

### 3e. 仕様書(段 3)

| 仕様書 | 行 | 目標 |
|--------|----|------|
| `docs/spec/rust/03-definition-format.md` | `:38-59`(§2.2 の表と注) | 新しい順の表。「v0.9.5 までは、パスのメタデータで大きさを先に見たため、読む権限のない 1 MiB 超のファイルは `tooLarge` だった」の注記。`:59` のシンボリックリンクの文(`std::fs::metadata` を使うため)を、開く操作がリンクを辿る旨に |
| `docs/spec/rust/02-architecture.md` | `:293`(`definition/intake.rs` の説明「メタデータの確認」) | 「開いて、開いたハンドルから大きさを確かめる」 |
| `docs/spec/language-agnostic/03-definition-format.md` | `:53-74`(§3.2 の表と注) | 同じ順の表と、同じ「v0.9.5 までは…」の注記 |
| `docs/spec/rust/09-converter.md`、`language-agnostic/09-converter.md` | — | 変えない |

注記の書き方の手本: `docs/spec/rust/04-cli.md:73`(「値は含めない。v0.9.4 までは、対象は値そのもの」)。

### 3f. 版の更新(0.9.6。手本は `git show --stat ea09b46`)

- `Cargo.toml` × 4(`crates/{core,runtime,converter,cli}/Cargo.toml:3`)、`Cargo.lock` の 4 つの版。
- テスト: `crates/cli/tests/binary_basic.rs:246`(`"conductor 0.9.5\n"`)。期待の文字列の変更(T18)。
- 仕様書: `0.9.5` を含む 16 ファイル(rust: `00-README.md` 2、`03` 1、`04` 3、`09` 1、`11` 6、`12` 2。language-agnostic: `00-README.md` 2、`01` 1、`02` 1、`04` 2、`05` 1、`06` 1、`07` 1、`08` 1、`11` 2、`12` 1)。ただし、段 3 で足す「v0.9.5 までは…」の注記と、段 2 の「v0.9.5 までは…」の注記は、昔の版を指すので**上げない**。
- README には `0.9.5` が無い(確かめた)。
