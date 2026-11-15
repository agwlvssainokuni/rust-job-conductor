# 単体テストの手順: Windows 向けデモ

## 準備

- 既存のテストの仕組みを、そのまま使う(`cargo test`、`tempfile`、`crates/cli/tests/bin_support`)。追加のインストールは要らない。
- 新しいテストは、`crates/cli/tests/windows_demo.rs`(`#[cfg(windows)]`)。Windows 以外では、中身は動かない(0 件)。

## このテストだけを動かすコマンド(Windows)

```
cargo test -p conductor-cli --test windows_demo
```

## 既存のテストが、緑のままであることの確認

```
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

## テストの内容(5 件、最小の方針)

| # | 要件 | 確かめること |
|---|---|---|
| 1 | FR1.3 | `demo/windows/conductor.toml` が読めて、4 つのジョブと、`extract` の警告の遷移を持つ |
| 2 | FR2.1(場面 1) | 全部成功 → 終了コード 0 |
| 3 | FR2.1・FR2.2(場面 2) | `DEMO_EXTRACT_EXIT=1` → `transform` と `load` を飛ばして `notify` へ。終了コード 0 |
| 4 | FR2.1・FR2.2(場面 3・4) | `DEMO_LOAD_EXIT=2` → 終了コード 2。環境変数なしでもう一度 → `load` から再開して完了 |
| 5 | FR2.1(場面 5・6) | `run`、`reset`、`convert --to yaml` が終了コード 0 |

## 実機での確認(自動のテストではない)

- `demo\windows\run-demo.cmd` を、このマシン(Windows 11)で最後まで動かす(NFR3)。
- `set DEMO_PAUSE=1` と、`DEMO_LANG` を付けた実行を、1 回ずつ確かめる。

## カバレッジ

- 行カバレッジ 80% の下限(チームの方針)は、全体のテストで、Build and Test が確かめる。この作業は Rust のコードを変えないので、測る対象の行は増えない。
- 新しいテストは、環境変数と一時ディレクトリを使う。各テストは、自分の一時ディレクトリで動き、ほかのテストに影響しない。

## テストデータと、代役(モック)

- テストの入力は、`demo/windows/conductor.toml` と `demo/windows/scripts\*.cmd` そのもの(写して使う)。代役は使わない。
- `DEMO_SLEEP` は、テストでは 0 にして、待ち時間をなくす。
