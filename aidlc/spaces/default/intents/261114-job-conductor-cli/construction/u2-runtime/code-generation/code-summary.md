# Code Summary(u2-runtime)

16 ステップをすべて完了した。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`(この Unit は 152 件、失敗 0。ワークスペース全体で 379 件、失敗 0)が通った。

## 作成・変更したファイル

- `crates/u2-runtime/`(パッケージ `conductor-runtime`。`publish = false`): `Cargo.toml`、`src/{lib,types,error,split,runner}.rs`、`src/platform/{mod,unix,windows}.rs`、`src/state/{mod,reader,writer,consistency,record}.rs`、`src/bin/conductor-test-helper.rs`(テスト専用。`test = false`、`doc = false`)。
- テスト: `tests/{toml_depth,runner_basic,runner_signal,state_read,state_write,flow_with_state,nfr_source,nfr_robust,nfr_content,nfr_performance}.rs`、`tests/common/mod.rs`、`src/` 内の単体テスト(78 件)。
- ワークスペース: `Cargo.lock`(`conductor-runtime` のエントリ。`libc 0.2.191` は、すでにロックに入っていた)。

## Step 1 の確認結果

- `std::os::unix::process::CommandExt::process_group`: 標準のライブラリ(1.64 以降)。手元の cargo 1.97 で、コンパイルと動作を確認した。**最小の Rust のバージョン 1.85 のツールチェインでの実ビルドは、できていない**(この環境には 1.97 のみ)。CI の build-check に委ねる。
- `libc 0.2.191`: `rust-version = "1.85"`(レジストリのソースで確認)。1.85 での実ビルドは未確認。
- `toml 1.2.1`: 読み取り・書き出し、`i64`・真偽値・配列・表・`[[jobs]]` の往復が通る。深さ 1000 と 100,000 の入れ子(配列、インラインのテーブル)は、テストのスレッドの標準のスタックで、異常終了せず `Err` を返す(`tests/toml_depth.rs`)。
- `cargo tree -p conductor-runtime --edges normal` に `-sys` はない(`libc` のみ)。

## 主な実装の判断

- 待ちは、設計どおり、1つのスレッドの `try_wait` のくり返し(1ms から 10ms)で、回収はこのくり返しだけが行う(`Waitable` トレイトで、テストでは、`forward` と `sleep` を差し替える)。シグナルの送信は、`platform/unix.rs` の1か所だけが `unsafe`(`#![deny(unsafe_code)]`、モジュールだけ許可)で、子プロセスの pid だけに、`{ SIGHUP, SIGINT, SIGQUIT, SIGTERM }` だけを、回収の前に1回送る。
- `apply_*` は、新しい `PipelineState` を返す純粋関数。`apply_skip` は、`AlreadyExecuted` を変えず、error のジョブがあれば `resume_job_id` と `completed` を変えず、それ以外では「いま skipped にしたジョブが最後で、pending が無い」ときに `completed`、`AfterEnd` で `end_reached`(単調)を書く。
- NFR1.1(macOS、debug ビルド): 起動前の処理(ジョブ 10 個、100 回、`Fast`)の p95 は約 8.6 ms(仮の基準 20 ms 以内)。結果の書き込み(`Durable`)の p95 は約 9.7 ms(基準なし。記録のみ)。
- 安定性: `runner_basic`、`runner_signal`、`flow_with_state` を 5 回連続で実行して、すべて成功した。権限による失敗のテストは、非 root の UNIX でだけ実行し、root では、「スキップ: root のため…」を標準エラーに出して終わる。

## 設計との差(計画からの逸脱と、設計の成果物への反映)

1. **補助の実行ファイルの出力先:** 設計は標準出力だが、`run_job` がジョブの標準出力を引き継ぎ、テストが読めないため、`--out <file>` に追記する形にした(指定がなければ標準出力)。モードは `echo`、`exit`、`env`、`cwd`、`pid`、`sleep [秒]`、`run <label> <code>`。
2. **テストが持つ子プロセスの強制終了:** `run_job` が子を所有するため、時間の上限(30 秒)を超えたとき、補助の実行ファイルが書いた pid を読み、`kill -KILL`(UNIX)/`taskkill /F`(Windows)で終わらせてから失敗にする。`sleep` モードは、25 秒以内に自分で終わる。
3. **「シグナルが 1 回だけ」の検証:** 補助の実行ファイルは、ハンドラを持たない(`libc` を使わない)ため、数えられない。UNIX のテストで `sh -c 'trap ... TERM'` を使い、受信のたびに 1 行書かせて、約 400 ms 待ち、1 行だけであることを確かめる(`sh` を呼ぶのは `tests/` だけで、`src/` は呼ばない)。2回目の合図の無視は、`InterruptSignal` の仕様と、WaitLoop の `forwarded` フラグ(fake の子での単体テスト)で確かめた。
4. **API の形(計画に明記のない決定):** `read_state(definition_path) -> Result<Option<PipelineState>, StateError>`(`None` が空)、`write_state(definition_path, &PipelineState, Durability)`、`reset_state(definition_path)`、`check_consistency(&PipelineState, &PipelineDefinition)`、`initial_state(&PipelineDefinition, &Path)`。`StateError.path` は `PathBuf`。
5. **`apply_outcome` の安全側の処理(計画にない):** `status` が success・warning・error 以外なら `exit_code` を `None` にする。分類を書いたときは `skip_reason` を消す。`skippedJobIds` に、いま結果を記録するジョブ自身が含まれても、触れない。`apply_skip` は、定義にあって状態に記録のないジョブを pending として扱い、状態にないジョブIDは何も変えずに返す(パニックしない)。`OutOfOrder` は `AlreadyExecuted` と同じく不変。
6. **`try_wait` の失敗:** `start_failure = true` の JobExecution を返す。「固定の理由の文」は、`JobExecution` に持たせる場所がないため、値としては返らない(起動の失敗と区別がつかない。安全側)。表示は u3-cli が決める。
7. **`128 + signalNumber` の飽和:** この Unit には、その計算がない(u1-core の領分)。この Unit が扱うシグナル番号の範囲外の値でも、パニックも桁あふれもしないことを、テストで確かめた。
8. **command の空白:** `\r` も空白に含めた(CRLF の command)。`''` だけのコマンド名は `Empty`。
9. **設計の成果物への差分(書き換えていない。この記録が、反映までの正):** security-requirements と security-design の「既存の成果物との差分」の表のとおり(長さ 0 のステートは空、`write_state` の耐久性の指定、待ちのくり返し、起動前の初期ステートは `Fast`)を実装した。

## 設計の隙間(後のステージ・Unit で解決する)

- **契約 C3 の更新(u3-cli の設計):** `check_command`、`initial_state`・`apply_outcome`(JobExecution を受け取る)・`apply_skip`(定義を受け取る)・`apply_interrupt`、`read_state`/`write_state`/`reset_state` の入力が定義ファイルのパス、`write_state` の `durability`、`check_consistency` の入力にパイプライン名を含む定義。E3: `definition_path` の追加、`definition_key` を定義ファイルのパスに確定、ステートファイルの場所。
- **UNIX 系で、ジョブを別のプロセスグループで起動する選択**(対話的なジョブは対象外、conductor が強制終了されたときに孤児が残りやすい): 人の確認が、まだである(functional-design の memory.md の引き継ぎ)。
- **functional-design の NOT-READY のまま完了した件(2 回目のレビューの Critical):** `apply_skip` の完了の導出は、直した設計どおりに実装し、`@end` の個別実行(A → `@end`、B・C が pending のまま、B のスキップで完了しない、最後の C のスキップで完了)をテストした(`tests/flow_with_state.rs`、`src/state/record.rs`)。

## テスト(152 件)

| 範囲 | 件数 |
|------|------|
| `src/` の単体テスト | 78 |
| toml_depth | 3 |
| runner_basic | 8(UNIX)。`cfg(windows)` の `.cmd` 2 件は含まない |
| runner_signal(UNIX のみ) | 6 |
| state_read | 12 |
| state_write | 10 |
| flow_with_state | 20 |
| nfr_source | 5 |
| nfr_robust | 6 |
| nfr_content | 3 |
| nfr_performance | 1 |

## レビュー(1 回目、READY)で残った Minor(次の機会に直す)

- 「回収後に送らない」性質を確かめる WaitLoop の単体テストがない(既存の「先に終わったジョブは転送なしで観測される」は、`run_job` が返ったあとに合図を立てるため、この性質に触れない)。
- 古い一時ファイルの後始末(`remove_stale_temps`)は、接頭辞だけで消すため、`.<名>.tmp.bak` のような通常のファイルも、24 時間以上たてば消える。名前の型(プロセス番号・時刻・連番)まで照合するように、厳しくする。
- `try_wait` の失敗のとき、子を止めずに `start_failure` で返すため、再開すると、同じジョブが重複して動きうる。この経路を、lib.rs の「保証の範囲」に書く。
- シグナルのテストは、`sleep 1`/`sleep 2` の時間の余裕に依存する。SIGINT のテストは、SIGINT を無視する環境から実行されると失敗する。
- `File::open` が、FIFO で永久にブロックしうる(reader.rs)。通常のファイルでなければ開く前に拒否する、または、開く前の `metadata` で確かめる。
- 記録のずれ: `cfg(windows)` のコードは、`cargo check --target x86_64-pc-windows-msvc` で、型検査まで通る(実行は未確認)。unit-test-instructions の「約 80 件」は、実際は 152 件。コミットに、IDE の設定ファイル(`.idea/rust-job-conductor.iml`)の変更が含まれる(この Unit の変更ではない)。

## 実行していないテスト・測定

- **行カバレッジ:** `cargo llvm-cov` が未導入のため、測っていない。80% の判定は Build and Test が行う(`--ignore-filename-regex 'bin/conductor-test-helper'`)。`cfg(windows)` のコードは、Linux の測定に入らない。
- **`cargo +1.85 build`:** 1.85 のツールチェインがなく、実行できなかった。CI の build-check に委ねる。
- **Windows 固有のテスト(`.cmd` の危険な引数と安全な引数の 2 件。NFR4.9)と、`platform/windows.rs`:** `cfg(windows)` で書いただけで、コンパイルも実行もしていない。CI での確認まで、未確認。期待と異なる場合は、危険な引数を、この Unit が先に拒否する方向で設計を見直す。
- **macOS 以外(特に Linux、Windows):** 実行していない。CI(CI Pipeline のステージで作成)で確かめる。CI のワークフローは、まだない。
