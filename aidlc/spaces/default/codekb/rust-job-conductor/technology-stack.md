# Technology Stack

## 言語・ビルド

| 項目 | 内容 | 証拠 |
|------|------|------|
| 言語 | Rust(edition 2024) | `Cargo.toml` の `[workspace.package]` |
| 最小の Rust(MSRV) | 1.100(README・CI・`Cargo.toml` の一致を `msrv_consistency` テストが検査) | `rust-version = "1.100"` |
| ビルド | Cargo ワークスペース(`resolver = "3"`、members = `crates/*`)、`Cargo.lock` をコミット | `Cargo.toml` |
| リリースプロファイル | `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` | `Cargo.toml` |
| 実行ファイル | `conductor`(cli の `[[bin]]`)。テスト専用に `conductor-cli-test-helper`、`conductor-test-helper`(`test = false`、配布に含めない) | `crates/cli/Cargo.toml`、`crates/runtime/Cargo.toml` |
| 対応 OS | Linux(x86_64、musl 向けのリリース)、macOS(Apple シリコン、Intel はクロスビルド)、Windows(x86_64 MSVC) | README、`release.yml` |
| ローカルの Rust | cargo 1.100.0 | 実行環境(参考) |

## 依存ライブラリ(`Cargo.lock` の版)

| ライブラリ | 版 | 使う crate | 用途 |
|------------|-----|------------|------|
| toml | 1.2.1(`default-features = false`) | core(`parse`、`std`)、runtime・converter(`parse`、`std`、`display`、`serde`) | 定義(位置つき)とステートの TOML、converter の書き出し |
| thiserror | 2.0.21 | core、runtime、converter | エラー型の Display |
| saphyr-parser | 0.3.4 | converter | YAML のイベント列 |
| lexopt | 0.3.2 | cli | 引数の字句分け |
| signal-hook | 0.4.6(`default-features = false`、`iterator`) | cli(UNIX) | シグナルの監視 |
| ctrlc | 3.5.2 | cli(Windows) | Ctrl+C |
| libc | 0.2 | runtime(UNIX) | `kill`、`pid_t`、シグナル番号 |
| tempfile | 3.27.0 | dev(4 crate) | テスト |

- serde の derive は使わない(`toml` の `serde` 機能は有効だが、自前の走査で型付けする)。HTTP・非同期ランタイム・ログのライブラリ・時刻のライブラリはない。
- CI が「C のビルドを要する依存(`cc`・`cmake`・`pkg-config`・`bindgen`)と `-sys` crate(`linux-raw-sys`・`windows-sys` を除く)が通常の依存にないこと」を検査する。

## 品質ツール・CI

| 項目 | 内容 |
|------|------|
| フォーマット | `cargo fmt --all -- --check`(`rustfmt` 既定) |
| リント | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| テスト | `cargo test --workspace --locked`(ubuntu-24.04、macos-latest、windows-2022)、MSRV 1.100 でも実行 |
| カバレッジ | `cargo llvm-cov --workspace --locked --fail-under-lines 80`(テスト用バイナリは除外) |
| 脆弱性 | `cargo audit`(週次と PR。警告のみ、マージはブロックしない) |
| 起動時間 | `nfr_perf`(警告のみ。`STARTUP_BENCH_ENFORCE=true` で強制) |
| リリース | タグ `v*` で `release.yml` が、タグが main 上にあり crate の版と一致することを確認し、3 OS の成果物と `SHA256SUMS` を作って下書きで公開 |
| 文書の検査 | `docs-check`: README に `SHA-256`・`Gatekeeper`・`終了コードの表`・`ステートファイルの場所`・`at-least-once`・`シグナル`・`convert` が含まれること |

## 実行時の特性

- 単一プロセス。ジョブは子プロセスとして直接起動(シェルなし)。待ちは 1 スレッドのポーリング(1 ms から 10 ms)、シグナルの監視は専用のスレッド 1 つ。
- ファイルの上限: 定義・ステート・変換の入力は各 1 MiB。
- 外部のサービス・ネットワークへの接続はなし。
