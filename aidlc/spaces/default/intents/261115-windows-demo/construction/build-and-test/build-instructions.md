# ビルドの手順

## 前提

- Rust(安定版。`rust-version` は 1.85 以上)と Cargo。
- Windows 11(x86_64)。ほかの OS でも、Rust のビルドとテストは同じ手順で動く(Windows 専用のテストは 0 件になる)。
- 追加のライブラリのインストールは要らない。依存は `Cargo.lock` で固定している(`--locked`)。
- 行カバレッジを測るときだけ、次の 2 つを入れる:
  - `rustup component add llvm-tools-preview`
  - `cargo install cargo-llvm-cov --locked`

## ビルド

```
cargo build --release --locked --bin conductor
```

確認: `target\release\conductor.exe --version` が `conductor 0.3.0` を出す。

## 静的検査

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## テスト

```
cargo test --workspace --locked
cargo test -p conductor-cli --test windows_demo --locked     # この作業のテストだけ(Windows のみ)
cargo llvm-cov --workspace --locked --ignore-filename-regex "bin[\\/]conductor-(cli-)?test-helper" --summary-only
```

## デモの実行(実機の確認)

```
demo\windows\run-demo.cmd
set DEMO_PAUSE=1 & demo\windows\run-demo.cmd     (各場面の後で Enter を待つ)
```

## つまずきやすい点

- `.cmd` は CRLF で保存する。LF だけだと、`goto` やラベルが壊れることがある。
- 待ち時間は `ping` を使う。`timeout` は、標準入力が端末でないと失敗する。
- 日本語の表示が崩れるときは、コードページ(`chcp 65001`)を確かめる。`run-demo.cmd` は、実行中だけ UTF-8 にして、終わりに戻す。
