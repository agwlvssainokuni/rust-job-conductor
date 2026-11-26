# ビルドの手順: v0.10.1

出典: `construction/code-generation/code-summary.md`、`unit-test-instructions.md`、team.md の Code Style・Testing Posture、`.github/workflows/ci.yml`。

## 前提

- Rust の toolchain(安定版。最小の版は MSRV 1.100)と cargo。カバレッジには `cargo-llvm-cov` と `llvm-tools-preview`。
- 依存は `Cargo.lock` で固定し、常に `--locked` を付ける。v0.10.1 で依存は増えていない(`Cargo.lock` の package は 46 のまま)。
- 環境変数や外部のサービスは要らない。デモのテスト(`unix_demo.rs`)は UNIX 系の `sh` を使う。

## ビルド

```bash
cargo build --workspace --locked
```

## ビルドの確かめ

```bash
cargo fmt --all -- --check
```

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

```bash
cargo test --workspace --locked
```

```bash
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

- ubuntu・macos・windows と MSRV 1.100 の確かめは、`main` への push で動く CI(`ci.yml` の `lint`、`build-check`、`test`、`msrv-test`、`coverage`、`docs-check`、`startup-bench`)が行う。

## よくある失敗

- `--locked` で `Cargo.lock` の更新を求められる: 版を上げたときに `Cargo.lock` を合わせていない。`cargo build --workspace` で一度合わせてからコミットする。
- `cargo llvm-cov` が無い: `cargo install cargo-llvm-cov` と `rustup component add llvm-tools-preview`。
- 性能のテスト(`*_perf.rs`)がカバレッジの計測の下で上限を超える: v0.10.0 の CI で 2 回あった(F3)。上限は下げない。再び起きたら、`build-and-test-summary.md` の「原因不明の失敗の記録の手順」に従って記録する。
