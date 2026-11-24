# 意図の把握の質問(Intent Capture Questions): converter の testing モジュールの削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 3: converter の testing モジュールの削除(動きを変えない。版は上げない)。converter の lib.rs の #[doc(hidden)] pub mod testing(Value の再公開と read_tree)はテストからしか使われない。それだけが使う TargetFormat::opposite と、本番で使われない Value の手書きの PartialEq も合わせて扱う。テストの側(tests/common/mod.rs の yaml()・toml()、read_resolve.rs、convert_roundtrip.rs、convert_ac.rs、nfr_security.rs、types.rs の単体テスト)は、木を比べるテストを src の単体テストへ移すか、公開の convert_text の結果で確かめる形に直す。判断が要るもの: テストの行き先、Value の PartialEq の扱い。仕様書 rust の testing の記述を直す。範囲の外: cli のライブラリの公開範囲の大きい部分"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方と既存のテストの扱い

背景: 前の 2 つの intent と同じ形で進められる見込みです。テストの置き場所や書き方は変わりますが、確かめる中身(どの入力がどう読まれ、どの誤りになるか)は変えません。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。確かめる中身は変えない。移す・変える・消すテストは検索で網羅した一覧にし、消すものは理由を添えて設計の段で承認を取る
- B. 段ごとに止まり、結果を見せて、続けるかを確かめる
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 読み取りの途中の表現(木)を比べるテストの行き先

背景: converter の統合テストのうち約 30 件(`read_resolve.rs` の大半、`convert_roundtrip.rs`、`convert_ac.rs`・`nfr_security.rs` の一部)は、`testing::read_tree` で入力を読んだ途中の表現(`Value` の木)を、期待の木と比べています。`testing` をなくすと、外からは木が見えなくなります。

- A. src の単体テストへ移す(`#[cfg(test)]` の中なら `read_value` と `Value` を直接使える。期待の木と比べる今の確かめ方をそのまま残せる。移すのは機械的)
- B. 公開の `convert_text` だけで確かめる(期待の木を、変換後の文字列に書き直す。利用者に見える入口だけで確かめられるが、期待の値をすべて書き直すことになる)
- C. 組み合わせる(誤りだけを確かめるテストは `convert_text` で、木を比べるテストは単体テストへ)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] `Value` の手書きの `PartialEq`

背景: `Value` の比べ方(浮動小数の NaN どうしを等しいとみなす、マップはキーの順によらず比べる)は手書きで、本番では使われず、テストが木を比べるのにだけ使います。

- A. テストのときだけコンパイルする形にする(`#[cfg(test)]`。本番の成果物には入らない。Q2 で A か C を選んだ場合)
- B. テストの側の比べる関数に移す(`impl PartialEq` をなくし、テストの補助の関数で比べる)
- C. 残す(範囲から外す)
- X. Other (please specify)

[Answer]: A
