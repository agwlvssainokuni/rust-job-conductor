# Code Summary: 完了後もステートを残す(0.7.0)

ゼロ Unit(express)。方式は test-after。コミット・タグはしていない(オーケストレーターが行う)。

## 作成・変更したファイル

### 定義(`crates/core`)— FR2.2、FR2.4
- `src/types.rs`: `PipelineDefinition.remove_on_complete: bool`(省略時 `false`)
- `src/definition/walk.rs`: `PIPELINE_KEYS` に `remove_on_complete` を追加。真偽値の取り出し(`Walker::boolean`)を新設。型の誤りは既存の `TypeMismatch`(64)の経路
- `src/flow/{decide,next_step,resume}.rs`: テスト用の `PipelineDefinition` の初期値に追加
- `tests/definition_errors.rs`: 省略・`true`・`false`、型の誤り、他のキーとの共存と未知のキーの拒否

### ステートの読み取り(`crates/runtime`)— FR3.1、FR3.2、FR3.4
- `src/state/reader.rs`: `read_state_raw` を新設(完了のステートも返す)。`read_state` は `read_state_raw` の結果から `completed` を空にする形に変更(挙動は同じ)。検査(版、破損、run ID の食い違い、大きさの上限)は共通の `parse_state_raw_for`
- `src/lib.rs`: 再公開、関数の表、呼び出しの順序の説明
- `src/state/{consistency,record}.rs`、`tests/{flow_with_state,nfr_content,nfr_performance}.rs`: テスト用の `PipelineDefinition` の初期値に追加
- `tests/state_read_raw.rs`(新規): `read_state_raw` と `read_state` の差、ファイルなし・長さ 0、同じ誤り

### CLI(`crates/cli`)— FR1、FR2.1、FR2.3、FR2.5、FR2.6、FR3、FR4
- `src/types.rs`: `CliCommand.remove_on_complete`
- `src/args.rs`: `--remove-on-complete`(値なし)。`run`・`pipeline` だけ許可、ほかは `OptionNotAllowed`(68)。2 回以上と `=値` は `InvalidValue`。冒頭の表を更新
- `src/report/messages.rs`: 使い方(英語・日本語)に追加、`WATCH_COMPLETED`(`completed` / `完了`)
- `src/report/watch.rs`: 完了のステートの表の下に完了の行を足す
- `src/ctx.rs`: `Session.remove_on_complete`(オプション or 定義の値。環境変数は見ない)を `prepare` で決める
- `src/pipeline.rs`、`src/run.rs`: 完了の削除(`reset`)を、実効の `remove_on_complete` が真のときだけ呼ぶ。偽なら完了のステートを残して 0
- `src/deps.rs`: `Runtime::read_state_raw` と `SystemRuntime` の実装
- `src/aux_commands.rs`: `watch` が `read_state_raw` を使う
- テスト: `tests/common/mod.rs`(フェイクの `read_state_raw`、`state_raw`・`state_raw_of`・`with_remove_on_complete`)、`args_grammar.rs`、`flow_pipeline.rs`、`flow_run.rs`、`flow_run_id.rs`、`flow_aux.rs`、`binary_basic.rs`、`binary_run_id.rs`、`nfr_perf.rs`、`unix_demo.rs`、`windows_demo.rs`

### 文書・デモ・版
- `README.md`: クイックスタートの 5・6、コマンド表と使い方、オプションの表、定義のキー、`ステートファイルの場所`、run ID の小節、終了コードの表、保証の範囲、版の例(0.7.0)。`docs-check` の 7 語句と見出しは変更なし
- `demo/run-demo.sh`、`demo/windows/run-demo.cmd`(CRLF・UTF-8 を保持)、`demo/README.md`: 場面 4 と 8
- 4 クレートの `Cargo.toml`、`Cargo.lock`: 0.7.0

## 主な実装の判断
- ステートの形式は変えない。完了のステートは、今までも削除の前に書いていたので、削除の呼び出しを条件つきにするだけで FR1 を満たした。
- `read_state` の挙動は変えない(完了は空)。`watch` だけが `read_state_raw` を使うので、`pipeline` の再実行は先頭から始まり、完了後に先頭以外を `run` すると 66 になる。
- テスト用のフェイク(`MemRuntime`)の `read_state` は、本物と同じく完了を空として返す形に直した(完了のステートが残るようになったため)。
- `--remove-on-complete=値` は、`--all` と同じく `InvalidValue` にした(値なしのオプション)。
- 日本語の `watch` の完了の行は `完了`、状態の列の語は英語のまま。

## テストの要約
- 要件ごとに 1 件以上: FR1.1(`pipeline` の 3 つの完了経路と `run` の 2 つ)、FR1.2(形式)、FR1.3(再実行・66)、FR1.4(完了の書き込み失敗で 65)、FR2.1〜FR2.3(オプション・定義・優先順位)、FR2.4(型の誤り 64)、FR2.5(args と統合で 68)、FR2.6(環境変数で残る)、FR3.1〜FR3.4(英語・日本語・run ID・`no state`・壊れたステート 65)、FR4.1(`reset`・`reset --all`)、FR6.1(`conductor 0.7.0`)
- 既存テストの前提を、新しい挙動の期待に直した(弱めていない): `flow_pipeline`・`flow_run`・`flow_run_id`・`flow_aux`・`binary_basic`・`binary_run_id`・`nfr_perf`・`unix_demo`・`windows_demo`
- 実行結果(macOS、安定版): `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets` がすべて成功。`docs-check` の 7 語句は 1 語ずつ確認(すべて残っている)。
- README のクイックスタート(定義の作成、`validate`、`FAIL=3 pipeline -v`、`watch`、再実行、`watch`、再度の `pipeline`)を、`target/` の下の一時ディレクトリで、リリースビルドの `conductor`、`LANG=C` で実行し、README の出力と一致することを確認した(確認後に一時ディレクトリは削除)。

## 実行していないテストとその理由
- `windows_demo.rs`(`cfg(windows)`)と、Windows の `run-demo.cmd` の実行: 開発機は macOS。CI の Windows で実行する。`run-demo.cmd` の変更は、UNIX 版と同じ場面の対応で、CRLF が保たれていること(242 行すべてが CRLF)だけ確認した。
- カバレッジの測定(`cargo llvm-cov`): Build and Test で測る。閾値は変えていない。
- Linux・Windows での `cargo test`: CI で実行する。

## 計画からのずれ
- 計画にない変更: `tests/common/mod.rs` のフェイクの `read_state` が完了を空にするよう変更、`nfr_perf.rs`・`binary_run_id.rs` の期待の修正(`cargo test --workspace` の失敗から発見)。
- `crates/runtime/tests/state_read_raw.rs` を新規ファイルにした(`state_read.rs` を触らないため)。
- `README.md` の版の例(`0.6.0` → `0.7.0`)も更新した。
- `traceability.json` はオーケストレーターが作る。
