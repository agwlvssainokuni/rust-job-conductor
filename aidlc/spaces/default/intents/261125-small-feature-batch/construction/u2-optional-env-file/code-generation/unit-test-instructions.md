# 単体テストの手順: u2-optional-env-file

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。
- core の統合テストは `crates/core/tests/`、runtime は `crates/runtime/tests/`、cli の流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime`)、実バイナリのテストは `crates/cli/tests/binary_*.rs` と `unix_demo.rs`、cli の内部のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

```bash
cargo test -p conductor-core --test env_definition --test env_merge --test env_types --test nfr_security --test env_refs_perf --locked
```

```bash
cargo test -p conductor-runtime --test env_loader --test flow_with_state --locked
```

```bash
cargo test -p conductor-cli --test flow_env --test flow_dry_run --test binary_env --test unix_demo --locked
```

- 型の変更は多くのテストの組み立てに及ぶため、最後に `cargo test --workspace --locked` で全体が通ることも確かめる(Build and Test でも走る)。
- `windows_demo.rs` は Windows でだけ動くため、CI の windows の `test` で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。Build and Test と、ユニットの確認のコマンド(`cargo llvm-cov --workspace ... --fail-under-lines 80`)が全体で測る。上のユニットのコマンドでは判定しない。下限は下げない。
- このユニットの新しい分岐(表の読み取り、required の論理和、無いときの読み飛ばし)は、規則ごとに正しい場合と誤りの場合のテストで通る。

## 偽物と差し替え

- core のテストは差し替え無し(core は純粋)。定義の文字列を `parse` して確かめる。
- runtime のテストは、`tempfile` の一時ディレクトリに実際のファイル(またはディレクトリ)を置いて確かめる。
- cli の流れのテストは、今の偽の `Runtime` を使い、引数の型だけ合わせる。本番にテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- 定義と `.env` は各テストの中の文字列で作る。無いファイルは、一時ディレクトリの中の置いていない名前で表す。
- 秘密のテストは、今の目印の文字列(`SENTINEL`)を値の位置に入れる。
