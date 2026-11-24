# 意図の記述(Intent Statement): converter の testing モジュールの削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 3: converter の testing モジュールの削除(動きを変えない。版は上げない)。converter の lib.rs の #[doc(hidden)] pub mod testing(Value の再公開と read_tree)はテストからしか使われない。それだけが使う TargetFormat::opposite と、本番で使われない Value の手書きの PartialEq も合わせて扱う。テストの側(tests/common/mod.rs の yaml()・toml()、read_resolve.rs、convert_roundtrip.rs、convert_ac.rs、nfr_security.rs、types.rs の単体テスト)は、木を比べるテストを src の単体テストへ移すか、公開の convert_text の結果で確かめる形に直す。判断が要るもの: テストの行き先、Value の PartialEq の扱い。仕様書 rust の testing の記述を直す。範囲の外: cli のライブラリの公開範囲の大きい部分"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。確かめる中身は変えない。移す・変える・消すテストは検索で網羅し、消すものは理由を添えて設計で承認を取る)
- [Q2] `intent-capture-questions.md` Q2: A(読み取りの途中の表現を比べるテストは、src の単体テストへ移す)
- [Q3] `intent-capture-questions.md` Q3: A(`Value` の `PartialEq` は `#[cfg(test)]` でテストのときだけコンパイルする)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- converter は、テストのためだけに `#[doc(hidden)] pub mod testing`(途中の表現 `Value` の再公開と `read_tree`)を公開している。[desc]
- それだけが使う `TargetFormat::opposite` と、本番で使われない `Value` の `PartialEq` も本番のコードに含まれる。[desc]
- 解決したいのは、利用者から見える動きを変えずに、これらを本番の公開範囲と成果物から外すことである。[desc] [Q3]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- converter の `testing` モジュールと `TargetFormat::opposite` が本番のコードから消え、参照が 0 件になる。[desc]
- 途中の表現(`Value` の木)を比べるテストは src の単体テストへ移り、期待の木と比べる今の確かめ方と期待の値をそのまま残す。[Q2]
- `Value` の `PartialEq` は `#[cfg(test)]` の中にだけあり、本番の成果物には入らない。[Q3]
- 仕様書 rust の `testing` と `opposite` の記述が実装と合う。確かめる手段: 直した後に `docs/spec/rust` で `testing::` と `opposite` の参照が 0 件になることを検索で確かめる。[desc]
- 確かめる手段: 設計の段で、移す・変えるテストの一覧(元の場所、移す先。`tests/common/mod.rs` の `yaml()`・`toml()` の補助と `types.rs` の単体テストの扱いを含む)と、終わったと判断する基準(「`testing::` の参照が 0 件」「`impl PartialEq for Value` が `#[cfg(test)]` の中だけにある」)を書き、実装の後に検索とテスト名の比較で確かめる。移すテストの名前を変える必要があるかどうかも設計の段で一覧にし、変えるものは承認を取る。[Q1] [Q3]
- 利用者から見える動き(変換の出力、誤りの表示、終了コード)は変わらない。版は上げない。[desc]
- 既存のテストの扱い: 移す・書き方を変える・消すテストは認める。確かめる中身と期待の値は変えない。一覧は検索で網羅し、消すテストは理由を添えて、設計の段で承認を取る。[Q1]
  - 確かめる手段: 実装の前と後でテストの件数を記録する。合格は、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。[Q1]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 6 回目のリファクタリングの調査(プロダクトコードの不要なコードの洗い出し)で挙がった候補を 4 つの intent に分けた、その 3 つ目。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: converter の `testing` モジュール、`TargetFormat::opposite`、`Value` の `PartialEq`、それらを使うテスト、仕様書 rust の該当箇所。[desc] [Q2] [Q3]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: cli のライブラリの公開範囲の大きい部分(別の intent)。[desc]

## Assumptions & Open Questions

None.
