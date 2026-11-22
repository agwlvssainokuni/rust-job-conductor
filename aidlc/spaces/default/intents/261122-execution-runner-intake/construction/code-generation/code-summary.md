# コード生成のまとめ(Code Summary): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

計画 `code-generation-plan.md` の Step 1〜11 を実装した。基準は HEAD d3e7fac(コードは v0.9.5)。コードと仕様書のコミットは 11 件(段 1: 4、段 2: 3、段 3: 4)。ほかに、この記録のコミットが 1 件。push、タグ、リリースはしていない。

## コミットの一覧

| # | Step | ハッシュ | 内容 |
|---|------|----------|------|
| 1 | 1 | `a5de482` | core・runtime・cli: `JobExecution::not_started` と `start_failed` を足して使う(項目の形と動きは同じ。テストは変えない) |
| 2 | 2 | `f00c7b9` | core・runtime・cli: `JobExecution` を `ExecutionResult` に切り替える(T1〜T12、T19、T20) |
| 3 | 3 | `c8cf160` | core のテスト: 三つの場合の分類を表のとおりに確かめる(N1) |
| 4 | 4 | `0fd3452` | 仕様書(rust の 02、05、07、11、12): `JobExecution` を 3 つの場合の言葉に直す |
| 5 | 5 | `10fb5a3` | runtime・cli: `run_job` が分割済みのコマンドを受け取り、分け直さない(T13〜T17) |
| 6 | 6 | `d857085` | runtime のテスト: 渡した分割済みのコマンドで起動し、分け直さない(N2) |
| 7 | 7 | `c49ff0b` | 仕様書: rust の 02、05、07、11、12 と、language-agnostic の 05、07 の最小限 |
| 8 | 8 | `33d9890` | core: 定義ファイルを開いてから、開いたものの大きさを見る(形 B) |
| 9 | 9 | `f623b1b` | core のテスト: 読み込みの順序(N3、N4、N5) |
| 10 | 10 | `31d6f56` | 仕様書: rust の 03・02、language-agnostic の 03(「v0.9.5 までは…」の注記) |
| 11 | 11 | `ee9f409` | 版を 0.9.6 にする(`ea09b46` と同じ 22 ファイル。T18) |

## 段ごとのテストの件数(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug)

| 時点 | 通過 | 無視 | 失敗 | 基準からの増分 | 内訳 |
|------|------|------|------|----------------|------|
| 基準 | 1,137 | 1 | 0 | — | inventory.md §0 と一致 |
| Step 1 のあと | 1,137 | 1 | 0 | +0 | テストの変更なし |
| Step 2 のあと | 1,137 | 1 | 0 | +0 | 消したテストは無い |
| 段 1 のあと(Step 4) | **1,138** | 1 | 0 | +1 | N1 |
| Step 5 のあと | 1,138 | 1 | 0 | +1 | T17 は名前の変更で件数は同じ |
| 段 2 のあと(Step 7) | **1,139** | 1 | 0 | +2 | N2 |
| Step 8 のあと | 1,139 | 1 | 0 | +2 | 既存の取り込みのテストは変えずに通った |
| 段 3 のあと(Step 11、最終) | **1,142** | 1 | 0 | **+5** | N3、N4、N5 |

見込み(1,138 → 1,139 → 1,142)と一致する。

## N3・N5 の直す前と後(Step 8 の前の木 c49ff0b で、N3〜N5 を一時的に足して走らせ、記録してから Edit で外した)

| テスト | 直す前(c49ff0b) | 直した後(f623b1b) |
|--------|------------------|--------------------|
| N3 `権限のない_1_mib_超のファイルは_unreadable` | **落ちた**(`left: TooLarge`、`right: Unreadable`) | 通った |
| N4 `上限超過は_utf8_でないより先` | 通った | 通った |
| N5 `名前付きパイプは開かずに_unreadable` | 通った(5 秒の打ち切りに届かずに返った) | 通った |

- N5 の FIFO は `mkfifo` コマンドを `std::process::Command` で呼んで作った(`unsafe`、`libc` を使わない)。
- N3 は、開けてしまうとき(root など)は確かめない(`File::open` が成功したら確認を飛ばす)。

## 最後の確かめ

- `cargo fmt --all -- --check`: 通った。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 通った。
- `cargo test --locked --workspace`: 1,142 通過、1 無視、0 失敗(rustc 1.100.0。MSRV と同じ版)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 行 98.23%、関数 98.29%、領域 97.88%。
- `Cargo.toml`・`Cargo.lock` の差分は版の 8 行だけ(新しい依存なし)。converter の差分は `Cargo.toml` の版の 1 行だけ。
- 足した `#[cfg(unix)]` は core の `definition/intake.rs` の単体テスト(`#[cfg(test)]`)の 2 か所だけ。`unsafe` は足していない。`nfr_source.rs` は通る。
- `JobExecution` の古い項目(`start_failure`、`.started`、`.interrupted`、`signal_number`)は src と tests に残っていない(`InterruptSignal::signal_number()`、`FakeSignals.started`、`flow_with_state.rs` の `Step.started` は別のもの)。
- `crates/runtime/src/runner.rs` の非テストのコードに `command_line`・`split(`・`check_command` の呼び出しは無い(ドキュメントコメントと単体テストだけ)。`Command::new(&line.program)` は残る。

## テストの変更の一覧(承認済みの T1〜T20)

| ID | ファイル | 変えたこと | 期待の文字列 |
|----|----------|------------|--------------|
| T1 | `crates/cli/tests/common/mod.rs` | `ScriptedRuntime`・`MemRuntime` の `run_job` を `ExecutionResult` から `JobExecution::new`・`not_started` で作る。実行中の合図は `Exited` のときだけ `interrupted_by` に入れる。import | 変えない |
| T2 | `crates/core/tests/flow_sequences.rs` | 補助 `exited`、起動しなかった中断(`NotStarted { signal: 15 }`)。import | 変えない |
| T3 ★ | `crates/core/tests/retry_flow.rs` | 補助 2 つと 4 つのテストの組み立て(`StartFailed`、`Exited { None, Some(9), None }`、`Exited { Some(75), None, Some(2) }`、`NotStarted { signal: 15 }`)。import | 変えない |
| T4 | `crates/core/tests/flow_perf.rs`、`plan_perf.rs`、`graph_edges.rs` | `JobExecution::exited(&def.job[0], 0)`、補助は `Exited { Some(code), None, None }` | 変えない |
| T5 | `crates/runtime/tests/nfr_content.rs` | `result: Exited { Some(3), None, None }`。import | 変えない |
| T6 | `crates/runtime/tests/runner_basic.rs` | §4.1 の規則で読み方を置き換える | 変えない |
| T7 | `crates/runtime/tests/runner_signal.rs` | 同上 | 変えない |
| T8 | `runner_env.rs`、`nfr_robust.rs`、`flow_with_state.rs` | 同上(`started: !matches!(…, NotStarted { .. })`) | 変えない |
| T9 ★ | `crates/core/src/flow/classify.rs`(単体) | 補助を `exec(result)` に。(1) `Exited { Some(0), Some(9), Some(15) }` → 143、(2) `StartFailed` → 127、(3) 「起動せず中断でもない」の 2 行を消す | 変えない |
| T10 | `crates/core/src/flow/next_step.rs`(単体) | 補助と 4 つの場合 | 変えない |
| T11 ★ | `crates/core/src/flow/record.rs`(単体) | 補助。起動しなかった中断は `NotStarted { signal: 2 }` | 変えない |
| T12 | `crates/runtime/src/runner.rs`(単体) | 読み方の置き換え | 変えない |
| T13 | `crates/cli/tests/common/mod.rs` | 偽物の `run_job` に `_line: &CommandLine` | 変えない |
| T14 | `runner_basic.rs`、`runner_signal.rs`、`runner_env.rs`、`flow_with_state.rs`、`nfr_robust.rs` | 呼ぶ前に `check_command(&j.command).expect("分けられる")` で作って渡す | 変えない |
| T15 ★ | `runner_basic.rs` の `配列の_command_の先頭が空なら起動失敗` | `CommandLine` を直接作って渡す(先頭を `program`、無ければ空、残りを `args`) | 変えない |
| T16 ★ | `nfr_robust.rs` の `起動できないコマンドや範囲外の番号でパニックしない` | 分けられない 3 つは `check_command` の誤り、残り 3 つは `StartFailed` | 変えない |
| T17 ★ | `runner.rs`(単体) | `分けられない_command_は起動失敗` → `空の_program_は起動失敗`(`""` と `"  "`)。残り 2 つは引数だけ | 変えない |
| T18 ★ | `crates/cli/tests/binary_basic.rs:246` | `"conductor 0.9.5\n"` → `"conductor 0.9.6\n"` | **この 1 行だけ** |
| T19 ★ | `crates/core/tests/flow_sequences.rs` の `優先順位_中断と起動失敗とシグナル` | 3 つの値(`Exited { Some(0), Some(9), Some(2) }` → 130、`StartFailed` → 127、`Exited { Some(0), Some(9), None }` → 137)。assert の文は変えない | 変えない |
| T20 | `flow_sequences.rs:115`、`runner_basic.rs:110・150・156・214・268`、`runner_signal.rs:164・180`、`runner_env.rs:30` | 消えた項目(主に `exit_code`)の読み方を §4.1 の規則で置き換える | 変えない |

足したテストは N1〜N5 の 5 件(上の件数の表)。

## 計画からのずれ

- **T19・T20 の追加**: Step 2 で、承認済みの T1〜T12 の行の一覧に無い箇所(`flow_sequences.rs` の `優先順位_中断と起動失敗とシグナル` と、読み方だけの 9 か所)を変えないとビルドが通らなかった。いったん Step 2 をコミットせずに止めて報告し、ユーザーの承認(2026-11-23)を受けて、functional-spec.md §4 に T19・T20 として足してから、Step 2 をコミットした。
- **classify の内部の `signal_exit` の引数**: `fn signal_exit(signal: Option<i32>)` を `fn signal_exit(signal: i32)` に変えた。中断の番号が型で必ずあるので、`None` のときの `128 + 0` の扱いがいらなくなったため。非公開の関数で、返す値は変わらない。
- cli の `step.rs` に、中断の表示のための非公開の補助 `interrupted_signal` を足した(`report_execution` の `if execution.interrupted` の置き換え)。
- language-agnostic の 05 §1.2 は、手順 1 のほかに、同じ節の `start_failure` の行(「command を引数に分けられない」)も「プログラム名が空」に直した。手順 1 と食い違わないようにするため。真偽値の観測モデルは変えていない(D2)。
- rust の 07 §4.4 は、順 2(起こらない)を消して、順の番号を 1〜9 に振り直した。05 §2.3 も順 1' を消した。番号を参照するほかの記述は無かった。
- rust と language-agnostic の 03 の表は、形 B の順(12 の行)にした。開くときに消えたファイルの `fileMissing`(順 5)と、開いたものが通常のファイルでない(順 8)の行を足した。
- functional-spec.md(T19・T20 の追記)は、コーディネーターが書いた未コミットの変更のまま残した(このステージの記録のファイルではないため)。
