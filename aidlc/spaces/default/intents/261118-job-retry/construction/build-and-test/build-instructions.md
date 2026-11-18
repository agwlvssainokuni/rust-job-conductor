# Build Instructions — job-retry

## 前提

- Rust 1.100.0 以降(`rustc --version`)。`Cargo.toml` の `rust-version` と一致する
- 依存のインストールの手順は不要(`cargo` が `Cargo.lock` に従って取得する)。新しい外部クレートは、この変更で増えていない
- 環境変数や設定ファイル、ローカルのサービスは不要
- カバレッジの測定には、`cargo-llvm-cov` と `llvm-tools-preview` が要る(CI と同じ)

## ビルドのコマンド

| 目的 | コマンド |
|---|---|
| ビルド(全クレート) | `cargo build --workspace --locked` |
| 書式の検査 | `cargo fmt --all -- --check` |
| 静的解析(CI の `lint` と同じ) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Windows 向けのコンパイルと静的解析(補助。テストは実行できない) | `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` |

## ビルドの確認

1. `cargo build --workspace --locked` が成功し、警告がないこと
2. `./target/debug/conductor --version` が版を表示すること
3. `./target/debug/conductor validate --config <定義ファイル>` が、`[job.retry]` を含む定義で成功すること

## つまずきやすい点

- `cargo llvm-cov` の値が、実際より低く出るとき(古い成果物が残っている): `cargo clean --target-dir target/llvm-cov-target` のあと、もう一度測る(この変更の途中で、1 回起きた。u1 の確認)
- `--locked` を付けると、`Cargo.lock` が古いときに失敗する。依存を変えていなければ、起きない
