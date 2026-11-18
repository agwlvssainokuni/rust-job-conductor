# Integration Test Instructions — job-retry

テスト戦略は Standard のため、Unit をまたぐ、主な境界のテストをここにまとめる。単体のテストは、各 Unit の `unit-test-instructions.md` にある。

## テストの枠組み

- `cargo test`(Rust 標準)。統合テストは、各クレートの `tests/` にある
- テストデータは、テスト内で組み立てる(定義は文字列、ステートは一時ディレクトリ)。外部のサービスは使わない

## 境界ごとのテスト

| 境界 | テスト | 確かめること |
|---|---|---|
| 定義の読み取り(core)と実行(cli) | `crates/cli/tests/flow_retry.rs` | `[job.retry]` を読んだ定義で、`run`・`pipeline` が、リトライし、待ち時間の列(固定、倍率と上限)、使い切ったときの終了コード、成功・警告での進行が設計どおりになる(偽の実行環境で、実際には待たない) |
| 判断(core)と待機(runtime) | `crates/runtime/tests/wait_interruptible.rs` | 待ちが経つと `Elapsed`、合図が立つと 1 秒以内に `Interrupted`(長さ 3600 秒の待ちを含む) |
| 待機中のシグナル | `crates/cli/tests/flow_retry.rs`(偽の実行環境が、待機中の合図を再現する) | 待機中のシグナルは、128 + 番号で終わり、再開位置をそのジョブとして記録し、エラーとしては記録しない。本物のシグナルでの確認は、下の「実バイナリの通しの確認」の 2(自動のテストにはない) |
| 定義の検証と `validate`・`convert` | `crates/cli/tests/flow_validate.rs`、`crates/converter/tests/convert_roundtrip.rs` | `[job.retry]` の誤りは 64、`validate` が同じ誤りを検出、YAML と TOML の往復で値が保たれる |
| README と読み取り | `crates/cli/tests/docs_retry.rs` | README の例の定義が、実際の読み取りを通り、必須の語が残っている |
| 既存の動作の維持 | 既存のすべてのテスト | `[job.retry]` を書かない定義の、動作、出力、終了コードが変わらない |

## 実バイナリの通しの確認(手動。この Build and Test で実施)

実行環境を偽らずに、本物のバイナリで確かめる。作業用のファイルは、プロジェクトの外の一時ディレクトリに置く。

1. 3 回までリトライして使い切る(`exit_codes = [75]`、`max = 2`、`interval = "1s"`、ジョブは `sh -c 'echo attempt; exit 75'`)
   - コマンド: `./target/debug/conductor pipeline --config <定義> <パイプライン名>`
   - 期待: `attempt` が 3 回、`conductor: retry: ...` の行が 2 回(標準エラー)、約 2 秒、終了コード 75、ステートのジョブが `status = "error"`、`exit_code = 75`、リトライの回数の記録がない
2. 1 時間の待機中にシグナルを受ける(`interval = "1h"`、`max = 3`)
   - 手順: 起動して 1 秒後に `kill -TERM <pid>`
   - 期待: 1 秒ほどで終了、終了コード 143、ステートのジョブが `status = "pending"`、`resume_job_id` がそのジョブ

## OS ごとの扱い

- ローカルは macOS で実行する。Linux と Windows のテストは、CI(GitHub Actions)で実行する。この変更は、まだ push していないため、CI は実行していない
- Windows 向けのコンパイルと静的解析は、ローカルで確かめられる(`build-instructions.md`)。ただし、テストの実行の代わりにはならない

## カバレッジの目標

- 行カバレッジ 80% 以上(全体)。測定: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`
