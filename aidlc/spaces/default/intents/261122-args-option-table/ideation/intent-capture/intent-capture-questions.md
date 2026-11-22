# 意図の把握の質問(Intent Capture Questions): args.rs のオプションの表化

## Sources

- [desc] Initial description: "args.rs のオプションの読み取りを表にまとめる refactor(誤りの報告の順を保つ。convert --to の誤りが入力の値を含む件と、enter_flow の到達しない分岐も扱う)"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER エラーの表示に入力の値を含めない。ステートに環境変数を書かない(`code-quality-assessment.md` の既存の性質)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] `convert --to` の誤りが入力の値を含む件

背景: `--to` の値が `yaml`/`toml` 以外のとき、誤りの対象に**入力の値そのもの**を出します(`--format` などは、オプションの名前だけ)。これは仕様書にも書かれた今の動きです(`docs/spec/language-agnostic/04-cli.md` の 79 行と 134 行。「対象は値」)。一方で、プロジェクトの規則は「エラーの表示に入力の値を含めない」です。ほかにも、知らないコマンドの名前、余分な位置引数、知らないオプションの綴りは、入力をそのまま対象に出します(どこが悪いかを利用者に示すため)。

- A. 今回は動きを変えない(表にまとめるだけ)。`--to` の扱いは、仕様書どおりのまま残す
- B. `--to` の誤りの対象を `--to`(オプションの名前)に変える。利用者から見える変更なので、仕様書(rust と language-agnostic)とテストを直し、版の更新の候補にする。ほかの対象(コマンド名など)は変えない
- X. Other (please specify)

[Answer]: B

## Q2. [プロダクト] 表にする範囲

背景: 今は、オプションごとに「値を取るか」(字句の段階)、「どのコマンドで使えるか」と「何回まで」(組み立ての段階)が、別々の `match` と `if` の並びに書かれています。誤りの報告の順(仕様書の 04 章の 4.2 の 6〜8a)は、この `if` の並びの順で決まっています。

- A. オプションを 1 つの表(名前、値を取るか、使えるコマンド、回数の上限)にまとめ、字句と組み立ての検査をその表から行う。誤りの報告の順は、表の並びと検査の順で今と同じにする。既存のテスト(`args_grammar.rs` ほか)は変えずに通す
- B. 字句の段階(名前と値を取るか)だけを表にし、組み立ての段階の `if` の並びは残す(変更を小さくする)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] `enter_flow` の「位置引数なし」の分岐

背景: 引数の読み取りは、`run` と `pipeline` に位置引数が無ければ先に誤りにするので、コマンドから使う限り、この分岐には来ません。ただし `CliCommand` は公開の型で、テストやライブラリとして使う側は `job_id: None` のまま流れを呼べるため、その道では到達します。

- A. 残す(公開の型から呼ばれたときの守り)。到達しない理由ではなく「公開の型から来たときの守り」とコメントを直す
- B. `CliCommand` をコマンドごとの型に分け、位置引数が必ずある形にして分岐をなくす(流れのテストの組み立てを多く変えることになる)
- X. Other (please specify)

[Answer]: A
