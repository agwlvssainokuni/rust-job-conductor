# Unit Test Instructions(state-job-key)

## 実行の環境

- 言語とツール: Rust(edition 2024、最小 1.85)、`cargo`。外部のサービスやネットワークは要らない。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は各クレートの `tests/`)。
- 設定ファイル: 変更なし。

## この作業のテストの実行

このスコープは、Unit に分けない(zero-Unit)。変更は `conductor-runtime` が中心のため、層ごとの確認は、そのクレートだけを動かし、最後に全体を 1 回動かす。

```bash
cargo test -p conductor-runtime --locked      # 層ごとの確認(Step 2〜4)
cargo test --workspace --locked               # 全体の確認(Step 7)
```

## 整形と静的検査・最小の Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets
```

## 期待する網羅(Minimal の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| runtime: ステートの読み書き | 約 4 | 書き出しに `[[job]]` があり `jobs` がない(書いて読み戻すと等しい)、`[[job]]` と `format_version = 1` が読める、`[[jobs]]` は `corrupt` で拒否、エラー文面に「jobs」が残らない |

合計: 約 4 件。ステートの TOML を持つ既存のテストは、キー名を直して成功させる。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。全体の下限は、Build and Test が、全テストで判定する(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80`)。この作業のテストだけで 80% を下回っても、テストの失敗としては扱わず、結果に記録する。この測定は、ほかの命令とは別の命令として実行する。

## モックとスタブ

- ステートの読み書きは、一時ディレクトリ(`tempfile`)の実際のファイルで確かめる(モックにしない)。

## テストデータ

- ステートの例は、テストの中の文字列(`[[job]]` の形)。
