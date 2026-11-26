# ビルドの手順: v0.10.0(261125-small-feature-batch)

出典: 各ユニットのコードのまとめ(`construction/u1-pipeline-retry` 〜 `u7-watch-follow` の `code-generation/code-summary.md`)、team.md の Code Style・Testing Posture、project.md の Mandated・Forbidden。

## 前提

- Rust の toolchain(stable と、最小の版 1.100.0)。`rustfmt`、`clippy`、`llvm-tools-preview`(カバレッジ)。
- `cargo-llvm-cov`(カバレッジを測るときだけ)。
- 新しい依存はこの版で足していない。`Cargo.lock` を追跡し、すべてのコマンドに `--locked` を付ける。
- ネットワークのライブラリ、C のコンパイルを要する依存、`-sys` クレート(`linux-raw-sys`・`windows-sys` を除く)は無い(CI の `build-check`)。

## 環境

- 環境変数や設定のファイル、ローカルのサービスは要らない。
- テストは一時ディレクトリを使い、テストのプロセスの環境を変えない。

## ビルド

```bash
cargo build --workspace --locked
```

```bash
cargo build --release --locked -p conductor-cli
```

- 配布するのは `conductor` のバイナリだけ。テスト専用の補助バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)は配布物に含めない(`release.yml`)。

## ビルドの確かめ

```bash
cargo fmt --all -- --check
```

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

```bash
./target/release/conductor --version
```

- `--version` が `0.10.0` を出すこと。
- 最小の版でのビルドとテストは CI の `build-check` と `msrv-test` が確かめる。

## うまくいかないとき

- `--locked` で失敗する: `Cargo.lock` を手で変えていないか確かめる。依存を足していないので、`cargo update` はしない。
- `clippy` の警告: 抑止(`#[allow]`)を足さずにコードを直す(抑止は `platform/unix.rs` の `unsafe_code` だけ)。
- Windows の CRLF: `demo/windows/run-demo.cmd` は CRLF のまま保つ(`.gitattributes`)。
