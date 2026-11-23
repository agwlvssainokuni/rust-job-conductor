# 意図の把握の質問(Intent Capture Questions): テストだけが使う本番のコードの削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 1: テストだけが使う本番のコードの削除(小。動きを変えない。版は上げない)。削除: core の Command の From<&str>・From<String>、cli の RunIdArg::as_str、runtime の InterruptSignal の is_raised・PartialEq・Eq・Default、本番で使われない Default(core の JobStatus、runtime の Durability、core の StateOverlay、cli の Language)、core の sanitize(escape_display を包むだけ)、core の EnvFileError::reason()。公開範囲を狭める: core の decide_before_run(テストは decide_from_state に)、core の flow::classify・MAX_RUN_ID_LEN・is_valid_env_name・is_reserved_env_name・JobDefinition::effective_success_codes の公開、cli の report::escape の escape_display の再公開、cli の lib.rs の pub mod 4 つ(args、deps、panic_hook、types)と根の GraphOptions・DryRunOptions の再公開。判断が要るもの: core の RunIdError の 4 種を 1 つに、runtime が core の関数と型を再公開している部分、誤りの種類の Display と Error、本番で使われない Hash、ライブラリの誤りの型の Display(残す推奨)。テストは偽物の側か公開の入口で代わりを持つ。範囲の外(別の intent): 読まれない項目(FlowEdge.explicit、EnvOrigin、EnvFileContent.path と EnvEntry.line、format_version と description)、converter の testing モジュール、cli のライブラリの公開範囲の大きい部分"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方と既存のテストの扱い

背景: 項目は小さく、1 項目 1 コミットで進められる見込みです。消す・狭める本番のコードを使っているテストは、偽物の側か公開の入口で代わりを持つ書き方に直します。消える関数そのものを試しているだけのテスト(`sanitize` と `escape_display` を比べる 1 件など)は、関数と一緒に消えます。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。確かめる中身と期待する出力の文字列は変えない。変える・消すテストは検索で網羅した一覧にし、消すものは理由を添えて設計の段で承認を取る
- B. 段ごとに止まり、結果を見せて、続けるかを確かめる
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] core の `RunIdError` の 4 種(空、長すぎる、使えない文字、先頭が `.`)

背景: 本番の呼ぶ所はすべて種類を見ずに捨て、同じ固定の文を出します。種類を分けて見るのは core の単体テストだけです。1 つにまとめると、単体テストは「通らない」ことだけを確かめる形になります(4 つの場合の入力は残す)。

- A. 種類のない誤りの型 1 つにまとめる(テストは 4 つの場合の入力が通らないことを確かめる)
- B. 残す(理由の種類は、将来の表示や仕様書の説明として意味がある。範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] runtime が core の関数と型を再公開している部分

背景: runtime は `check_consistency`、`apply_interrupt`・`apply_outcome`・`apply_skip`、`initial_state`、`ConsistencyResult`、`MismatchKind` を、core から再公開しています(「これまでの道を保つため」)。cli は core にも直接依存しているので、同じものへの入口が 2 つあります。再公開をやめると、cli の本番のコード 5 か所と、runtime と cli のテスト数ファイルの `use` を `conductor_core` に直します(中身は変わらない)。

- A. 再公開をやめ、使う所を `conductor_core` からの `use` に直す
- B. 残す(範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 誤りの「種類」の `Display`・`Error` と、本番で使われない `Hash`

背景: 次のものは、本番では使われないか、テストだけが使います。
- core の `DefinitionErrorKind` と `EnvFileErrorKind` の `Display` と `Error`(`Display` はテスト 2 か所の `to_string()` だけ。代わりは `label()`)、converter の `ConversionErrorKind` の `Error`(`Display` は本番で使うので残す)
- `Hash`: converter の `TargetFormat`、core の `EnvFileErrorKind` は使用 0 件。core の `DefinitionErrorKind` と converter の `ConversionErrorKind` は、テストが集合に入れるのに使うだけ

- A. すべて消す(テストは `label()` で確かめ、集合は並べて比べるなど別の形にする)
- B. 使用 0 件のものだけ消す(`Error` と、使用 0 件の `Hash`)。テストが使う `Display`・`Hash` は残す
- C. 残す(派生や慣例の実装は、使われていなくても害がない。範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] ライブラリの誤りの型の `Display`(日本語の文)

背景: core・runtime・converter の誤りの型は `Display` と `std::error::Error` を持ちますが、cli は誤りの型の項目から英語・日本語の文を自分で組み立てるので、`Display` を使いません。ただし、ライブラリの誤りの型が `Display` と `Error` を持つのは Rust の慣例で、「表示に入力の値を含めない」ことを確かめる多くのテストがこれを使います。

- A. 残す(推奨。慣例であり、表示の安全の確かめの対象でもある)
- B. 消す(cli が使わない文は不要。テストは cli の表示で確かめる)
- X. Other (please specify)

[Answer]: A
