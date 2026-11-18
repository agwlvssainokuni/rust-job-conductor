# Test Results — job-retry

実施日: 2026-11-18(UTC)。環境: macOS(Darwin)、rustc 1.100.0、ローカル。対象の状態: `main` の `ae8324a`(コード・README・テストは、u3 の完了時点から変更なし)。

## ビルドと静的解析

| コマンド | 終了コード | 結果 |
|---|---|---|
| `cargo build --workspace --locked` | 0 | 成功、警告なし |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 件 |
| `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` | 0 | 警告 0 件(Windows 向けのコンパイルの確認。テストの実行ではない) |

## テスト

`cargo test --workspace --locked`(終了コード 0): 合計 **803 件が成功、失敗 0 件、無視 1 件**(無視の 1 件は、この変更の前からある)。

| 段階 | 件数(累計) |
|---|---|
| u1 の完了後 | 755 件 |
| u2 の完了後 | 799 件 |
| u3 の完了後(今回) | 803 件(+4: `docs_retry.rs`) |

この変更の前の件数は、記録していない。

## 全体のカバレッジ

`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(終了コード 0、下限 80% を満たす):

| 種類 | 値 |
|---|---|
| 行(Lines) | **97.80%**(7212 行のうち、未実行 159 行) |
| 関数(Functions) | 97.73% |
| 領域(Regions) | 97.50% |

この変更で足したソースファイルの行カバレッジ(実測): `crates/core/src/flow/retry.rs` 100%、`crates/core/src/definition/duration.rs` 100%、`crates/cli/src/retry.rs` 100%、`crates/runtime/src/wait.rs` 100%。コマンドを再実行すれば再現できる。

## 実バイナリの通しの確認(手動)

`integration-test-instructions.md` の手順。実バイナリ(`./target/debug/conductor`)で実施した。

1. 3 回までリトライして使い切る(`max = 2`、`interval = "1s"`、ジョブは常に終了コード 75): `attempt` が 3 回出力され、標準エラーに知らせが 2 回(`conductor: retry: job 'flaky' exited with code 75; retry 1 of 2 in 1 seconds`、`... retry 2 of 2 in 1 seconds`)、所要 2.08 秒、終了コード 75。ステートは、`status = "error"`、`exit_code = 75`、`resume_job_id = "flaky"`、リトライの回数の記録なし。結果: 設計どおり
2. 1 時間の待機中に SIGTERM(起動の 1 秒後): 知らせが 1 行(`retry 1 of 3 in 3600 seconds`)、**1 秒で終了、終了コード 143**。ステートは、`status = "pending"`、`resume_job_id = "flaky"`(エラーとして記録されない)。結果: 設計どおり

## 既存のアサーションの変更(NFR1 に関わる)

`5b7aabc`(リトライの実装の前)から今回までの、テストを含む全ファイルの差分で、**削除または変更されたアサーションは 1 件**:

- `crates/core/src/error.rs` の、誤りの種類の一覧の件数の確認: `assert_eq!(codes.len(), 20)` → 30。リトライの設定の誤りの種類を 10 個足したため、数え直しが避けられない。u1 のコード生成で、変更として記録し、レビューを経て、あなたが u1 の完了を承認している

それ以外の、既存のテスト(ファイル)は、変更されていない。

## 実行できなかったもの

- Linux と Windows でのテストの実行: ローカルは macOS のため。CI(GitHub Actions)で実行する。この変更は、まだ push していない
- 本物のシグナルで、リトライの待機を打ち切る自動のテスト: 自動のテストは、偽の実行環境と、`wait_for` の単体のテストで確かめている。本物のバイナリでの確認は、上の手動の 2 のみ

## 失敗の判定

このステージは、失敗の条件に当たる: 目標 NFR5(3 つの OS)が `Unverified`(`build-and-test-summary.md`)。ビルドとテストのコマンドは、すべて成功している。原因は、ローカルが macOS のみで、Linux と Windows のテストを実行できないこと。コード、テスト、設定の側に、直せる欠陥は見つかっていない(直せる方法が、ない)。解消するには、push して、CI(GitHub Actions)の 3 つの OS の結果を確認するしかない。

## Loop-Back Log

なし(ループバックは、していない)。
