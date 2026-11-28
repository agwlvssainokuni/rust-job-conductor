# ビルドの手順: v0.10.3

出典: `construction/code-generation/unit-test-instructions.md`、`code-summary.md`、team.md、project.md。

## 前提

- Rust の stable を使う。最小の版(MSRV)は 1.100 で、`msrv-test` の CI で確かめる。
- `cargo-llvm-cov` はカバレッジを測るときだけ要る。
- 新しい依存は無い。`Cargo.lock` の package は 46 で、v0.10.2 と同じ。

## ビルドと確かめ

コマンドは 1 つずつ走らせる。

```bash
cargo build --locked
```

```bash
cargo test --workspace --locked
```

```bash
cargo test -p conductor-cli --test spec_numbers --locked
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

## 仕様書の数の検査

- `spec_numbers` は、次のファイルを文字列として読み、12 の数を突き合わせる。
  - `docs/spec/rust/`
  - `docs/spec/language-agnostic/`
  - `crates/*/src/`
  - `demo/`
- 落ちたときは、失敗の文に次のものが出る。
  - 数の名前
  - ファイルと行番号
  - 仕様書の値と実装の値
- 直す側は 2 通りある。
  - 実装を変えた場合: 仕様書を直す。
  - 履歴の数を書いた場合: 同じ行に版の注記(`v0.N.N まで`、`v0.N.N 以前`、`の時点`)を付ける。

## よくある失敗

- カバレッジの計測は、テストを重い負荷で走らせる。そのため、時間に敏感なテスト(`runtime` の `runner_log` など)が、まれに落ちることがある。走らせ直して通れば、計測の負荷によるものとして扱う。続けて落ちる場合は調べる。
