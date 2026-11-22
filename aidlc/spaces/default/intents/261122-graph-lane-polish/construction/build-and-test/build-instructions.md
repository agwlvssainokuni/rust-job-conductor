# ビルドの手順(Build Instructions): graph の text 表示の改善

対象: ワークスペース `conductor`(`conductor-core`、`conductor-runtime`、`conductor-converter`、`conductor-cli`)。この作業の変更は core と cli に限られる。

## 前提

- Rust 1.100 以上(MSRV 1.100、edition 2024)。このマシンは `rustc 1.100.0`(MSRV と同じ)。
- カバレッジの計測: `cargo-llvm-cov`(`llvm-tools-preview`)。
- 新しい依存は足していない(`Cargo.toml`、`Cargo.lock` に、この作業の差分はない)。

## コマンド

```bash
cargo build --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --locked --workspace
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

README の必須語句(CI の `docs-check`): `SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert` が、`README.md` に残っていること。

## このマシンで確かめられないもの

- Windows と ubuntu のビルドとテスト、`msrv-test`(1.100.0。このマシンの rustc が同じ版なので、ここでの結果が兼ねる): CI(`.github/workflows/ci.yml`。push で動く)で確かめる。この作業の変更は、OS に依存するコードを足していない。

## よくある問題

- 計測つきの実行は遅い。マシンが忙しいと、時間のかかるテスト(`unix_demo` の一部、`conductor-converter` の 16 MiB のテスト)が 30 秒の上限を超えることがある。単独で再実行すると通る。
