# Technology Stack

| 区分 | 内容 |
|------|------|
| 言語 | Rust(edition 2024、最小 1.85)。デモは POSIX sh と Windows cmd |
| ビルド | Cargo ワークスペース(resolver 3、全クレート 0.5.0)。release は opt-level 3、thin LTO、codegen-units 1、strip |
| 配布ターゲット | x86_64-unknown-linux-musl、aarch64-apple-darwin、x86_64-apple-darwin、x86_64-pc-windows-msvc(zip、`conductor.exe`) |
| CI | GitHub Actions(ubuntu-24.04、macos-latest、windows-2022)。ツールチェーンは dtolnay/rust-toolchain(前回の記録。今回は未走査) |
| ライセンス | Apache-2.0 |

ライブラリの版は dependencies.md を参照。Windows 上のデモ実行には、PowerShell と cmd(OS 標準)を使う。`sh` は標準では入っていないため、Windows 版は cmd のスクリプト(`demo/windows/`)になっている。cmd のデモは、待ちに `ping`、コードページの切り替えに `chcp` を使う。ステートと定義はどちらも TOML(`toml` クレート。ステートは `toml::Table` で書き、`toml::Table` で読む)。
