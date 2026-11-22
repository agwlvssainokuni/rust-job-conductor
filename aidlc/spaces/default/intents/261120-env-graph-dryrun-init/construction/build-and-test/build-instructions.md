# ビルドの手順(Build Instructions): v0.9.0 の機能拡張

対象: `conductor` のワークスペース(`conductor-core`、`conductor-runtime`、`conductor-converter`、`conductor-cli`)。4 つのユニット(U1 環境変数と `.env`、U2 `graph`、U3 `--dry-run`、U4 `init`)の成果を、まとめてビルドして、確かめる。

## 前提(依存のインストール)

- Rust のツールチェーン: 1.100 以上(MSRV 1.100。`Cargo.toml` の `rust-version = "1.100"`、edition 2024)。このマシンは `rustc 1.100.0`。
- カバレッジの計測: `cargo-llvm-cov`(`rustup component add llvm-tools-preview` も必要)。CI の `coverage` ジョブと同じ。
- 新しい外部ライブラリは足していない(`Cargo.toml` と `Cargo.lock` に、この作業の差分は無い)。`cargo` は、`Cargo.lock` に固定された依存を使う(`--locked`)。

## 環境の設定

- 環境変数や設定ファイルは、ビルドに要らない。
- テストは、一時ディレクトリを使う(終わると消える)。テスト専用の補助の実行ファイル(`conductor-cli-test-helper` など)は、`cargo test` が自動でビルドする(配布しない)。

## ビルドのコマンド

```bash
cargo build --workspace --locked
```

リリース用のビルド(配布の対象は `conductor` だけ): `cargo build --release --locked -p conductor-cli --bin conductor`(リリースの作成は、この作業の範囲の外)。

## ビルドの確認

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --locked --workspace
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

(最後の 1 つは、プロジェクト全体の行カバレッジ 80% の下限を確かめる。CI の `coverage` と同じ。)

README の必須語句(CI の `docs-check`): `SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert` が、`README.md` に残っていること(`grep -q -- '<語句>' README.md`)。

## このマシンで確かめられないもの

- Windows と ubuntu でのビルドとテスト(CI の `test` の行列で確かめる)。`#![cfg(windows)]` の `crates/cli/tests/windows_demo.rs`(場面 11〜14 のテストを含む)と、`demo/windows/run-demo.cmd`。
- 最小の Rust(1.100.0)での `msrv-test`: このマシンのツールチェーンは、ちょうど 1.100.0 で、MSRV と同じなので、ここでのビルドとテストが、MSRV での確認を兼ねる。

## よくある問題と対処

- 計測つき(`cargo llvm-cov`)の実行では、時間のかかるテスト(`unix_demo` の一部、`conductor-converter` の `expanded_size_budget_boundary_with_the_default_16_mib`)が、30 秒の上限を超えて失敗することがある。マシンが忙しいとき(ほかのビルドが動いているとき)に、起きやすい。単独で再実行すると通る。繰り返すときは、他の重い処理を止めてから、再実行する。
- `--locked` で `Cargo.lock` の更新を求められたら、依存を足していないか確かめる(この作業は、依存を足さない)。
- clippy の `large_enum_variant` が出たら、大きな型を持つ欄を、別の取り出しの関数に切り出す(U4 で、`init` のオプションを、そのようにした)。
