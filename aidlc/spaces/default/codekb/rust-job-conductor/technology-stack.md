# 技術スタック

| 項目 | 内容 | 根拠 |
|------|------|------|
| 言語 | Rust(edition 2024)。最小の Rust は 1.85(`rust-version`) | `Cargo.toml:4-6` |
| ビルド | Cargo ワークスペース(`crates/*`、`resolver = "3"`)。`--locked` を使う | `Cargo.toml`、CI |
| release プロファイル | `opt-level=3`、`lto="thin"`、`codegen-units=1`、`strip=true` | `Cargo.toml` |
| 配布 | 単一の静的に近い実行ファイル `conductor`。Linux は musl で静的リンク | `release.yml` |
| 対象 OS | Linux(x86_64 musl)、macOS(aarch64・x86_64)、Windows(x86_64 msvc)。CI のテストは ubuntu-24.04・macos-latest・windows-2022 | `release.yml`、`ci.yml` |
| 定義・ステートの形式 | TOML(`format_version = 1`)。変換先に YAML | `core`、`runtime`、`converter` |
| ライブラリ | `toml` 1.2.1、`thiserror` 2.0.21、`saphyr-parser` 0.3.4、`lexopt` 0.3.2、`signal-hook` 0.4.6(UNIX、`iterator` のみ)、`ctrlc` 3.5.2(Windows)、`libc` 0.2(UNIX)。dev: `tempfile` 3.27.0 | `crates/*/Cargo.toml`、`Cargo.lock`(46 パッケージ) |
| ネットワーク・DB | なし | `cli/src/lib.rs` の doc |
| テスト | `cargo test`。カバレッジは `cargo-llvm-cov`(行 80% 未満で失敗) | `ci.yml` |
| 整形・静的解析 | `rustfmt`(`cargo fmt`)、`clippy`(`-D warnings`)。設定ファイルなし(既定) | `ci.yml` |
| 脆弱性の検査 | `cargo-audit`(警告のみ) | `audit.yml` |
| CI | GitHub Actions(`actions/checkout@v6`、`dtolnay/rust-toolchain`、`taiki-e/install-action@v2`、`upload-artifact@v7`、`download-artifact@v8`) | `.github/workflows/` |
| 文書・コメント | 日本語 | 全体 |
