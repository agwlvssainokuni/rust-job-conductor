# テスト結果(Test Results): graph の text 表示の改善

実行した日: 2026-11-22。環境: macOS(Darwin)、`rustc 1.100.0`。リポジトリの状態: `main` の、コミット `826cddf`。

## ビルドと静的な確認

- `cargo build --workspace --locked`: 成功。
- `cargo fmt --all -- --check`: 差分なし。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。

## テスト(プロジェクト全体)

- `cargo test --locked --workspace`: **1,038 件が成功、失敗 0 件**(前の 1,017 件から 21 件増)。
- 構造の規則の検査: `nfr_source`(cli 6 件)と `nfr_security`(core 5 件)が成功。
- 範囲を絞ったコマンド(`unit-test-instructions.md` の 5 つ)は、コード生成の段階で成功済み(開発担当の報告と、全体の実行で確認)。

## カバレッジ(プロジェクト全体、1 回の全体の実行)

- コマンド: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`。
- 行カバレッジ **97.94%**(9,328 行のうち未通過 192 行)。下限 80% を超えている。
- 変更したファイル: `core/src/graph/layout.rs` 98.56%、`cli/src/report/messages.rs` 99.72%。

## 依存と README

- `Cargo.toml`、`Cargo.lock`、`crates/*/Cargo.toml` の、版を 0.9.0 にしたコミット(`9949237`)以降の差分: なし(新しい依存なし)。
- README の必須語句 7 つ: すべて残っている。

## 失敗の詳細

なし。

## 実行しなかったもの(理由つき)

- Windows と ubuntu のテスト、最小の Rust(1.100.0)での `msrv-test`(rustc が同じ版なので、ここでの結果が兼ねる)、起動の性能(`startup-bench`)、`cargo audit`: CI の仕事。この作業の変更は、OS に依存するコードを足していない。CI Pipeline の段階は、既存の `ci.yml` で足りるため省略した。push のあとの CI の結果で確かめる。
