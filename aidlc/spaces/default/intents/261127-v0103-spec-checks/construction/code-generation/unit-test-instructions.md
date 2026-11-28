# 単体テストの手順: v0.10.3 の仕様書の粗の直しと数の検査

## テストの仕組み

- テストは Rust の標準の `cargo test` で行う。新しい道具も依存も足さない。
- 足すテストは `crates/cli/tests/spec_numbers.rs` の 1 ファイル(統合テスト)。
  - 仕様書(`docs/spec/rust/`、`docs/spec/language-agnostic/`)、ソース(`crates/*/src/`)、デモ(`demo/`)を文字列として読み、数を突き合わせる。
  - 数ごとにテスト関数を 1 つ置き、12 個とする。
  - 前例は `crates/runtime/tests/nfr_source.rs`(ソースを読む)と `crates/cli/tests/msrv_consistency.rs`(リポジトリのファイルを読む)。
- 本番のコードは変えない。テストのために `pub` や関数を足さない。

## この作業のテストの走らせ方

どのコマンドも 1 つずつ走らせ、後ろに別の手順を付けない。

数の検査(Step 6、Step 7):

```bash
cargo test -p conductor-cli --test spec_numbers --locked
```

テスト名の直し(Step 3):

```bash
cargo test -p conductor-core --test env_types --locked
```

全体と形の確かめ(Step 1、Step 6、Step 10):

```bash
cargo test --workspace --locked
```

```bash
cargo fmt --all -- --check
```

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## 期待するカバレッジ

- 行カバレッジ 80% の下限は、プロジェクト全体にかかる。測るのは Build and Test で、全体を次のコマンドで測る。

  ```bash
  cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
  ```

- 本番のコードは変えないので、v0.10.2 の 98.22% から変わらない見込み。下限は下げない。

## 効くことの確かめ(Step 7)

- 確かめる方法: 仕様書を一時的にずらし、`spec_numbers` が落ちることを見る。ずらすのは次の 3 つで、1 つずつ行う。
  - 場面の数
  - 注記の無い「34 種」
  - 決まった見出し
- 戻し方: ずらした文はファイルの道具で元の文に書き戻す。`git checkout`・`git restore`・`git stash` は使わない。
- 書き戻した後に `git diff --stat docs/` が空であることを確かめる。

## 偽物と差し替え

- 無い。読むファイルはリポジトリの実物。

## テストのデータ

- 無い。リポジトリの今の仕様書とソースを使う。
