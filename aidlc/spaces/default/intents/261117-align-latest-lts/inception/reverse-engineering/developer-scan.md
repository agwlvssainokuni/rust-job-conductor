## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`
  - `.github/workflows/ci.yml`, `.github/workflows/audit.yml`, `.github/workflows/release.yml`
  - `README.md`
  - `crates/core/src/lib.rs`、`crates/runtime/src/lib.rs`、`crates/cli/src/lib.rs`、`crates/converter/src/lib.rs`(クレートの契約と構成を記した文書コメント)
  - `crates/cli/src/deps.rs`(差し替え口の trait)
  - `.gitignore`
- **Skimmed only**:
  - `crates/*/src/`(ファイル名と行数、属性・`unsafe` の grep)
  - `crates/*/tests/`(ファイル名、`#[test]` の総数)
  - `demo/`(README の冒頭、ファイル構成)
  - `.idea/`、`LICENSE`
- **Left out**: `target/`(ビルド出力)、`local-docs/`(`.gitignore` で除外)、`.git/`、AI-DLC 自身のインストール(`.claude/`、`aidlc/` など)

### Packages Found
- conductor-core — library — Rust — 定義(TOML)の読み込み・検証と、フローの判断(純粋関数)
- conductor-runtime — library — Rust — ジョブの直接起動、シグナル転送、ステートの読み書き
- conductor-converter — library — Rust — 定義の YAML と TOML の相互変換
- conductor-cli — library + binary(`conductor`) — Rust — 引数解釈、流れの組み立て、表示、終了コード

### Build System
- **Type**: Cargo ワークスペース(`resolver = "3"`、メンバー `crates/*`)
- **Config Files**: `Cargo.toml`、`crates/*/Cargo.toml`、`Cargo.lock`(version 4、コミット済み)。`rust-toolchain*`、`rustfmt.toml`、`clippy.toml`、`.cargo/`、`deny.toml` は無い
- **Build Dependencies**: cli → core, runtime, converter / runtime → core / converter → (内部依存なし) / core → (内部依存なし)

### APIs Discovered
- CLI — `conductor` — サブコマンド 6(run, pipeline, watch, reset, validate, convert)と `--help`/`--version`
- ライブラリ公開関数 — core 5、runtime 約 10、converter 2、cli 4(`run_cli` ほか)
- ネットワーク API は無し

### Frameworks & Libraries
- lexopt — 0.3.2 — 引数の字句解析
- toml — 1.2.1(`default-features = false`) — TOML の読み書き
- saphyr-parser — 0.3.4 — YAML のイベント解析
- thiserror — 2.0.21 — エラー型
- signal-hook — 0.4.6(unix) — シグナル監視
- ctrlc — 3.5.2(windows) — Ctrl+C
- libc — 0.2(unix) — `kill`
- tempfile — 3.27.0(dev) — テスト

### Test Coverage
- **Test Directories**: `crates/{cli,core,runtime,converter}/tests/`(合計約 11,600 行、`#[test]` 約 716)、`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`
- **Test Frameworks**: 標準の `cargo test`。テスト用の補助バイナリ `conductor-test-helper`、`conductor-cli-test-helper`
- **Coverage Config**: CI の `cargo llvm-cov --fail-under-lines 80`(設定ファイルは無し)

### Code Quality Indicators
- **Linting**: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`(CI の lint ジョブ)。設定ファイルは無し
- **CI/CD**: `ci.yml`(lint、build-check、test、msrv-test、coverage、docs-check、startup-bench)、`audit.yml`(cargo audit)、`release.yml`(タグ `v*`)
- **Documentation**: README が詳細(日本語)。各クレートの `lib.rs` に契約を記した文書コメント。`demo/` に UNIX / Windows のデモ

### Technical Debt Signals
- `Cargo.toml` の `rust-version = "1.85"` と、CI の `dtolnay/rust-toolchain@1.85.0`(build-check、msrv-test)で、最小版が 2 か所に別々に書かれている(`ci.yml`、`Cargo.toml`)。README にも「Rust(1.85 以降)」(README.md:84)
- 他の CI は `@stable` で、版を固定していない。ローカルの `rustc` は 1.100.0(スキャン時の観測)
- CI のランナーは `ubuntu-24.04`、`windows-2022` の固定と、`macos-latest` の浮動が混在(`ci.yml`、`release.yml`)
- アクションの版: `actions/checkout@v6`、`upload-artifact@v7`、`download-artifact@v8`、`taiki-e/install-action@v2`
- `libc = "0.2"` だけ範囲指定が緩く、他は完全な版(`Cargo.lock` で固定)
- `unsafe` は `crates/runtime/src/platform/unix.rs:72` の 1 か所のみ
- TODO / FIXME は無し

## Handoff Summary
- **Intent-relevant finding**: 版を持つ項目は、(1) `Cargo.toml` の `edition = "2024"`、`rust-version = "1.85"`、`resolver = "3"`、(2) CI の最小版 1.85.0 と `@stable`、(3) ランナー(`ubuntu-24.04`、`windows-2022`、`macos-latest`)、(4) アクションの版(`checkout@v6` など)、(5) 依存の版(`Cargo.lock` version 4)。どれも「LTS」という語で表されておらず、`rust-toolchain` ファイルは無い。
- **Risks / follow-up**: 「LTS」が何を指すかは、この走査では決まらない(Rust に LTS は無い。ランナーの OS イメージ、MSRV の方針などが候補)。MSRV 1.85 は CI の 2 ジョブ、`Cargo.toml`、README の 4 か所に分散している。変更時は 4 か所の整合と、`-sys` crate / C ビルド依存を禁じる CI の検査(`ci.yml` build-check)を保つこと。
