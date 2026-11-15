# Test Results

実行日: 2026-11-16。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## 文書の確認

| 確認 | 命令・方法 | 結果 |
|------|-----------|------|
| docs-check の 7 語句 | CI の `docs-check` と同じ `grep` の繰り返し | すべて見つかる(`MISSING` なし)。整理前も通っていた |
| 目次とリンク | README の `](#...)` を、見出しから作った GitHub 形式の slug と突き合わせるスクリプト | 17 個、切れ 0 個。`demo/README.md` と `LICENSE` へのリンクは実在 |
| run ID の規則の集約 | 値の規則(`A-Za-z0-9._-`)、`1〜64`、優先順位(`環境変数は見ません`)、`--run-id ""` の固定、`reset --all` の「中身は読まずに」を検索 | いずれも README に 1 回だけ |
| 整理前後の事実の突き合わせ | `git show` の整理前の README と比べ、バッククォートの語句 110 個、終了コードの行(0・ジョブの値・1・64・65・66・67・68・70・127・128+番号)、成功・警告の規則、at-least-once の箇条書き 5 つ、シグナルの動作、convert の制限、ロケールの規則を確認 | すべて残っている。文単位で一致しなかった 24 件は、意図した書き換え(表の短縮で小節へ移した、FR4 の修正) |
| 変更の範囲 | `git diff --stat 5bdcbbf HEAD -- . ':!aidlc'` | `README.md`(203 行の増減)と `demo/README.md`(15 行の増減)だけ |

## README に書いた手順の実行

| 手順 | 結果 |
|------|------|
| クイックスタート(`LANG=C`、リリースビルド) | `validate` が 0、`FAIL=3 pipeline -v hello` が 3(`step2` で止まる)、`watch` が `step2 error`、再実行が 0(`step1` はスキップ、`step2` から再開)、`watch` が `no state`。**README に載せた出力と、独立にもう一度実行した出力が、一致** |
| ソースからのビルド | `cargo build --release --locked --bin conductor` 成功、`cargo test --workspace --locked` 成功、`cargo install --path crates/cli --locked --bin conductor --root <一時ディレクトリ>` 成功(`conductor --version` は `conductor 0.6.0`)。`git ls-remote` でクローンの URL は実在 |
| リリースのアーカイブの展開(v0.6.0、aarch64-apple-darwin) | 最上位に `conductor-0.6.0-aarch64-apple-darwin/`、その中に `conductor` と `LICENSE`。`SHA256SUMS` は `OK`。README の手順どおり |

## 既存の検査(コードは変えていない)

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo test --workspace --locked` | 0 | 666 件成功、失敗 0、無視 1(前回と同じ) |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |

## 実行できていないもの

- Windows のクイックスタート(`cmd /c` の読み替え)の実行。Windows の zip の構造(`release.yml` から読んだだけ)の実物の確認。Linux・Intel macOS のアーカイブの実物の確認。
- `shasum` と Gatekeeper の手順(OS の動作で、コードの外)。
