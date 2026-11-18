# Code Summary — u2-retry-execution(リトライの実行と、途中の知らせ)

根拠: `code-generation-plan.md`(承認済み)、`unit-test-instructions.md`、設計 `functional-design/`(`rules.md` BR2.1〜BR2.16、`entities.md`、`functional-spec.md`)、`requirements.md`(FR1.2 と FR1.6 の動き、FR2.1、FR2.2、FR3.1〜FR3.6、FR4.1、FR4.2、FR5.1、FR5.2、FR6.1、FR6.2 の最初の文言、NFR3、NFR7)。

## 作成・変更したファイル

コミットは 4 件(`main` 上。push はしていない)。

| コミット | 内容 |
|---|---|
| ea3f694 | リトライの判断(core の純粋な関数) |
| be5c494 | 待機(runtime) |
| 99e28a1 | 差し込み口(`Runtime::wait`)、偽の `Runtime`、表示の種類と文言 |
| 19d7b1a | 流れ(`run` と `pipeline` の共通の処理) |

### ソース

- `crates/core/src/flow/retry.rs`(新規): `RetryDecision`(`Stop` と `Retry { wait_secs }`)と、純粋な `retry_decision(job, execution, retries_done, last_wait_secs)`。対象の判定(BR2.1、BR2.16)、回数(BR2.2)、待ち時間(BR2.3。固定と、倍率と上限。桁あふれは飽和)
- `crates/core/src/flow/mod.rs`、`crates/core/src/lib.rs`: 公開面に `RetryDecision`、`retry_decision` を追加
- `crates/runtime/src/wait.rs`(新規): `WaitOutcome`(`Elapsed` と `Interrupted(i32)`)と `wait_for(secs, &InterruptSignal)`。標準ライブラリだけを使い、50 ミリ秒の間隔で合図を確かめる(BR2.13、BR2.14)
- `crates/runtime/src/lib.rs`: `wait` モジュールの宣言と公開
- `crates/cli/src/retry.rs`(新規): `run` と `pipeline` が共有する唯一の関数 `run_job_with_retry`(BR2.11)。実行 → 判断 → 知らせ → 待つ → 繰り返し。待機が中断されたときは、「起動していない、中断された」実行結果を作る(BR2.5)
- `crates/cli/src/deps.rs`: `Runtime` に `wait` を追加。`SystemRuntime` は `wait_for` を呼ぶ
- `crates/cli/src/run.rs`、`crates/cli/src/pipeline.rs`: `run_job` の呼び出しを `run_job_with_retry` に置き換え。結果の表示とステートの記録は、最終の結果について、これまでどおり 1 回だけ(BR2.10、BR2.12)
- `crates/cli/src/lib.rs`: `retry` モジュールの宣言
- `crates/cli/src/types.rs`: 表示のレベル `Level::Notice` を追加(「常に標準エラー」で、エラーの文言ではない)
- `crates/cli/src/report/mod.rs`: `DisplayEvent::RetryScheduled { job_id, last_exit_code, retry_number, max, wait_secs }`(レベルは `Notice`)
- `crates/cli/src/report/messages.rs`: リトライの知らせの英語と日本語の固定の文(最初の文言。最終の確定は U3)

リトライの知らせの文言(最初のもの):

- 英語: `conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 40 seconds`
- 日本語: `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。40 秒後にリトライします(2/3 回目)`
- ジョブ ID は、既存のエスケープを通す。それ以外は、固定の文と整数だけ(BR2.15)

### テスト(追加 44 件)

| ファイル | 件数 | 内容 |
|---|---|---|
| `crates/core/tests/retry_flow.rs`(新規) | 16 | 対象と対象外の全パターン、回数の境界、待ち時間の列(固定、倍率と上限、桁あふれしない) |
| `crates/runtime/tests/wait_interruptible.rs`(新規) | 6 | 待機が経つ、待ち 0 秒、すでに合図が立っている、3600 秒の待機を途中のシグナルで 1 秒以内に中断(NFR7) |
| `crates/cli/tests/flow_retry.rs`(新規) | 18 | 偽の `Runtime` で、2 回目に成功、使い切り、対象外、設定なし、倍率と上限の待ち時間、待機中のシグナル、`pipeline`、ステートが最終の 1 回だけ、`-v` なしで標準エラーに知らせが出ること |
| `crates/cli/tests/report_messages.rs` | 4 | 英語と日本語の文、`-v` なしで出ること、入力の値が出ないこと |

既存のテストへの変更(アサーションの変更はない):

- `crates/cli/tests/common/mod.rs`: 偽の `Runtime` 2 つ(`ScriptedRuntime`、`MemRuntime`)に、`wait`(待たずに、呼ばれた待ち時間を記録し、指定の回数目に `Interrupted` を返せる)を追加
- `crates/cli/tests/report_messages.rs` の既存のイベント一覧テストの入力(`with_all_events`)に、新しいイベントを 1 件追加

## 主な実装上の判断

- 知らせのレベルは、新しい `Level::Notice` にした。`Level::Error` にすると、既存のテスト「エラーの行は `conductor: error:` で始まる」(Error レベルのすべてのイベントが対象)に反するため。出力先は標準エラーで、常に表示する
- 待機が中断されたときは、`started = false`、`interrupted = true`、`signal_number = Some(n)` の実行結果を作り、既存の分類と記録の流れをそのまま使う(分類なし、128 + n、再開位置はそのジョブ、エラーとしては記録しない)
- 待機の確認の間隔は 50 ミリ秒(設計の「100 ミリ秒以内」を満たす)
- `retry_decision` の `last_wait_secs` は `u64` で、`retries_done` が 0 のときは使わない

## 計画からのずれ

- `Level::Notice` の追加(`crates/cli/src/types.rs`)は、計画に名前を挙げていなかった。表示のレベルを足すために必要だったため、追加した(上の判断を参照)
- 開発担当が、作業の規則(ファイルはファイルツールで編集する)に反して、一部の編集をシェル(`python3`、`cat >>`、`sed -i`)で行ったと申告している。内容は意図どおりで、`cargo fmt` と `cargo clippy` を通り、私も差分を確認した

## 確認の結果(ローカル、macOS、rustc 1.100.0)

- `cargo test --workspace --locked`: 799 件が成功、失敗 0 件、無視 1 件(既存)。変更前は 755 件(今回の追加 44 件を足すと 799 件)
- `cargo fmt --all -- --check`: 問題なし
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告 0 件
- カバレッジ(`cargo llvm-cov ... --fail-under-lines 80`、開発担当の実測): 行 97.80%。新しいコード(`flow/retry.rs`、`wait.rs`、`cli/src/retry.rs`)は、いずれも 100%
- `cargo tree`: `-sys` クレートと C のビルド依存はない。新しい外部クレートも追加していない。`Cargo.toml` と `Cargo.lock` は変更なし
- 既存のアサーションの変更: なし

## 実行していない(書いたが実行していない)テストと検査

- Linux と Windows でのテスト: ローカルは macOS のため。CI で実行する
- 実プロセスで、実際の時間(間隔)を待つ統合テスト: 計画の前提どおり、書いていない。待機の実時間の確かめは、`wait_interruptible.rs` の短い待機と、偽の `Runtime` を使う流れのテストで行った
- CI での実行: この時点では、プッシュしていないため、CI は実行されていない

## この Unit の外

- ステートファイルの形式、定義の読み取りと検証、README は、触れていない
