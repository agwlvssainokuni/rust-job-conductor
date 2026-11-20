# 発見した規則

> 人が確認した強い制約を記す。既存の制約はインタビュー Q7 で、新しい依存の禁止は Q4 で、人が述べた。

## Mandated

- ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。
- ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。
- ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。
- ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。
- ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。

## Forbidden

- NEVER C のコンパイルを要する依存や `-sys` クレートを実行時の依存に加えない(CI の `build-check`)。
- NEVER `unsafe` を runtime の `platform/unix.rs` の外に書かない。cli では `unsafe` を禁止する(`nfr_source.rs`)。
- NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない(`nfr_source.rs`)。
- NEVER エラーの表示に入力の値を含めない。ステートに環境変数を書かない(`code-quality-assessment.md` の既存の性質)。
- NEVER テスト専用の補助バイナリをリリースの成果物に含めない(`release.yml`)。
- NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。
