# 変える場所の一覧(Inventory): 動きを変えない小さな整理とテストの補助

HEAD af018bc(v0.9.6)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。直前の調査の候補と実物が違ったところは「(調査との違い)」と書く。各段の設計の要点、テストの変更の一覧(T…)、足すテスト(N…)、コミットの並びは `functional-spec.md`。

## 0. 基準のテスト件数(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug、HEAD af018bc)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 112 | 261 | 373 |
| runtime | 71 | 125 | 196 |
| converter | 53 | 63 通過 + 1 無視 | 116 通過 + 1 無視 |
| cli | 0 | 457 | 457 |
| 全体 | | | **1,142 通過、1 無視、0 失敗** |

この intent に関わるファイルの件数(基準): cli `args_grammar.rs` 60、`flow_run.rs` 52、`flow_pipeline.rs` 33、`flow_retry.rs` 18、`flow_run_id.rs` 24、`nfr_perf.rs` 5、`nfr_robust.rs` 4、`docs_dry_run.rs` 2、`docs_env.rs` 2、`docs_graph.rs` 8、`docs_init.rs` 2、`docs_retry.rs` 4、`binary_signal.rs` 2、`signals_order.rs` 1、`signals_real.rs` 1、`report_messages.rs` 35。runtime `state_read.rs` 14、`state_write.rs` 10、`nfr_content.rs` 3、`nfr_performance.rs` 1、`flow_with_state.rs` 20。core `definition_errors.rs` 28、`definition_retry.rs` 36、`env_definition.rs` 9、`env_file_parse.rs` 10、`env_merge.rs` 8、`env_merge_perf.rs` 2、`env_types.rs` 5、`flow_perf.rs` 1、`flow_sequences.rs` 9、`graph_edges.rs` 9、`graph_overlay.rs` 5、`plan_pipeline.rs` 10、`plan_run.rs` 5、`plan_types.rs` 3、`retry_flow.rs` 16。converter `convert_roundtrip.rs` 14。

前提(前回と同じ): cli の `ctx`・`step`・`run`・`pipeline`・`dry_run`・`aux_commands`・`graph`・`init`・`retry` は非公開のモジュールで、テストから見えない。`report::messages` は公開だが、テストのファイルは参照していない(`grep -rn "messages::" crates/cli/tests` で 0 件)。runtime の `state`・`file_create`・`platform` は非公開。

---

## 段 1(説明のずれと、使われないもの)

### 1a. core `types.rs` の「生成は runtime」

- 今: `crates/core/src/types.rs:17-19`「`JobExecution`、`JobState`、`PipelineState` は conductor-runtime が生成・保持する」、`:211`「生成は conductor-runtime と、cli のリトライの待機」。
- 実際: `PipelineState`・`JobState` は core の `flow/record.rs`(`initial_state`、`apply_*`。`:37`、`:88`、`:135`、`:175`)が作る。runtime は読み取り(`state/reader.rs:314`)と書き込みだけ。`JobExecution` は runtime の `runner.rs`(`:108`〜`:142`)、cli の `retry.rs:93`(`not_started`)、core の `plan/simulate.rs:280`(`exited`。`--dry-run` の模擬)が作る。
- 目標: `:17-19` を「`PipelineState`・`JobState` の生成と更新は core の `flow::record`。runtime は読み書きだけ。`JobExecution` は runtime の起動の結果、cli のリトライの待機の中断、core の `--dry-run` の模擬が作る」に。`:211` も同じ趣旨に。
- 影響するテスト: なし(コメントだけ)。

### 1b. `sanitize`・`escape_display` の説明と、テスト名「64 文字を超えない」

- 今: `crates/core/src/error.rs:235`、`:255-256`「`max_chars` 文字までに収め、切ったときは末尾に `…` を付ける」。単体テスト `:383-387` の名前 `エスケープ後も_64_文字を超えない` が、実際には `64 + 1`(`…` を含めて 65 文字)を確かめる(`:386`)。`:19`(モジュールの説明)「64 文字までに切って表示する」。
- 実際: `escape_display`(`:236-250`)は、エスケープ後の文字数が上限を超える手前で止め、`…` を足す。本文は上限以下(エスケープの 1 片が収まらなければ、それより短い)、全体は最大で上限 + 1 文字。
- 目標: 3 か所の説明を「本文はエスケープ後の文字数で `max_chars` 文字以内。切ったときは末尾に `…` を足す(全体は最大で `max_chars + 1` 文字)」に。**テスト名は変えない**(Q2)。`:383` のテストの中に「名前の『64 文字』は `…` を除いた本文の文字数」の注記を 1 行足す(`functional-spec.md` の T40。D1: 決定(ユーザー、2026-11-23))。
- 影響するテスト: `error.rs` の単体テスト 1 件のコメントだけ(確かめる中身は変えない)。

### 1c. cli `args.rs` の文法の表が一部だけ

- 今: `crates/cli/src/args.rs:23-32` の表の列は `-v`、`--config`、`--to`、`--run-id`、`--all`、`--remove-on-complete` だけ。`--dry-run`、`--assume`、`--format`、`--state`、`--force` は表の下の箇条書き(`:34-42`)にしかない。
- 実際: 許される場所は `OPTIONS`(`:101-185`)の 11 行。
- 目標: 表に 5 つの列(`--dry-run`、`--assume`、`--format`、`--state`、`--force`)を足し、`OPTIONS` と一致させる(`run`・`pipeline` の `--dry-run` は「可」、`--assume` は「`--dry-run` と一緒のときだけ」、`graph` の `--format`・`--state` は「可」、`init` の `--force` は「可」、ほかは「不可」)。箇条書きは値の検査などの説明として残す。
- 影響するテスト: なし。

### 1d. 仕様書 08 章の拒否の理由の表

- 今: `docs/spec/rust/08-output-and-i18n.md:233-235` に「拒否の理由 `SkipTarget`・`AfterEnd`・`AlreadyExecuted`」の 3 行。
- 実際: 拒否の理由は `RejectReason::OutOfOrder` だけ(前の intent の 4b)。3 つはスキップの理由 `SkipCause` で、表示は同じ章の `:387-389` にある。英語・日本語の文(`skip target` など)は、もうコードに無い(`grep -rn "skip target" crates` で 0 件)。
- 目標: 3 行を消す。
- (調査との違い)同じ種類のずれが `docs/spec/rust/05-execution-flow.md:185`「理由が `AlreadyExecuted` / `OutOfOrder` → 何も変えない」にもある(`apply_skip` は `SkipCause` を受け取るので `OutOfOrder` は渡せない)。「理由が `AlreadyExecuted` → 何も変えない」に直す。
- 影響するテスト: なし。

### 1e. 使われない `Command::program_name`(Q4)

- 今: `crates/core/src/types.rs:79-87`(`impl Command` の `program_name`)。呼ぶのは同じファイルの単体テスト `:627-634` `プログラム名は_line_では先頭の語_argv_では先頭の要素` だけ。仕様書 `docs/spec/rust/02-architecture.md:76`「`program_name() -> Option<&str>`」。
- 検索: `grep -rn "program_name" crates docs/spec/rust docs/spec/language-agnostic README.md` → 上の 3 か所(テストの 4 行を含む)だけ。
- 目標: メソッドと `impl Command` のブロック(ほかのメソッドは無い)を消す。単体テスト 1 件を消す(件数 −1。T38)。仕様書の行は「`Line(String)` / `Argv(Vec<String>)`」だけにする。
- 影響するテスト: core 単体 1 件の削除(T38)。

---

## 段 2(文言の定数の重複)

### 2a. cli `report/messages.rs` の同じ文言

| 今(`crates/cli/src/report/messages.rs`) | 使う場所 | 目標 |
|---|---|---|
| `:506` `SKIPPED_WORD`、`:507` `SKIPPED_LIST`(どちらも `skipped`/`スキップ`) | `report/mod.rs:414`、`:431` | `SKIPPED_LIST` を消し、`:431` も `SKIPPED_WORD` を使う |
| `:531` `NOTE_BY_TRANSITION`、`:532` `NOTE_BY_END` と、`skip_reason`(`:509-515`)の `SkipTarget`・`AfterEnd` の文が同じ | `report/watch.rs:52-53`、`report/mod.rs:415` | 定数 `SKIPPED_BY_TRANSITION`・`SKIPPED_AFTER_END` を置き、`skip_reason` と watch の補足の両方が使う(`NOTE_BY_*` は消す) |
| (調査との違い)`skip_reason` の `AlreadyExecuted`(`:513`)と `plan_disposition` の `AlreadyExecuted`(`:672`)が同じ `already executed`/`実行済み` | `report/mod.rs:415`、`report/plan_text.rs` | 定数 `ALREADY_EXECUTED` を置き、両方が使う |
| `:552` `GRAPH_RESUME_HERE`、`:642` `PLAN_RESUME_HERE` | `graph_text.rs:52`、`plan_text.rs:145` | `RESUME_HERE` の 1 つに |
| `:554` `GRAPH_DETAIL_COMMAND`・`:556` `GRAPH_DETAIL_RETRY`、`:644` `PLAN_DETAIL_COMMAND`・`:646` `PLAN_DETAIL_RETRY`(`:645` `PLAN_DETAIL_ENV`) | `graph_text.rs:124`、`:130`、`plan_text.rs:161`、`:174`、`:183` | `DETAIL_COMMAND`・`DETAIL_RETRY`(と、そろえて `DETAIL_ENV`) |
| `:546` `GRAPH_KIND_SUCCESS`・`:548` `GRAPH_KIND_WARNING` と、`plan_class_word`(`:683-690`)の `Success`・`Warning` | `graph_text.rs:80-81`、`plan_text.rs`(`plan_assumed` の中 `:701`) | `plan_class_word` を `class_word` に改め、graph の `kind_label` も `class_word(lang, Classification::Success)` などを使う。`GRAPH_KIND_BOTH`(`:550`。区切りが `・` で、組み立てると文が変わる)は残す |
| `:527` `WATCH_COMPLETED` と、`plan_end`(`:723-734`)の `Completed` の `completed`/`完了` | `watch.rs:126`、`plan_text.rs:112` | `COMPLETED` の 1 つに(3e で `plan_end` を分けるときに使う)。`plan_next_step` の `done`/`完了`(`:755`)は英語が違うので別のまま(調査との違い: `:755` は重複ではない) |
| `:59` `INTERNAL_ERROR`、`:60` `SIGNAL_SETUP_FAILED` が `ERROR_PREFIX`(`:51`)を使わず、`conductor: error: ` を直書き | `panic_hook.rs:28`、`:32`、`report/mod.rs:311` | マクロ `error_prefix!()`(文字列のリテラルに展開)を置き、`ERROR_PREFIX = error_prefix!()`、`INTERNAL_ERROR = concat!(error_prefix!(), "internal error")` などにする(`&str` の定数どうしは `const` で連結できないため) |

- 仕様書: `docs/spec/rust/08-output-and-i18n.md:315-316` の `SKIPPED_WORD`・`SKIPPED_LIST` の 2 行を 1 行に(「`SKIPPED_WORD`(スキップ行の語と、遷移行の `(…: ID, …)`)」)。ほかの名前は仕様書に無い(`grep -n "RESUME_HERE\|DETAIL_COMMAND\|GRAPH_KIND\|plan_class_word\|WATCH_COMPLETED\|NOTE_BY" docs/spec/rust/*.md` で 0 件)。
- 影響するテスト: なし(出力は 1 文字も変わらない。`report_messages.rs`、`graph_report.rs`、`plan_report.rs`、`flow_aux.rs` などがそのまま確かめる)。

### 2b. core の環境変数の名前の規則の文(3 か所)

- 今: 同じ句「英字または `_` で始まり、英数字と `_` だけ」が 3 か所。
  - `crates/core/src/definition/walk/env.rs:51`「`env` の名前 {} は、英字または `_` で始まり、英数字と `_` だけで書きます」
  - `crates/core/src/definition/walk/mod.rs:285`「`run_id_env` は、英字または `_` で始まり、英数字と `_` だけの名前で書きます」
  - `crates/core/src/envfile/mod.rs:133`「名前は、英字または `_` で始まり、英数字と `_` だけで書きます」
- 規則の本体は `crates/core/src/run_id.rs:74` `is_valid_env_name`。
- 目標: `run_id.rs` の `is_valid_env_name` の隣に、句に展開するマクロ `env_name_rule!()` を置き(`pub(crate) use`)、3 か所を `concat!`(`format!` の第 1 引数にも使える)で組む。文は 1 文字も変えない。
- 影響するテスト: なし(`env_definition.rs`、`env_file_parse.rs`、`definition_errors.rs` の文の検査がそのまま通ること)。

### 2c. `CONDUCTOR_` の直書き

- 今: `crates/core/src/definition/walk/env.rs:62`「…`CONDUCTOR_` で始まるため書けません」、`crates/core/src/envfile/mod.rs:134`「`CONDUCTOR_` で始まる名前は、書けません」。定数 `RESERVED_ENV_PREFIX`(`envfile/mod.rs:48`)がある。
- 目標: `envfile/mod.rs` にマクロ `reserved_env_prefix!()` を置き、`RESERVED_ENV_PREFIX = reserved_env_prefix!()`、2 つの文は `concat!` で組む。
- 影響するテスト: なし。

### 2d. `"format_version"` の直書き(core の walk)

- 今: `crates/core/src/definition/walk/mod.rs:41` `ROOT_KEYS = &["format_version", "pipeline", "job"]`。定数 `version::VERSION_KEY`(`definition/version.rs:25`)がある。
- 目標: `&[VERSION_KEY, "pipeline", "job"]`。
- 範囲: `version.rs:44`・`:53` の理由の文の中の `format_version` と、runtime の `state/reader.rs:196`・`:218` は今回は触らない(文の中の語、別のファイル形式の別のキー)。
- 影響するテスト: なし。

### 2e. 間隔の文「1 秒以上 24 時間以下」と `MIN_SECS`・`MAX_SECS`

- 今: 文は `crates/core/src/definition/walk/retry.rs:197`。定数は `crates/core/src/definition/duration.rs:18`、`:20`(非公開)。説明は `duration.rs:25`、`types.rs:131`。
- 目標: `duration.rs` の定数を `pub(super)` にし、`walk/retry.rs` の文の隣に `const _: () = assert!(MIN_SECS == 1 && MAX_SECS == 24 * 60 * 60);`(文の数と定数をコンパイル時に結ぶ。前の intent の BR1.8 と同じ方法)。文は変えない。
- 影響するテスト: なし。

### 2f. `CONDUCTOR_RUN_ID` の二重(判断して記録)

- 今: `crates/core/src/envfile/merge.rs:46` は、ジョブに渡す自動の変数の名前(出力)。`crates/core/src/run_id.rs:27` `DEFAULT_RUN_ID_ENV` は、`run_id_env` を省略したときに conductor が run ID を**読む**環境変数の名前(入力)。
- 判断: **まとめない**。読む名前は `[pipeline].run_id_env` で変えられるが、渡す名前は常に `CONDUCTOR_RUN_ID`(仕様書 `docs/spec/rust/03`、README の自動の変数の表)。同じ定数にすると、片方を変えたときにもう片方が黙って変わる。`merge.rs:46` の上に「`DEFAULT_RUN_ID_ENV` とは意味が別(こちらは渡す名前で固定)」の注記を 1 行足す(D2: 決定(ユーザー、2026-11-23))。
- 影響するテスト: なし。

---

## 段 3(処理の重複と、モジュールの境界)

### 3a. runtime の一時ファイルの作り方の二重と、`state` の内部への依存

- 今:
  - `crates/runtime/src/state/writer.rs:85-97`(プロセス番号、時刻、`create_temp` と `SEQUENCE` と `temp_name`)と、`crates/runtime/src/file_create.rs:138-149`(同じ形)。
  - 連番の `static SEQUENCE: AtomicU32` が 2 つ(`writer.rs:43`、`file_create.rs:30`)。
  - `file_create.rs:27-28` が `crate::state::dir_or_current` と `crate::state::writer::{create_temp, temp_name}` を使う。`platform/unix.rs:34` が `crate::state::dir_or_current` を使う。`state/reset_all.rs:33`、`:60`、`writer.rs:33`、`:179` も `dir_or_current` を使う。
  - `dir_or_current` の本体は `state/mod.rs:27-35`。`temp_name` は `writer.rs:139-141`、`create_temp` は `:145-159`、`TEMP_ATTEMPTS` は `:38`。後始末の接頭辞 `format!(".{state_name}.tmp.")`(`writer.rs:178`)は `temp_name` の形の写し。
- 目標: 中立のモジュール `crates/runtime/src/temp_file.rs`(非公開。名前は依存の `tempfile` クレートと紛れないように `temp_file`)に、`dir_or_current`、`TEMP_ATTEMPTS`、`temp_prefix(target_name)`、`temp_name`(`temp_prefix` を使う)、`create_temp`、連番 `SEQUENCE`(1 つ)、と、2 か所の共通部分 `create_temp_beside(dir, target_name) -> io::Result<(PathBuf, File)>`(プロセス番号・時刻・連番で名前を作り、`create_temp` を呼ぶ)を置く。`writer.rs`・`file_create.rs` は `create_temp_beside` を呼ぶ。`remove_stale_temps` は `temp_prefix` を使う。`file_create`・`platform/unix.rs`・`state/*` は `crate::temp_file` を使い、`state` の内部を使わない。
- 守ること: 呼び出し側が渡す `dir` は今のまま(`writer.rs` は親が空なら空のまま `join`。前の intent で「一時ファイルのパスの文字列が変わる」ため `dir_or_current` を通さないと決めた。`file_create.rs` は `dir_or_current` を通す)。一時ファイルの名前の形 `.<名前>.tmp.<pid>.<ナノ秒>.<連番>` は変えない。
- 変わること(利用者に見えない): 連番が 1 つになるので、同じプロセスでステートと `init` の出力先の両方を書くとき、連番の値が今と違う(一時ファイルは置き換えか削除で消え、名前は残らない。D5: 連番を 1 つにすると決定(ユーザー、2026-11-23))。
- `lib.rs:56-70` のモジュールの表に `temp_file` の行を足す。仕様書 `docs/spec/rust/02-architecture.md` の 5.2 の表(`:318-329`)に `temp_file.rs` の行を足し、`docs/spec/rust/06-state-file.md:455` の `create_temp` の説明に置き場所を書く。
- 影響するテスト: `writer.rs` の単体テストのうち 4 件(`一時ファイルの名前の形` `:312-318`、`名前が重なったら連番を進める` `:329-344`、`名前が_16_回とも重なれば失敗` `:345-357`、`作成のそれ以外の失敗はすぐに返す` `:358-367`)を、名前と中身を変えずに `temp_file.rs` の単体テストへ移す(T22)。統合テスト `state_write.rs`(一時ファイルの残り方)、`file_create.rs`(8 件)はそのまま通ること。

### 3b. cli の `push_line` の 3 つ

- 今: `crates/cli/src/report/graph_text.rs:172-175`、`crates/cli/src/report/plan_text.rs:235-238`(同じ本体)、`crates/cli/src/report/watch.rs:120-121`(同じ 2 行を直書き)。
- 目標: `report/mod.rs` に `pub(crate) fn push_line(out: &mut String, line: &str)` を 1 つ置き、3 か所が使う。`watch.rs:126-127`(完了の行)は `trim_end` しない今の形を保つ(`COMPLETED` の語は末尾に空白が無いので、`push_line` を使っても出力は同じだが、変える理由が無い)。
- 影響するテスト: なし。

### 3c. `job '<ID>'` の場所の組み立て(4 か所)

- 今: `crates/cli/src/report/mod.rs:360`(`Rejected`)、`:368`(`StartFailure`)、`:377`(`SuccessCodeNotListed`)、`:551`(`command_line` の `"{definition}: job '{}'"`)。
- 目標: `fn job_place(job_id: &str) -> String`(`format!("job '{}'", escape_name(job_id))`)を置き、4 か所が使う(`:551` は `format!("{definition}: {}", job_place(job_id))`)。
- 影響するテスト: なし。

### 3d. reset の案内の選び方の二重

- 今: `crates/cli/src/report/mod.rs:347-351`(`ConsistencyMismatch` の `with_run_id` で `RESET_HINT_RUN_ID` か `RESET_HINT`)と、`:571-580`(`state_line` の `Read`・`ReadRunId` で同じ選び方)。bool から `StateContext` を選ぶのは `crates/cli/src/ctx.rs:450-457` の `read_context`(呼ぶのは `ctx.rs:312`、`aux_commands.rs:21`(import)、`:36`)。
- 目標: `messages.rs` に `pub fn reset_hint(with_run_id: bool) -> Text` を置き、2 か所が使う(`state_line` は `Read | ReadRunId` の 1 つの枝で `reset_hint(context == StateContext::ReadRunId)`)。`read_context` は型の関連関数 `StateContext::for_read(with_run_id: bool)`(`pub(crate)`)に移し、`ctx.rs`・`aux_commands.rs` はそれを使う。`StateContext` の値(5 つ)は変えない(テスト `report_messages.rs:158-161`、`:470`、`:503`、`:516-520` が値を使うため)。
- 影響するテスト: なし。

### 3e. `plan_end` の引数の二重

- 今: `crates/cli/src/report/plan_text.rs:103-106` が `PlanEnd::StoppedAtError(id)` から ID を取り出してエスケープし、`plan_end(lang, &plan.end, &stopped)`(`messages.rs:723`)に `end` と一緒に渡す(`plan_end` は `StoppedAtError(_)` の中身を捨てる)。
- 目標: `plan_text.rs` で 1 回だけ `match &plan.end` し、`Completed` は `COMPLETED`(2a)、`EndReached` は新しい定数 `PLAN_END_REACHED`、`StoppedAtError(id)` は `plan_stopped_at_error(lang, &escape_path(id))` を使う。`plan_end` は消す(messages の関数は「エスケープ済みの値を受け取る」約束を保つ)。
- 影響するテスト: なし(`plan_report.rs`、`flow_dry_run.rs`、`docs_dry_run.rs` の出力の検査がそのまま通ること)。

### 3f. `graph_mermaid.rs` の let-else の二段

- 今: `crates/cli/src/report/graph_mermaid.rs:74-83`(`(Some(from), to) = (…, match …)` の let-else のあと、`let Some(to) = to else { continue };`)。
- 目標: `from` と `to` をそれぞれ `Option` で作り、`let (Some(from), Some(to)) = (from, to) else { continue };` の 1 段に。
- 影響するテスト: なし。

---

## 段 4(到達しない分岐と、効かない処理)

### 4a. core `flow/decide.rs` の規則 5

- 今: `crates/core/src/flow/decide.rs:74-82`(`nothing_started && index == 0` なら `Run`)。説明 `:30`。
- 理由: 規則 1〜4 を通った時点で `status` は `Pending`(`Skipped` は 1、`Success`・`Warning` は 3、`Error` は 4 で返る。`JobStatus` はこの 5 つ。`types.rs:274-281`)。`index == 0` なら規則 6 の `definition.job[..0]` は空で `all` は真なので、規則 6 が同じ `Run` を返す。規則 5 は結果に効かない。
- 目標: `:74-82` のコードを消す。説明の番号(BR3.5 の 1〜7)は残し、`:30` に「規則 6 に含まれる(前のジョブが無いので、規則 6 が `Run` を返す)。コードは規則 6 で判定する」と書く。仕様書 `docs/spec/rust/05-execution-flow.md:219` の表の 5 の行に同じ注記を足す。
- 影響するテスト: なし(`decide.rs` の単体テストの表、`flow_sequences.rs`、`plan_run.rs`、`flow_run.rs` がそのまま通ること)。

### 4b. cli `args.rs` の lexopt の到達しない枝

- 今: `crates/cli/src/args.rs:323-334`(`parser.next()` の誤り。`UnexpectedValue` なら `continue`、ほかは `break`)、`map_lex_error`(`:375-393`)の `MissingValue` の枝(`:377`)と `_` の枝(`:391`)。
- 確かめたこと: `Cargo.lock` の lexopt は 0.3.2。`Parser::next` が返す誤りは `Error::UnexpectedValue` だけ(lexopt 0.3.2 の `src/lib.rs:166-235` の `next` の `Err` は 3 か所とも `UnexpectedValue`。lexopt は MIT のライブラリで、読んでよい)。値の取り出し(`parser.value()`)の誤りは `:350-354` で別に扱っている。
- 目標: `map_lex_error` は `UnexpectedValue` の場合だけを扱う関数にし、ループの誤りの扱いを 2 つの枝にする: `Err(lexopt::Error::UnexpectedValue { option, .. })` は今と同じ(最初の誤りを記録して続ける)、`Err(_)` は 1 つの守りの枝(`InvalidValue` を記録して `break`)に「lexopt 0.3.2 の `next` は `UnexpectedValue` しか返さない。版を上げたときの守り」と注記して残す(`lexopt::Error` は列挙の枝を網羅する必要があり、守りの枝は消せない。D3: 決定(ユーザー、2026-11-23))。`MissingValue` の枝(`:377`)は消す。
- 影響するテスト: なし(`args_grammar.rs` の誤りの検査がそのまま通ること)。

### 4c. cli `report/mod.rs` の到達しない `=> String::new()`

- 今: `crates/cli/src/report/mod.rs:307-316`(`SignalSetupFailed`・`Version` を先に返し、その後で `let lang = self.language();`)、`:471-472`(2 つ目の `match` の網羅のための、到達しない `=> String::new()`)。
- 目標: `match` を 1 つにする。`SignalSetupFailed`・`Version` の枝は本当の文を返し、ほかの枝は言語を必要なときにだけ判定する(`let lang = || self.language();` として各枝で `lang()`。`language()` は `OnceCell` で 1 回だけ判定する。`:265-267`)。`SignalSetupFailed`・`Version` で言語を判定しない今の性質(ロケールを読まない。`flow_run.rs:71`、`flow_pipeline.rs:44` が確かめる)を保つ。
- 影響するテスト: なし。

### 4d. cli `retry.rs` の使われない既定値 0

- 今: `crates/cli/src/retry.rs:72`(`job.retry.as_ref().map_or(0, …)`)、`:75-81`(`ExecutionResult::Exited { exit_code: Some(code), .. } => code, _ => 0`)。
- 理由: `retry_decision` が `Retry` を返すのは、リトライの設定があり(`core/src/flow/retry.rs:78-80`)、結果が `Exited` で終了コードがあるとき(`is_retry_target`。`:35-44`)だけ。0 は表示されない(表示されたら「最大 0 回」の誤った文になる)。
- 目標: `let (Some(policy), ExecutionResult::Exited { exit_code: Some(last_exit_code), .. }) = (job.retry.as_ref(), &execution.result) else { return execution; };` の 1 つの let-else にし、既定値を消す。else は「リトライしない」と同じ結果で、注記を付ける。
- 影響するテスト: なし(`flow_retry.rs` がそのまま通ること)。

### 4e. core `plan/simulate.rs` の `unwrap_or(Classification::Error)` と、`retryable` の写し

- 今: `crates/core/src/plan/simulate.rs:283`(`outcome.classification.unwrap_or(Classification::Error)`。`JobExecution::exited` の結果は必ず分類がある)、`:284-287`(`retryable` を `r.exit_codes.contains(&exit_code)` で求める。`flow/retry.rs:35-44` の `is_retry_target` の写し)。
- 目標:
  - 分類: `flow/classify.rs` の `classify` の手順 5(`:88-96`。成功コード → 成功・0、警告コード → 警告・0、成功コードに無い 0 → エラー・`EXIT_UNLISTED_ZERO`(1)・`conductor_error = true`、そのほか → エラー・その終了コード・`false`)を、次の形で切り出す(D4: 決定(ユーザー、2026-11-23))。
    - 戻り値の型(非公開。`pub(crate)`): `struct ExitCodeClass { classification: Classification, reported_exit_code: i32, conductor_error: bool }`。関数は `pub(crate) fn classify_exit_code(job: &JobDefinition, code: i32) -> ExitCodeClass`。返す終了コードと `conductor_error`(`code == 0` の判定)まで、この 1 か所で決める。
    - `classify` の手順 5 は `let c = classify_exit_code(job, code); outcome(job, Some(c.classification), c.reported_exit_code, c.conductor_error)` だけにする。手順 1〜4 と「起動して終わったのに終了コードがない」(`:85-87`。エラー・1・`true`)は `classify` に残す(終了コードが無いので `classify_exit_code` の入力にならない)。
    - `simulate.rs` は `classify_exit_code(job, exit_code).classification` で `SimulatedResult.classification` を決め、`unwrap_or` を消す。`classify_and_decide` の呼び出し(次の動きと `JobOutcome` のため)は今のまま。
    - 出力と終了コードが変わらない理由: `classify_exit_code` の 4 つの場合は今の `:88-96` の 4 つの枝と同じ順・同じ値。`simulate` の実行は `JobExecution::exited`(シグナル・中断なし、終了コードあり)なので、`classify` は必ず手順 5 に来て、`outcome.classification` と `classify_exit_code(..).classification` は同じ値になる。返す終了コード(`reported_exit_code`)は `classify` が今と同じ値を作る。テストでは `classify.rs` の単体テスト(`終了コードの分類と返す値` など)、`flow_sequences.rs` の `成功コードを書いたジョブの_0_はエラーで_conductor_のエラー`、`plan_*.rs`、`flow_dry_run.rs`、`binary_dry_run.rs` がそのまま通ること。
  - `retryable`: `flow/retry.rs` の `is_retry_target` を `pub(crate)` にし、`simulate.rs` は `job.retry.as_ref().is_some_and(|p| is_retry_target(p, &execution))` にする(`execution` は `JobExecution::exited` で、シグナル・中断なし。結果は今と同じ)。`RetryPolicy` に共有のメソッドを足す形(型の変更)は範囲外。
  - `simulate.rs:269-270` の「分類を 2 回しない」の説明は「`classify_and_decide` の分類と、同じ規則(`classify_exit_code`)で結果の語を決める」に直す。
- 影響するテスト: なし(`plan_*.rs`、`flow_dry_run.rs` がそのまま通ること)。

### 4f. core `walk/retry.rs` の `u32::try_from(m).ok()`

- 今: `crates/core/src/definition/walk/retry.rs:78`、`:80`。`max` は `1..=100`、`multiplier` は `2..=10` の範囲を `retry_integer`(`:141-170`。`Option<i64>` を返す)で確かめ済みで、変換は必ず成功する。
- 目標: `retry_integer` が `RangeInclusive<u32>` を受け取り `Option<u32>` を返す(`integer_of(value).and_then(|v| u32::try_from(v).ok()).filter(|v| range.contains(v))`)。`RawRetry` の組み立ては `max`、`multiplier` をそのまま渡す。理由の文(`{} 以上 {} 以下`)の数は同じ。
- 影響するテスト: なし(`definition_retry.rs` の境界の検査 `br1_9_…`、`br1_12_…` がそのまま通ること)。

### 4g. core `duration.rs` の余分な `n >= 1`

- 今: `crates/core/src/definition/duration.rs:40`(`n >= 1 && (MIN_SECS..=MAX_SECS).contains(&secs)`)。`n == 0` なら `secs == 0` で、`MIN_SECS = 1` の範囲に入らない。
- 目標: `(MIN_SECS..=MAX_SECS).contains(&secs).then_some(secs)`。説明 `:25` の「整数は 1 以上」は結果の性質として残す。
- 影響するテスト: なし(`duration.rs` の単体テストと `definition_retry.rs` の `br1_10_…` がそのまま通ること)。

---

## 段 5(文字列での比較を、型と定数に)

### 5a. コマンド名の逆引き

- 今: `crates/cli/src/args.rs:426-436`(`match first.as_str() { "run" => …, … }`)。名前の表は `crates/cli/src/types.rs:72-83` の `CommandKind::name`。
- 目標: `types.rs` に `pub const ALL: [CommandKind; 8]` と `pub fn from_name(name: &str) -> Option<Self>`(`ALL` から `name()` が一致するものを探す)を足す。`args.rs` は `CommandKind::from_name(&first).ok_or_else(…)`。
- 仕様書 `docs/spec/rust/02-architecture.md:157` の `CommandKind` に `ALL`、`from_name` を足す。
- 影響するテスト: なし。新しいテスト N1。

### 5b. 位置引数の `matches!`

- 今: `crates/cli/src/args.rs:493-501`(`wants_positional`、`allows_operand` を否定の `matches!` で決める。新しいコマンドを足すと、黙って「必須」になる)。
- 目標: 非公開の `enum Operand { Required, Optional, Forbidden }` と、網羅の `match` で決める `fn operand(kind: CommandKind) -> Operand` を `args.rs` に置く(`Run`・`Pipeline`・`Convert` は必須、`Validate`・`Graph`・`Init` は省略可、`Watch`・`Reset` は不可)。`:504-512` はそれで分ける。誤りの種類と順序は変えない。
- 影響するテスト: なし。

### 5c. 誤りの対象の直書き

- 今: `crates/cli/src/args.rs:523` `"--to"`、`:547` `"--assume"`、`:569` `"--format"`、`:578` `"--run-id"`、`crates/cli/src/types.rs:43` `"--run-id"`(`RunIdArg::parse`)。同じ綴りは `Opt::subject`(`args.rs:214-221`)が表から作る(`:439-445` などはすでに使う)。
- 目標: `args.rs` の 4 か所は `Opt::To.subject()` などにする。`types.rs:43` は、`types.rs` に定数 `pub(crate) const RUN_ID_OPTION: &str = "--run-id";` を置いて使う(`RunIdArg::parse` は公開の関数で、テスト `run_id_arg.rs`、`args_grammar.rs:632` が呼ぶので、形は変えない)。
- 定数と表を結ぶ: `args.rs` の今の表の検査(`:185-192` の `const _: () = { … }`)の隣に、`RUN_ID_OPTION` が `"--"` と `OPTIONS[Opt::RunId as usize].long` をつないだものと等しいことを、バイトを比べる小さな `const fn` で `const _: () = assert!(…)` として確かめる(コンパイル時に結ぶので、テストの件数は変わらない。`const fn` のループは MSRV 1.100 で使え、同じファイルに前例がある)。実行時にも `run_id_arg.rs:40`(`--run-id`)と、`args_grammar.rs` の `--run-id` の誤りの検査が同じ綴りを確かめる。
- 採らない形: `types.rs` から `crate::args::Opt::RunId.subject()` を呼ぶ(`Opt` を `pub(crate)` にする)。`args` は `types` を使うので、`types` から `args` を使うと 2 つのモジュールが互いを参照する。
- 影響するテスト: なし(`run_id_arg.rs:40` が対象 `--run-id` を確かめる)。

### 5d. runtime `state/reader.rs` の reset --all の判定が理由の文の一致

- 今: `crates/runtime/src/state/reader.rs:146`(`Err(e) if e.reason == reason::RUN_ID_MISMATCH => Inspected::NotOurs`)。理由の文は `error.rs:122`、作るのは `reader.rs:262`。
- 目標: 非公開の `ParseFailure`(`:160-171`)に原因の列挙 `cause: FailureCause`(`RunIdMismatch` と `Other`)を足し、`:262` は `run_id_mismatch()` で作る。`:146` は `Err(ParseFailure { cause: FailureCause::RunIdMismatch, .. })` で見分ける。`ParseFailure` を作る `corrupt`(`:166-171`)と `:201-204` は `Other`。公開の `StateError` は変えない。
- 影響するテスト: なし(`reader.rs` の単体テストと、`run_id_state.rs`、`flow_run_id.rs` の `reset --all` の検査がそのまま通ること)。

---

## 段 6(walk の整理)

### 6a. `success_declared` の真偽値

- 今: `crates/core/src/definition/walk/mod.rs:81-83`(`success: Option<(Vec<i32>, usize)>` と `success_declared: bool`)、`:369-370`(初期値)、`:386-387`(読み取り)、`:462`(警告との重なり)、`:587`(組み立て)、`crates/core/src/definition/walk/retry.rs:211-215`(リトライとの重なり)。
- 目標: 非公開の `enum SuccessCodes { Absent, Invalid, Valid(Vec<i32>, usize) }` の 1 つのフィールドに。`Absent` は書いていない(省略時は `[0]`)、`Invalid` は書いて誤り(重なりを調べない)、`Valid` は正しい値と位置。
- 影響するテスト: なし(`definition_retry.rs` の `br1_7_…` 4 件、`definition_errors.rs` の `br1_4_…` がそのまま通ること)。

### 6b. `build_jobs` の clone

- 今: `crates/core/src/definition/walk/mod.rs:560-595`(`&[RawJob]` を受け取り、`id`、`command`、`warning`、`success`、遷移先、`env`、`env_file` を clone)。呼ぶのは `:534`。
- 目標: `build_jobs(raw: Vec<RawJob>)` で所有権を受け取り、`into_iter` で値を移す(`raw_jobs` は `:534` より後で使わない)。
- 影響するテスト: なし。

### 6c. `integer_of` の重複

- 今: `crates/core/src/definition/walk/mod.rs:482-488`(`i64::from_str_radix(i.as_str(), i.radix())`)と、`crates/core/src/definition/version.rs:56` の同じ式。
- 目標: `definition/mod.rs` に `pub(crate) fn integer_value(int: &DeInteger<'_>) -> Option<i64>` を置き、両方が使う(`version.rs` は「整数でない」と「範囲外」を別の誤りにするため、`DeValue::Integer` の let-else は今のまま残す)。`DeInteger` の公開の道(`toml::de::DeInteger`)は実装で確かめる。
- 影響するテスト: なし。

### 6d. `depth.rs` の `Table`/`Array` の同じ形の枝

- 今: `crates/core/src/definition/depth.rs:44-66`(2 つの枝が、深さの検査と子の走査を同じ形で持つ)。
- 目標: 子の並びを受け取る補助 `fn walk_children<'a>(start, depth, children: impl Iterator<Item = &'a Spanned<DeValue<'a>>>) -> Option<usize>` を置き、2 つの枝はそれを呼ぶ(深さの検査は子の走査より先、空のテーブル・配列でも検査する、の順序を保つ)。
- 影響するテスト: なし(`definition_errors.rs` の `深さの上限と巨大な入れ子`、`深さ_32_は受け入れ_33_と_1000_は_syntax`、`depth.rs` の単体テストがそのまま通ること)。

---

## 段 7(テストの補助)

この段の「変えるテストの場所」は、すべて `functional-spec.md` §4 の T 表に、ファイル・行・数・理由で挙げる。ここでは補助ごとに、今の写しの場所と、まとめる先を書く。

### 7a. cli: README の読み取りと必須語句

- 今: `fn readme()` が 5 つ(`docs_dry_run.rs:19-25`、`docs_env.rs:21-26`、`docs_graph.rs:19-25`、`docs_init.rs:22-28`、`docs_retry.rs:21-27`。本体は同じ)。docs-check の 7 語の配列が 5 つ(`docs_dry_run.rs:40-48`、`docs_env.rs:59-67`、`docs_graph.rs:41-49`、`docs_init.rs:43-51`、`docs_retry.rs:92-100`。語と順は CI の `docs-check`(`.github/workflows/ci.yml:124`)と同じ)。見出しの後のコードブロックを取る関数が 2 つ(`docs_env.rs:31-40` `first_block_after_heading(text, fence)`、`docs_retry.rs:32-40` `first_toml_block_after_heading(text)`。`fence = "toml"` のとき本体は同じ)。
- 必須語句を確かめるループも 5 つのファイルで同じ形(`for word in [..] { assert!(text.contains(word), "<文> {word} がない"); }`)。失敗のときの文は 2 種類: 各機能の語句は「README に {word} がない」(`docs_dry_run.rs:37`、`docs_env.rs:56`、`docs_graph.rs:37`、`docs_init.rs:40`、`docs_retry.rs:52`)、docs-check の 7 語は「docs-check の語句 {word} がない」(`docs_dry_run.rs:49`、`docs_env.rs:68`、`docs_graph.rs:50`、`docs_init.rs:52`)。**`docs_retry.rs:101` だけ、docs-check の 7 語にも「README に {word} がない」を使う**。
- まとめる先: 新しいモジュール `crates/cli/tests/docs_support/mod.rs`(`bin_support` と同じ形。`#![allow(dead_code)]`)に次を置く。
  - `readme()`、`DOCS_CHECK_WORDS: [&str; 7]`、`block_after_heading(text, heading, fence)`。
  - `assert_readme_has(text, words)`: 各語が無ければ「README に {word} がない」で失敗する。
  - `assert_docs_check_words(text)`: `DOCS_CHECK_WORDS` の各語が無ければ「docs-check の語句 {word} がない」で失敗する。
- 失敗の文は変えない: 4 つのファイルの docs-check の検査は `assert_docs_check_words(&text)`、`docs_retry.rs:92-102` は `assert_readme_has(&text, &DOCS_CHECK_WORDS)` にして、今の「README に {word} がない」を保つ。各機能の語句の配列は各ファイルに残し、`assert_readme_has(&text, &[..])` に渡す。失敗の文は期待の文字列ではないが、この形なら 1 文字も変わらないので、新しい承認は要らない。
- 範囲の外: `docs_graph.rs:183-192` の行ごとの検査(`line.contains`、文「{word} がない」)、`docs_graph.rs:39` の見出しの検査と `docs_retry.rs:54`・`:59-67` の個別の検査(形が違う)。
- 範囲: `docs_dry_run.rs:56-60`、`docs_graph.rs:57-61`、`docs_init.rs:59-63` の ```` ```text ```` の取り出しは、節の見出しの接頭辞で探し、閉じの探し方(`"```"`)も違うので、今回はまとめない(まとめると、例の中に ```` ``` ```` がある場合の結果が変わりうる)。
- 失敗のときだけ出る `expect` の文は、`block_after_heading` の中で共通の文「見出し `<見出し>` がある」になる(今は `docs_env.rs:32`「環境変数の設定の小節がある」、`docs_retry.rs:33`「リトライの設定の小節がある」)。テストの合否と期待する文字列は変わらない。`functional-spec.md` §10 の「既定で進めるもの」で、承認待ち。

### 7b. cli: `status_of` の 3 つ

- 今: `flow_run.rs:52-60`、`flow_pipeline.rs:23-31`(同じ。`&Harness` から読み、無ければ失敗)、`flow_retry.rs:74-80`(`&PipelineState` から読み、無ければ `Pending`)。
- まとめる先: `crates/cli/tests/common/mod.rs` に `pub fn status_in(state: &PipelineState, id: &str) -> JobStatus`(無ければ失敗)と、`pub fn status_of(h: &Harness, id: &str) -> JobStatus`(`status_in(&h.state().expect(…), id)`)。`flow_retry.rs` の 5 か所は `status_in` を呼ぶ。
- `flow_retry.rs` の「無ければ `Pending`」は「無ければ失敗」になる。5 か所の対象のジョブ `a` は、いつもステートにある(`initial_state` が定義の全ジョブを入れる)ので、結果は同じ。

### 7c. cli: `send(signal)` の 3 つ

- 今: `binary_signal.rs:31-37`(`send(signal, pid)`)、`signals_order.rs:29-35`、`signals_real.rs:29-36`(自分のプロセスへ。`send(signal)`)。
- まとめる先: 新しいモジュール `crates/cli/tests/signal_support/mod.rs` に `send(signal, pid)` と `send_self(signal)`(`send(signal, std::process::id())`)。3 つのファイルは `#![cfg(unix)]` のまま。

### 7d. cli: `args_grammar.rs` の `MapEnv`

- 今: `crates/cli/tests/args_grammar.rs:611-629`(`MapEnv` と `env_of`)。使うのは `:633` だけ。同じ働きの `FakeEnv::with` が `common/mod.rs:57-66` にある。
- まとめる先(推奨 A): `args_grammar.rs` に `mod common;` を足し(glob の `use` はしない。名前の衝突を避ける)、`:633` は `&common::FakeEnv::with(env)`。
- A の副作用: `args_grammar.rs` のテストのバイナリが、`common/mod.rs` の全体(偽の `Runtime`・`Sink`・`Signals`、`Harness`、`MemRuntime`、定義の組み立て。850 行)を一緒にコンパイルする。使わない部分の警告は、`common/mod.rs:16` の `#![allow(dead_code)]` が抑えるので出ない(`mod common;` を書くほかの 13 のテストのファイルと同じ扱い)。動きとテストの件数は変わらない。コンパイルの時間がわずかに増える。
- 別の道 B: `FakeEnv` を `common/fake_env.rs` に分け、`common/mod.rs` は `mod fake_env; pub use fake_env::FakeEnv;`、`args_grammar.rs` は `#[path = "common/fake_env.rs"] mod fake_env;` で `FakeEnv` だけを取り込む。大きな `common` を取り込まないが、`#[path]` の属性と、`common` のファイルの分割(変えるファイルが 1 つ増える)が要る。
- 比べて: A はほかのテストのファイルと同じ書き方で、読む人が迷わない。B は取り込む量が少ないが、このリポジトリに `#[path]` の前例が無い。推奨は A(承認待ちの T14 に含める)。

### 7e. cli の偽物(`crates/cli/tests/common/mod.rs`)

| 今 | 目標 | 影響するテスト |
|---|---|---|
| `MemRuntime.writes`(`:681`、初期値 `:698`、加算 `:782`)を、どのテストも読まない(`grep -rn "\.writes\b" crates/cli/tests` は `common/mod.rs` の 2 行だけ) | フィールドと 2 行を消す | なし |
| `MemRuntime::run_job`(`:734-747`)が、合図が立っていて起動しないときも、台本(`exit_codes`)を先に 1 つ消費する。`ScriptedRuntime::run_job`(`:334-383`)は、合図のときは台本を消費しない | 合図を先に見て、起動するときだけ `pop_front` する(`ScriptedRuntime` と同じ順) | `MemRuntime` を使う 4 ファイル(`flow_retry.rs:354-370`、`nfr_perf.rs:161-163`、`nfr_robust.rs:83-87`・`:109-125`・`:192-213`・`:218-220`、`flow_run_id.rs:350-355`)。どれも `run_mem` の `FakeSignals::new(SignalMode::Noop)` で、合図が立つのは `flow_retry.rs:365` の待機の中断だけ。中断のあと `run_job` が呼ばれても、確かめるのは終了コード 130 と待ち時間 `[30]` で、残りの台本の数は確かめない。結果は変わらない |
| `ScriptedRuntime::run_job` が分割済みのコマンド `_line`(`:337`)を捨てる | フィールド `run_lines: RefCell<Vec<RunLine>>`(`RunLine = (String, CommandLine)`。ジョブ ID と、受け取った `CommandLine` の写し)を足し、起動のたびに記録する(合図で起動しないときも、`run_envs` と同じく記録する) | なし(新しいテスト N2、N3 が使う) |
| 手書きの `impl Default for ScriptedRuntime`(`:225-250`) | `#[derive(Default)]`。そのため `WriteFailure`(`:171-178`)に `#[derive(Default)]` と `#[default] Never` を足す(`SystemRuntime` は `Default` を持つ。`deps.rs:151`) | なし |

- 仕様書 `docs/spec/rust/02-architecture.md:262-271` の偽物の表に、`ScriptedRuntime` の `run_lines` を足す。

### 7f. runtime: ステートの包みと `definition()`

- 今: run ID なしの包み(`read_state`・`reset_state`・`initial_state`・`write_state` を `None` で呼ぶ)が 5 ファイル: `state_read.rs:23`(別名の import)、`:25-28`、`state_write.rs:25-28`、`:30-39`、`nfr_content.rs:25-29`、`:31-43`、`nfr_performance.rs:26-30`、`:32-41`、`flow_with_state.rs:34-38`、`:41-57`。`definition()` の直書き(`PipelineDefinition` の全フィールド)が `nfr_content.rs:117-131`、`nfr_performance.rs:46-64`、`src/state/consistency.rs:30-50`(7g で core へ移る)。
- まとめる先: `crates/runtime/tests/common/mod.rs` に `pub mod without_run_id { read_state, reset_state, initial_state, write_state }`(glob の `use common::*` をしている `flow_with_state.rs` で、別名の import と衝突しないように、下位のモジュールにする)と、`pub fn definition(name: &str, jobs: Vec<JobDefinition>) -> PipelineDefinition`。

### 7g. runtime の `src/state/consistency.rs` を core へ

- 今: `crates/runtime/src/state/consistency.rs`(110 行。`#[cfg(test)]` だけ。core の `check_consistency` の単体テスト 4 件: `一致すれば_ok_で_パスの違いは見ない`、`パイプライン名の違い`、`ジョブidの追加_削除_改名`、`並びだけの違い`)。`state/mod.rs:15-18`(説明)、`:20`(`pub(crate) mod consistency;`)、`lib.rs:66`(表の行)。
- まとめる先: `crates/core/src/consistency.rs` の末尾に `#[cfg(test)] mod tests` として、4 件を名前と中身を変えずに移す(`use conductor_core::…` を `use crate::…` に)。runtime のファイル、モジュールの宣言、表の行を消し、`state/mod.rs` の説明を直す。
- 仕様書: `docs/spec/rust/02-architecture.md:285`(core の `consistency.rs` の行に「単体テストも持つ」)、`:326`(runtime の `state/consistency.rs` の行を消す)、`docs/spec/rust/12-rebuild-guide.md:58`(「単体テストは runtime の `state/consistency.rs`」を「core の `consistency.rs`」に)。
- 件数: runtime 単体 −4、core 単体 +4(全体は変わらない)。

### 7h. `JobExecution` の手組みを、組み立ての関数へ

| 場所 | 今 | 目標 |
|---|---|---|
| `crates/runtime/tests/nfr_content.rs:139-146` | 全フィールドの手組み(`Exited { Some(3), None, None }`) | `JobExecution::exited(&d.job[0], 3)` |
| `crates/core/tests/flow_sequences.rs:46-56` | 補助 `exited(job: &str, code)`(`command: "c"` の手組み) | core の `tests/common` の `exited(job_id, code)`(`JobExecution::exited(&common::job(job_id), code)`)を使う。呼ぶ側 18 か所は変えない |
| `crates/core/tests/flow_sequences.rs:300-304` | `NotStarted { signal: 15 }` の手組み | `JobExecution::not_started(job, 15)` |
| `crates/core/tests/retry_flow.rs:39-49` | 補助 `exec(j)`(`Exited { None, None, None }`) | `JobExecution::new(j, ExecutionResult::Exited { exit_code: None, … })` |
| `crates/core/tests/retry_flow.rs:51-60` | 補助 `exited(j, code)` | `JobExecution::exited(j, code)` |
| `crates/core/tests/retry_flow.rs:109-112`、`:119-126`、`:133-140`、`:147-150` | `JobExecution { result: …, ..exec(&j) }` | `JobExecution::start_failed(&j)`、`JobExecution::new(&j, …)`(シグナル 9)、`JobExecution::new(&j, …)`(中断 2)、`JobExecution::not_started(&j, 15)` |
| `crates/core/tests/graph_edges.rs:158-168`、`:176` | 補助 `exec(job_id, code)` | 補助を消し、`:176` は `JobExecution::exited(job, code)`(`job` は `&JobDefinition`) |
| `crates/core/src/flow/classify.rs:116-122`(単体テスト) | 補助 `exec(result)` の手組み | `JobExecution::new(&job(None, Vec::new()), result)`(同じファイルの `job` の補助。ID `a`) |
| `crates/core/src/flow/next_step.rs:226-236`(単体テスト) | 補助 `exec(job_id, code)` の手組み | `JobExecution::exited(&job(job_id, vec![]), code)` |
| `crates/core/src/flow/next_step.rs:355-378`(単体テスト) | 4 つの `JobExecution { result: …, ..exec("j1", 0) }` | `JobExecution::start_failed(j)`、`JobExecution::new(j, …)`、`JobExecution::new(j, …)`、`JobExecution::not_started(j, 2)`(`j` は `&d.job[1]`、ID `j1`) |
| `crates/core/src/flow/record.rs:227-237`(単体テスト) | 補助 `exec(job, code: Option<i32>)` の手組み(`command: "echo"`) | `JobExecution::new(&definition(&[job]).job[0], ExecutionResult::Exited { exit_code: code, … })` |

- 変わること: 手組みの `command`(`"c"`・`"echo"`)が、ジョブの定義の `command` になる。`JobExecution.job_id`・`command` を読む本体の処理は無い(`grep -rn "execution\.job_id\|execution\.command" crates` で 0 件。分類・判断・記録は `JobDefinition` と `result` だけを見る)ので、結果は変わらない。
- 検索で拾った手組みの全体: `grep -rn "JobExecution {" crates` → 上の表と、型の定義(`core/src/types.rs`)、`impl` の行(`types.rs:239`)だけ。

### 7i. core の `tests/common` を作る

- 今の写し:
  - `run`・`kinds`: `definition_errors.rs:22-28`・`:35-37`、`definition_retry.rs:21-31`(同じ)。`env_definition.rs:30-37` の `kinds`(`parse(...).unwrap_err()` 経由。誤りの種類の並びは同じ。受理された入力では今は失敗するが、そう呼ぶテストは無い)。
  - `median`: `env_merge_perf.rs:39-43`、`flow_perf.rs:33-37`(同じ)。
  - `SECRET`(`"s3cr3t-VALUE-do-not-print"`): `env_file_parse.rs:19`、`env_merge.rs:22`、`env_types.rs:24`、`plan_types.rs:22`、(調査との違い)`env_definition.rs:19` も同じ値で 5 ファイル。cli(4 ファイル)と runtime(`env_loader.rs:22`)にも同じ値があるが、クレートをまたいだ `tests` の共有はできないので範囲外。
  - ステートの補助: `plan_run.rs:53-74` `state_with(statuses)`、`plan_pipeline.rs:58-79` `state(jobs, resume, completed)`、`graph_overlay.rs:23-44` `state(statuses, resume, completed)`。違いは、`definition_key`・`definition_path`(`"t.toml"` と `"k"`・`"conductor.toml"`)と、`Skipped` の `skip_reason`(`graph_overlay.rs` だけ `ByTransition`)。
  - `JobExecution` の補助(7h)。
- まとめる先: `crates/core/tests/common/mod.rs`(`#![allow(dead_code)]`)に `SECRET`、`run`、`kinds`、`median`、`state(jobs, resume, completed)`(キーとパスは `"t.toml"`、`Skipped` は `ByTransition`)、`job(id) -> JobDefinition`、`exited(job_id, code) -> JobExecution`。
- 同じ値になることの確かめ: `plan_pipeline.rs` で `Skipped` を渡すのは `:182-191`(直後に `ByTransition` を書く)と `:203-213`(直後に `ByEnd` を書く)だけで、書いた後の値は今と同じ。`definition_key`・`definition_path` は計画と図の処理が読まない(`grep -rn "definition_key\|definition_path" crates/core/src/plan crates/core/src/graph` で 0 件)。

### 7j. converter: `TargetFormat` の反対の形式

- 今: `crates/converter/src/lib.rs:177-180`(`testing::read_tree`)、`crates/converter/tests/convert_roundtrip.rs:37-40`(`assert_roundtrip`)。
- まとめる先: `crates/converter/src/types.rs` の `TargetFormat` に `pub fn opposite(self) -> Self`。2 か所が使う。新しいテスト N5。

---

## 8. 使った検索のコマンド(テストの変更の一覧の網羅のため)

リポジトリのルートで実行した。結果は §1〜§7 と `functional-spec.md` §4 に反映した。

```sh
# 基準の件数
cargo test --locked --workspace 2>&1 | awk '/Running |Doc-tests/{n=$0} /^test result/{print n " | " $0}'
# 文言の定数
grep -rn "SKIPPED_WORD\|SKIPPED_LIST\|NOTE_BY_TRANSITION\|NOTE_BY_END\|skip_reason\|GRAPH_RESUME_HERE\|PLAN_RESUME_HERE\|GRAPH_DETAIL_\|PLAN_DETAIL_\|GRAPH_KIND_\|plan_class_word\|WATCH_COMPLETED\|plan_end\b\|INTERNAL_ERROR\|SIGNAL_SETUP_FAILED\|ERROR_PREFIX" crates docs/spec/rust
grep -rn "messages::\|report::messages" crates/cli/tests
grep -rn "CONDUCTOR_\|RESERVED_ENV_PREFIX\|format_version\|VERSION_KEY\|MIN_SECS\|MAX_SECS\|24 時間\|1 秒以上\|DEFAULT_RUN_ID_ENV" crates/core/src
# 一時ファイルと境界
grep -rn "create_temp\|temp_name\|dir_or_current\|static .*Atomic" crates/runtime/src
# cli の表示の補助と reset の案内
grep -rn "job '{}'" crates/cli/src
grep -rn "RESET_HINT\|read_context\|with_run_id\|ReadRunId" crates/cli/src
grep -rn "StateContext::\|with_run_id" crates/cli/tests
grep -n "fn \|push_line" crates/cli/src/report/graph_text.rs crates/cli/src/report/plan_text.rs crates/cli/src/report/watch.rs
# 文字列での比較
grep -rn '"--[a-z]' crates/cli/src
grep -rn "RunIdArg::parse\|CommandKind::\|\.name()" crates/cli/src crates/cli/tests
grep -rn "RUN_ID_MISMATCH" crates
# lexopt の誤り(Cargo.lock の版と、0.3.2 の Parser::next の Err)
grep -A3 'name = "lexopt"' Cargo.lock
grep -n "Err(\|pub fn next\|Error::" ~/.cargo/registry/src/*/lexopt-0.3.2/src/lib.rs
# 使われない program_name
grep -rn "program_name" crates docs/spec/rust docs/spec/language-agnostic README.md
# 生成元
grep -rn "PipelineState {\|JobExecution::\(new\|exited\|not_started\|start_failed\)\|fn initial_state\|JobState::pending" crates/core/src crates/runtime/src crates/cli/src
# walk
grep -n "success_declared\|\.success\b\|job.success\|raw.success" crates/core/src/definition/walk/*.rs
# テストの補助(cli)
grep -n 'fn readme\|readme()\|"SHA-256",\|"convert",\|first_block_after_heading\|first_toml_block_after_heading\|HEADING\|^use std::path' crates/cli/tests/docs_*.rs
grep -rn '"Gatekeeper"\|fn readme\|README.md' crates/cli/tests
grep -n "fn status_of\|status_of(" crates/cli/tests/*.rs
grep -n "fn send\|send(" crates/cli/tests/binary_signal.rs crates/cli/tests/signals_order.rs crates/cli/tests/signals_real.rs
grep -n "MapEnv\|env_of" crates/cli/tests/args_grammar.rs
grep -rn 'MemRuntime\|\.writes\b\|exit_codes\.borrow\|run_mem' crates/cli/tests
grep -rn 'interrupt_wait\|SignalMode::' crates/cli/tests
grep -rn "ScriptedRuntime {\|ScriptedRuntime::default\|WriteFailure::" crates/cli/tests
# テストの補助(runtime)
grep -n "_with(" crates/runtime/tests/state_read.rs crates/runtime/tests/state_write.rs crates/runtime/tests/nfr_content.rs crates/runtime/tests/nfr_performance.rs crates/runtime/tests/flow_with_state.rs
grep -rn "fn definition()" crates/runtime
grep -n "#\[test\]" -A1 crates/runtime/src/state/writer.rs
# テストの補助(core)
grep -rn 'const SECRET' crates
grep -rn 'fn median\|fn kinds\|fn run(text' crates/core/tests
grep -n "state_with(\| state(&\|= state(\|(state(" crates/core/tests/plan_run.rs crates/core/tests/plan_pipeline.rs crates/core/tests/graph_overlay.rs
grep -n "Skipped" crates/core/tests/plan_run.rs crates/core/tests/plan_pipeline.rs crates/core/tests/graph_overlay.rs
grep -rn "definition_key\|definition_path" crates/core/src/plan crates/core/src/graph
grep -rn "JobExecution {" crates
grep -n "exited(\|exec(" crates/core/tests/flow_sequences.rs crates/core/tests/retry_flow.rs crates/core/tests/graph_edges.rs crates/core/src/flow/classify.rs crates/core/src/flow/next_step.rs crates/core/src/flow/record.rs
grep -rn "execution\.job_id\|execution\.command" crates
# 仕様書
grep -n "create_temp\|temp_name\|dir_or_current\|state::consistency\|state/consistency\|consistency\.rs\|success_declared\|push_line\|is_retry_target\|CommandKind\|SKIPPED_LIST" docs/spec/rust/*.md docs/spec/language-agnostic/*.md
```
