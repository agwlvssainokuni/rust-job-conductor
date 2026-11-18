# Build and Test Summary — job-retry

## ビルドの状態と前提

- ビルド、書式、静的解析、全テスト(803 件)、全体のカバレッジ(行 97.80%)は、ローカル(macOS)で成功した。詳細は `test-results.md`
- 前提: Rust 1.100.0 以降。手順は `build-instructions.md`

## 作成したテストの種類

| 種類 | 場所 |
|---|---|
| 単体のテスト | 各 Unit の `unit-test-instructions.md` と、各クレートの `tests/`(u1: 設定の読み取りと検証、u2: 判断・待機・実行の流れ・表示、u3: README) |
| 統合のテスト(Standard) | `integration-test-instructions.md` |
| 実バイナリの通しの確認(手動) | `integration-test-instructions.md` の末尾。実施済み(`test-results.md`) |
| 性能・セキュリティ | 作成しない(Standard のため。性能と安全性の要件の NFR ステージは省いた) |

## カバレッジの期待

- 全体: 行カバレッジ 80% 以上。実測 97.80%
- 各 Unit: この変更で足したソースファイル(`flow/retry.rs`、`definition/duration.rs`、`cli/src/retry.rs`、`runtime/src/wait.rs`)は、いずれも 100%

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| NFR1 | `requirements.md` NFR1 | リトライを設定しない定義の、動作・出力・終了コードを変えない。既存のテストが、アサーションの変更なしで通る | 既存のテストは、すべて通る(803 件、失敗 0)。ただし、アサーションの変更が **1 件**ある: `crates/core/src/error.rs` の誤りの種類の件数 `20` → `30`(誤りの種類を 10 個足したための、避けられない数え直し)。u1 で、変更として記録し、レビューとあなたの承認を経ている | `cargo test --workspace --locked`(終了コード 0)、`git diff 5b7aabc HEAD` の削除されたアサーションの検索(1 件)、`test-results.md` | build-and-test | Met(例外 1 件を、ゲートで提示する) |
| NFR2 | `requirements.md` NFR2 | `unsafe` の禁止(cli)、`platform/` の外での OS 別の `cfg` の禁止、ネットワークを使わない。既存の `nfr_source` のテストが通る | `crates/cli/tests/nfr_source.rs`、`crates/runtime/tests/nfr_source.rs` が通る | `cargo test --workspace --locked`(終了コード 0) | build-and-test | Met |
| NFR3 | `requirements.md` NFR3 | 間隔が長いリトライのテストが、実際に待たずに完了する | 偽の `Runtime`(`wait` が待たずに、待ち時間を記録する)で、リトライの流れのテスト(`flow_retry.rs`)が、実際に待たずに完了する。`wait_for` の単体のテストは、長さ 3600 秒の待ちを、合図で打ち切って完了する(1 秒ほど)。なお、間隔が 1 時間の定義を、偽の実行環境で流すテストは、ない(間隔は 30 秒・10 秒・1 秒。待ち時間は記録するだけで、長さに依存しない) | `crates/cli/tests/flow_retry.rs`、`crates/runtime/tests/wait_interruptible.rs`、スイート全体が数秒で完了 | build-and-test | Met |
| NFR4 | `requirements.md` NFR4、`team.md`(Q4: A) | 行カバレッジ 80% の下限を維持する(CI の `coverage` ジョブが通る) | 全体の行カバレッジ 97.80%。ローカルで、CI と同じ形のコマンド(`--fail-under-lines 80`)が、終了コード 0。CI の `coverage` ジョブも成功(実行 44415057031) | `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(終了コード 0) | build-and-test | Met |
| NFR5 | `requirements.md` NFR5 | Linux、macOS、Windows で同じ動作にする(CI の 3 つの OS のテストが通る) | ローカル(macOS)で 803 件が成功。push 後の CI(GitHub Actions、`main`、実行 44415057031)で、`test`(ubuntu-24.04、macos-latest、windows-2022)、`msrv-test`、`build-check`(ubuntu-24.04、windows-2022)、`startup-bench`(3 つの OS)、`lint`、`coverage`(行 80% 以上)、`docs-check` の全ジョブが成功 | `gh run watch 44415057031 --exit-status`(終了コード 0、全ジョブ ✓)。ローカルの補助: `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings`(終了コード 0) | build-and-test | Met |
| NFR6 | `requirements.md` NFR6 | 新しい外部クレートを追加しない(`-sys` と C のビルド依存を増やさない) | `Cargo.toml` と `Cargo.lock` に、リトライの実装の前(`5b7aabc`)からの差分がない | `git diff 5b7aabc HEAD --stat -- Cargo.toml Cargo.lock crates/*/Cargo.toml`(出力なし) | build-and-test | Met |
| NFR7 | `requirements.md` NFR7 | 待機中のシグナルで、1 秒以内に待機をやめる | `wait_for` の単体のテスト(待ち 3600 秒、合図は 200 ミリ秒後、全体で 1.2 秒未満)が通る。実バイナリで、間隔 1 時間の待機中に SIGTERM を送ると、1 秒で終了、終了コード 143 | `crates/runtime/tests/wait_interruptible.rs`、`test-results.md` の手動の確認 2 | build-and-test | Met |
| TC-CI | Testing Contract の「テストを CI で実行してからマージする」 | CI の実行 | 計画に CI Pipeline の段階(`ci-pipeline`)がない(SKIP) | `aidlc-state.md` の `ci-pipeline — SKIP` | — | N/A(「計画が CI Pipeline を実行しない」ため) |
| TC-STYLE | `team.md` Code Style(Q6: A) | `cargo fmt` の差分なし、`cargo clippy -D warnings` が通る | どちらも、終了コード 0 | `test-results.md` | build-and-test | Met |

## 準備の判断

- ビルドの準備: できている(成功)
- テストの準備: できている。ローカル(macOS)と、CI の 3 つの OS(Linux、macOS、Windows)で成功
- デプロイの準備: 対象の全目標が Met。リリース(版の決定とタグ)は、あなたの判断

## 残っていること・制限

1. **NFR1 の例外 1 件**: 誤りの種類の件数のアサーション(20 → 30)。避けられない変更で、u1 で承認済み。このゲートで、あらためて確認してほしい
2. 英語の知らせは、待ち時間が 1 のとき `in 1 seconds` と複数形になる(見た目だけの問題。u2 の文言の限界で、u3 で記録した)
3. 本物のシグナルでの、リトライの待機の打ち切りには、自動のテストがない(手動の確認 1 回のみ)。自動のテストは、偽の実行環境と、`wait_for` の単体のテスト
4. README の版の表記 `v0.8.0 以降` は、次のリリースの版が 0.8.0 になるという前提。リリースの版は、あなたが決める(タグを付ける前に、違っていれば直す)
