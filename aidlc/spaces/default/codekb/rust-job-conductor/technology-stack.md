# Technology Stack

## 言語とビルド

| 項目 | 値 | 場所 |
|------|-----|------|
| 言語 | Rust | |
| edition | 2024 | `Cargo.toml` `[workspace.package]` |
| rust-version(MSRV) | 1.85 | `Cargo.toml` `[workspace.package]` |
| resolver | 3 | `Cargo.toml` `[workspace]` |
| Cargo.lock | あり、`version = 4`、コミット済み | `Cargo.lock` |
| rust-toolchain ファイル | なし | |
| rustfmt / clippy の設定ファイル | なし(既定) | |
| ライセンス | Apache-2.0 | `Cargo.toml` |
| クレートの版 | 0.7.0(4 クレートとも) | `crates/*/Cargo.toml` |
| release プロファイル | opt-level 3、lto "thin"、codegen-units 1、strip | `Cargo.toml` |
| スキャン時のローカルのツールチェーン(観測) | rustc 1.100.0、cargo 1.100.0 | 実行環境 |

## CI(`.github/workflows/ci.yml`)

トリガ: main への push と pull_request、`workflow_call`。`actions/checkout@v6`。

| ジョブ | ランナー | Rust |
|--------|----------|------|
| lint(fmt、clippy `-D warnings`) | ubuntu-24.04 | `dtolnay/rust-toolchain@stable` |
| build-check(`--locked --all-targets`、C ビルド依存と `-sys` の検査) | ubuntu-24.04、windows-2022 | `dtolnay/rust-toolchain@1.85.0` |
| test | ubuntu-24.04、macos-latest、windows-2022 | `@stable` |
| msrv-test | ubuntu-24.04 | `@1.85.0` |
| coverage(`taiki-e/install-action@v2` で cargo-llvm-cov、行 80%) | ubuntu-24.04 | `@stable` + `llvm-tools-preview` |
| docs-check(README の必須語の検査) | ubuntu-24.04 | なし |
| startup-bench(`continue-on-error`) | ubuntu-24.04、macos-latest、windows-2022 | `@stable` |

## 監査(`audit.yml`)

pull_request(main)と毎週月曜 03:00(cron)。ubuntu-24.04、`@stable`、`taiki-e/install-action@v2`(cargo-audit)、`continue-on-error: true`。

## リリース(`release.yml`)

タグ `v*`。verify-tag(ubuntu-24.04)→ checks(`ci.yml` を呼ぶ)→ build → draft-release。

| ターゲット | ランナー |
|-----------|----------|
| x86_64-unknown-linux-musl | ubuntu-24.04(`musl-tools` を apt で導入) |
| aarch64-apple-darwin | macos-latest |
| x86_64-apple-darwin(クロスビルド、スモークテストなし) | macos-latest |
| x86_64-pc-windows-msvc | windows-2022 |

Rust は `dtolnay/rust-toolchain@stable`。アクション: `actions/checkout@v6`、`actions/upload-artifact@v7`、`actions/download-artifact@v8`。

## その他

- README のビルド要件: 「Rust(1.85 以降)」(README.md:84)
- `.idea/`(IDE の設定)がコミットされている
- デモ: `demo/`(sh と cmd)
