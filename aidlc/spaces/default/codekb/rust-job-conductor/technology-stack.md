# Technology Stack

| 区分 | 内容 |
|------|------|
| 言語 | Rust(edition 2024、最小 1.85)。デモは POSIX sh |
| ビルド | Cargo ワークスペース(resolver 3)。release は opt-level 3、thin LTO、codegen-units 1、strip |
| 配布ターゲット | x86_64-unknown-linux-musl、aarch64-apple-darwin、x86_64-apple-darwin、x86_64-pc-windows-msvc(zip、`conductor.exe`) |
| CI | GitHub Actions(ubuntu-24.04、macos-latest、windows-2022)。ツールチェーンは dtolnay/rust-toolchain |
| ライセンス | Apache-2.0 |

ライブラリの版は dependencies.md を参照。Windows 上のデモ実行に使える既定の環境は、PowerShell と cmd(OS 標準)。`sh` は標準では入っていない。
