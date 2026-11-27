# テストの結果: v0.10.2

出典: この段階で走らせたコマンドの出力、CI の実行 46079873817、`unit-test-instructions.md`、`build-instructions.md`、`code-generation-plan.md` の付録 B。

## ビルドと品質の検査(手元の macOS、コミット 01a6d15 の時点)

| コマンド | 結果 |
|---|---|
| `cargo build --workspace --locked` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo doc -p conductor-runtime --no-deps --locked` | exit 0。警告 4 件(作業の前と同じ数) |
| `target/debug/conductor --version` | `conductor 0.10.2` |
| 依存 | `Cargo.lock` の package 46(変わらず) |

## 単体のコマンド(`unit-test-instructions.md`。各 1 回)

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-runtime --lib --locked` | exit 0。88 passed |
| `cargo doc -p conductor-runtime --no-deps --locked` | exit 0 |
| `cargo test -p conductor-cli --test binary_basic --locked` | exit 0。26 passed |
| `cargo test -p conductor-cli --lib --locked -- tests::docs` | exit 0。22 passed |

## 全体のテストとカバレッジ

- `cargo test --workspace --locked`: exit 0。1,437 passed、0 failed、1 ignored(前からのもの)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: exit 0。行カバレッジ 98.22%(10,823 行のうち 193 行が未通過)。v0.10.1 と同じ(コードの動きを変えていないため)。

## 仕様書の確かめ(付録 B)

- `grep -rn '33 種\|13 種' docs/spec/rust/ docs/spec/language-agnostic/`: rust の `12-rebuild-guide.md:105`・`107`、language-agnostic の `12-rebuild-guide.md:129` の 3 行だけ(どれも v0.9.0 の段の履歴として残す行)。
- `grep -rn '足すだけ\|取り除かない\|取り除く書き方\|読まない・保存しない\|1 回だけ書\|0〜14' docs/spec/rust/ docs/spec/language-agnostic/`: rust の `05:170`・`05:277`・`05:713`・`03:394`、language-agnostic の `05:729`・`03:561`・`11:924`(AC-452)だけ(どれも付録 A の「残してよい記述」)。
- 数・語の出現・テストの数の確かめは、コード生成の記録(`code-summary.md`)のとおりで、この段階のレビューで抜き取りでも一致を確かめた。

## CI 46079873817(コミット 01a6d15)

- 12 のジョブがすべて成功: `lint`、`build-check`(ubuntu、windows)、`test`(ubuntu-24.04、macos-latest、windows-2022)、`msrv-test`、`coverage`、`docs-check`、`startup-bench` ×3。

## Target Verification Matrix の最終の判定

`build-and-test-summary.md` の表を参照。すべて Met。
