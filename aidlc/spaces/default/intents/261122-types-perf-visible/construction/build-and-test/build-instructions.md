# ビルドの手順(Build Instructions)

既存の Rust のワークスペースをそのまま使う。新しい設定と依存は無い(`Cargo.toml` と `Cargo.lock` の差分は版の行だけ)。

## 前提

- Rust 1.100.0 以上(MSRV 1.100)。手元の rustc は 1.100.0。
- 行カバレッジの測定には `cargo-llvm-cov`。
- 名前付きパイプのテスト(N12、UNIX だけ)は `mkfifo` コマンドを使う(ubuntu・macOS の標準)。

## コマンド(CI と同じ)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --locked --workspace
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

上流: `construction/code-generation/code-generation-plan.md`(Step 1〜13)、`construction/code-generation/unit-test-instructions.md`(実行のコマンド)。
