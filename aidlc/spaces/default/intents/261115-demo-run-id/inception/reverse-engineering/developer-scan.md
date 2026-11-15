## Developer Code Scan Results

対象: 焦点を絞った走査(brownfield)。意図 `demo-run-id`(デモを、実装済みの run ID(v0.5.0)に合わせて更新する)。HEAD は 1d8a46119c9ad5e5cc03726cb38ab23718b3e7ea。実行ファイルの版は `crates/cli/Cargo.toml:3` の `0.5.0`。

### Scan Coverage
- **Analyzed deeply**:
  - `demo/`(`run-demo.sh`、`conductor.toml`、`scripts/*.sh`、`README.md`、`windows/run-demo.cmd`、`windows/conductor.toml`、`windows/scripts/*.cmd`)
  - `crates/cli/tests/windows_demo.rs`(全体)、`crates/cli/tests/bin_support/mod.rs`(`conductor`・`run_limited`・`text` の部分)
  - `crates/cli/src/report/messages.rs`(USAGE、`definition_valid`、`RESET_HINT_RUN_ID`、`RESET_ALL_KEPT`、`WATCH_RUN_ID`、`usage_kind`)、`crates/cli/src/report/mod.rs`(`usage_text`、`ResetAllKept`、`DefinitionValid`)、`crates/cli/src/report/watch.rs:95-130`
  - `crates/cli/src/args.rs`(`--run-id` と `--all` の検査、`resolve_run_id`)、`crates/cli/src/aux_commands.rs`(`run_watch`・`run_reset`)、`crates/cli/src/ctx.rs`(run ID の決定、`aux_run_id`)
  - `crates/core/src/run_id.rs`、`crates/runtime/src/state/reset_all.rs`、`crates/runtime/src/state/reader.rs`(`TOP_KEYS`)・`writer.rs`(`run_id` の直列化)・`types.rs`(`StateLocation::of`)
  - `README.md` 全体(使い方、定義ファイル、終了コード、ステートファイルの場所、run ID の節)
  - 既存の CodeKB 9 成果物(読み、更新の土台にした)
- **Skimmed only**: `crates/cli/tests/binary_run_id.rs`・`flow_run_id.rs`(並行実行と run ID の期待の確認だけ)、`crates/runtime/src/lib.rs` の表、`crates/cli/src/pipeline.rs` の run ID の受け渡し、`crates/core/src/definition/walk.rs` の `run_id_env` の検査、そのほかの `crates/*/tests/`。
- **Left out**: `crates/converter/`(浅く: 一覧のみ)、`.github/`(走査せず。CI は前回の記録を引き継ぐ)、`.claude/`・`aidlc/`・`.idea/`・`target/`・`local-docs/`(範囲外)。

### Packages Found
- conductor-cli — bin+lib — Rust — 実行ファイル `conductor`、引数・流れ・表示(版 0.5.0)
- conductor-core — lib — Rust — 定義の検証、フローの判断、`RunId`(`run_id.rs`)
- conductor-runtime — lib — Rust — ジョブの起動、ステートの読み書き、`reset_all_states`
- conductor-converter — lib — Rust — YAML と TOML の変換(今回は走査対象外)
- demo — スクリプト — sh / cmd — デモ(Rust のパッケージではない)

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`)
- **Config Files**: `Cargo.toml`、`Cargo.lock`
- **Build Dependencies**: cli → core・runtime・converter、runtime → core

### APIs Discovered
- CLI — `crates/cli/src/args.rs` — 6 コマンド。`run`・`pipeline`・`watch`・`reset` が `--run-id`、`reset` だけが `--all` を取る。
- 環境変数 — 定義の `[pipeline].run_id_env`(既定 `CONDUCTOR_RUN_ID`)。

### Frameworks & Libraries
- 今回は再確認していない(前回の記録: lexopt、signal-hook、ctrlc、libc、toml、saphyr-parser、thiserror、tempfile)。

### Test Coverage
- **Test Directories**: `crates/*/tests/`、各 `src/*.rs` の `mod tests`
- **Test Frameworks**: 標準の `cargo test`、`tempfile`
- **Coverage Config**: 前回の記録(CI で cargo-llvm-cov、行 80%)。今回は未確認。

### Code Quality Indicators
- **Linting**: rustfmt、clippy(前回の記録)
- **CI/CD**: `.github/workflows/`(未走査。前回の記録)
- **Documentation**: README.md(run ID の説明あり)、`demo/README.md`(run ID の記述は、まだない)

### Technical Debt Signals
- デモに run ID の場面がない。`demo/README.md`・両デモ・`windows_demo.rs` のどれも `--run-id`・`reset --all`・`validate` に触れていない(`grep` で確認済み)。
- UNIX 版デモは `windows_demo.rs` のような自動テストを持たない(`crates/cli/tests/` に UNIX 用のデモのテストはない)。

## Handoff Summary

### (1) デモの構造

#### `demo/run-demo.sh`(POSIX `sh`。`set -u`。`demo/run-demo.sh:20`)
- 前提と環境変数: `here`・`root`(`:21-22`)。`CONDUCTOR`(未指定なら `$root/target/release/conductor`。無ければ `cargo build --release --locked --bin conductor`。`:24-30`)。`DEMO_LANG`(既定 `ja_JP.UTF-8`、`LANG` に設定して export。`:33`。`LC_ALL`・`LC_MESSAGES` は消さない)。`DEMO_PAUSE`(`=1` で Enter を待つ。`:50-52`)。`DEMO_<JOB>_EXIT` と `DEMO_SLEEP`(既定 0.5。`scripts/*.sh:18`)はジョブのスクリプトが読む。
- 一時ディレクトリ: `work=$(mktemp -d)`、`trap 'rm -rf "$work"' EXIT`、`conductor.toml` と `scripts/` を写して `cd`(`:35-39`)。ステートファイルは、この中の `conductor.toml.state`(run ID ありなら `conductor.toml.<ID>.state`)に出来て、終了時にディレクトリごと消える。デモ中に明示的に消すのは `reset`(場面 5)だけ。
- 補助関数: `scene()`(太字の見出し。`:41-43`)、`cmd()`(シアン色で `$ <コマンド>` を表示 → `"$@"` を実行 → `code=$?` → 黄色で `→ 終了コード N`。`:44-49`)、`pause()`(`:50-52`)。`cmd` が表示するのは、`"$CONDUCTOR"` の展開後のフルパスで始まる行(`$ /…/target/release/conductor pipeline -v nightly`)で、環境変数の指定は表示されない。
- 場面(`scene` の呼び出し): 0 定義の表示(`cat conductor.toml`。`:54`)、1 全部成功(`pipeline -v nightly`。`:58-59`)、2 警告で分岐(`DEMO_EXTRACT_EXIT=1`。`:62-63`)、3 エラーで停止(`DEMO_LOAD_EXIT=2`、続けて `watch`。`:66-70`)、4 再開(同じコマンド、続けて `watch` が空。`:73-77`)、5 個別実行 `run -v extract` と `reset`(`:80-82`)、6 `convert conductor.toml --to yaml`(`:85-90`。最後に「デモは以上です。」)。`validate` は使わない。
- 1 コマンドだけの環境変数の限定: サブシェル `( DEMO_X_EXIT=N; export DEMO_X_EXIT; cmd "$CONDUCTOR" ... )`(`:63`、`:67`)。`VAR=値 cmd ...` の前置きは、`cmd` が関数なので使わない(POSIX では、関数呼び出しへの前置きの代入が呼び出し後に残るかが未規定)。
- 終了コード: `cmd` が毎回 `→ 終了コード N` を出す。サブシェルを使った場面でも、`cmd` の出力はサブシェルの中で出る。
- UNIX 固有: 色(ANSI の `\033[…`、`:42`・`:45`・`:48`)、`mktemp`、`trap`、`sleep` の小数、`sh scripts/*.sh`。`conductor watch` の色は、標準出力が端末で `NO_COLOR` が未設定のときに付く(`README.md:278`)ので、端末ではデモの `watch` も色付きになる。

#### `demo/conductor.toml`
- `[pipeline] name = "nightly"`、`description`。`run_id_env` なし(`:3-5`)。ジョブ 4 つ: extract(`command = "sh scripts/extract.sh"`、`warning_exit_codes = [1]`、`on = "warning"` → `next = "notify"`。`:8-15`)、transform(`:17-19`)、load(`:21-23`)、notify(配列 `["sh", "scripts/notify.sh"]`。`:25-28`)。
- `scripts/*.sh`: 各 `[<job>] 実行中...` を出し、`sleep "${DEMO_SLEEP:-0.5}"`、`[<job>] 終了コード N で終わります` を出して `DEMO_<JOB>_EXIT`(既定 0)で `exit`(`scripts/extract.sh:17-21` ほか)。ジョブは、親の環境を継承する(runtime は環境を変えない。`crates/runtime/src/runner.rs` に `.env(` なし)。`--run-id` は子へ渡らず、環境変数 `CONDUCTOR_RUN_ID` を export したときだけ、ジョブにも見える。

#### `demo/windows/run-demo.cmd`(UTF-8、CRLF。BOM なし)
- `@echo off`、`setlocal EnableExtensions`(`:22`)。`root`・`here`(`:24-25`)。`CONDUCTOR` 既定は `%root%\target\release\conductor.exe`、無ければ `cargo build`(`:27-39`)。
- 言語: `DEMO_LANG`(既定 `ja_JP.UTF-8`)を `LANG` に設定し、`LC_ALL`・`LC_MESSAGES` を空にする(`:42-45`)。UNIX 版と違い、上位の変数を消す。
- コードページ: `chcp` の出力から旧コードページを取り(`:48-50`)、`chcp 65001 >nul`(`:51`)。最後に戻す(`:106`)。
- 一時ディレクトリ: `%TEMP%\conductor-demo-%RANDOM%%RANDOM%`(`:53`)、`copy`・`xcopy`、`cd /d`(`:54-57`)。最後に `cd /d "%root%"`、`rmdir /s /q "%work%"`(`:104-105`)。`trap` に当たるものはなく、途中で Ctrl+C すると一時ディレクトリが残る。
- サブルーチン(`:110-129`): `:scene`(`echo === %~1 ===`)、`:runcmd`(`echo $ conductor %*`、`"%CONDUCTOR%" %*`、`set "code=%errorlevel%"`、`echo → 終了コード %code%`、`exit /b 0`)、`:pause`(`set /p`)。表示されるコマンドは、`conductor` と引数だけ(パスは出さない)。
- 場面は UNIX 版と同じ 0〜6(`:59-101`)。1 コマンドだけの環境変数: `setlocal` / `set "DEMO_X_EXIT=N"` / `call :runcmd ...` / `endlocal`(`:68-71`、`:75-78`)。場面 3 の `watch` は `endlocal` のあとで呼ぶ。
- Windows 固有: ジョブの待ちは `ping -n <秒+1> 127.0.0.1 >nul`(`scripts/extract.cmd:22-23`。`DEMO_SLEEP` は整数の秒、既定 1。`set /a wait_count=DEMO_SLEEP+1`)。ジョブの終了コードは `exit /b %DEMO_<JOB>_EXIT%`。定義の `command` は `'scripts\extract.cmd'`(TOML のリテラル文字列。バックスラッシュはそのまま)と、notify の配列 `["cmd", "/c", "scripts\\notify.cmd"]`(`demo/windows/conductor.toml:11`、`:29`)。色なし。

### (2) `crates/cli/tests/windows_demo.rs`(`:17` `#![cfg(windows)]`)
- 検証の方法: `run-demo.cmd` は実行しない。`demo/windows/` の `conductor.toml` と `scripts/{extract,transform,load,notify}.cmd` を一時ディレクトリへ写し(`Demo::new`。`:34-53`)、本物の `conductor`(`CARGO_BIN_EXE_conductor`)を、`bin_support::conductor(args)`(`LANG=C` で英語に固定、`LC_ALL`・`LC_MESSAGES`・`NO_COLOR` を外す。`bin_support/mod.rs:49-57`)で、そのディレクトリを `current_dir` にして起動する(`Demo::run`。`:56-69`)。`DEMO_SLEEP=0` を設定し、`DEMO_{EXTRACT,TRANSFORM,LOAD,NOTIFY}_EXIT` を外し、追加の環境変数を足す。`run_limited`(30 秒の上限、標準出力・標準エラーを集める)で実行し、`text()` で UTF-8 の文字列にする。
- 場面ごとの検証:
  - 定義(`:72-83`): `convert conductor.toml --to yaml` が 0、YAML に `id: extract|transform|load|notify`、`next: notify`、`warning_exit_codes`。
  - 場面 1(`:85-94`): `pipeline -v nightly` が 0。標準出力に `[extract|transform|load|notify] 実行中`(ジョブのスクリプトの日本語。言語の設定に依らない)。
  - 場面 2(`:96-108`): `DEMO_EXTRACT_EXIT=1` で 0。`[notify] 実行中` があり、`[transform]`・`[load]` はない。
  - 場面 3 と 4(`:110-123`): `DEMO_LOAD_EXIT=2` で終了コード 2。同じ一時ディレクトリで再実行すると 0 で、`[extract]`・`[transform]` が出ず、`[load]`・`[notify]` が出る。
  - 場面 5 と 6(`:125-141`): `run -v extract`、`reset`、`convert …` が、それぞれ 0。
- 新しい場面をテストする手がかり: 新しいテストは `demo.run(&[...], &[(環境変数, 値)])` の形で足せる。ステートの確認は、一時ディレクトリの中のファイルの有無(`demo.dir.path()`)か、`watch` の標準出力(`Run ID: <ID>`)で行う。同じ `Demo` の中でステートが続く(場面 3 → 4 と同じ)。`watch` は英語の固定(`LANG=C`)で、出力の表の見出し(`Pipeline:`、`Run ID:`)は、英語と日本語で `Run ID` だけが同じ(`Pipeline` は日本語だと `パイプライン`)。`reset --all` の「残した」警告は、標準エラーにあるので `output.stderr` を見る。
- 落とし穴: `Demo::run` は `CONDUCTOR_RUN_ID` を外さない。開発者や CI の環境に `CONDUCTOR_RUN_ID` があると、既存の場面も影響を受けるため、`env_remove("CONDUCTOR_RUN_ID")` を足す必要がある(定義の `run_id_env` を変えるなら、その名前も)。`validate` を足すなら `conductor validate -v` は標準エラーに 1 行を出す(`stderr` を見る)。`cfg(windows)` なので、macOS・Linux の開発機では、このテストは実行されない(CI の windows-2022 だけ)。

### (3) run ID 関連の表示(実装どおり。`messages.rs`・`report/mod.rs`・`watch.rs`)
接頭辞は、エラー `conductor: error: `(`messages.rs:48`)、警告 `conductor: warning: `(`:50`)、`-v` の行 `[conductor] `(`:52`)。
- **`watch` の `Run ID:` の行**(標準出力。英語・日本語とも同じ見出し `Run ID`。`watch.rs:110-111`、`messages.rs:331`): 先頭が `Pipeline: nightly`(日本語は `パイプライン: nightly`)、次の行が `Run ID: <ID>`、そのあとに表(`JOB  STATUS  NOTE` / `ジョブ  状態  補足`)。run ID がなければ、`Run ID` の行は出ない。ステートがないとき: `no state (nothing has run yet)` / `ステートはありません(まだ実行されていません)`(`messages.rs:332-335`)。run ID があっても、ステートがなければこの行になる(`Run ID` の行は出ない)。
- **`reset --all` の「残した」警告**(標準エラー。`report/mod.rs:265-271`): 1 行目 `conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):`(日本語: `conductor: warning: 読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):`)。続けて、ファイル名を 1 行ずつ、2 文字の字下げで(`  conductor.toml.bad.state`)。終了コードは 0。消すものも残すものもなければ、何も出さずに 0。成功しても、消したファイルの数は表示しない。
- **使い方の誤り**(標準エラーに、誤りの 1 行 + 空行 + 使い方の全文。終了コード 68。`usage_text`。`report/mod.rs:412-421`):
  - 規則に合わない `--run-id` の値: `conductor: error: invalid value: '--run-id'` / `conductor: error: 値が正しくありません: '--run-id'`(値は含めない。`args.rs:312`)。
  - 規則に合わない環境変数の値: `conductor: error: invalid value: 'run ID (environment variable)'`(日本語は、`値が正しくありません: 'run ID (environment variable)'`。対象の部分は英語の固定。`args.rs:334`)。
  - `--all` と `--run-id` の併用: `conductor: error: invalid value: '--all'`(`args.rs:256-258`)。`reset` 以外への `--all`: `conductor: error: option not allowed for this command: '--all'`(`args.rs:236-238`)。`validate`・`convert` への `--run-id`: `option not allowed for this command: '--run-id'`(`:229-235`)。`--run-id` の値がない: `option requires a value: '--run-id'`(`MissingValue`)。
  - 日本語: `このコマンドでは使えないオプションです: '--all'`、`オプションに値がありません: …`。
- **`validate -v`**(標準エラー 1 行。`messages.rs:103-111`): `[conductor] definition is valid (4 jobs)`(1 個のときは `1 job`)/ `[conductor] 定義は正しいです(ジョブ 4 個)`。`validate` は `--run-id` を受け付けず(68)、環境変数も見ない。`run_id_env` の型や形が誤りなら、定義の誤りで 64(`validate` も同じ。`walk.rs:237-243`)。
- ステートの食い違い・読めない: run ID つきなら、案内が `run `conductor reset --run-id <ID>` to discard the state` / `` `conductor reset --run-id <ID>` でステートを初期化できます``(`messages.rs:223-226`。終了コード 65)。run ID なしは `RESET_HINT`(`conductor reset`)。
- `pipeline -v` / `run -v` の行(`job=<id> …`、`transition: …`)は run ID を含まない(run ID の表示は `watch` だけ)。

### (4) run ID の意味(デモが守ること)
- 優先順位(`args.rs:320-343`、`ctx.rs:106-115`): `--run-id` があれば環境変数は見ない(空の `--run-id ""` は、環境変数があっても run ID なしに固定)。なければ、環境変数(名前は `[pipeline].run_id_env`、省略時 `CONDUCTOR_RUN_ID`)。未定義と空は run ID なし。
- 値の規則(`crates/core/src/run_id.rs:35-55`): `A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない。`run_id_env` の名前の形は `A-Za-z_` で始まり `A-Za-z0-9_` が続く(`:58-65`)。
- ステートのファイル名(`crates/runtime/src/types.rs:114-120`): run ID なし `<定義名>.state`、あり `<定義名>.<ID>.state`(例 `conductor.toml.run-1.state`)。中身には、run ID があるときだけ `run_id = "<ID>"`(`writer.rs:278`。`TOP_KEYS` は 10 個、`reader.rs:184`)。
- 再開: 同じ run ID で同じコマンドを呼ぶと、止まったジョブから再開する。完了した run のステートは、これまでどおり消える(run ID つきも)。別の run ID の再開位置は、互いに変わらない(`binary_run_id.rs:59-104` が、並行する 2 つと、片方だけの再開を確認している)。
- `watch`・`reset`(`--all` なし): `--run-id` があれば、定義を読まない。なければ、定義から `run_id_env` だけをゆるく読む(`aux_commands.rs:29-37`、`ctx.rs:303-315`)。環境変数に規則違反の値があれば、`watch`・`reset` も 68。
- `reset --all`(`reset_all.rs:1-12`): 環境変数を見ない。定義ファイルと同じディレクトリだけ(再帰しない)で、`<定義名>.state` と、名前の run ID が有効で中身の `run_id` が一致する `<定義名>.<ID>.state` を消す。残すもの(黙って): 一時ファイル `.<名前>.tmp.*`、別の定義のステート(中身に `run_id` がない、または違う)、ディレクトリ、シンボリックリンク。残して警告するもの: 読めない・壊れているファイル。ステートが 1 つもなくても 0。
- 注意: `--run-id` を与えたデモのステートは、完了すると消える。`reset --all` の場面を見せるには、複数の run ID を、エラーで止めたままにしておく必要がある(`DEMO_LOAD_EXIT=2` で止める)。
- 注意: 完了した run のステートは自動では消えない場合がある、という記述は、README にある(`README.md:235`)が、これは、止まったままの古いステートのこと。完了の run は消える。

### (5) 落とし穴
- **利用者の環境の `CONDUCTOR_RUN_ID`**: 両デモは、これを消さない。設定されていると、既存の場面 1〜5 のステートが、`conductor.toml.<ID>.state` に分かれてしまう(場面 3 → 4 の再開や `watch` は、同じ環境変数を見るので動くが、出力に `Run ID:` が現れる)。デモの冒頭で、`unset CONDUCTOR_RUN_ID`(sh)/ `set "CONDUCTOR_RUN_ID="`(cmd)にすると安全。同様に、`run_id_env` を変えた定義を使うなら、その名前の変数も消す。
- **cmd の環境変数**: `set X=1 & cmd` は、`1` のあとの空白が値に入る(`1 `)。既存どおり、`set "X=1"`(引用符で囲む)と `setlocal` / `endlocal` を使う。`call :runcmd` の中で `setlocal` が不要な範囲で閉じられるよう、`endlocal` を忘れない。`%*` は引用符を保つので、`--run-id ""` は `call :runcmd run -v --run-id "" extract` のまま渡せる。値に `%`・`^`・`&` を含めない(run ID の規則に合う値だけなら安全)。サブルーチンの中の `if (...)` で、`)` を含む文字列を `echo` するときは `^)` が要る(`run-demo.cmd:31` の例)。
- **環境変数の場面の表示**: `$ conductor pipeline …` の行は、環境変数の指定を出さない。`CONDUCTOR_RUN_ID=…` を見せたいなら、場面の前に自分で `echo` する(sh は `printf`、cmd は `echo`)。
- **並行実行**: sh は `cmd ... &` と `wait` で並行にできるが、2 つのジョブの出力が混ざり、順序が不定になる(`binary_run_id.rs:59` は並行で検証しているが、出力は見ていない)。cmd の `start /b` も同様に混ざり、`call :runcmd` のサブルーチンは `start` に渡せない(別の cmd プロセスで、`%CONDUCTOR%` などの変数を引き継ぐか、別途スクリプトが要る)。ステートのファイルは run ID ごとに別でも、デモの「画面の順序」と「`code` の変数」(sh の `cmd` は大域の `code` を使う)が壊れる。見せるなら、逐次(run ID `a` を失敗で止める → run ID `b` を失敗で止める → 2 つの `watch` → `a` だけ再開)が安定。`DEMO_SLEEP` の既定(sh 0.5 秒、cmd 1 秒)で、待ち時間も増える。
- **ロケール**: sh は `LANG` だけ設定し `LC_ALL`・`LC_MESSAGES` を消さない。利用者の `LC_ALL` が優先され、デモの `DEMO_LANG` が効かない場合がある(既存の問題)。cmd は 3 つとも整えている。`Run ID:` の見出しは、英語・日本語で同じ。エラーや警告の文言は、`DEMO_LANG` に従う。
- **cmd の制限**: 待ちは `ping`(整数の秒だけ)。`timeout` は標準入力のないとき失敗する。文字は、コードページを 65001 にして出し、`.cmd` の書き換えは UTF-8・CRLF のまま保つ(LF だと、ラベルや `call :` が壊れることがある)。`exit /b` の値を `%errorlevel%` で読む(`if errorlevel` の「以上」の意味と混ぜない)。`dir /b` の結果の並び、`ls` の並びは、名前の順(ロケールで変わりうる)。ステートファイルを見せるときは、`ls conductor.toml*`(sh)/ `dir /b conductor.toml*`(cmd)が使える(ほかに `scripts/` や一時ファイルが混ざらない)。
- **`reset --all` の対象**: 同じディレクトリの `conductor.toml.*.state` だけ。デモの一時ディレクトリには、ほかのステートはないので、「中身が違うので残す」場面は、手で作らない限り見せられない(壊れたファイルを `printf 'x' > conductor.toml.bad.state` で作ると、警告の場面を見せられる)。
- **大文字小文字を区別しないファイルシステム**(macOS・Windows): run ID を、大文字小文字だけで区別しない(`README.md:236`)。

### (6) `demo/README.md` の構成と新しい文章の置き場所
- 先頭(`:1-12`): 概要、UNIX 版のコマンド例(`DEMO_PAUSE`、`DEMO_LANG`、`CONDUCTOR`)、「一時ディレクトリで動くので…」。
- 場面の表(`:14-21`、列は「場面|見せること」。場面 1〜6)。ここに、run ID の場面の行を足す(場面の番号は、スクリプトの `scene` と揃える)。
- 箇条書き(`:23-24`): 定義・スクリプト・`DEMO_<ジョブ名>_EXIT`・`DEMO_SLEEP`、`notify` の配列形式。run ID 用の補足(`run_id_env`、`CONDUCTOR_RUN_ID`)はここ。
- 「## Windows 版」(`:26-53`): 場面は UNIX 版と同じと書いている(`:28`。場面を増やすなら、ここの文言「場面 0〜6」を直す)。コマンド例(`:30-45`)、PowerShell の注(`:47`)、箇条書き(`:49-53`: 定義、`DEMO_SLEEP`、色なし、`DEMO_LANG`、コードページ)。
- 場面の表の下に run ID の節を足すか、表に行を足すのが自然。`README.md:122` が `demo/README.md` を参照するだけ(リンクのみ)で、デモの場面の番号は書いていない。

### (7) デモに触れる既存のテストと検査
- `crates/cli/tests/windows_demo.rs`(`cfg(windows)`。上の (2))だけが、デモのファイルを開く。`grep -rIil demo crates README.md` の結果は、`windows_demo.rs`、`README.md`(リンクの 1 行)、`crates/converter/src/yaml_write.rs`(`name = "demo"` というテストの文字列で、デモのファイルとは無関係)。
- UNIX 版デモを検査するテストは、リポジトリ内にない。デモの出力を grep する検査も、ない(`demo/` を `grep` する `crates/` のテストは `windows_demo.rs` だけ)。
- CI(`.github/workflows/ci.yml` の docs-check など。未走査。前回の記録): README の必須語の検査と、`cargo test --workspace`(windows-2022 を含む)が、`windows_demo.rs` を走らせる。docs-check が `demo/README.md` を見るかは、今回は確認していない(要確認。`ci.yml` を読むこと)。
- `README.md` の記述との整合: `README.md:75-120`(使い方・例)、`:227-237`(run ID の節)が、デモの説明の元。デモの文言は、これと矛盾しないこと。
