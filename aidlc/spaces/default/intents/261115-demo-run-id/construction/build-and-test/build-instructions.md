# Build Instructions

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85(`rust-version = "1.85"`、edition 2024)。`rustup` で入れる。
- カバレッジの測定: `cargo install cargo-llvm-cov --locked` と `rustup component add llvm-tools-preview`(開発機のみ)。
- デモの実行: UNIX 版は POSIX の `sh`。Windows 版は `cmd.exe` だけ(PowerShell、WSL、Git Bash は要らない)。

## ビルドと確認

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor        # デモが使う実行ファイル(古い版が残っていないこと)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets
cargo +1.85 test --workspace --locked
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked   # Windows 専用のテストの型検査
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

## デモの確認

```bash
DEMO_SLEEP=0.1 sh demo/run-demo.sh                             # UNIX 版。場面 0〜9。終了コード 0
DEMO_LANG=en_US.UTF-8 DEMO_SLEEP=0.1 sh demo/run-demo.sh      # 英語の表示
cargo test -p conductor-cli --test unix_demo --locked          # UNIX 版のデモの自動確認
```

```bat
rem Windows 版(Windows の実機で、リポジトリの根から)
set DEMO_SLEEP=1
demo\windows\run-demo.cmd
```

- Windows 版の `run-demo.cmd` は、CI では実行されない(CI は `windows_demo.rs` が、定義とスクリプトを写して `conductor` を直接呼ぶ)。実機での実行は、人の確認の項目。

## README の文書チェック(CI の `docs-check` と同じ)

```bash
for kw in 'SHA-256' 'Gatekeeper' '終了コードの表' 'ステートファイルの場所' 'at-least-once' 'シグナル' 'convert'; do
  grep -q -- "$kw" README.md || echo "MISSING $kw"
done
```

何も出力しないこと。

## よくあるビルドの問題

| 症状 | 原因と対処 |
|------|-----------|
| デモが古い版で動く、場面 7〜9 が失敗する | `target/release/conductor` が古い。`cargo build --release --locked --bin conductor` で作り直す |
| `run-demo.cmd` が文字化けする、ラベルが見つからない | 文字コードが UTF-8、改行が CRLF であること。LF だけの行があると壊れる |
| UNIX 版の日本語の表示が英語になる | 利用者の `LC_ALL` が優先される古い版。このデモは、冒頭で `LC_ALL`・`LC_MESSAGES` を消す |
| `error[E0658]: let expressions in this position are unstable` | 1.85 で let チェーンを使っている。`is_some_and` などに書き換える |
