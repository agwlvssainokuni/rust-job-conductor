# 単体テストの手順: v0.10.4 の大きなファイルの分割

## テストの仕組み

- テストは Rust の標準の `cargo test` で行う。新しい道具も依存も足さない。
- この作業では、新しいテストを足さない。動きを変えないので、既存のテストが名前も期待の値もそのままで通ることで確かめる。
- テストの中身を直すのは、`src/tests/report_messages.rs` の `with_all_events` の一覧(FR4.4)だけ。
- 分けるファイルごとに、それを覆うテストは次のとおり。
  - `walk`: core の統合テスト(`definition_*.rs`、`env_definition.rs` など)と、`crates/cli/tests/spec_numbers.rs`(キーの表を読む)
  - `args`: `crates/cli/src/tests/args_grammar.rs`(69 テスト)と cli の統合テスト
  - `report`: `crates/cli/src/tests/report_messages.rs`、`escape_table.rs`、`docs.rs`、cli の統合テスト
  - 全体: `crates/cli/tests/nfr_source.rs`(ソースの検査)

## この作業のテストの走らせ方

どのコマンドも 1 つずつ走らせ、後ろに別の手順を付けない。

`walk` の各ステップ(Step 2〜4):

```bash
cargo test -p conductor-core --locked
```

```bash
cargo test -p conductor-cli --test spec_numbers --locked
```

`args` の各ステップ(Step 5〜8)。誤りの順は `args_grammar` が確かめる:

```bash
cargo test -p conductor-cli --lib --locked -- tests::args_grammar
```

`report` の各ステップ(Step 9〜12)。694 行と 726 行の判定の後回しのテストを含む:

```bash
cargo test -p conductor-cli --lib --locked -- tests::report_messages
```

```bash
cargo test -p conductor-cli --lib --locked -- tests::escape_table
```

層の終わりと最後(Step 4、8、12、16):

```bash
cargo test -p conductor-cli --locked
```

```bash
cargo test --workspace --locked
```

```bash
cargo fmt --all -- --check
```

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

テストの数の確かめ(Step 1 と Step 17)。`#[test]` は 1,472 のまま:

```bash
grep -rcE '^\s*#\[test\]' crates --include='*.rs'
```

## 期待するカバレッジ

- 行カバレッジ 80% の下限は、プロジェクト全体にかかる。Build and Test で、全体を次のコマンドで測る。
  ```bash
  cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
  ```
- コードを移すだけなので、v0.10.3 の 98.22% から大きくは変わらない見込み。下限は下げない。

## 偽物と差し替え

- 新しいものは無い。既存の偽の `Env`・`Sink` をそのまま使う。

## テストのデータ

- 新しいものは無い。
