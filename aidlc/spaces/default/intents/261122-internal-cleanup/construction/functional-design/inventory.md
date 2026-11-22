# 変える場所の一覧(Inventory): 動きを変えない整理のリファクタリング

HEAD 38a8c97(v0.9.4)の時点で、読んで作った一覧。コードは変えていない。

## 0. 基準のテスト件数(`cargo test --locked --workspace`、macOS、rustc 1.100.0)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 105 | 264(28 ファイル) | 369 |
| runtime | 71 | 124(15 ファイル) | 195 |
| converter | 53 | 63 通過 + 1 無視(4 ファイル) | 116 通過 + 1 無視 |
| cli | 0 | 443(39 ファイル) | 443 |
| 全体 | | | 1,123 通過、1 無視、0 失敗 |

graph のテストの件数: `graph_lane_polish` 14、`graph_branch_lanes` 9、`graph_crossing_lanes` 9、`graph_mixed_shapes` 6、`graph_layout_unify` 10、`graph_drawable` 16(この 6 ファイルで 64)。

cli の前提: `ctx`・`graph`・`aux_commands`・`dry_run`・`run`・`pipeline`・`step`・`retry` は非公開のモジュールで、テストから見えない。テストが見る内部の型は、公開の `report::DisplayEvent` だけ。偽物の `check_command`(`tests/common/mod.rs:293`、`:714`)は本物に委ね、呼び出しの回数を数えない。

## 段 1(cli)

### 1a. run ID の決定、ステートの読み取りと照合

- 今: `crates/cli/src/ctx.rs:155-166`(run ID の決定、誤りは 68)、`:171-188`(`read_state` → 誤りの表示 → `check_consistency`、食い違いは 65)。`crates/cli/src/graph.rs:80-90`、`:92-109` は同じ処理の写し(違いは `read_state_raw` で読むことだけ)。`crates/cli/src/aux_commands.rs:41-45` は `ctx::read_context`(`ctx.rs:366-372`)を使わずに同じ判断を書く。
- 目標(`impl Ctx`): `definition_run_id(&mut self, cmd, definition) -> Result<Option<RunId>, i32>` と、`enum StateRead { Live, Raw }`、`read_checked_state(&mut self, path, definition, run_id, read: StateRead) -> Result<Option<PipelineState>, i32>`。`prepare_checked` は「run ID → `.env` → ステート」の順で呼び、`graph::read_overlay` は「run ID → ステート(Raw)」。watch は `read_context` を使う。`aux_run_id`(`ctx.rs:378-392`。定義をゆるく読む規則)は残す。
- **順序を崩さない**: prepare は「定義 → ジョブ → `--assume` → run ID → `.env` → ステート」、graph は「定義 → command → `.env`(警告を含む)→ run ID → ステート」。run ID とステートは別の関数にする。`read_state` と `read_state_raw` は偽物のログ(`"read"`、`"read-raw"`)で区別されるので、呼び分けを保つ。

### 1b. コマンドの分割を定義の検査の 1 回に

- 今: `ctx.rs:236-245`(`check_definition` で全ジョブを分割し、結果を捨てる)、`dry_run.rs:53-76`(もう一度分割。誤りの枝 `:64-76` は通らない)、`ctx.rs:320-325`(`report_execution` で再分割。`unwrap_or_else` は通らない)。`conductor_runtime::run_job` の中の分割(`runner.rs:119`)は `Runtime` トレイトの向こうなので範囲の外。
- 目標: `struct CheckedDefinition { path, definition, commands: Vec<CommandLine> }` を `check_definition` が返す。誤りは今のとおり全件を集めて 64。`Checked` と `Session` に `commands` を持たせる。dry_run は `checked.commands` を写すだけ(誤りの枝と `EXIT_DEFINITION` の import を消す)。`report_execution(&mut self, job, command: &CommandLine, execution, outcome, next)`。`step::run_step` に `command: &CommandLine` を足す(run は `&session.commands[index]`、pipeline は `job` と `commands` を並べて回す)。

### 1c. `check_definition_and_env` を ctx.rs へ

- 今: `aux_commands.rs:101-117`(validate `:124` と graph `graph.rs:40` が呼ぶ)。目標: `impl Ctx { pub(crate) fn check_definition_and_env(&mut self, cmd) -> Result<CheckedDefinition, i32> }`。処理(`.env` の警告の表示を含む)は変えない。

### 1d. run と pipeline の入口の重複、通らない else

- 今: `run.rs:38-46` と `pipeline.rs:41-49` が同じ形(位置引数の欠け → 68、`--dry-run` → `run_dry_run`)。位置引数の欠けは `parse_args` で先に 68 になるので、この分岐には来ないが、動きを保つため残す。`run.rs:75-78` の `job_index` の `None`(通らない)は、prepare の中(`ctx.rs:139-145`)で検査済み。
- 目標: 入口の共通の関数(`DryRunTarget` を `FlowTarget` に改名してよい)。prepare を部品(`require_job` → 位置 `usize`、`require_known`、run ID → `.env` → ステート)に分け、`prepare_run(cmd, job_id) -> Result<(Session, usize), i32>` が位置を返す。run はその位置でジョブを引くので、通らない else が型の上から消える。
- **順序を保つ**: シグナル → 定義(64)→ command(64)→ ジョブ(68)→ `--assume`(68)→ run ID(68)→ `.env`(64)→ ステート(65)。

## 段 2(型)

### 2a. 分類と遷移を型で表示に渡す

- 今: `ctx.rs:306-311`(分類を `"success"` などの文字列に)、`:335-339`(分類 → `TransitionOn`。同じ写しが `core/src/flow/next_step.rs:108-113`)、`:347-350`(`TransitionOn` → 文字列)。表示の側: `report/mod.rs:163-170`(`JobResult { result: &'static str }`)、`:175-181`(`Transition { on: &'static str }`)、`:405-410`、`:417-424`、`job_result_line` の `result == "error"`(`:619`)。
- 目標: core に `Classification::as_str`、`TransitionOn::as_str`、`Classification::transition_on(self) -> Option<TransitionOn>`(ctx と next_step の両方で使う)。`DisplayEvent` を `JobResult { result: Option<Classification>, .. }`、`Transition { on: TransitionOn, .. }` に(フィールド名は変えない)。表示は `result.map_or("interrupted", Classification::as_str)`、比較は `result == Some(Classification::Error)`。
- **直す必要のあるテスト(承認が要る)**: `crates/cli/tests/report_messages.rs` の `with_all_events`(`:84〜`。`:237` の `["success","warning","error","interrupted"]`、`:246-255` の `on: "warning"`・`"success"`。この関数を使う 3 つのテストに波及)と `v_の文言`(`:524-566`。`:530`、`:539`、`:548`)。`DisplayEvent` が公開の型で、テストが文字列の値で組み立てているため。**期待する出力の文字列は 1 文字も変わらない**(値を `Some(Classification::Warning)`、`None`、`TransitionOn::Warning` などに置き換え、import を足すだけ)。

### 2b. JobStatus と SkipReason の文字列の対応を core に

- 今: `runtime/src/state/writer.rs:199-213`、`runtime/src/state/reader.rs:228-245`、`cli/src/report/watch.rs:30-38`(公開の `status_word`)。
- 目標(`core/src/types.rs`): `JobStatus::as_str`、`JobStatus::parse(&str) -> Option<Self>`、`SkipReason::as_str`、`SkipReason::parse`(`"byTransition"`、`"byEnd"`)。reader は `JobStatus::parse(t).ok_or_else(...)`。watch は `status.as_str()`(`status_word` は薄い包みとして残してよい)。テストの変更なし。

### 2c. `check_consistency` を core へ

- 今: `runtime/src/state/consistency.rs:22-46`(純粋)。型 `MismatchKind`(`runtime/src/types.rs:138-147`)、`ConsistencyResult`(`:149-154`)も一緒に移す。
- 目標: core に置き、`lib.rs` で公開。runtime は `pub use conductor_core::{ConsistencyResult, MismatchKind, check_consistency};` で道を保つ(cli、runtime のテストは変えずに通る)。
- **runtime の単体テスト 4 件はその場に残す**: runtime の `consistency.rs` を、core からの `pub(crate) use` と `#[cfg(test)] use` だけのファイルにし、`mod tests` は 1 文字も変えない(件数も動かない)。

### 2d. reader.rs の上限つきの読み取り

- 今: `reader.rs:76-91`(`read_state_file_raw`)と `:109-121`(`inspect_run_state`)が同じ手順。
- 目標: `enum BoundedError { Unreadable, TooLarge }`、`fn read_bounded(path) -> Result<Option<Vec<u8>>, BoundedError>`(`Ok(None)` はファイルが無い)。今の写像(inspect は open と metadata の失敗を `Broken`)を保つ。`read_limited`(`:132`)は単体テストが呼ぶので、名前と引数を変えない。

## 段 3(core)

### 3a. walk.rs を分ける

- `walk.rs` を `walk/mod.rs` にし、`walk/retry.rs`(`RETRY_KEYS`、`RawRetry`、`retry`、`retry_missing`、`retry_exit_codes`、`retry_integer`、`retry_duration`、`check_retry`、`build_retry`)と `walk/env.rs`(`env_table`、`env_file`、`env_file_item`)を作る。`is_valid_env_name` は mod.rs でも使うので残す。親から呼ぶ子のメソッドは `pub(super)`、`RawRetry` は `pub(super) struct`。公開の道 `conductor_core::definition::walk::walk` は変わらない。呼ぶ順序を変えない。テストの変更なし。

### 3b〜3e. 小さな重複

- 3b: 既定の成功コード `[0]`(`types.rs:144`、`walk.rs:746`)→ `types.rs` に `pub(crate) const DEFAULT_SUCCESS_EXIT_CODES: &[i32] = &[0];`。
- 3c: `exit_code_of`(`walk.rs:801-809`)を `integer_of(item).and_then(|v| i32::try_from(v).ok())` で書き、`:569` もそれを呼ぶ。
- 3d: 入れ子の上限の文(`core/src/definition/depth.rs:79`)を `format!` で `MAX_NESTING_DEPTH`(`:27`)から作る(文は 1 文字も変わらない。`:19` の説明も)。converter の `error.rs:138-139` は `&'static str` のまま、`const _: () = assert!(crate::raw::MAX_DEPTH == 32);` で定数と文を縛る(converter は core に依存しない)。表示の上限 256(`core/src/error.rs:27`、`core/src/envfile/mod.rs:47`)は error の `pub(crate)` に寄せる。
- 3e: 空の `NextStep`(`core/src/flow/next_step.rs:89-99`、`:120-128`)→ 非公開の `empty_step(job)`。

### 3f. 終了コードで終わった実行結果を作る関数

- `impl JobExecution { pub fn exited(job: &JobDefinition, exit_code: i32) -> Self }` を `types.rs` に。段 3 では本番の `core/src/plan/simulate.rs:282-291` だけを置き換える(テストは段 1〜3 で変えない)。

## 段 4(テストと文書)

### 4a. graph のテスト

- 補助の重複: `lanes(n, transitions)` が 5 か所(`graph_lane_polish:28`、`graph_branch_lanes:25`、`graph_crossing_lanes:28`、`graph_mixed_shapes:25`、`graph_layout_unify:24`)、`both(from, to)` が 2 か所(`lane_polish:35`、`layout_unify:31`)。`graph_common` に `lanes`、`both`、`letter_definition(n, transitions)` を置く。
- **入力と期待(出力の全文)がまったく同じテスト(まとめる対象)**:
  1. `graph_lane_polish::交差する通常の飛び越しは_レーンの図になる` と `graph_crossing_lanes::c1_a_が_d_へ_b_が_e_へ飛ぶ交差は_行_a_と行_b_で描く`
  2. `graph_lane_polish::交差と同じ行き先に_両方の飛び越しが混ざる形は_レーンの図になる` と `graph_mixed_shapes::交差に_両方の飛び越しが混ざる形は_レーンの図になる`
  3. `graph_lane_polish::d1_両方の飛び越しは_通常の飛び越しと同じ線で描き_主線の線だけ省く` と `graph_layout_unify::d1_両方の飛び越しの図は_変わらない`
  4. `graph_lane_polish::両方の飛び越しが交差する形は_通常の飛び越しが交差する形と同じくレーンの図になる` と `graph_layout_unify::n1_両方の飛び越しと通常の飛び越しの交差`
  5. `graph_branch_lanes::b1_分岐は_2本の線を同時に開き_実行されうるジョブの下だけ主線を描く` と `graph_layout_unify::b1_分岐の図は_実行されうるジョブの下の合流の行だけ主線を描く`
  6. 性能: `graph_crossing_lanes::幅の広い交差の最悪の形は_1000_ジョブでも_1秒以内` と `graph_perf::` の同名(`graph_perf` のほうが広く確かめるので、crossing のほうを消す)
- 一部しか確かめていないだけのもの(入力は同じだが、期待が全文でない)は、まとめずに残す(意図の記述の「同じ例」の定義に当たらない)。
- 形ごとのファイルの案: `graph_shape_skip.rs`(通常の飛び越しと交差)、`graph_shape_same_target.rs`(同じ行き先)、`graph_shape_both.rs`(両方の飛び越し)、`graph_shape_branch.rs`(分岐)、`graph_drawable.rs`(判定。辺の一覧に残る形 3 件と主線の列 0 の 2 件を寄せる。主線は `graph_main_line.rs` に分けてもよい)。64 件から 6 件を除いて 58 件。
- 直す文書: `docs/spec/rust/11-quality-and-build.md:201`、`docs/spec/rust/05-execution-flow.md:804`、`docs/spec/language-agnostic/11-quality-and-build.md` の AC-476a、AC-476b。

### 4b. converter のテスト名(117 件、すべて英語)

- 件数: src 53(`error.rs` 5、`intake.rs` 9、`resolve.rs` 4、`scalar.rs` 6、`toml_read.rs` 7、`toml_write.rs` 5、`types.rs` 3、`yaml_read.rs` 9、`yaml_write.rs` 5)、tests 64(`convert_ac.rs` 8、`convert_roundtrip.rs` 14、`nfr_security.rs` 13(1 件は無視)、`read_resolve.rs` 29)。
- 一緒に直すもの: `tests/nfr_security.rs:203-209` の `env_independence` は、子プロセスを `--ignored --exact env_probe_child` で起動する。`env_probe_child`(`:191`)を改名するなら、この文字列も同時に変える。`docs/spec/language-agnostic/11-quality-and-build.md` が converter のテスト名を 24 個参照している(改名に合わせて直す)。

### 4c. cli のテストの偽物の共通部分

- `wait` の処理が `ScriptedRuntime`(`common/mod.rs:365-379`)と `MemRuntime`(`:751-764`)で同じ(違いは Scripted がログを足すことだけ)。`flow_retry.rs` は `runtime.waits`、`runtime.interrupt_wait` に直接触る(17 か所)ので、フィールドは残し、自由関数 `fake_wait(waits, interrupt_wait, secs, signal) -> WaitOutcome` を足す最小の形にする。`run_job` の土台は 3f の `JobExecution::exited` を使う。

### 4d. README の終了コード 68

- 直す対象: `README.md:515`(68 の行)、`:493`。関連: `:254`(`--remove-on-complete` を許さないコマンドに graph と init が抜けている)、`:267`、`:664`、`:693`、`:710`。
- 今の実装で 68 になるもの(要約): コマンドと位置引数の誤り、知らないオプション、値を取らないオプションへの値、値の欠け、許されない場所のオプション(`-v` を watch・reset・convert・init に、`--config` を convert に、`--to` を convert 以外に、`--run-id` を validate・convert・init と `--state` なしの graph に、`--all` を reset 以外に、`--remove-on-complete`・`--dry-run` を run・pipeline 以外に、`--force` を init 以外に、`--format`・`--state` を graph 以外に、`--assume` を `--dry-run` なしで)、オプションの重複、`--all` と `--run-id` の併用、値の誤り(`--run-id`、`--format`、`--to`、`--assume`、init の名前)、定義を読んだ後(ジョブID・`--assume` のジョブID が定義に無い、環境変数の run ID が規則に合わない)。表は代表を挙げ、詳しくは本文の節を指す形にする。
- 制約: `docs-check` と `docs_*.rs` が必須の語句を検査する。`終了コードの表` と `| 69 |` は残す。

### 4e. 5 つの仕様書の凍結の明記

- `docs/spec/{go,python,typescript,csharp,powershell}/00-README.md` は、1 行目が見出し、2 行目が空行。1 行目の見出しの直後に引用のブロックを置く:
  > この仕様書は v0.8.0 時点の挙動で凍結されている(v0.9.0 以降の環境変数の注入と `.env`、`graph`、`--dry-run`、`init` を含まない)。最新は [rust](../rust/00-README.md) と [language-agnostic](../language-agnostic/00-README.md)。
- これらを読むテストはない。

## 段 1〜3 のあとの仕様書(docs/spec/rust)

内部の名前(`prepare_checked`、`check_definition_and_env`、`DryRunTarget`、`run_individual`、`aux_commands.rs`、`consistency.rs`、`walk.rs`)が、02・04・05・06・07・10・12 章にある。新しい置き場所と名前に合わせて直す。
