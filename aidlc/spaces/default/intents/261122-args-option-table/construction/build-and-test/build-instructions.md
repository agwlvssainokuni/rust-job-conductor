# ビルドの手順(Build Instructions): args.rs のオプションの表化

新しい依存も設定も無い。既存の手順のまま。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --locked --workspace
cargo test --locked --workspace
cargo llvm-cov --workspace --locked --fail-under-lines 80 --ignore-filename-regex 'test-helper'
```

- MSRV 1.100 と、ubuntu・windows のテストは、push の後の CI(`build-check`、`msrv-test`、`test`)で確かめる。
- `Cargo.toml` と `Cargo.lock` は ba1cdfb から変わっていない。
