# テストの結果: v0.10.3

出典: `construction/code-generation/code-summary.md`、`unit-test-instructions.md`、`build-instructions.md`。

## ビルドと品質の検査(手元の macOS、コミット 7b7db42 の時点)

| 検査 | 結果 |
|---|---|
| `cargo build --locked` | 成功。`conductor --version` は `conductor 0.10.3` |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `Cargo.lock` の package | 46(v0.10.2 と同じ) |

## 単体のコマンド(`unit-test-instructions.md`)

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-cli --test spec_numbers --locked` | 12 passed、0 failed |
| `cargo test -p conductor-core --test env_types --locked` | 通った(コード生成の記録。テスト名 1 つの直し) |
| `cargo test --workspace --locked` | 1,449 passed、0 failed、1 ignored |

- `#[test]` の数は 1,472 で、v0.10.2 の 1,460 から 12 増えた。
- 走ったテストの数(1,449 + 無視 1)との差は 22 で、v0.10.2 と同じ。macOS では組み立てない `cfg(windows)` のテストの分。

## 全体のカバレッジ

- 1 回目: `cargo llvm-cov … --fail-under-lines 80` が `runtime` の `runner_log` で 1 つ落ちて止まった。
  - 落ちたテストの名前は、出力を絞って走らせたために残っていない。
  - `runner_log` だけを `cargo llvm-cov` で 7 回走らせ直した。7 回とも 8 passed で、再現しなかった。
  - `runner_log` には 10 秒・2 秒などの時間の上限を持つテストがあり、計測の重い負荷で時間を越えたと見ている。
  - この intent は `crates/runtime/` を変えていない(差分は版の `Cargo.toml` だけ)。
- 2 回目: 全体が通った。行カバレッジは **98.22%**(10,823 行のうち 193 行が未通過。下限は 80%)。

## 効くことの確かめ(コード生成の Step 7)

`code-summary.md` のとおり、次の 3 つを一時的に行い、毎回 `spec_numbers` が落ちた。どれも書き戻してある。

- 場面の数をずらす
- 注記の無い「34 種」をずらす
- 決まった見出しを変える

## 承認の後の直し(コード生成のレビューの R-01・R-02)

- d646030: R12 の段 10 の「13 種」に、版の注記を付けた。
- 7b7db42: AC-214 のテスト名の `_` を空白にした。
- 直した後に `spec_numbers` を走らせ、12 passed だった。

## CI

- まだ走っていない(push の前)。結果はここに書き足す。

## Target Verification Matrix の最終の判定

- CI の結果を待つ。
