# 技術スタック

## 言語とツールチェーン

| 項目 | 値 | 根拠 |
|------|----|------|
| 言語 | Rust(edition 2024) | 根の `Cargo.toml` の `[workspace.package]` |
| 最小の Rust(MSRV) | 1.100 | `rust-version = "1.100"`。CI の `build-check`・`msrv-test` が `1.100.0` で確かめる。README と `ci.yml` との一致は `crates/cli/tests/msrv_consistency.rs` |
| ビルド | Cargo ワークスペース(`resolver = "3"`、`members = ["crates/*"]`) | 根の `Cargo.toml` |
| 版 | 4 クレートとも 0.10.3(`publish = false`) | `crates/*/Cargo.toml:3` |
| リリースの最適化 | `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` | `[profile.release]` |
| 依存の固定 | `Cargo.lock` を追跡。CI は常に `--locked` | `ci.yml` |
| 書式・静的検査 | `rustfmt`、`clippy -D warnings`(設定ファイルなし、既定) | `ci.yml` の `lint` |
| カバレッジ | `cargo-llvm-cov`(行 80% 以上。テスト専用の補助バイナリを除く) | `ci.yml` の `coverage` |
| 脆弱性の検査 | `cargo-audit`(警告のみ) | `audit.yml` |
| C のビルド | 持たない(`-sys` クレートは `linux-raw-sys`・`windows-sys` を除いて拒否) | `ci.yml` の `build-check` |

## 実行時の依存(直接)

| クレート | 版(`Cargo.toml` / ロック) | 使う場所 | 目的 |
|----------|---------------------------|----------|------|
| `toml` | 1.2.1 / 1.2.1+spec-1.1.0 | core(`parse`、`std`)、runtime・converter(`parse`、`std`、`display`、`serde`) | 定義・ステートの読み取り(core の `walk` は `toml::de::{DeTable, DeValue}` と `Spanned` で位置を持つ木を走査)、ステートと変換の書き出し |
| `thiserror` | 2.0.21 | core、runtime、converter | エラー型 |
| `lexopt` | 0.3.2 | cli(`args.rs` だけ。`lexopt::prelude::*`) | 引数の字句の分割。`next` が返す誤りは `UnexpectedValue` だけ(`args.rs:381` の注記) |
| `saphyr-parser` | 0.3.4 | converter | YAML のイベントの読み取り |
| `signal-hook` | 0.4.6(`default-features = false`、`iterator`) | cli(UNIX) | シグナルの監視 |
| `ctrlc` | 3.5.2 | cli(Windows) | Ctrl+C の監視 |
| `libc` | 0.2 / 0.2.191 | runtime(UNIX) | `kill`(1 か所) |

## 開発時の依存

| クレート | 版 | 目的 |
|----------|----|------|
| `tempfile` | 3.27.0 | テストの一時ディレクトリ |

推移的な依存と依存の方針は `dependencies.md` を参照。

## 標準ライブラリで自前に書いているもの

新しい依存を加えない方針(`project.md` の Forbidden、`team.md` の Code Style)に従い、標準ライブラリで書いている主なもの。

- command の分割、プロセスの起動と待ち、出力の複製(`std::thread`、`std::sync::mpsc`)
- 時刻(グレゴリオ暦の計算、RFC 3339 と詰めた形)と時計
- `.env` の解析、図の配置と整形(text、Mermaid、DOT)
- 表示の遅延の判定(`std::cell::OnceCell`。`report/mod.rs`)、表示の escape、英日の文言、ロケールの判定、端末の色、`watch --follow` の描き直し
- 引数の表と検査の順(`args.rs` の `const` の表とコンパイル時の `const _` の検査、`const fn is_long_spelling`)
- ファイルの原子的な置き換え、上限つきの読み取り、ログのファイルの排他的な作成
- 文書とソースを読むテスト(`std::fs::read_to_string` と `str` の検索。Markdown の解析器や正規表現のライブラリは使わない。`spec_numbers.rs` も同じ)

## 文書の技術

- 仕様書(`docs/spec/`)と README は Markdown。図は文字の図。
- 仕様書の数は `crates/cli/tests/spec_numbers.rs` が実装と突き合わせる(v0.10.3)。仕様書を生成する道具は無い。

## 対象の OS と配布

| 対象 | ビルド | 動作の確認 |
|------|--------|------------|
| `x86_64-unknown-linux-musl` | ubuntu-24.04(musl-tools) | `--version` と静的リンクの確認 |
| `aarch64-apple-darwin` | macos-latest | `--version` |
| `x86_64-apple-darwin` | macos-latest(クロスビルド) | 確認なし(リリースの注記に明記) |
| `x86_64-pc-windows-msvc` | windows-2022 | `--version` |

- 配布物: `conductor`(Windows は `conductor.exe`)と `LICENSE` を tar.gz(Windows は zip)に入れ、`SHA256SUMS` を添える。テスト専用の補助バイナリは含めない。署名・公証はしない。
- リリースは下書き(draft)として作られ、人が確認して公開する。

## CI の実行環境

- `test`: ubuntu-24.04、macos-latest、windows-2022(stable)
- `build-check`: ubuntu-24.04、windows-2022(1.100.0)
- `msrv-test`、`coverage`、`lint`、`docs-check`: ubuntu-24.04
- `startup-bench`: 3 つの OS(`continue-on-error`、`STARTUP_BENCH_ENFORCE: "false"`)
- 性能のテストは debug で走り、固定の時間の上限を超えれば不合格。

## 開発環境

- 主な開発環境の OS は macOS(`team.md`)。デモは UNIX 系の `sh` と Windows の cmd。
- `.idea/` の IntelliJ 系のプロジェクト設定を追跡している(この走査では開いていない)。
