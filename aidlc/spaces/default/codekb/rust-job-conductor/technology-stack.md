# Technology Stack

| 項目 | 内容 |
|------|------|
| 言語 | Rust(edition 2024、`rust-version = 1.85`) |
| ビルド | Cargo ワークスペース(resolver 3)。release は `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` |
| ライセンス | Apache-2.0(各ソースにライセンスコメント) |
| 実行環境 | Linux(musl)、macOS(Apple シリコン/Intel)、Windows(MSVC)。外部ランタイム不要 |

## ライブラリ(`Cargo.lock` の版)

| 名前 | 版 | 用途 | 使うクレート |
|------|----|------|--------------|
| toml | 1.2.1 | 定義・ステートの読み書き。runtime は `parse`・`std`・`display`・`serde` 機能(`default-features = false`)で、`toml::Table` と `toml::Value` を手で歩く | core, runtime, converter |
| thiserror | 2.0.21 | エラー型 | core, runtime, converter |
| saphyr-parser | 0.3.4 | YAML のイベント読み取り | converter |
| lexopt | 0.3.2 | 引数解析 | cli |
| signal-hook | 0.4.6 | UNIX のシグナル | cli |
| ctrlc | 3.5.2 | Windows の Ctrl+C | cli |
| libc | 0.2(ロック 0.2.191) | プロセスグループ・シグナル送信 | runtime(UNIX) |
| tempfile | 3.27.0 | テスト | 全クレート(dev) |

runtime の `Cargo.toml` は v0.2.0(今回確認)。上の版は前回の記録を引き継ぎ、今回は runtime の `Cargo.toml` の指定だけを確認した。

## ツール・CI

- フォーマッタ `rustfmt`、リンタ `clippy`(`-D warnings`)、`cargo llvm-cov`(行 80% 以上)、`cargo audit`(警告のみ)。設定ファイル(`rustfmt.toml` など)は無し。
- GitHub Actions: `ci.yml`、`release.yml`(タグで 4 ターゲットのバイナリと `SHA256SUMS`)、`audit.yml`。
- C のビルドが要る依存(`cc`、`cmake`、`-sys` クレート)は CI で禁止(`linux-raw-sys`・`windows-sys` を除く)。
