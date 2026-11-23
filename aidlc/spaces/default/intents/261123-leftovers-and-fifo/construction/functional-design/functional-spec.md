# 機能の仕様(Functional Spec): 残りの整理と読み込みの順序のそろえ

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 A、Q4 C、Q5 A、Q6 A、Q7 A)。**変える場所の正確な一覧(ファイルと行、目標の形、完了の判定、使った検索のコマンド)と、基準の件数と、計算量の直す前の測定は `inventory.md`**(HEAD 025067a で作った)。この文書は、方針、段ごとの設計、**承認が要るテストの変更(§4 の T 表)と消すテストの対の表(§4.6)**、足すテスト(§5 の N 表と性能の基準)、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- 2 つの段を順に進める。段 1(動きを変えない整理。計算量を含む)は、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` と `graph` の出力)を変えない [desc]。段 2(利用者から見える変更)を最後に置き、最後のコミットで版を 0.9.8 に上げる [desc] [Q5]。
- 各段は小さなコミットの積み重ねで、どのコミットでもビルド、`clippy -D warnings`、テストが通る [Q1] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は段の順に進め、各段の終わりにテストの件数を記録し、結果は最後にまとめて見せる [Q1]。
- 既存のテストは、型の変更に伴う直しと補助の置き換えだけを変える。確かめる中身と期待する出力の文字列は変えない [Q2]。例外は §4 の ★(T1b の 1 行の置き換え、段 2 の 2b で期待が変わる T23、T22 の版)だけ。重複したテストを消すときは、§4.6 で残るテストと対にして示す。
- 新しい依存を足さない [memory:M5]。`unsafe` は runtime の `platform/unix.rs` だけ。`cfg(unix)`/`cfg(windows)` は src の `platform` の外に置かない(テストのファイル `tests/` の `#[cfg(unix)]` は前例どおり可。core と converter の src のテストの既存の `cfg(unix)` は触らない。Q4: C)。誤りの表示に入力の値を含めない。ステートに環境変数を書かない。runtime の製品のコード(src の `#[cfg(test)]` より前の部分)に `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない(`crates/runtime/tests/nfr_source.rs` の `製品のコードに_パニックしうる書き方がない` が検査する。1c の `bounded_read.rs`・`replace_via_temp`、2b の形 B も同じ)。converter は core に依存しない。MSRV 1.100。テスト名とドキュメントコメントは日本語。
- 仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` を直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない [Q2]。段 2 の変更には「v0.9.7 までは…」と注記する [Q5]。

## 2. 段ごとの設計(詳細は `inventory.md` の同じ番号)

### 2.1 段 1(動きを変えない整理)

| 項目 | 要点 |
|------|------|
| 1a | 古くなった説明 13 か所を直す(runtime の `lib.rs` の `run_job` の行と run ID の関数の並び、`runner.rs` の「`job` は結果の `job_id` と `command` のため」、`read_state_raw`・`write_state` の説明、`messages.rs` の「U3」、`step.rs`・`run.rs` の経緯の文、`types.rs` の `is_raised`、`report/mod.rs` の切る・切らないの説明、`dry_run.rs`・`aux_commands.rs` の守りの理由(守りは残し、理由を「`parse_args` を通る限り起こらない、内部の不変条件の守り」に)、仕様書 rust 05 の `apply_interrupt_in`) |
| 1b | 使われないもの: `apply_interrupt_in`、`EffectiveEnv` の `job_id` と `names()`、core のテストの補助の使わない引数(D2)、模擬の分類の二重の計算(`classify_exit_code`・`ExitCodeClass` を非公開に)、`decide_before_run` の常に真の条件、cli の `interrupted_signal`(`result_detail` を使う)、runtime の `command_line`(`check_command` に畳む) |
| 1c | 二重の処理: 「一時ファイルに書いて置き換える」を `temp_file::replace_via_temp` の 1 か所に、ステートの読み込みの段(`read_state_file_raw`・`parse_state_for`)をなくす、上限つきの読み取りを新しい `bounded_read.rs`(`read_limited` と形 B の `read_regular_file`)に置き `.env` が使う、`build_retry` は値を移し欠けたらジョブを組み立てない(D3)、ごく小さい整理 5 つ(`hint` の型は `Option<usize>` にそろえる。D4) |
| 1d | cli の `Runtime::run_job` から `job` を外す(Q3: A。§2.3) |
| 1e | core の `flow`・`plan`・`envfile`・`graph` の下位モジュールを非公開に(関数と型は今の再公開の名前のまま)。定数 `EXIT_*`・`ASSUME_EXIT_CODE_*`・`RESERVED_ENV_PREFIX`・`MAX_REPORTED_ERRORS` を `pub(crate)` に。`MAX_ENV_FILE_BYTES`・`MAX_ENV_ENTRIES` は使われるので狭めない |
| 1f | runtime の `pub(crate)` のうちモジュールの中でしか使わない 11 個を非公開に(`write_state_file`、`persist`、`remove_stale_temps`、`serialize`、`temp_name`、`create_temp`、`split`、`read_limited`、`Waitable`・`Waited`・`wait_loop`) |
| 1g | 型と名前: cli の `escape_path` を `escape_full` に(D6)、run ID の有無の真偽値を `StateContext` に(`for_read(Option<&RunId>)`、`ConsistencyMismatch.context`、`reset_hint(StateContext)`。D5)、converter の行番号の 0 の番兵を `Option<usize>` に、`ConversionError.reason` を `&'static str` に |
| 1h | 計算量: 定義の `env` の表の大文字小文字の検査を `HashSet` に、遷移の重複の確認を種類ごとの真偽値に、`env_file_refs` の重複の確認を `HashSet` に(この 3 つに性能のテスト P1〜P3)、cli の `-v` の図の辺を出どころのジョブで 1 回まとめる、`record_skip` の土台の複製をやめる(この 2 つはテストを足さず前後の時間を記録。D8) |
| 1i | テスト: core の src の単体テストの定義の手組みを `#[cfg(test)]` の `test_support` に、cli の tests の `FakeEnv` の言語分け・`Deps` を組んで `run_cli_with` を呼ぶ形・`state_error` の二重を `common` に(D12)、runtime の tests の `def`・`put`・`paths` を `common` に、深さの境界の重複したテストを 4 つ消す(§4.6)、runtime のテストの `run_job` の名残(`job(..).command`)を `common::command_line` に |

### 2.2 段 2(利用者から見える変更と版)

| 項目 | 要点 |
|------|------|
| 2a | converter の読み込み(`read_input`)を、定義ファイルの `read_definition` と同じ形 B(パスで種別 → 通常のファイルだけ開く → 開いたハンドルで種別と大きさ → 上限 + 1 バイトまで → UTF-8)に。書き手のいない名前付きパイプでも止まらずに `unreadable`(`通常のファイルではありません`)。ほかの場合の種類と理由は変わらない |
| 2b | ステートの読み込み(`read_state`、`read_state_raw`、`reset --all` の `inspect_run_state`)を、`.env` と共有する `bounded_read::read_regular_file`(形 B)に。名前付きパイプ・ディレクトリのステートは、どちらも `unreadable`、理由は今の `開く、または読み取ることができません`(D7)。`/dev/zero` へのリンクのステートも開かずに `unreadable` になり、★ T23 の期待が変わる(§4.7)。仕様書 06 の §5.1 の表を改訂 |
| 2c | 文字列の `command` の先頭の語が空白だけ(`'  '`)なら、配列の `["  "]` と同じく `Empty`(定義の誤り、終了コード 64)。今は 127 |
| 2d | 最後のコミットで版を 0.9.8 に(`7239aa2` と同じ 22 ファイル) |

各項目のコミットで、`docs/spec/rust` と `docs/spec/language-agnostic` の該当の行を合わせる(`inventory.md` の各項目の「仕様書」)。段 1 の仕様書の変更は内部の名前と形だけで、利用者から見える挙動の記述は変えない。

### 2.3 1d の設計(cli の `Runtime::run_job` の `job`。Q3: A)

- **理由(ユーザー、2026-11-23)**: リファクタリングで、トレイト `Runtime` の `run_job` の `job` の引数は本番では不要になった(`SystemRuntime` は受け取って捨てる)が、テストのコードの都合(偽物が `job.id` を参照する)で残っていた。**テストのコードの都合で本番のコードが曲げられるのは本来あるべきではない**ので、`job` を外し、本番の `conductor_runtime::run_job(line, env, signal)` と同じ形にする。
- **消えるのは、トレイトの `run_job` に渡す引数だけ**。`crates/cli/src/retry.rs` の中で `job` を使う所は今のまま残る: `merge_env(&def, job, …, &EnvContext { job_id: &job.id, … })` の環境変数の合成(`:52-61`)、`retry_decision(job, &execution, retries_done, last_wait_secs)` のリトライの判断(`:65-69`)、リトライの知らせのジョブ ID。`run_job_with_retry(ctx, session, job, line)` の引数も、`step.rs` からの呼び出しも変わらない。
- 偽物(`ScriptedRuntime`)は、渡された `env` の `CONDUCTOR_JOB_ID` からジョブ ID を取る。これは**テストの側の事情**として扱う(本番の形を決める理由にしない)。core の `merge_env` は自動の変数の層(最後の層)で `CONDUCTOR_JOB_ID` をいつも足し、利用者は予約の接頭辞 `CONDUCTOR_` を書けないので、ほかの値に上書きされない。偽物は今と同じ記録(`run:<id>`、`skip-run:<id>`、`run_envs`、`run_lines`)、台本の引き当て、`during_job`・`after_job` の呼び出しを保つ。`CONDUCTOR_JOB_ID` が無いとき(本番の流れでは起こらない)は、偽物がパニックで止める(D1)。`MemRuntime` は `job` を使っていないので、署名だけ変える。

## 3. 実物で確かめて調査と違った点、範囲から外したもの

- **調査と違った点**(実物に従った):
  - 1a-11: `dry_run.rs` の守りの理由「公開の型 `CliCommand` を直接組み立てて呼ばれたとき」は誤り。`CliCommand` を受け取る関数はすべて `pub(crate)` で、公開の入口は `run_cli`・`run_cli_with`(引数を `parse_args` で読む)だけ。
  - 1b-2: `EffectiveEnv::names()` と `job_id` を読むのはテスト `env_merge.rs` だけ。`job_id` の確認(`:150`)は、同じ値を `CONDUCTOR_JOB_ID` の確認に置き換える(★ T1b)。
  - 1c-2: `parse_state_for` の完了を空にする処理は、`inspect_run_state` の結果に効かない(完了でも `Ours`)。`.env` の読み取り(`envfile.rs:123-130`)は前回すでに形 B で、`read_limited` の写しを持つ。共有の `bounded_read.rs` を段 1 で作り(`.env` だけが使う。動きは同じ)、段 2 でステートも使う。
  - 1f: `runner.rs` の `Waitable`・`Waited`・`wait_loop` も、モジュールの中だけで使う `pub(crate)`。足した。
  - 1g-4: `reason` を `&'static str` にすると、cli のテストの組み立ての `reason: "…".into()` が `clippy::useless_conversion` に当たる(T19)。
  - 1h: 定義の `env` の表と `env_file_refs` は、定義ファイル 1 MiB で約 8 万項目書け、40,000 項目で 8〜9 秒(debug)かかる。遷移の重複は、重複の誤りのある入力だけで二乗になる(`inventory.md` §0.2)。
  - 1h-2: `TransitionOn` は `Hash` を持たない。公開の型に derive を足さず、種類ごとの真偽値 2 つで持つ。
  - 1i-2c: `report_messages.rs` の「英語の表示は英語の文だけで日本語の文字を含まない」は、すべてのイベントの表示を ASCII と確かめるので、`common::state_error` の理由(今は日本語)を ASCII の `"reason"` にしないと共有できない(D12)。
  - 1i-4: 深さの境界は、core では単体テスト 2 つが統合テストと重なり、converter では YAML の「とても深い入力」の単体テストが統合テストに含まれ、逆に統合テストの「32 と 33」が単体テストに含まれる。TOML の単体テストは統合テストに無い入力を持つので残す。
  - 2b: `reset --all` は、候補を `symlink_metadata` で通常のファイルに絞ってから読むので、名前付きパイプの候補は今も読まない。読む関数を形 B にそろえる(動きは同じ)。
  - 2c: 空白だけの先頭の語で 127 になることを確かめる既存のテストは無い(期待の変わるテストは無い)。
  - 2b(確認担当の指摘で見つけた): `crates/runtime/tests/state_read.rs:112-121` の `終わりのない入力でも上限で止まる`(`#[cfg(unix)]`)は、ステートのパスを `/dev/zero`(文字デバイス)へのシンボリックリンクにして `TooLarge` を期待する。形 B では、パスのメタデータ(リンクを辿る)が通常のファイルでないので、開かずに `Unreadable` になり、このテストは落ちる。★ T23 として期待を変える(§4.2)。`/dev/` を使うテストはこの 1 つだけ(`grep -rn "/dev/" crates/*/tests crates/*/src`。`.env`(`env_loader.rs`)、converter、定義ファイル(core)のテストには無い。終わりのない入力は、converter の `intake.rs` の単体テストの `Endless` と core の `intake.rs:192` の単体テストがメモリの読み手で確かめ、ファイルを使わないので形 B の影響を受けない)。
- **範囲から外したもの**:
  - core と converter の src のテストの `cfg(unix)`(Q4: C)。
  - 計画の型で方式ごとの項目を結ぶこと(Q7: A)。
  - runtime の `load_env_files` の `seen` の線形の探索(`envfile.rs:43-47`。前回も範囲の外。D11)。
  - `record_skip` の公開の `apply_skip` の中の複製と、書いたかを決める全体の比較(1h-5。比較は要る。公開の関数の契約の中の処理)。
  - core の `DefinitionError.reason: String`(名前を埋める理由があるため。1g-4)。
  - `tests/common/mod.rs:63`(core)の `exited(_job_id, code)`(意図して残した。D2)。
  - 凍結した仕様書、タグとリリース。

## 4. 変えるテストの一覧(ユーザーの承認が要る)

Q2: A に従い、変えるのは**型の変更に伴う直し(組み立て、import、項目の読み方)と補助の置き換え**だけで、**テストの名前、確かめる中身、期待する出力の文字列は変えない**。例外は ★ の 3 つ(T1b の 1 行の置き換え、段 2 の 2b で期待が変わる T23、段 2 の版 T22)。件数が変わるのは §4.6 の消すテスト(−4)だけ。src の中の単体テスト(`#[cfg(test)] mod tests`)の変更もここに挙げる。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §8。T 表を作るのに使った主な検索(型・項目・関数の名前のすべての参照):

```sh
grep -rn "\.names()\|EffectiveEnv\b\|e3\.job_id\|env\.job_id" crates
grep -n "exec(\|outcome(\|step(" crates/core/src/flow/record.rs crates/core/src/flow/next_step.rs
grep -n "exec(\|exited(" crates/core/tests/retry_flow.rs
grep -rn "conductor_core::\(flow\|plan\|envfile\|graph\)" crates | grep -v "^crates/core/src"
grep -n "fn job\|fn def\b\|fn definition" crates/core/src/*.rs crates/core/src/flow/*.rs
grep -rn "parse_state\b\|parse_state_for\|read_limited" crates/runtime/src
grep -n "job(\|\.command\|fn job\|fn line" crates/runtime/src/runner.rs crates/runtime/tests/runner_basic.rs crates/runtime/tests/runner_env.rs crates/runtime/tests/runner_signal.rs crates/runtime/tests/nfr_robust.rs
grep -n "^fn \|mod common" crates/runtime/tests/state_read_raw.rs crates/runtime/tests/run_id_state.rs crates/runtime/tests/state_read.rs crates/runtime/tests/state_write.rs
grep -rn "impl Runtime for\|fn run_job\|job\.id" crates/cli/tests/common/mod.rs
grep -rn "escape_path" crates
grep -rn "with_run_id\|for_read\|reset_hint" crates/cli
grep -rn "ConversionError {" crates/cli/tests
grep -rn "FakeEnv::with(&\[(\"LANG\"\|fn env_for\|fn state_error\|run_cli_with\|Deps {" crates/cli/tests
grep -rn "RawNode\|\.line\b" crates/converter/src
grep -rn "深\|depth\|with_timeout" crates/core/src/definition/depth.rs crates/core/tests/definition_errors.rs crates/converter/src/yaml_read.rs crates/converter/src/toml_read.rs crates/converter/tests/read_resolve.rs
grep -rn "'  '\|空白だけ" crates/*/tests crates/runtime/src/split.rs
grep -rn "0\.9\.7" crates
# 段 2 の形 B で種別が変わる、特別なファイル(デバイス・名前付きパイプ・ディレクトリ)を使うテスト
grep -rn "/dev/\|mkfifo\|create_dir\|symlink(" crates/*/tests crates/*/src
grep -rn "Endless\|io::repeat\|repeat(b'" crates/*/src crates/*/tests
# Deps を組んで run_cli_with を呼ぶ所(1i-2b)
grep -rn "Deps {\|run_cli_with(" crates/cli/tests
```

検索で参照が見つからず、テストを変えずに通るもの: 1a(コメント)、1b-1・1b-4〜1b-7、1c-1・1c-3・1c-4、1e の T5 以外、1f(単体テストは同じモジュールの中)、1g-3(`RawNode.line` を読むテストが無い)、1h の 5 つ、段 2 の 2a・2c と、T23 を除く 2b(期待の変わるテストが無い)。

### 4.1 core(`crates/core/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T1a | `tests/env_merge.rs:85`、`:167`、`:169`、`:170` | 4 | `env.names()` を、ファイルの中の補助 `fn origins(env: &EffectiveEnv) -> Vec<(&str, EnvOrigin)>`(`env.entries.iter().map(\|e\| (e.name.as_str(), e.origin)).collect()`)に置き換える(同じ並びと値。確かめる中身は同じ) | 1b-2 |
| T1b ★ | `tests/env_merge.rs:150` | 1 | **`assert_eq!(e3.job_id, "a")` を `assert_eq!(get(&e3, "CONDUCTOR_JOB_ID"), Some("a"))` に**(フィールドが無くなる。「`e3` がジョブ `a` の環境である」ことを、残る表し方で確かめる。同じテストの他の確認はそのまま) | 1b-2 |
| T2 | `src/flow/next_step.rs:227`、`:239`、`:410`、`src/flow/record.rs:278-303` と呼ぶ所 35 か所(`exec` 12、`outcome` 12、`step` 11)(単体テスト) | 2 ファイル | 補助の使わない引数を外す: `exec(code)`、`outcome(c)`、`step()`。呼ぶ所の第 1 引数(ジョブ ID の文字列)を消す | 1b-3(D2) |
| T3 | `tests/retry_flow.rs:39-49` と呼ぶ所 15 か所(`exec` 1、`exited` 14) | 1 ファイル | `exec()`、`exited(code)`。呼ぶ所の `&j` を消す | 1b-3(D2) |
| T5 | `tests/graph_types.rs:92` | 1 | `use conductor_core::graph::overlay::node_state_of;` を `use conductor_core::graph::node_state_of;`(再公開の道)に | 1e |
| T6 | 構造体を手で組んでいる行(`awk` で `#[cfg(test)]` より後の `JobDefinition {`・`PipelineDefinition {` を数えた): `src/types.rs:486`(補助 `job`)・`:595`(テスト `ジョブの位置は記述順` の中の定義)、`src/consistency.rs:71-72`・`:80`、`src/flow/resume.rs:58-59`・`:66`、`src/flow/decide.rs:93-94`・`:101`、`src/flow/next_step.rs:196`・`:209-210`、`src/flow/classify.rs:128`、`src/flow/record.rs:254`・`:262`(単体テスト)、新しい `src/test_support.rs` | 8 ファイル | 補助の名前と引数は今のまま、本体を `crate::test_support::{job, definition}` と構造体の更新の書き方で書く(フィールドの全部の手書きをやめる) | 1i-1 |

### 4.2 runtime(`crates/runtime/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T4 | `src/state/reader.rs:236-240`(単体テストの補助 `parse_state`)、`:461-467`(単体テスト `上限_1_つぶんだけ読む`) | 2 | `parse_state` の本体を `parse_state_raw_for(bytes, None).map(\|s\| s.filter(\|s\| !s.completed))` に(テストの本体は変えない)。`上限_1_つぶんだけ読む` は、名前と中身を変えずに `src/bounded_read.rs` の単体テストへ移す(`read_limited` と一緒に。件数は同じ) | 1c-2 |
| T7 | `src/runner.rs:228-245`、`:263-264`、`:270-271`(単体テスト) | 3 | 補助 `job(command)` と `line(&job)` を `line(command: &str) -> CommandLine` 1 つに。`JobDefinition` の import を消す | 1i-5 |
| T8 | `tests/runner_basic.rs:27-28`、`:108`、`:175-176`、`:221-222`、`:265-266`、`tests/runner_signal.rs:38-39`、`:119-120`、`:156-157`、`:199-200`、`tests/runner_env.rs:29-30`、`tests/nfr_robust.rs:180-181`、`:188-189` | 12 | `let j = job("j", command); let line = check_command(&j.command)…` を `let line = command_line(command);` に。`:108` の使わない `let _j = job(..)` を消す。`:175-176` は `check_command(&Command::from("'"))` | 1i-5 |
| T9 | `tests/state_read_raw.rs:24-31` | 1 | `def`・`put` の写しを消し、`mod common;` と `use common::{def, put};` | 1i-3 |
| T10 | `tests/run_id_state.rs:55-61` | 1 | 同上 | 1i-3 |
| T11 | `tests/state_read.rs:45-47` | 1 | `paths` の写しを消し、`common::paths` を使う | 1i-3 |
| T12 | `tests/state_write.rs:28-30` | 1 | 同上 | 1i-3 |
| T13 | `tests/common/mod.rs`(足すだけ) | — | `def`、`put`、`paths`、`command_line` を足す(既存の補助は変えない) | 1i-3、1i-5 |
| T23 ★(承認待ち) | `tests/state_read.rs:112-121`(`#[cfg(unix)]` の `終わりのない入力でも上限で止まる`) | 1 | **期待を `StateErrorKind::TooLarge` から `StateErrorKind::Unreadable` に変え、テスト名を `文字デバイスへのリンクのステートは開かずに_unreadable` に、説明を「`/dev/zero`(通常のファイルでないもの)へのシンボリックリンクのステートは、開かずに `unreadable`(形 B。終わりのない読み取りに入らない)」に変える**。時間の上限(`run_with_limit`)と、リンクの作り方はそのまま。コミット 40(形 B にするコミット)で変える。「終わりのない入力でも上限 + 1 バイトで読み止める」性質は、N8(`bounded_read.rs` の単体テスト)と、移した `上限_1_つぶんだけ読む`(T4)が確かめる。詳細と代案は §4.7 | 2b |

### 4.3 converter(`crates/converter/`)

既存のテストの書き換えは無い。消すテスト 2 つは §4.6。

### 4.4 cli(`crates/cli/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T15 | `tests/common/mod.rs:315-366`(`ScriptedRuntime::run_job`)、`:716-728`(`MemRuntime::run_job`)、`:257-263`(`state_error`)、`:633-651`(`Harness::run_os`)、`:816-830`(`run_mem`)、`impl FakeEnv`(`:48-66`) | 6 | 偽物の `run_job` から `job` を外し、`ScriptedRuntime` は `job_id_of(env)`(`CONDUCTOR_JOB_ID`。無ければパニック。D1)で今と同じ記録・台本・仕掛けを行う。`state_error` の理由を ASCII の `"reason"` に(D12)。`run_os` と `run_mem` は新しい `run_with_fakes` を呼ぶ。`FakeEnv::for_language(lang)` を足す | 1d、1i-2 |
| T16 | `tests/escape_table.rs:21`、`:69` | 2 | `escape_path` を `escape_full` に(import と呼び出し。テスト名と期待は変えない) | 1g-1 |
| T17 | `tests/graph_long_id.rs:17`(説明の文) | 1 | 説明の中の `escape_path` を `escape_full` に | 1g-1 |
| T18 | `tests/report_messages.rs:178` | 1 | `with_run_id: false` を `context: StateContext::Read` に | 1g-2 |
| T19 | `tests/report_messages.rs:206`、`:213`、`:1080`、`tests/flow_aux.rs:364` | 4 | `ConversionError` の組み立ての `reason: "…".into()` の `.into()` を外す(型が `&'static str` になり、`clippy::useless_conversion` に当たるため。文字列は同じ) | 1g-4 |
| T20 | `tests/report_messages.rs:34-39`(`env_for`)、`:41-47`(`state_error`) | 2 | `env_for` は `FakeEnv::for_language(lang)` の 1 行の包みに、`state_error(kind)` は `common::state_error(Path::new("dir/c.toml.state"), kind)` の 1 行の包みに(呼ぶ所 33 か所と 3 か所は変えない。パスと理由の値は今と同じ) | 1i-2 |
| T21 | `tests/flow_init.rs:35-64`(`run_with`) | 1 | 言語の `match` を `FakeEnv::for_language(lang)` に、`Deps` の組み立てと `run_cli_with` を `run_with_fakes` に | 1i-2 |
| T24 | `tests/nfr_robust.rs:136-142`(`utf8_でない引数でパニックしない` の中の `Deps` の組み立てと `run_cli_with`) | 1 | 4 か所目の「`Deps` を組んで `run_cli_with` を呼ぶ形」。`run_with_fakes(&runtime, &mut sink, &env, &signals, args)` に寄せる(同じ `sink` を使い回すので、引数の形がそのまま合う。終了コードの範囲の確認は変えない) | 1i-2 |
| T25 | converter の `tests/nfr_security.rs:328` | 1 | `assert!(shown.contains(&e.reason), …)` の `&e.reason` を `e.reason` に(`reason` が `&'static str` になると `clippy::needless_borrows_for_generic_args` に当たるため。確かめる中身は同じ)。実装の中で見つかった一覧の漏れで、ユーザーが 2026-11-23 に追加を認めた | 1g-4 |
| T22 ★ | `tests/binary_basic.rs:246` | 1 | **`"conductor 0.9.7\n"` を `"conductor 0.9.8\n"` に**(版の更新。前回 `7239aa2` と同じ) | 2d |

### 4.5 テストを変えずに通るもの(参考)

- 段 2 の 2a・2c と、★ T23 を除く 2b: 既存のテストの期待は変わらない(2a・2b はほかの場合の種類と理由が同じで、既存のテストがそのまま確かめる。2c は空白だけの先頭の語を確かめる既存のテストが無い)。2b で期待が変わるのは、通常のファイルでない `/dev/zero` へのリンクを使う ★ T23 だけ(§4.0 の `/dev/` などの検索で確かめた)。
- 1h の計算量の変更: 公開の関数の形と結果は同じ。既存の `env_definition.rs`(大文字小文字の衝突)、`nfr_security.rs`(遷移の重複)、`env_merge.rs`・計画のテスト(`env_file_refs`)、`graph_report.rs`(`-v` の図)、`flow_pipeline.rs`(スキップの記録)がそのまま確かめる。

**合計**: 変えるテストのファイルは core 10(統合 3: `env_merge.rs`、`retry_flow.rs`、`graph_types.rs`。src の単体テスト 7 ファイル: `types.rs`、`consistency.rs`、`flow/resume.rs`、`flow/decide.rs`、`flow/next_step.rs`、`flow/classify.rs`、`flow/record.rs`)と新しい `src/test_support.rs`、runtime 11(統合 9: `runner_basic.rs`、`runner_signal.rs`、`runner_env.rs`、`nfr_robust.rs`、`state_read_raw.rs`、`run_id_state.rs`、`state_read.rs`(T11 と ★ T23)、`state_write.rs`、`common/mod.rs`。src の単体テスト 2: `state/reader.rs`、`runner.rs`)と新しい `src/bounded_read.rs` の単体テスト、cli 8(`common/mod.rs`、`escape_table.rs`、`graph_long_id.rs`、`report_messages.rs`、`flow_aux.rs`、`flow_init.rs`、`nfr_robust.rs`、`binary_basic.rs`)。消すテストのあるファイルは core 1、converter 2(§4.6)。期待が変わるのは ★ T23(種類とテスト名)と ★ T22(版の文字列)。★ T1b は確かめ方の置き換え。

### 4.6 消すテストと、同じことを確かめて残るテスト(Q2: A)

| 消すテスト | 確かめていること | 同じことを確かめて残るテスト | 残るテストで確かめられない点 |
|------------|------------------|-------------------------------|-----------------------------|
| core `src/definition/depth.rs` の単体テスト `配列の深さ_32_は受理し_33_は拒否` | `check_depth` で、配列の深さ 32 は通り、33 は `syntax`(行 1) | core `tests/definition_errors.rs` の `深さ_32_は受け入れ_33_と_1000_は_syntax`(公開の入口で、配列の 32 は深さの検査を通り(`versionMissing`)、33 と 1000 は `syntax`)と `深さの上限と巨大な入れ子`(配列の 32・33) | 境界(深さ 33)のときの行の番号(1)。消したあと、行の番号を確かめるのは、残る単体テスト `深い入力でも行番号を返す`(深さ 40 の配列で行 3)だけになる(境界の 33 での行の番号は確かめなくなる。超えた位置の行を返す処理は深さによらず同じ) |
| core `src/definition/depth.rs` の単体テスト `インラインテーブルの深さ_32_は受理し_33_は拒否` | `check_depth` で、インラインテーブルの深さ 32 は通り、33 は誤り | core `tests/definition_errors.rs` の `深さ_32_は受け入れ_33_と_1000_は_syntax`(インラインテーブルの 32 は `versionMissing`、33 と 1000 は `syntax`) | なし |
| converter `src/yaml_read.rs` の単体テスト `とても深い入力は_異常終了せずに失敗する`(時間制限の手書き) | YAML の流れの配列(閉じた `[[…]]`)・流れのマップ(閉じない `a: {a: {a: …`)・ブロックの配列(末尾の値の無い `- - - …`)の 1000 段と 100,000 段、インデントの 1,300 段が、30 秒以内に、パニックせずに失敗する(失敗の種類は見ない) | converter `tests/read_resolve.rs` の `極端に深い入力は_異常終了せずに文法の誤りで失敗する`(YAML の閉じた流れの配列 `a: [[…]]`、閉じた流れのマップ `a: {a: …1}`、末尾に値のあるブロックの配列 `- - … x`、インデントの 1,300 段、TOML の配列・インラインの表の 1000 段と 100,000 段。共通の `with_timeout`(30 秒)。種類が `syntax` であることまで確かめる) | **入力は同じではない**: 単体テストの「閉じない流れのマップ」と「末尾の値の無いブロックの配列」は統合テストに無い(統合テストは閉じた形と末尾の値のある形)。確かめる性質(とても深い入力で、止まらず・パニックせずに失敗する)は、深さの上限の検査が入れ子を読み進める途中で働くことで、入力の終わりの形(閉じているか)によらない。残るテストは、同じ深さの入れ子で、失敗の種類(`syntax`)まで確かめるので強い。閉じない形・値の無い形そのものを消すことになる点は、承認で判断してもらう(実装で、消す前に閉じない形を統合テストの入力で一時に試し、`syntax` で 30 秒以内に返ることを確かめて記録する)。単体テストは `yaml_read::read` だけを、統合テストは読み取りと展開(`read_tree`)を通す |
| converter `tests/read_resolve.rs` の `読み取りの深さの上限_32_と_33` | YAML の `a: [[…]]` の 31 段の配列は受理、32 段は `syntax` | converter `src/yaml_read.rs` の単体テスト `深さ32は受理し_33は文法の誤り`(同じ入力の 31 段で受理、32 段で `syntax` と理由 `TOO_DEEP`。ブロックの配列 32・33 も) | 統合テストは展開(`resolve`)も通す。31 段の配列は展開でも深さが増えない(エイリアスなし)ので、受理の結果は同じ |

- 残すもの: core の `テーブルの見出しの深さも数える`(見出しの 32・33 は統合テストに無い)、`通常の定義の深さは浅い`、`深い入力でも行番号を返す`、`深さの上限と巨大な入れ子`。converter の `toml_read.rs` の `とても深い入力は_異常終了せずに失敗する`(閉じない配列、ドットのキー、見出しは統合テストに無い)、`toml_read.rs` の `深さ32は受理し_33は文法の誤り`。
- 件数: core −2(単体)、converter −2(単体 1、統合 1)。

### 4.7 ★ T23(段 2 の 2b で期待が変わるテスト。承認待ち)

- 今のテスト(`crates/runtime/tests/state_read.rs:112-121`、`#[cfg(unix)]`): `終わりのない入力でも上限で止まる`。ステートのパスを `/dev/zero` へのシンボリックリンクにし(大きさは 0 と報告されるが、読むと終わらない)、30 秒の上限の中で `read_state` が `TooLarge` を返すことを確かめる。
- 形 B にしたあと: パスのメタデータ(シンボリックリンクを辿る)で、`/dev/zero` は文字デバイスで通常のファイルでないので、開かずに `Unreadable`(理由は D7 の決定のとおり今の `開く、または読み取ることができません`)。今の期待のままでは落ちる。
- **推奨案**: 期待を `Unreadable` に変え、テスト名を `文字デバイスへのリンクのステートは開かずに_unreadable` に、説明を今の意味(通常のファイルでないものは開かない。終わりのない読み取りに入らない)に変える。リンクの作り方と時間の上限はそのまま(止まらないことも引き続き確かめる)。「終わりのない入力でも上限 + 1 バイトで読み止め、`TooLarge` にする」性質は、ファイルを使わない N8(`bounded_read.rs` の単体テスト)で確かめる。テスト名が変わるので、§7 の「テスト名の一覧の差」に、この 1 つの名前の変更を含める。
- 代案と、採らない理由:
  - (B)テスト名と説明は変えず、期待だけを `Unreadable` に変える: 名前(「上限で止まる」)が、確かめる中身(開かない)と食い違う。
  - (C)テストを消す: 「通常のファイルでないリンクのステートを開かない」ことと、止まらないことの確かめが無くなる(N5 は名前付きパイプで、デバイスへのリンクではない)。
  - (D)文字デバイスだけは開いて読む(形 B の例外にして `TooLarge` を保つ): 定義ファイル・`.env`・converter の形 B(通常のファイルでなければ開かない)と食い違い、開くと止まりうるデバイスもあるので、そろえる目的に反する。
- 同種のテストの確認: `grep -rn "/dev/\|mkfifo\|create_dir\|symlink(" crates/*/tests crates/*/src` で、特別なファイルを使うテストは、`/dev/zero` のこの 1 つ、名前付きパイプ(runtime `env_loader.rs`、core `intake.rs` の単体テスト。どちらもすでに形 B で「開かずに `unreadable`」)、ディレクトリ(runtime `state_read.rs` の `読めないファイルは_unreadable` は今も後も `Unreadable`、`run_id_state.rs` の `reset --all` は対象外のまま)、シンボリックリンク(`run_id_state.rs` の `reset_all_はシンボリックリンクを辿らず消さない`。`reset --all` の候補の選び方は変えない)だけ。`.env`・converter・定義ファイルのテストに `/dev/` を使うものは無い。終わりのない入力は、converter `intake.rs` の単体テスト(`Endless`)と core `intake.rs:192` の単体テストがメモリの読み手で確かめ、形 B の影響を受けない。

## 5. 足すテスト

test-after(各項目の実装のコミットの後に、テストのコミット)。各テストは、正しい場合と、境界または誤りの場合を 2 つ以上確かめる(性能のテストは、時間と結果の中身を確かめる)。

### 5.1 一覧

| ID | 場所 | 確かめること | 件数 |
|----|------|--------------|------|
| N1 | `crates/core/tests/definition_perf.rs`(新規) | 性能 P1(§5.2)。結果: 定義は正しく読め、`[pipeline.env]` は 10,000 項目で書いた順(`K0` … `K9999`)。時間を測るのは、この最初の読み(10,000 項目の正しい定義の `parse_definition`)だけ。続けて(時間は測らず)、同じ定義の `[pipeline.env]` の最後の項目(`K9999`)の次の行に `k0 = "x"` を足した定義を読むと、`envNameCaseConflict` が 1 件だけ(大文字小文字の検査が働く) | 1 |
| N2 | 同上 | 性能 P2。結果: 誤りは 39,998 件で、すべて `duplicateTransition`(`success` の 2 個目以降 19,999 件と `warning` の 2 個目以降 19,999 件。最初の `success` と最初の `warning` は誤りでない)。誤りは位置の順 | 1 |
| N3 | `crates/core/tests/env_refs_perf.rs`(新規) | 性能 P3。結果: `env_file_refs(def, None)` は 10,000 個で、`[pipeline].env_file` に書いた順。`env_file_refs(def, Some("a"))` も同じ 10,000 個(ジョブの記述は全部重複)。ジョブの記述に新しい 1 個を足すと末尾に 1 個増える | 1 |
| N4 | `crates/converter/tests/convert_ac.rs`(`#[cfg(unix)]`) | 書き手のいない名前付きパイプ(`mkfifo`)を `convert` に渡すと、別のスレッドで 5 秒以内に返り、種類は `unreadable`、理由は `通常のファイルではありません`(前例 runtime `env_loader.rs` の `名前付きパイプの_env_は開かずに_unreadable`)。直す前の木では 5 秒で返らず落ちる | 1 |
| N5 | `crates/runtime/tests/state_read.rs`(`#[cfg(unix)]`) | ステートのパスが書き手のいない名前付きパイプのとき、`read_state` と `read_state_raw` が 5 秒以内に `unreadable` を返す。`reset_all_states` は 5 秒以内に返り、名前付きパイプ(`<定義>.<run-id>.state` の名前)を消さず、`kept` にも入れない(今と同じ)。あわせて、読む権限の無い(mode 000)1 MiB 超のステートは `unreadable`(`tooLarge` ではない。root などで開けてしまうときは、この確認だけを飛ばす。前例 runtime `env_loader.rs` の N11、core `intake.rs` の `権限のない_1_mib_超のファイルは_unreadable`)。直す前の木では `read_state` が名前付きパイプで返らず落ちる(権限の確認は直す前も通る) | 1 |
| N6 | `crates/runtime/src/split.rs` の単体テスト(`cfg` なし) | `'  '`、`"\t"`、`'  ' x`、`"  " x` は `Empty`。`' a'` は `program = " a"`(空白だけでない先頭の語は今のまま)。直す前の木では落ちる | 1 |
| N7 | `crates/cli/tests/flow_validate.rs` | `command = "'  '"` の定義は、`validate` が 64 で `command is empty`(英語のロケール)とジョブ ID を出し、入力の値(空白)を含めない。`run a` も 64 で、ジョブを起動しない(`run_calls == 0`)。直す前の木では `validate` が 0 で落ちる | 1 |
| N8 | `crates/runtime/src/bounded_read.rs` の単体テスト(`cfg` なし) | 開いたあとの読み取りが、報告された大きさより長く続いても(終わりのない読み手 `io::repeat`)、上限 + 1 バイトで止まり `TooLarge` を返す。上限ちょうどは受理、上限 + 1 は `TooLarge`。ファイルを使わずに確かめるため、`read_regular_file` の 4 の段を、読み手と上限を受け取る小さな関数(`read_opened<R: Read>(reader: R, limit: u64) -> Result<Vec<u8>, ReadFailure>`)に分け、それを呼ぶ。★ T23 で `/dev/zero` のテストが「開かずに `unreadable`」に変わったあとも、「終わりのない入力でも読み止める」性質をこのテストで保つ(移した `上限_1_つぶんだけ読む` は `read_limited` が `上限 + 1` バイトを返すことだけを確かめ、`TooLarge` への写しは確かめないので、それだけでは足りない) | 1 |

- 合計の見込み: **+8 件**。消すテスト **−4 件**(§4.6)。実装で増減したら、その数と理由を記録する。

### 5.2 性能の基準(Q6: A)

測り方と直す前の結果は `inventory.md` §0.2(debug、5 回の中央値)。各テストは、同じ処理を **5 回測った中央値** を上限と比べ(core の `tests/common` の `median`)、測った値を `eprintln!` で出す(前回の `env_parse_perf.rs` などと同じ形)。上限は、前回の D6 と同じ考え方で、**直した後の見積もりの約 5 倍(丸めた数)**の余裕のある上限にする(D9)。

| テスト | 測る処理 | 入力の大きさ(根拠は `inventory.md` §0.2) | 直す前 | 直した後の見積もり | 合格の上限 | 直した後の余裕 | 直す前の形との比 |
|--------|----------|------------------------------------------|--------|--------------------|------------|----------------|------------------|
| P1(N1) | `parse_definition`(`[pipeline.env]` の別々の名前) | 10,000 項目(`MAX_ENV_ENTRIES` にそろえる。118,977 バイト) | 618 ms | ≒ 72 ms | **350 ms** | 4.9 倍 | 手元で超える(1.77)。遅い CI では当てにならない |
| P2(N2) | `parse_definition`(1 つのジョブに `success` 20,000 個、`warning` 20,000 個の遷移) | 1 MiB に収まる最大に近い(960,120 バイト) | 2,115 ms | ≒ 324 ms | **1,600 ms** | 4.9 倍 | 手元でわずかに超える(1.32)。当てにならない |
| P3(N3) | `env_file_refs(def, None)`(`[pipeline].env_file` 10,000 個 + ジョブに同じ 10,000 個) | 10,000 個(`MAX_ENV_ENTRIES` にそろえる) | 630 ms | ≒ 7 ms | **40 ms** | 5.7 倍(数 ms の揺れを見込んで丸めた) | 手元で大きく超える(15.8) |

- 余裕の考え方: 手元(M2)より遅い CI の機械(GitHub Actions の ubuntu・macos・windows、debug。`coverage` は計測つき)で落ちないことを優先する。**大きな退行(見積もりの 5 倍を超える遅れ)だけを止める**。
- CI の `coverage`(計測つき、debug、ほかのテストと並行)での余裕は、手元では測れない。実装の後に CI(`test` の 3 つの OS と `coverage`)で P1〜P3 が出した時間(`eprintln!` の行)を記録し、上限との余裕が 3 倍を切っていたら、上限を上げる(上げた値と理由を記録する)。
- **直す前の形に戻ったことは、P1・P2 では確実には検知できない**(手元で 1.3〜1.8 倍超えるだけで、機械の速さによっては通る)。P3 は直す前の時間が上限の約 16 倍なので、戻れば落ちる見込み。改善が保たれていることは、テストの合否ではなく、直す前と後の時間の記録で示す。
- 前後の記録: 各テストを直す前の木(直すコミットの 1 つ前)でも走らせた時間と、直した後の時間を、code-summary(コード生成の記録)と test-results(ビルドとテストの記録)に書く。1h-4(`-v` の図。§0.2 の M4)と 1h-5(`record_skip`。M5)は、テストを足さず(D8)、同じ場面を一時のテストで前後に測って記録する(測ったら消す)。
- 今ある性能のテスト(`plan_perf.rs`、`graph_perf.rs`、`flow_perf.rs`、`env_merge_perf.rs`、`plan_simulate_perf.rs`、`graph_lookup_perf.rs`、`env_parse_perf.rs`)と、起動の性能(`nfr_perf.rs`。警告のみ)の扱いは変えない。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 115 − 2(§4.6)= **113** | 268 + 3(N1〜N3)= **271** | **384** | +1 |
| runtime | 68 + 2(N6、N8)= **70** | 127 + 1(N5)= **128** | **198** | +3 |
| converter | 54 − 1(§4.6)= **53** | 64 − 1(§4.6)+ 1(N4)+ 1 無視 = **64 + 1 無視** | **117 + 1 無視** | −1 |
| cli | 0 | 461 + 1(N7)= **462** | **462** | +1 |
| 全体 | | | **1,161 通過、1 無視、0 失敗** | +4(+8 − 4) |

ファイルごと: core `definition_errors.rs` 28(変わらない)、新規の `definition_perf.rs` 2、`env_refs_perf.rs` 1。runtime `state_read.rs` 15(UNIX。Windows は 14。★ T23 は名前が変わるだけで件数は同じ)。converter `read_resolve.rs` 28、`convert_ac.rs` 10(UNIX。Windows は 9)。cli `flow_validate.rs` 13。ほかのファイルの件数は変わらない。基準と見込みは macOS の件数(N4・N5 は Windows では数えない)。

## 7. 確かめ方

- 段の前と後で、`cargo test --locked --workspace` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「基準 1,157 − 消したテスト 4 + 足したテスト 8 = 1,161」に一致すること [Q1] [Q2] [Q6]。
- テストのファイルの変更は §4 の T 表、§4.6 の消すテスト、§5 の N 表だけ。段ごとに `git diff --stat -- 'crates/*/tests'` と、src の単体テストの差分を見る。テスト名が変わっていないことは、段の前後で `cargo test --locked --workspace -- --list 2>/dev/null | sort` の差が、§4.6 の 4 つの削除、N1〜N8 の追加、★ T23 の 1 つの名前の変更(`終わりのない入力でも上限で止まる` → `文字デバイスへのリンクのステートは開かずに_unreadable`。段 2 のコミット 40)だけであることで確かめる(T4 の `上限_1_つぶんだけ読む` はモジュールのパスだけが `state::reader::tests` から `bounded_read::tests` に変わる。名前は同じ)。
- 段 1 の完了: `inventory.md` の各項目の「完了の判定」の検索を 1 つずつ行う [Q1]。たとえば:
  - 1a: `grep -n "JobDefinition" crates/runtime/src/lib.rs`、`grep -n "U3" crates/cli/src/report/messages.rs`、`grep -n "直接組み立てて" -r crates/cli/src`(どれも 0 件)
  - 1b: `grep -rn "apply_interrupt_in\|fn interrupted_signal\|fn command_line" crates docs/spec`(0 件)、`grep -rn "\.names()" crates/core`(0 件)、`grep -rn "pub(crate) fn classify_exit_code" crates`(0 件)
  - 1c: `grep -n "fs::rename" -r crates/runtime/src`(`temp_file.rs` の 1 件)、`grep -rn "read_to_end" crates/runtime/src`(`bounded_read.rs` の 1 件)、`grep -rn "fn read_state_file_raw\|fn parse_state_for\b" crates/runtime/src`(0 件)
  - 1d: `grep -rn "job: &JobDefinition" crates/cli/src/deps.rs crates/cli/tests/common/mod.rs`(0 件)
  - 1e・1f: `cargo build --workspace --all-targets --locked` と `cargo clippy … -D warnings` が通ること、`inventory.md` §8 の 1e の検索で外からの参照が 0 件のままであること(T5 の後)
  - 1g: `grep -rn "escape_path" crates`(0 件)、`grep -rn "with_run_id" crates/cli`(0 件)、`grep -n "line: 0\|pub reason: String" crates/converter/src`(0 件)
  - 1h: `grep -n "seen.iter().any\|out.iter().any\|out.contains" crates/core/src/definition/walk crates/core/src/envfile/mod.rs -r`(0 件)、`grep -n "filter(|e| e.from_job" crates/cli/src/report/graph_text.rs`(0 件)、`grep -n "\.cloned()" crates/cli/src/step.rs`(0 件)
  - 1i: `grep -c "warning_exit_codes:" crates/core/src/types.rs crates/core/src/consistency.rs crates/core/src/flow/*.rs`(単体テストの手組みが無い。`test_support.rs` の 1 件と製品のコードを除く)、`grep -rn "check_command(&j.command)" crates/runtime`(0 件)
- 段 1 の性能: N1〜N3 が通り、直す前と直した後の時間(1h-4・1h-5 は一時の測定)を記録する(D9。上限は大きな退行だけを止めるので、改善の確かめは記録で行う)。
- 段 2: N4・N5 が直す前の木で落ち(5 秒で返らない)、直した後で通ることを記録する。★ T23 が、直す前の期待(`TooLarge`)のままでは直した後に落ち、変えた期待(`Unreadable`)で通ることを記録する。N8 が通ること(直す前も後も、`read_opened` の性質として通る)を記録する。N6・N7 が直す前の木で落ち、直した後で通ることを記録する。仕様書の「v0.9.7 までは…」の注記が rust 06・07・09、language-agnostic 06・07・09 にある。
- 版: `grep -rn "0\.9\.7" crates Cargo.lock` が 0 件(このため、この intent で書くコードのコメントには「v0.9.7 までは」などの版の番号を書かない。版の注記は仕様書だけに書く。inventory 1a-1)、仕様書の `0.9.7` が「v0.9.7 以降」「v0.9.7 までは」の注記だけ。`cargo run -q -p conductor-cli --bin conductor -- --version` が `conductor 0.9.8`。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分は版の 4 行だけ)[memory:M5]。`unsafe` と src の `cfg` を足さない(runtime と cli の `nfr_source.rs`)。converter の `Cargo.toml` に core を足さない。README と `docs-check` の語句に触れる変更は無い。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

段 1(36 コミット):

1. runtime: 古くなった説明(1a-1〜1a-5、1a-9)
2. cli: 古くなった説明と守りの理由(1a-6〜1a-8、1a-10〜1a-12)
3. core: `apply_interrupt_in` を消す(1b-1。仕様書 rust 05:172)
4. core: `EffectiveEnv` の `job_id` と `names()` を消す(1b-2。T1a、★ T1b。仕様書 rust・language-agnostic 05)
5. core のテスト: 補助の使わない引数を外す(1b-3。T2、T3)
6. core: 模擬の分類を `classify_and_decide` の結果から取り、`classify_exit_code`・`ExitCodeClass` を非公開に(1b-4。仕様書 rust 05)
7. core: `decide_before_run` の常に真の条件(1b-5)
8. core: 記録の `hint` を `Option<usize>` にそろえ、`From` の書き方を直す(1c-4c、1c-4d。仕様書 rust 05:884)
9. cli: `interrupted_signal` を消す(1b-6)
10. runtime: `command_line` を `check_command` に畳む(1b-7。仕様書 rust 07)
11. runtime: `temp_file::replace_via_temp`(1c-1。仕様書 rust 02・06)
12. runtime: ステートの読み込みの段をなくす(1c-2 の前半。T4 の `parse_state`)
13. runtime: `bounded_read.rs`(`read_limited` を移し、形 B の `read_regular_file` と、その 4 の段の `read_opened`。`.env` が使う)(1c-2 の後半。T4 の移動。仕様書 rust 02)。`read_limited` は、ステートの `read_bounded`(`state/reader.rs`)が段 2 まで呼ぶので、このコミットでは `pub(crate)` のまま
14. core: `build_retry` は値を移し、欠けたらジョブを組み立てない(1c-3)
15. core・converter: ごく小さい整理(1c-4a、1c-4b、1c-4e)
16. cli: `Runtime::run_job` から `job` を外す(1d。T15 の偽物の `run_job`。仕様書 rust 02)
17. core: `flow` の下位モジュールと `EXIT_*` を非公開に(1e)
18. core: `plan::simulate`、`envfile::{merge, parse}`、`graph::{edges, layout, overlay}` と定数を非公開に(1e。T5。仕様書 rust 02・03・05・10)
19. runtime: `pub(crate)` を狭める(1f。`read_limited` を除く 10 個。`read_limited` はコミット 40)
20. cli: `escape_path` を `escape_full` に(1g-1。T16、T17。仕様書 rust 02・08・11)
21. cli: run ID の有無を `StateContext` で渡す(1g-2。T18。仕様書 rust 02)
22. converter: 行番号を `Option<usize>` に(1g-3)
23. converter・cli: `ConversionError.reason` を `&'static str` に(1g-4。T19。仕様書 rust 02・10)
24. core: 定義の `env` の表の大文字小文字の検査を `HashSet` に(1h-1)
25. core: 遷移の重複の確認を種類ごとの真偽値に(1h-2)
26. core: `env_file_refs` の重複の確認を `HashSet` に(1h-3)
27. core のテスト: N1、N2、N3(直す前の木でも走らせ、時間を記録)
28. cli: `-v` の図の辺を出どころのジョブで 1 回まとめる(1h-4。前後の時間を一時に測って記録)
29. cli: `record_skip` の土台を借りる(1h-5。同上)
30. core のテスト: `test_support`(1i-1。T6)
31. cli のテスト: `FakeEnv::for_language`、`run_with_fakes`、`state_error` の共有(1i-2。T15 の残り、T20、T21、T24)
32. runtime のテスト: `def`・`put`・`paths` を `common` に(1i-3。T9〜T13)
33. runtime のテスト: `run_job` の名残を `command_line` に(1i-5。T7、T8、T13)
34. core のテスト: 深さの境界の重複した単体テストを消す(1i-4。§4.6 の 1・2 行目)
35. converter のテスト: 深さの重複したテストを消す(1i-4。§4.6 の 3・4 行目)
36. 仕様書: rust 11 の性能の表とテストのファイルの表(新しい 2 ファイル。消したテスト)、language-agnostic 11 の性能の表(目安の行)

段 2(10 コミット):

37. converter: 読み込みを形 B に(2a)
38. converter のテスト: N4(直す前の木で落ちることを記録)
39. 仕様書: converter の取り込み(rust・language-agnostic 09、language-agnostic 11。「v0.9.7 までは…」)
40. runtime: ステートの読み込みを `read_regular_file`(形 B)に(2b。★ T23 の期待と名前を同じコミットで変える。`read_bounded` が無くなるので、`read_limited` を `bounded_read.rs` の中だけの非公開にする(1f の残り))
41. runtime のテスト: N5、N8
42. 仕様書: ステートの読み込み(rust・language-agnostic 06、rust 02・03。「v0.9.7 までは…」)
43. runtime: 空白だけの先頭の語を `Empty` に(2c)
44. runtime・cli のテスト: N6、N7
45. 仕様書: 空白だけのコマンド名(rust・language-agnostic 07、rust 12、language-agnostic 11。「v0.9.7 までは…」)
46. 版を 0.9.8 にする(2d。`Cargo.toml` × 4、`Cargo.lock`、★ T22、仕様書の版の表記。`7239aa2` と同じ 22 ファイル)

合計 **46 コミット**ほど(実装で分けたりまとめたりしたら記録する)。タグとリリースはしない。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」。
- core と converter の src のテストの `cfg(unix)`(Q4: C)、計画の型で方式ごとの項目を結ぶこと(Q7: A)。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- タグとリリース(終わった後に頼まれたら行う)。

## 10. 決めてほしい点

D1〜D12 は、ユーザーがすべて推奨のとおりに決めた(2026-11-23)。D10 は「重複したテストを消す方針」の決定で、個々のテストの削除(§4.6 の 4 つ)は、★ T1b・T22・T23 と同じく、設計の承認で認めてもらう(承認待ち)。

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | cli の偽物で、`env` に `CONDUCTOR_JOB_ID` が無いとき(1d) | **パニックで止める**(「CONDUCTOR_JOB_ID がありません(core の merge_env がいつも足す)」) | 空の ID や `?` で続ける | 本番の流れでは起こらない。起きたら core か cli の契約が崩れたので、テストが落ちて気付けるほうがよい。続けると、台本の引き当てが外れて別の理由で落ち、原因が分かりにくい | 決定: パニックで止める(推奨のとおり。ユーザー、2026-11-23) |
| D2 | core のテストの補助の使わない引数(1b-3) | **`record.rs`・`next_step.rs`・`retry_flow.rs` の補助は引数を外し、呼ぶ所を直す。`tests/common/mod.rs` の `exited(job_id, code)` は残す** | (B)すべて外す(`flow_sequences.rs` の 17 か所も直す)、(C)すべて残す | 前者は 1 つのジョブの関数を確かめるテストで、ジョブ ID の引数は読みやすさにも効かない。`flow_sequences.rs` は複数のジョブの並びを読むテストで、「どのジョブの結果か」が読めることに意味があり、前回も意図して残した | 決定: `record.rs`・`next_step.rs`・`retry_flow.rs` の補助は引数を外し、`tests/common/mod.rs` の `exited` は残す(推奨のとおり。ユーザー、2026-11-23) |
| D3 | `build_retry` で値が欠けたとき(1c-3) | **値を移し、欠けていれば、遷移と同じくジョブを組み立てない**(全体が「定義を組み立てられません」の防御に落ちる) | 値を移すだけにし、欠けたら今と同じく黙って `None`(リトライなし) | どちらも到達しない(欠けるのは誤りを記録したときだけで、そのときは組み立てに来ない)。推奨は、もし来ても「書いたリトライが黙って消える」ことを防ぎ、ほかの項目と同じ扱いになる | 決定: 値を移し、欠けていればジョブを組み立てない(推奨のとおり。ユーザー、2026-11-23) |
| D4 | 記録の `hint` の型(1c-4d) | **`apply_outcome_in`・`apply_skip_in` とも `Option<usize>`**(`None` = 先頭から。公開の関数) | 両方 `usize`(公開の関数は 0)。ただし `apply_outcome_in` のスキップの並びの「直前の次から探す」を、公開の関数でも行うことになる | 推奨は、公開の関数の今の動き(ジョブ ID が重なる入力でも、最初に一致したもの)を型で保てる。代案は、重なる入力で結果が変わりうる | 決定: 両方 `Option<usize>`(推奨のとおり。ユーザー、2026-11-23) |
| D5 | run ID の有無の渡し方(1g-2) | **`StateContext`(`Read`/`ReadRunId`)で渡す**(`ConsistencyMismatch.context`、`reset_hint(StateContext)`、`for_read(Option<&RunId>)`) | 案内の種類だけを表す小さな列挙型(`ResetHint { Plain, RunId }`)を新しく作る | 推奨は、初期の記述のとおりで、型を増やさない。`ConsistencyMismatch` に `JobExecuted` などを入れても意味は無いが、作る所は `ctx.rs` の 1 か所で、`for_read` を通すので `Read`/`ReadRunId` しか入らない | 決定: `StateContext` で渡す(推奨のとおり。ユーザー、2026-11-23) |
| D6 | `escape_path` の新しい名前(1g-1) | **`escape_full`**(切らずに、全体をエスケープする) | `escape_untruncated`、`escape_text` | 対になる `escape_name`(64 文字で切る)と並べて読んだとき、「全部出す」が伝わる短い名前。仕様書の 08 に「v0.9.7 までは `escape_path`」と 1 回書く | 決定: `escape_full`(推奨のとおり。ユーザー、2026-11-23) |
| D7 | ステートのパスが名前付きパイプ・ディレクトリのときの理由の文(2b) | **どちらも今の `unreadable` の理由「開く、または読み取ることができません」**(新しい理由の文を作らない) | 定義・converter と同じ「通常のファイルではありません」を足す | 推奨は、ディレクトリのステートの表示が今と 1 文字も変わらない(既存のテストの期待も変わらない)。代案は、原因が分かりやすいが、ディレクトリの表示が変わり、★ が増える | 決定: 今の理由「開く、または読み取ることができません」(推奨のとおり。ユーザー、2026-11-23) |
| D8 | 計算量の改善の小さい 2 つ(1h-4 の `-v` の図、1h-5 の `record_skip`)の性能のテスト | **足さない**。前後の時間を一時のテストで測って記録する | 足す(`-v` の図: ジョブ 1,000 個で上限 35 ms など) | 直す前でも `-v` の図は 21 ms、`record_skip` の部分は 165 ms(→ 91 ms。ファイルの書き込みが支配的)で、5 倍の上限では直す前の形も通り、テストが何も守らない。初期の記述も「改善が小さいものは前後の時間だけ」を認めている | 決定: 足さず、前後の時間を記録する(推奨のとおり。ユーザー、2026-11-23) |
| D9 | 性能のテストの上限(§5.2) | **直した後の見積もりの約 5 倍**: P1 350 ms、P2 1,600 ms、P3 40 ms(前回の D6 と同じ考え方) | P3 を 35 ms(ちょうど 5 倍)にする | P3 は数 ms の処理で、遅い CI の揺れが比で大きくなるので、少し丸めた。P1・P2 は直す前の形に戻っても通りうる(記録で示す)ことを承知で置く | 決定: P1 350 ms、P2 1,600 ms、P3 40 ms(推奨のとおり。ユーザー、2026-11-23) |
| D10 | 消すテスト(§4.6 の 4 つ) | **表のとおり消す**(core の単体 2、converter の単体 1 と統合 1) | 消さずに、補助の置き換えだけにする(深さの境界の重複は残る) | 消すテストが確かめていた点は、行の番号を除き、残るテストが同じかより強く確かめる。行の番号は別の単体テストが確かめる | 決定: 重複したテストを消す方針(推奨のとおり。ユーザー、2026-11-23)。個々のテストの削除(§4.6 の 4 つ)は設計の承認で認めてもらう(承認待ち) |
| D11 | runtime の `load_env_files` の `seen` の線形の探索 | **今回は範囲の外**(前回の決定のまま) | 1h-3 と同じく `HashMap` にする(テストは足さない) | 初期の記述に無い。異なる記述ごとにファイルの読み取りが要り、計算量の改善の効果が見えにくい。入れるなら小さな 1 コミットで済む | 決定: 今回は範囲の外(推奨のとおり。ユーザー、2026-11-23) |
| D12 | cli の `common::state_error` の理由の文(1i-2c) | **ASCII の `"reason"` にして、`report_messages.rs` と共有する** | 共有せず、`report_messages.rs` の補助を残す(二重が残る) | 理由の文を確かめるテストは無い(`grep` で確かめた)。英語の表示を ASCII と確かめるテストがあるので、共有するには ASCII が要る | 決定: ASCII の `"reason"` にして共有する(推奨のとおり。ユーザー、2026-11-23) |

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-23)**: §4 の T 表の置き換えの書き方(T1a の補助 `origins`、T20 の 1 行の包み)、★ T1b の確かめ方の置き換え、★ T22(版)、★ T23(期待を `Unreadable` に、テスト名を `文字デバイスへのリンクのステートは開かずに_unreadable` に。§4.7)、§4.6 の 4 つのテストの削除、T4 の単体テストのモジュールの移動(名前と中身は同じ)。

## 11. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 段 1(動きを変えない整理): 説明が実装と一致、使われないもの・二重のものが無い、公開範囲が狭まる、名前と型が使い方に合う。cli の `Runtime::run_job` から `job` を外す。項目ごとの場所・目標・判定の一覧 | BR1.1〜BR1.6、BR1.8、BR9.1 |
| SM2 | 計算量: 二乗の項目が線形に近い時間で終わる。性能のテストの上限は直した後の見込みの約 5 倍で、直す前と後の時間を記録する | BR1.7、BR2.1〜BR2.4、BR9.1 |
| SM3 | 段 2: converter とステートの読み込みを形 B に(名前付きパイプで止まらない。ほかの場合の種類は変わらない)、空白だけのコマンド名を定義の誤りに。仕様書に「v0.9.7 までは…」、値を含めない性質を保つ | BR3.1、BR3.2、BR3.3、BR8.1 |
| SM4 | 版: 最後のコミットで 0.9.8(タグとリリースはしない) | BR3.4 |
| SM5 | 既存のテストの扱い(T 表、消すテストの対の表、★ を検索で網羅して承認)と件数(基準 − 消したテスト + 足したテスト) | BR1.8、BR1.9、BR8.1、BR9.2、BR9.3 |
| SM6 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
