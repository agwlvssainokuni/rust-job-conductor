# 単体テストの手順: v0.10.2 の仕様書の整理

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。この作業は文書が主で、新しいテストは足さない(コードの動きを変えないため)。
- コードに触れるのは、`crates/runtime/src/lib.rs` のドキュメントコメント(Step 27)と、版の更新(Step 30。`crates/cli/tests/binary_basic.rs` の `--version` の期待を含む)だけ。
- 仕様書の確かめは、計画の付録 B の `grep` で行う(テストではない)。

## この作業のテストの走らせ方

どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

runtime のドキュメントコメント(Step 27):

```bash
cargo test -p conductor-runtime --lib --locked
```

```bash
cargo doc -p conductor-runtime --no-deps --locked
```

版の期待と文書のテスト(Step 30):

```bash
cargo test -p conductor-cli --test binary_basic --locked
```

```bash
cargo test -p conductor-cli --lib --locked -- tests::docs
```

- 版の更新は広い範囲に及ぶため、Step 29・30 で `cargo test --workspace --locked` で全体が通ることも確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。Build and Test が全体で測る(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`)。コードを変えないので、v0.10.1 の 98.22% から変わらない見込み。下限は下げない。

## 偽物と差し替え

- 無い(新しいテストを足さない)。

## テストのデータ

- 無い。
