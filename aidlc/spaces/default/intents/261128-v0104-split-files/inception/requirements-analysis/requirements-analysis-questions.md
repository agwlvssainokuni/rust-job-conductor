# 要件の分析: 質問

v0.10.4 で分ける範囲と、それに伴って直すものを決めます。

今の状態は、読み直したコードの知識と引き渡しに基づいています。
- コードの知識: `aidlc/spaces/default/codekb/rust-job-conductor/`
- 引き渡し: `inception/reverse-engineering/developer-scan.md` の「この intent のための詳細」の 1〜6

## Q1. [プロダクト] `Reporter::render` をどの単位で分けるか

`render` の `match` は 35 の枝で、37 の変種を扱います。分け方の候補は 2 つです。
- 表示の形で分けると 9 種(引き渡しの 1.3 の A〜I)
- 水準(誤り・警告・情報など)で分けると 4 種

A. 表示の形の 9 種ごとに関数にする。`render` は振り分けだけにする
B. 水準の 4 種ごとに関数にする
C. 関数に分けず、ファイルを分けるだけにする
X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] `walk/mod.rs` のキーの表を動かすか

`crates/cli/tests/spec_numbers.rs` は、`walk/mod.rs` の `const PIPELINE_KEYS`・`const JOB_KEYS` を文字列として読んでいます。

A. キーの表(`ROOT_KEYS`・`PIPELINE_KEYS`・`JOB_KEYS`・`TRANSITION_KEYS`)は `walk/mod.rs` に残す。テストは変えない
B. キーの表も外に出す。`spec_numbers.rs` の読み先を同じコミットで直す
X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] ファイルのパスを書いた文書を直すか

分けると、次の記述が古くなります。どのテストも検査していません。
- `docs/spec/rust/` の 02・07・08・12 章にある、モジュールとファイルの記述
- `crates/cli/src/lib.rs:47-59` と `crates/core/src/lib.rs:57` の文書コメント

language-agnostic 版は、ファイルの名前を書いていません。

A. どちらも、分けた後の形に直す(v0.10.2・v0.10.3 と同じく、rust 版の仕様書を実装に合わせる)
B. 文書コメントだけ直す。仕様書は後の intent に回す
X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] team.md の「使い方の表の 3 か所」の記述

team.md の Code Style に「使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える」とあります。オプションの表を `args` の下の別のファイルに移すと、このパスが変わります。

A. 移した先のパスに直す。v0.10.3 と同じく、コード生成の計画に今の文と書き換えた後の文を載せ、承認を経て直す
B. team.md は変えない(`args` のモジュールを指すものとして読む)
X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 読み直しで見つかった小さな古い記述

- `crates/core/src/error.rs:176` の文書コメントが、エスケープの関数を古い名前の `report::escape` で書いている。
- `crates/cli/src/tests/report_messages.rs:99` の `with_all_events` は手で並べた一覧で、`DefinitionValid` と `ResetAllKept` が入っていない。

A. どちらもこの intent で直す(分ける場所に近く、小さい)
B. 文書コメントだけ直す
C. どちらも直さない(後の候補)
X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] `args.rs` を分けた後のファイルの形

`report/` と `walk/` は `mod.rs` の形です。

A. 同じ形にそろえる。`args.rs` を `args/mod.rs` にし、`args/options.rs`・`args/tokens.rs`・`args/values.rs` を置く
B. `args.rs` は残し、`args/` の下に子のファイルを置く(Rust 2018 以降の形)
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- `report/mod.rs` を、`DisplayEvent` の定義、1 行の組み立て、エスケープの 3 つのファイルに分ける。あわせて `Reporter::render` を、表示の形の 9 種ごとの関数に分け、`render` は振り分けだけにする。言語と色の判定を必要な枝だけで行う今の形は保つ(Q1 A)。
- `walk/mod.rs` から、値の読み取り、遷移、ジョブをまたぐ検査を、`env.rs`・`retry.rs` と同じ形で外に出す。キーの表は `walk/mod.rs` に残し、`spec_numbers.rs` は変えない(Q2 A)。
- `args.rs` を `args/mod.rs` の形にし、オプションの表と検査、字句、値の解釈を子のファイルに分ける。`build_command` は、コマンドの枝ごとに関数に分ける(Q6 A)。
- 3 つとも、次の 3 つを保つ。
  - 動き(誤りの順と、最初に返す誤りを含む)
  - 公開の範囲(`pub use` で今の名前を残す)
  - `nfr_source.rs` の検査(`#[path]` は使わない)
- rust 版の仕様書の 02・07・08・12 章と、`lib.rs` の文書コメントを、分けた後の形に直す(Q3 A)。
- team.md の「使い方の表の 3 か所」のパスを、移した先に直す。コード生成の計画に前後の文を載せ、承認を経てから直す(Q4 A)。
- `core/src/error.rs:176` の古い名前を直す。`src/tests/report_messages.rs` の `with_all_events` に、足りない 2 つの変種を足す(Q5 A)。
- 最後に版を 0.10.4 にする。

- Looks correct
- Request changes

[Answer]: Looks correct
