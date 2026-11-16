# Technology Stack

| 項目 | 内容 |
|------|------|
| 言語 | Rust(edition 2024、`rust-version = 1.85`) |
| ビルド | Cargo ワークスペース(`resolver = "3"`)。リリースは `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` |
| 配布 | 単一バイナリ `conductor`(4 ターゲット: x86_64 Linux musl、aarch64・x86_64 macOS、x86_64 Windows MSVC) |
| ライセンス | Apache-2.0(`LICENSE`) |
| フォーマッタ・リンタ | `rustfmt`、`clippy -D warnings`(設定ファイルなし) |
| テスト | `cargo test`(約 681 件)。カバレッジ 80% を CI が要求 |
| CI | GitHub Actions(`ci.yml`、`release.yml`、`audit.yml`) |
| 表示 | 英語・日本語(`LANG` などで決める。`report/messages.rs`) |

ライブラリのバージョンは `dependencies.md`。ネットワーク接続もテレメトリもない。`unsafe` は `conductor-runtime` の UNIX の `kill` 呼び出し 1 か所だけ(`conductor-cli` は `forbid(unsafe_code)`)。
