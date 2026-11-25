# コード生成の計画: u5-log-dir(ジョブの出力をファイルにも残す)

## Summary

- Builds: `[pipeline] log_dir` で、試行ごとにジョブの出力(標準出力と標準エラー)を端末に出しつつファイルにも残す。時刻の整形(CoreTime)と現在の時刻(RuntimeClock)も作る
- Touches: core の `time.rs`(新規)・`types.rs`・`definition/walk/mod.rs`・`lib.rs`、runtime の `clock.rs`・`log.rs`(新規)・`runner.rs`・`platform/unix.rs`・`platform/windows.rs`・`lib.rs`、cli の `deps.rs`・`step.rs` ほか流れ・`report/mod.rs`・`messages.rs`、テスト(新規 `time_format.rs`・`log_file.rs`・`runner_log.rs`・`binary_log.rs` と既存)、`README.md`、`demo/`
- Tests: 単体・統合テスト約 40(時刻の表、名前の表、衝突、0600、出力の複製、失敗、待ち切り)、流れのテスト 5、実バイナリのテスト 5、デモ

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR6.2、`entities.md`)、非機能の設計(`security-design.md`、`performance-design.md`、`reliability-design.md`、`observability-design.md`、`logical-components.md`)、ユニットの定義(`unit-of-work.md` の U5)、要件(`requirements.md` の FR5.1〜FR5.5、FR6.6、FR8.1)、契約(`contract-summary.md` の C2、C3、C6)、設計の決定(ADR-002、ADR-005)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。core → runtime → cli の順に、層を作ってからその層のテストを書いて走らせる。

## データの形の変更

- `PipelineDefinition.log_dir: Option<String>`(core の `types.rs`。FR5.1、`entities.md` の PipelineDefinition)。
- `Timestamp`(core の `time.rs`。UNIX 時刻の秒。契約 C2、`entities.md`)。
- `LogTarget`(runtime。dir、job_id、started_at、attempt。`entities.md`)。
- `LogOutcome`(runtime。log_not_created、write_failed、output_cut_short。`entities.md` と `reliability-design.md` の 1)。
- ステートの形は変えない(U6)。

## 設計のレビューからの持ち越し(このユニットで扱う)

- 失敗の伝え方: `run_job` は `JobExecution` と `LogOutcome` を返し、`log_not_created` のとき流れは BR5.1 の 1 行だけを出す(`reliability-design.md` の 1)。
- 読み取りのスレッドの止め方: 端末への書き込みの失敗では読み捨てを続け、待ち切れでは「止める」印で以後どこにも書かない(`reliability-design.md` の 2)。
- 安全な名前(`security-design.md` の 1)に次を足す(非機能の設計のレビュー): Windows の予約名の判定は、最終のファイル名の最初の `.` より前の部分(大文字小文字を問わない)で行う。つまり置き換えた後の stem の最初の `.` より前が予約名なら、stem の先頭に `_` を足す(例 `NUL.x` → `_NUL.x`、`con` → `_con`)。
- 名前の長さ: stem は 100 バイトまで(UTF-8 の文字の途中で切らない)。時刻・番号・拡張子を足しても 255 バイトを超えない。
- 中断(シグナル)のときも、子の終了の後の出力の待ちは同じく最大 2 秒(BR4.5)。シグナルの転送と終了コードは今のまま。
- 2 つの流れの間の順は「読んだ順」で、契約 C6 の「子が書いた順」は流れの中でだけ保つ。README に書く(`observability-design.md`)。
- 性能のテストは 16 MiB の出力、待ち切りは CI で 10 秒を上限にする(`performance-design.md`)。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 変更の前に、`unit-test-instructions.md` の 3 つのコマンドを走らせ、結果を記録する。

### Step 2: 時刻(core)

- [x] `crates/core/src/time.rs` を新しく作り(`lib.rs` で公開)、`Timestamp`、`format_rfc3339`、`format_compact`、`parse_rfc3339`、`duration_secs` を書く(BR2.1、BR2.2、FR6.6)。範囲は 1970-01-01〜9999-12-31。日数と年月日の変換は自前で書く。ドキュメントコメントは日本語。

### Step 3: `log_dir` の定義(core)

- [x] `types.rs` の `PipelineDefinition` に `log_dir` を足し、`definition/walk/mod.rs` の `PIPELINE_KEYS` と `pipeline_section` で読む(文字列でなければ TypeMismatch、空なら EmptyField。BR1.1)。

### Step 4: core の層のテストを書いて走らせる

- [x] `crates/core/tests/time_format.rs`(新規)に、整形・詰めた形・読み取り・差の表(1970-01-01T00:00:00Z、2000-02-29、2024-02-29、2100-03-01(平年)、各月末、9999-12-31T23:59:59Z)、読み取りの誤り(小数の秒、時差、小文字、範囲外の月日、2023-02-29、桁の不足)、差が負にならないことを足す(NFR2.2)。
- [x] `definition_valid.rs` と `definition_errors.rs` に `log_dir` の正しい場合と誤り、`[job]` に書くと知らないキーを足す。`nfr_security.rs` の `cases` に `log_dir` の型の誤りを足す。
- [x] 定義を組み立てているテストの補助は、型の変更に合わせて `log_dir: None` を足すだけにする。
- [x] core のコマンドを走らせ、通ることを確かめる。

### Step 5: 時計とログのファイル(runtime)

- [x] `crates/runtime/src/clock.rs`(新規)に `now() -> Timestamp`(SystemTime。1970 年より前は 0、上限を超えれば上限。BR2.3)。
- [x] `crates/runtime/src/log.rs`(新規)に `LogTarget`、`safe_log_stem`(`security-design.md` の 1 と、上の持ち越しの予約名の判定と 100 バイト)、`open_log_file(target) -> Result<(File, PathBuf), PathBuf>`(ディレクトリの作成、`create_new`、`-2`〜`-1000` の番号、UNIX 系は platform で 0600。BR3.1〜BR3.4)。
- [x] `platform/unix.rs` に作成の権限を付ける関数、`platform/windows.rs` に何もしない同じ名前の関数を足す。`cfg` は platform の中だけ。

### Step 6: 出力の複製(runtime)

- [x] `runner.rs` の `run_job` に、ログの指定(`Option<&LogTarget>`)を受け取る形を足し、戻り値を `(JobExecution, LogOutcome)` にする(`log_dir` が無いときは今と同じ経路で、`LogOutcome` は空。BR1.2)。
- [x] ログの指定があるときは、合図の確認の後、起動の前に `open_log_file`。作れなければ起動せず `StartFailed` と `log_not_created`(BR5.1)。
- [x] 作れたら `Stdio::piped()` で起動し、標準出力と標準エラーをそれぞれ 1 つのスレッドで 8 KiB ずつ読み、端末(conductor の同じ流れ)へ書いて flush し、ファイルへは `Mutex` で順に書く(BR4.1、BR4.2)。端末への書き込みの失敗では端末に書くのをやめて読み続け、ファイルへの書き込みの失敗では `write_failed` を立ててファイルに書くのをやめる(`reliability-design.md` の 2)。
- [x] 待ちのくり返しとシグナルの転送は今の関数のまま(BR4.4)。子の終了の後、スレッドの終わりを合わせて最大 2 秒待ち、待ち切れたら「止める」印を立てて `output_cut_short`(BR4.5)。中断のときも同じ。
- [x] `lib.rs` から必要なものを公開する。

### Step 7: runtime の層のテストを書いて走らせる

- [x] `crates/runtime/tests/log_file.rs`(新規): 安全な名前の対の表(`a`、`a/b`、`../x`、`.env`、`a.`、`a `、`CON`、`com1`、`NUL.x`、`日本語`、制御文字、200 バイトの ID)、ディレクトリの作成、名前の形、同じ名前が 1 つ・2 つあるときの番号、元のファイルが変わらないこと、作れないとき(ファイルを置いた場所をディレクトリにする)、UNIX 系の 0600(`cfg` はテストの中で可)、番号の上限の境界(`log.rs` の `mod tests` で、上限の定数で名前を作る関数を確かめる)。
- [x] `crates/runtime/tests/runner_log.rs`(新規): テスト用の補助バイナリで、標準出力と標準エラーの両方がファイルにあり端末にも出ること、各流れの中の順、16 MiB の出力、`log_dir` 無しで `LogOutcome` が空、作れないときに起動しないこと、UNIX 系で孫が残っても 10 秒以内に戻り `output_cut_short`。書き込みの失敗は `runner.rs` の `mod tests` で、失敗する `Write` を渡して確かめる(本番に差し替えの引数は足さない。内部の関数を `mod tests` から呼ぶ)。
- [x] 補助バイナリに必要な動き(大量の出力、両方の流れへの出力、孫を残す)が無ければ、今のテスト用の補助バイナリ(`crates/runtime/src/bin/` または `crates/cli/src/bin/` のテスト専用のもの)に足す。リリースの成果物には含まれない(今のまま)。
- [x] runtime のコマンドを走らせ、通ることを確かめる。

### Step 8: 差し替え口と流れ(cli)

- [x] `crates/cli/src/deps.rs` の `Runtime` に `now() -> Timestamp` を足し、`run_job` にログの指定を渡して `(JobExecution, LogOutcome)` を受け取る形にする。`SystemRuntime` は runtime を呼ぶ。
- [x] 流れ(`step.rs` など、試行を動かす所)で、各試行の前に `now()` を 1 回取り、`log_dir` があれば定義ファイルのディレクトリで解決して LogTarget を作る(BR2.3、BR3.1)。`--dry-run`・`validate`・`graph` はログを作らない(BR5.4)。
- [x] `log_not_created` のときは BR5.1 の 1 行だけを出し、今の起動の失敗の表示は出さない。`write_failed`・`output_cut_short` のときは試行の後に警告を 1 行ずつ出す(BR5.2、BR5.3)。終了コードとリトライの扱いは今のまま。
- [x] `report/mod.rs` の `DisplayEvent` と `messages.rs` に 3 つの文(英日)を足す(固定の文と escape したパス)。

### Step 9: cli の層のテストを書いて走らせる

- [x] 偽の `Runtime`(`crates/cli/tests/common/mod.rs`)に `now()`(決めた時刻を順に返す)と、`run_job` の新しい形を足す(引数の型と戻り値を合わせるだけ)。今の流れのテストは組み立て方だけを直す。
- [x] `flow_run.rs`・`flow_pipeline.rs`・`flow_retry.rs` に、試行ごとに時刻を取りログの指定が渡ること(リトライで試行の番号が増える)、`log_dir` 無しでは指定が無いこと、`log_not_created` で 127 と 1 行だけ、`write_failed`・`output_cut_short` で警告と終了コードが変わらないことを足す。
- [x] `src/tests/report_messages.rs` に 3 つの文の英日を足す。
- [x] `crates/cli/tests/binary_log.rs`(新規): 実バイナリで、ファイルの中身がジョブの出力と一致、名前の形、リトライで 3 つ、作れないときの 127 と 1 行、`log_dir` あり・なしで端末の出力が同じ、`-v` の前後の行とジョブの出力の順。
- [x] `signals_real.rs` に、`log_dir` ありの中断で終了コードが今と同じことを 1 件足す(UNIX 系)。
- [x] cli のコマンドを走らせ、通ることを確かめる。

### Step 10: デモと README

- [x] `demo/` に `log_dir` を使う場面を足し(Unix と Windows)、できたファイルを表示してから消す。`demo/README.md` を合わせ、`unix_demo.rs`・`windows_demo.rs` に確認を足す。`cargo test -p conductor-cli --test unix_demo --locked` を走らせる。
- [x] `README.md` に、`log_dir` の書き方、名前の形と置き換え・番号・長さ、0600、作れないとき 127・書けないとき警告・待ち切り、子から見て端末でなくなること、流れの間の順、古いログは消さないことを書く。定義のキーの表と終了コード 127 の説明を合わせる。`cargo test -p conductor-cli --lib --locked -- tests::docs` を走らせる。

### Step 11: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR2.1、BR2.2 | FR6.6 |
| Step 3 | BR1.1 | FR5.1 |
| Step 5 | BR2.3、BR3.1〜BR3.4 | FR5.3、FR5.4 |
| Step 6 | BR1.2、BR4.1〜BR4.5、BR5.1 | FR5.2、FR5.4、FR5.5 |
| Step 8 | BR2.3、BR5.1〜BR5.4 | FR5.2〜FR5.4 |
| Step 4、7、9 | すべて | FR5.1〜FR5.5、FR6.6、NFR1〜NFR6 |
| Step 10 | BR6.1、BR6.2 | FR5.3、FR8.1 |
| Step 11 | | NFR7 |

## コミットの分け方

小さな単位でコミットする(件名は日本語、先頭に `core:`・`runtime:`・`cli:`・`docs:`)。各コミットで fmt と clippy が通るようにする(使われない関数で落ちないよう、使う側と一緒に入れるか、公開の関数として入れる)。例: Step 2 と 3 は core の 2 コミット、Step 4 のテスト、Step 5・6 は runtime(差し替え口の型の変更は cli と同じコミット)、Step 7、Step 8・9 は cli、Step 10 は `docs:` の 2 コミット。版は上げない。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "feature",
  "test_strategy": "standard",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 33、runtime 14、cli 25、converter 2 のファイル)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(11 ファイル)、`crates/converter/src/tests/`(4 ファイル)。`#[test]` は合計 1,170。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- v0.10.0 の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す。それ以外(コマンド・環境変数・出力)は今までどおり検査する(インタビュー Q7)。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Meet an 80% line-coverage floor.",
      "Run the selected tests in CI before merge."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Verify the existing test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Verify the existing test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:ed0086a4c967568436c54943aebacd308a988af279eee16ed426cf4f39a154fa",
  "contract_sha256": "sha256:3789717991ddc007f6cbca5d56d6b2d0b248de4ce1061637170c6787b5bf88e9"
}
```
