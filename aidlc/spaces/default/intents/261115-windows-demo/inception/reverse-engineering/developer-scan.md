## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`, `README.md`
  - `demo/`(README.md、conductor.toml、run-demo.sh、scripts/extract.sh、scripts/notify.sh。transform.sh・load.sh は extract.sh と同形のため冒頭のみ確認)
  - `.github/workflows/`(ci.yml、release.yml、audit.yml)
  - `crates/` の各 Cargo.toml、各クレートの `lib.rs` の説明、`crates/runtime/src/runner.rs`、`crates/runtime/src/platform/windows.rs`、`crates/cli/src/platform/windows.rs`、`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`(冒頭)
- **Skimmed only**:
  - `crates/*/src/` の上記以外のモジュール(ファイル名と lib.rs の表で把握)
  - `crates/*/tests/`(ファイル名のみ)
  - `.gitignore`、`LICENSE`、`Cargo.lock`
- **Left out**:
  - `.claude/`、`aidlc/`(AI-DLC 自身の導入物)
  - `.idea/`、`target/`(IDE・ビルド出力。開かない)

### Packages Found
- conductor-cli — バイナリ+ライブラリ(`conductor`、補助 `conductor-cli-test-helper`)— Rust — 引数、流れの組み立て、シグナル、表示、終了コード
- conductor-core — ライブラリ — Rust — 定義の読み込み・検証、フローの判断(純粋)
- conductor-runtime — ライブラリ(補助バイナリ `conductor-test-helper`)— Rust — ジョブ起動・シグナル転送、ステートの読み書き
- conductor-converter — ライブラリ — Rust — YAML と TOML の変換
- demo — スクリプト群(sh)— POSIX sh — UNIX 系専用のデモ

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`、resolver 3、edition 2024、rust-version 1.85)
- **Config Files**: `Cargo.toml`、`crates/*/Cargo.toml`、`Cargo.lock`。`[profile.release]` は thin LTO・strip
- **Build Dependencies**: cli → core, runtime, converter / runtime → core / converter は core に依存しない

### APIs Discovered
- CLI(`conductor`)— `crates/cli/src/args.rs` — サブコマンド 5(run・pipeline・watch・reset・convert)と --help/--version
- Rust ライブラリ API — 各 `lib.rs` — core 5 関数、runtime 10 関数、converter 2 関数
- 定義ファイル(TOML。YAML は convert 用)— README.md の「定義ファイルの書き方」

### Frameworks & Libraries
- lexopt — 0.3.2 — 引数の字句解析
- signal-hook — 0.4.6(unix のみ)— シグナル監視
- ctrlc — 3.5.2(windows のみ)— Ctrl+C
- libc — 0.2(unix のみ)— kill
- toml — 1.2.1 — TOML の読み書き
- saphyr-parser — 0.3.4 — YAML の読み取り
- thiserror — 2.0.21 — エラー
- tempfile — 3.27.0 — テスト

### Test Coverage
- **Test Directories**: `crates/cli/tests/`、`crates/core/tests/`、`crates/runtime/tests/`、`crates/converter/tests/`
- **Test Frameworks**: 標準の `cargo test`。実バイナリ起動の統合テスト、補助バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)
- **Coverage Config**: あり(CI の cargo-llvm-cov、行 80% 未満で失敗)

### Code Quality Indicators
- **Linting**: CI で `cargo fmt --check` と `cargo clippy -D warnings`(`rustfmt.toml` などの設定ファイルはなし)
- **CI/CD**: `.github/workflows/ci.yml`(lint・build-check・test・msrv-test・coverage・docs-check・startup-bench)、`release.yml`(タグ v* で 4 ターゲットをビルドし、下書きリリースを作る)、`audit.yml`(cargo audit、警告のみ)
- **Documentation**: README.md が詳細(日本語)。各 lib.rs に表形式の設計説明。`demo/README.md` あり

### Technical Debt Signals
- `demo/` は UNIX 系専用(`demo/run-demo.sh`、`demo/conductor.toml` の `command = "sh scripts/*.sh"`)。`demo/README.md:25` が「Windows では動かない」と明記
- Windows の Ctrl+C 経路は自動テストがなく、手動チェックリストのみ(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)
- 仕様 ID(BR・NFR・W など)が doc コメントに残っており、リポジトリ外の設計書への参照になっている
- `command` はシェルを介さず直接起動する。Windows では `.cmd`/`.bat` や組み込みコマンド(`echo`、`timeout`)の扱いに注意が要る(実行は未確認)

## Handoff Summary
- **Intent-relevant finding**: 既存デモは `sh` と UNIX コマンド(`mktemp`、`sleep`、`cp -R`、ANSI の `printf`)に依存する(`demo/run-demo.sh`、`demo/scripts/*.sh`)。`conductor.toml` の `command` は `sh scripts/<name>.sh` で、`conductor` はシェルを介さず直接起動するため、Windows 版には `.ps1` か `.cmd` のジョブスクリプトと、Windows 用の `conductor.toml` が要る。バイナリは `x86_64-pc-windows-msvc` の zip で配布され(`release.yml:67`)、`cargo build --release --locked --bin conductor` で `target\release\conductor.exe` ができる。Windows の CI は windows-2022 で test・build-check・startup-bench を実行する(`ci.yml`)。
- **Risks / follow-up**: (1) `command` の文字列の分割規則と、PowerShell・cmd の起動方法(`powershell -File` か `cmd /c`)の組み合わせを実機で確かめる必要がある。(2) 終了コードは Windows でも 0〜255 に丸められる。(3) Windows では色(ANSI)は `watch` に付かない(UNIX 系のみ)。(4) README の docs-check が必須語を検査しているため、README を編集するときは `SHA-256`・`Gatekeeper`・`終了コードの表`・`ステートファイルの場所`・`at-least-once`・`シグナル`・`convert` を残す。(5) チームの規約はコメント・説明を日本語、ファイル冒頭に Apache ライセンスヘッダを付ける。
