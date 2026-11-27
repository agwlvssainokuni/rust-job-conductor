# 技術スタック

## 言語とツールチェーン

| 項目 | 値 | 根拠 |
|------|----|------|
| 言語 | Rust(edition 2024) | 根の `Cargo.toml` の `[workspace.package]` |
| 最小の Rust(MSRV) | 1.100 | `rust-version = "1.100"`。CI の `build-check`・`msrv-test` が `1.100.0` で確かめる。README との一致は `crates/cli/tests/msrv_consistency.rs` |
| ビルド | Cargo ワークスペース(`resolver = "3"`、`members = ["crates/*"]`) | 根の `Cargo.toml` |
| 版 | 4 クレートとも 0.10.1(`publish = false`) | `crates/*/Cargo.toml` |
| リリースの最適化 | `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` | `[profile.release]` |
| 依存の固定 | `Cargo.lock` を追跡。CI は常に `--locked` | `ci.yml` |
| 書式・静的検査 | `rustfmt`、`clippy -D warnings`(設定ファイルなし、既定) | `ci.yml` の `lint` |
| カバレッジ | `cargo-llvm-cov`(行 80% 以上。テスト専用の補助バイナリを除く) | `ci.yml` の `coverage` |
| 脆弱性の検査 | `cargo-audit`(警告のみ) | `audit.yml` |
| C のビルド | 持たない(`cc`・`cmake`・`pkg-config`・`bindgen` と `-sys` クレートを検査で拒否。`linux-raw-sys`・`windows-sys` は例外) | `ci.yml` の `build-check` |

## 実行時の依存(直接)

| クレート | 版(`Cargo.toml` / ロック) | 使う場所 | 目的 |
|----------|---------------------------|----------|------|
| `toml` | 1.2.1 / 1.2.1+spec-1.1.0 | core(`parse`、`std`)、runtime・converter(`parse`、`std`、`display`、`serde`) | 定義・ステートの読み取り、ステートと変換の書き出し |
| `thiserror` | 2.0.21 | core、runtime、converter | エラー型の `Display` と `Error` |
| `lexopt` | 0.3.2 | cli | 引数の字句の分割 |
| `saphyr-parser` | 0.3.4 | converter | YAML のイベントの読み取り |
| `signal-hook` | 0.4.6(`default-features = false`、`iterator`) | cli(UNIX) | シグナルの監視 |
| `ctrlc` | 3.5.2 | cli(Windows) | Ctrl+C の監視 |
| `libc` | 0.2 / 0.2.191 | runtime(UNIX) | `kill`(1 か所) |

## 開発時の依存

| クレート | 版 | 目的 |
|----------|----|------|
| `tempfile` | 3.27.0 | テストの一時ディレクトリ(4 クレート)。runtime の `log.rs` の単体テストでも使う |

推移的な依存と依存の方針は `dependencies.md` を参照。v0.10.1 で `Cargo.lock` が変わったのは 4 クレートの版だけ。

## 標準ライブラリで自前に書いているもの

新しい依存を加えない方針(`project.md` の Forbidden、`team.md` の Code Style)に従い、標準ライブラリで書いている主なもの。

- command の分割(`runtime::split`)、プロセスの起動と待ち(`std::process`、`try_wait` のくり返し)
- ジョブの出力の複製(`std::thread` と `std::sync::mpsc`、8 KiB ずつの読み取り。`runtime::runner`)
- 時刻(UTC の秒、グレゴリオ暦の計算、RFC 3339 と詰めた形の整形・読み取り。`core::time`)と時計(`SystemTime`。`runtime::clock`)
- `.env` の解析(`core::envfile::parse`)
- 図の配置と整形(text、Mermaid、DOT)
- `watch --follow` の描き直し(ANSI の `ESC[H ESC[2J`)、表示の escape、英日の文言、ロケールの判定、端末の色
- ファイルの原子的な置き換え、上限つきの読み取り、ログのファイルの排他的な作成
- 待機(`std::thread::sleep` と `Instant`)
- 非同期ランタイム、ログの枠組み、日時のライブラリ、シリアライズの derive は使っていない

## 文書の技術

- 仕様書(`docs/spec/`)と README は Markdown。図は文字の図(`text` のコードブロック)で、Mermaid の図は使っていない(`docs/spec/rust/` と `docs/spec/language-agnostic/` に mermaid のコードブロックは 0 件。Mermaid は `graph --format mermaid` の出力の例としてだけ現れる)。
- 仕様書を生成・検査する道具は無い(手で書いた文書。テストと CI から読まれない)。

## 対象の OS と配布

| 対象 | ビルド | 動作の確認 |
|------|--------|------------|
| `x86_64-unknown-linux-musl` | ubuntu-24.04(musl-tools) | `--version` と静的リンクの確認 |
| `aarch64-apple-darwin` | macos-latest | `--version` |
| `x86_64-apple-darwin` | macos-latest(クロスビルド) | 確認なし(リリースの注記に明記) |
| `x86_64-pc-windows-msvc` | windows-2022 | `--version` |

- 配布物: `conductor`(Windows は `conductor.exe`)と `LICENSE` を tar.gz(Windows は zip)に入れ、`SHA256SUMS` を添える。テスト専用の補助バイナリは含めない。署名・公証はしない。
- リリースは下書き(draft)として作られ、人が確認して公開する。版に `-` を含むタグは prerelease。

## CI の実行環境

- `test`: ubuntu-24.04、macos-latest、windows-2022(stable)
- `build-check`: ubuntu-24.04、windows-2022(1.100.0)
- `msrv-test`、`coverage`、`lint`、`docs-check`: ubuntu-24.04
- `startup-bench`: 3 つの OS(`continue-on-error`、`STARTUP_BENCH_ENFORCE=false`、`--release`)
- 性能のテスト(`crates/core/tests/*_perf.rs` など)は、`test`・`msrv-test` では debug で、`coverage` では計測つきの debug で走る。上限は「debug の見積もりの約 5 倍」で決めている(例 `plan_simulate_perf.rs:39` の 150ms。v0.10.1 で下げずに、計画の模擬の内部の入口で走査を減らした)。

## 開発環境

- 主な開発環境の OS は macOS(`team.md`)。デモは UNIX 系の `sh` と Windows の cmd の 2 本。
- `.idea/` の IntelliJ 系のプロジェクト設定を追跡している(この走査では開いていない)。
- `crates/cli/tests/fixtures/*.toml` は `.gitattributes` で LF に固定。
