# Build Instructions

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85(`rust-version = "1.85"`、edition 2024)。`rustup` で入れる。
- カバレッジの測定: `cargo install cargo-llvm-cov --locked` と `rustup component add llvm-tools-preview`(開発機のみ。プロジェクトのファイルは変えない)。
- 外部のサービス、環境変数、設定ファイルは要らない。依存は最初の取得だけ公開のレジストリを使う。C のコンパイラ(`cc`)も要らない(`-sys` クレートを含まない)。

## 依存の取得とビルド

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor
```

- 配布するのは `conductor` だけ。テスト専用の `conductor-cli-test-helper` と `conductor-test-helper` は、`cargo build -p ...` で作られるが、配布物には入れない。
- ビルドの確認: `target/release/conductor --version` が版を表示する(約 971 KB)。

## 最小の Rust のバージョン(MSRV)の確認

```bash
rustup toolchain install 1.85 --profile minimal
cargo +1.85 build --workspace --locked --all-targets
```

- edition 2024 でも、let チェーン(`if let ... && ...`)は 1.88 で安定化されたため、1.85 では使えない。`is_some_and` などで書く。

## 整形と静的検査

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## 依存の確認(C のビルドを要する依存がないこと)

```bash
cargo tree -p conductor-cli -e normal,build | grep -E '(^| )(cc|cmake|pkg-config|bindgen) '
cargo tree -p conductor-cli -e normal | grep -E -- '-sys'
```

どちらも何も出力しないこと。

## よくあるビルドの問題

| 症状 | 原因と対処 |
|------|-----------|
| `error[E0658]: let expressions in this position are unstable` | 1.85 で let チェーンを使っている。`is_some_and` や入れ子の `if let` に書き換える |
| `cargo llvm-cov` が見つからない | 上の前提のとおり導入する |
| 実行ファイルのテストが 30 秒で失敗する | ハング。子プロセスの回収とシグナルの経路を確認する |
