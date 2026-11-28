# ビルドの手順: v0.10.4

出典: `construction/code-generation/unit-test-instructions.md`、`code-summary.md`、team.md、project.md。

## 前提

- Rust は stable を使う。最小の版(MSRV)は 1.100 で、`msrv-test` の CI で確かめる。
- `cargo-llvm-cov` は、カバレッジを測るときだけ使う。
- 新しい依存は無い。`Cargo.lock` の package は 46 で、v0.10.3 と同じ。

## ビルドと確かめ

コマンドは 1 つずつ走らせる。

```bash
cargo build --locked
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

```bash
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

## 分けた後のファイルの形

| 親 | 子 |
|---|---|
| `crates/core/src/definition/walk/mod.rs` | `values.rs`、`transition.rs`、`cross_check.rs`(と、前からある `env.rs`、`retry.rs`) |
| `crates/cli/src/args/mod.rs` | `options.rs`、`tokens.rs`、`values.rs` |
| `crates/cli/src/report/mod.rs` | `event.rs`、`escape.rs`、`lines.rs`、`render.rs`(と、前からある子モジュール) |

- 表示を足すときは、次の 3 か所を直す。
  - `event.rs` に変種を足す。
  - `render.rs` の節にメソッドを足す。
  - `mod.rs` の `render` の `match` に枝を足す。`match` は網羅なので、枝を書き忘れるとコンパイルが止まる。

## よくある失敗

- カバレッジの計測は、テストを重い負荷で走らせる。そのため、`runtime` の `runner_log` の `br4_1_標準出力と標準エラーの両方がファイルに残り_各流れの中の順を保つ` が、まれに落ちることがある。
  - 落ちる場所は 77 行の `LogOutcome::default()` との比べ。
  - ジョブの終わった後、出力を写す処理を待つ上限(`runner.rs` の `DRAIN_LIMIT` 2 秒)を、負荷のために超えると、`output_cut_short` が付いて落ちる。
- 走らせ直して通れば、計測の負荷によるものとして扱う。続けて落ちるときは調べる。
