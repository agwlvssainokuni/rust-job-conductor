# 単体テストの手順: u1-pipeline-retry

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しいテストの道具や依存は足さない。
- core の統合テストは `crates/core/tests/`、cli の流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime` を注入)、実バイナリのテストは `crates/cli/tests/binary_*.rs` と `unix_demo.rs`、cli の表示のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

```bash
cargo test -p conductor-core --test definition_retry --test definition_valid --test nfr_security --test definition_perf --locked
```

```bash
cargo test -p conductor-cli --test flow_retry --test flow_validate --test binary_basic --test unix_demo --locked
```

```bash
cargo test -p conductor-cli --lib --locked -- tests::graph_report tests::plan_report tests::docs
```

- `windows_demo.rs` は Windows でだけ動くため、CI の windows の `test` で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。Build and Test が、組み上がったプロジェクトのすべてのテストで測る(CI の `coverage` は `cargo llvm-cov --fail-under-lines 80`)。上のユニットのコマンドでは判定しない。下限は下げない。
- このユニットの新しいコード(`walk/retry.rs` と `walk/mod.rs` の追加分)は、規則ごとに正しい場合と誤りの場合のテストで通る。

## 偽物と差し替え

- core のテストは差し替え無し(core は純粋)。定義の文字列を `parse` して確かめる。
- cli の流れのテストは、今の `Harness` と偽の `Runtime`(終了コードを順に返す)を使う。本番のコードにテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- 定義は各テストの中の文字列で作る(今の `job_with`、`retry_def`、`with_retry` などの補助関数に倣う)。
- 性能のテストは、テストの中で 1 万のジョブの定義を組み立てる(ファイルを置かない)。
- 秘密のテストは、今の目印の文字列(`SENTINEL`)を、値とジョブの ID の位置に入れる。
