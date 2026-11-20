# 技術スタック

## 言語とツールチェーン

| 項目 | 値 | 根拠 |
|------|----|------|
| 言語 | Rust | `Cargo.toml` |
| edition | 2024 | ワークスペースの `[workspace.package]` |
| 最小の Rust(MSRV) | 1.100 | `rust-version = "1.100"`。CI の `build-check` と `msrv-test` が 1.100.0 で検査。README の表記との一致は `msrv_consistency.rs` が検査 |
| ワークスペース | resolver 3、`members = ["crates/*"]` | `Cargo.toml` |
| release プロファイル | opt-level 3、lto thin、codegen-units 1、strip | `Cargo.toml` |
| ライセンス | Apache-2.0 | `Cargo.toml`、`LICENSE` |
| 対応 OS | Linux、macOS、Windows | README、CI、release |
| ビルド/パッケージ管理 | Cargo(`Cargo.lock` を追跡。CI は `--locked`) | |

## ライブラリ(外部)

| クレート | 版(Cargo.toml) | 使う場所 | 用途 |
|----------|-----------------|----------|------|
| toml | 1.2.1(default-features 無効。core: parse, std。runtime/converter: さらに display, serde) | core、runtime、converter | 定義とステートの TOML の読み書き。core は `Spanned`、`DeTable`、`DeValue` で位置つきに読む |
| thiserror | 2.0.21 | core、runtime、converter | エラー型の `Display` |
| saphyr-parser | 0.3.4 | converter | YAML のイベント列の読み取り(自前で木と展開を組む) |
| lexopt | 0.3.2 | cli | 引数の字句分け |
| signal-hook | 0.4.6(default-features 無効、`iterator`) | cli(UNIX) | シグナルの監視 |
| ctrlc | 3.5.2 | cli(Windows) | Ctrl+C の監視 |
| libc | 0.2(ロックは 0.2.191) | runtime(UNIX) | `kill`。`unsafe` はここ 1 か所 |
| tempfile | 3.27.0 | 各クレートの dev-dependencies | テスト |

serde のデータ形式はステートの書き込みに使う(runtime の `toml` の `serde`、`display` 機能)。環境変数の読み込み(`.env`)や、グラフの描画のための外部のクレートは、現在は無い。

## 制約として守られていること(テストと CI で検査)

- C のコンパイルを要する依存と `-sys` クレートを持たない(CI の `build-check` の Linux)。
- `unsafe` は runtime の `platform/unix.rs` だけ。cli は `#![forbid(unsafe_code)]`(`nfr_source.rs`)。
- `cfg(unix)` / `cfg(windows)` は `platform` の外に置かない。
- `std::net` やネットワークのライブラリを使わない。
- 標準ライブラリだけの待機(`wait_for`)。

## 配布

タグ `v*` で `release.yml` が動く。対象 4 つ: `x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`(クロスビルド。動作の確認なし)、`x86_64-pc-windows-msvc`。成果物は tar.gz(Windows は zip)と `SHA256SUMS`。下書きのリリースを作り、人が公開する。署名・公証はしない。
