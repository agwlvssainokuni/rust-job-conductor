# ビルドの手順: v0.10.2

出典: `construction/code-generation/code-summary.md`、`unit-test-instructions.md`、`code-generation-plan.md` の付録 B、team.md の Code Style・Testing Posture、`.github/workflows/ci.yml`。

## 前提

- Rust の toolchain(MSRV 1.100)と cargo。カバレッジには `cargo-llvm-cov`。依存は `Cargo.lock` で固定し、常に `--locked`。v0.10.2 で依存は増えていない(package 46)。
- この版の変更は文書(`docs/spec/`、`team.md`)と、`crates/runtime/src/lib.rs` のドキュメントコメントと版だけ。コードの動きは変わらない。

## ビルドと確かめ

```bash
cargo build --workspace --locked
```

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
cargo doc -p conductor-runtime --no-deps --locked
```

```bash
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

## 仕様書の確かめ(計画の付録 B)

```bash
grep -rn '33 種\|13 種' docs/spec/rust/ docs/spec/language-agnostic/
```

```bash
grep -rn '足すだけ\|取り除かない\|取り除く書き方\|読まない・保存しない\|1 回だけ書\|0〜14' docs/spec/rust/ docs/spec/language-agnostic/
```

- 前者は履歴として残す 3 行(rust の 12:105・12:107、language-agnostic の 12:129)だけ、後者は付録 A の「残してよい記述」の行だけを返す。

## よくある失敗

- `cargo doc` の警告は作業の前から 4 件ある(増えていないことを確かめる)。
- 版を上げたあと `--locked` で `Cargo.lock` の更新を求められる: `Cargo.lock` を合わせていない。
- ubuntu・windows・MSRV は、`main` への push の後の CI(`ci.yml`)で確かめる。
