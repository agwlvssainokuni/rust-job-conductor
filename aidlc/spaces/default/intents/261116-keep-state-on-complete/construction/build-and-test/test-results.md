# Test Results

実行日: 2026-11-17。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。コミット: `ca0b642`(code-generation の承認の時点)。

## ビルドと既存の検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --release --locked --bin conductor` | 0 | 成功 |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo test --workspace --locked` | 0 | 701 件成功、失敗 0、無視 1(前回 666 件。35 件増) |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| README の docs-check(7 語句の `grep`) | 0 | `MISSING` なし |

## カバレッジ

`cargo llvm-cov --workspace --locked --summary-only`(全体のテストで、組み立てたプロジェクト全体を測定。閾値の設定は変えていない):

| 指標 | 値 |
|------|----|
| 行(Lines) | **97.39%**(7046 行中、未達 184) |
| 関数 | 97.89% |
| 領域 | 97.28% |

下限 80% を満たす(前回 97.74%。新しく足した、エラーの経路の一部が、未達の行に入った分だけ下がった)。

## 実機の確認(リリースビルドの `conductor`、`LANG=C`)

`target/` の下の一時ディレクトリ(確認後に削除)に、`format_version = 1` の定義(2 ジョブ)を置いて、実行した。

| 確認 | 結果 |
|------|------|
| `pipeline bt` のあと | 終了 0。`conductor.toml.state` が**残っている**(FR1.1) |
| `watch`(`LANG=C`) | 表(`Pipeline: bt`、ジョブ 2 行、すべて `success`)の下に `completed`。終了 0(FR3.1) |
| `watch`(`LANG=ja_JP.UTF-8`) | 表(`パイプライン: bt`、`補足` 列)の下に `完了`。状態の列は英語のまま(FR3.1) |
| 完了のステートがあるまま `pipeline -v bt` を再実行 | 先頭の `a` から再実行された(FR1.3) |
| `pipeline bt --run-id r1` のあと | `conductor.toml.r1.state` が残る。`watch --run-id r1` は `Run ID: r1` の行つきの表と `completed`(FR3.3) |
| 環境変数 `CONDUCTOR_REMOVE_ON_COMPLETE=1` を設定して `--run-id r2` | `conductor.toml.r2.state` が残る(環境変数は見ない。FR2.6) |
| `pipeline bt --run-id r3 --remove-on-complete` | 終了 0。ファイルなし(FR2.1) |
| `[pipeline].remove_on_complete = true` の定義で `pipeline bt` | 終了 0。ファイルなし(FR2.2) |
| `remove_on_complete = "yes"` の定義で `validate`・`pipeline` | 終了 64(`wrong value type: remove_on_complete は真偽値…`)(FR2.4) |
| `--remove-on-complete` を `watch`・`reset`・`validate`・`convert` に付ける | すべて 68(FR2.5) |
| `--remove-on-complete` を 2 回付ける | 68 |
| `run a` → `run b`(単独の呼び出しで完了) | どちらも 0。完了後、ファイルが残り、`watch` は表と `completed`(FR1.1) |
| 完了のステートがあるまま `run a` を再び呼ぶ | 0。先頭から実行し直し、`watch` は `a success`、`b pending` で、`completed` の行なし(FR1.3) |
| `reset`、`reset --all` | どちらも終了 0。ファイルがすべて消える(FR4.1) |
| `conductor --version` | `conductor 0.7.0`(FR6.1) |

## 実行していないもの

- Windows のテスト(`windows_demo.rs`)と、`demo\windows\run-demo.cmd` の実行: 開発機は macOS。CI の Windows で実行する。
- Linux・Windows の `cargo test`: CI で実行する(`ci.yml` の `test` ジョブが 3 OS で実行する)。
