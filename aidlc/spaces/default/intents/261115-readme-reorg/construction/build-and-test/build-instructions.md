# Build Instructions

この作業は、文書(`README.md` と `demo/README.md`)だけの変更で、コードは変わらない。ビルドと既存の検査は、これまでどおり。

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85。カバレッジの測定には `cargo-llvm-cov`(開発機のみ)。

## ビルドと確認

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo +1.85 build --workspace --locked --all-targets
```

## README の文書チェック

```bash
# CI の docs-check と同じ(.github/workflows/ci.yml)
for pat in 'SHA-256' 'Gatekeeper' '終了コードの表' 'ステートファイルの場所' 'at-least-once' 'シグナル' 'convert'; do
  grep -q -- "$pat" README.md || echo "MISSING $pat"
done
```

何も出力しないこと。見出しの語(`SHA-256`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`Gatekeeper` を含む見出し)は、変えない。

## README に書いた手順の確認

- クイックスタート: `target/` の下の一時ディレクトリで、README の定義を作り、リリースビルドの `conductor` で、`LANG=C` で、手順を順に実行する。出力と終了コードが、README と一致すること。
- ソースからのビルド: `cargo build --release --locked --bin conductor`、`cargo test --workspace --locked`、`cargo install --path crates/cli --locked --bin conductor --root <一時ディレクトリ>` が成功すること(`--bin conductor` を付けないと、テスト専用の補助の実行ファイルまで入る)。

## よくある問題

| 症状 | 原因と対処 |
|------|-----------|
| `docs-check` が失敗する | README から必須の語句が消えた。上の検索で、どれが無いかを確かめる |
| 目次のリンクが開かない | 見出しの名前を変えたのに、目次のリンク(アンカー)を直していない |
