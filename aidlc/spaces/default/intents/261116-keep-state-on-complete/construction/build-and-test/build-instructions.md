# Build Instructions

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85。カバレッジの測定には `cargo-llvm-cov`(開発機のみ)。
- 新しい依存や環境変数、設定ファイルは足していない。

## ビルドと確認

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo +1.85 build --workspace --locked --all-targets
```

版は 0.7.0(4 クレートと `Cargo.lock`)。`conductor --version` が `conductor 0.7.0` を出す。

## カバレッジ

```bash
cargo llvm-cov --workspace --locked --summary-only
```

下限は行カバレッジ 80%(team.md)。CI の `coverage` ジョブも同じ。

## README の文書チェック(CI の docs-check と同じ)

```bash
for pat in 'SHA-256' 'Gatekeeper' '終了コードの表' 'ステートファイルの場所' 'at-least-once' 'シグナル' 'convert'; do
  grep -q -- "$pat" README.md || echo "MISSING $pat"
done
```

何も出力しないこと。

## 動作の確認(リリースビルドの実機)

`target/` の下の一時ディレクトリに、`format_version = 1` を先頭に書いた定義を置き、`conductor pipeline <名前>`、`watch`、`--run-id`、`--remove-on-complete`、`[pipeline].remove_on_complete`、`reset --all` を実行する(`test-results.md` に実行結果)。

## よくある問題

| 症状 | 原因と対処 |
|------|-----------|
| 定義ファイルで 64(`format_version is missing`) | 定義の先頭に `format_version = 1` がない |
| `pipeline` で 68(`missing argument`) | `pipeline` にはパイプライン名の引数が要る(`conductor pipeline <名前>`) |
| 完了後のステートが残って困る | `--remove-on-complete`、または `[pipeline].remove_on_complete = true` で、従来どおり消える。残ったものは `conductor reset`(`--all`)で消す |
