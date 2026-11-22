# 意図の記述(Intent Statement): args.rs のオプションの表化

## Sources

- [desc] Initial description: "args.rs のオプションの読み取りを表にまとめる refactor(誤りの報告の順を保つ。convert --to の誤りが入力の値を含む件と、enter_flow の到達しない分岐も扱う)"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: B(`--to` の誤りの対象をオプションの名前に変える)
- [Q2] `intent-capture-questions.md` Q2: A(字句と組み立ての両方を 1 つの表に)
- [Q3] `intent-capture-questions.md` Q3: A(`enter_flow` の分岐は残してコメントを直す)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER エラーの表示に入力の値を含めない。ステートに環境変数を書かない(`code-quality-assessment.md` の既存の性質)。 (affirmed 2026-11-20)"


## Problem Statement

- 引数の読み取りでは、オプションごとの性質(値を取るか、どのコマンドで使えるか、何回まで)が、表ではなく、字句の段階と組み立ての段階の別々の並びに書かれている。オプションを足すたびに複数の場所を揃えて直す必要がある。[desc] [Q2]
- 誤りの報告の順は、その並びの順で決まっており、仕様書に書かれた順と一致させ続ける必要がある。[desc]
- `convert --to` の値の誤りだけが、入力の値そのものを誤りの対象に出し、「エラーの表示に入力の値を含めない」という規則と揃っていない。[desc] [memory:M6]
- `enter_flow` の「位置引数なし」の分岐は、コマンドからは来ないが、そのことがコメントで「到達しない」と書かれ、公開の型から来たときの守りであることが分からない。[desc] [Q3]

## Target Customer

- このコマンドラインツールのコードを保守し、オプションを足す開発者。[desc]
- `conductor convert` を使う人(`--to` の誤りの表示が変わる)。[Q1]

## Success Metrics

- オプションの性質(名前、値を取るか、使えるコマンド、回数の上限)が 1 つの表にあり、字句と組み立ての検査がその表から行われる。オプションごとの別々の `match` 腕と `if` の並びが残らない。[Q2]
- 誤りの報告の順は今と同じ。`--to` の件を除き、既存の引数のテスト(`args_grammar.rs` ほか)を 1 行も変えずにすべて通る。[Q2]
- オプションの性質の表化の確かめ方: 設計の段階で、今のオプションごとの `match` 腕と `if` の場所(ファイルと行)を一覧にし、実装の後に、それらが表の行と、表を読む共通の検査に置き換わったことを確かめる。[Q2]
- `convert --to` の値が `yaml`/`toml` 以外のとき(UTF-8 でない値を含む)、誤りの対象は `--to` になり、入力の値を含まない。仕様書(rust と language-agnostic)の該当の行と、それを確かめるテストを直す。[Q1] [memory:M6]
- 知らないコマンドの名前、余分な位置引数、知らないオプションの綴りの対象は変えない。どこが悪いかを利用者に示すために入力を出している今の動きで、Q1 の答え B もこれらを変えないとしている。規則 M6 との差は、この intent では扱わない。[Q1]
- 利用者から見える変更は `--to` の誤りの対象だけで、版の更新の候補として記録する。[Q1]
- `enter_flow` の分岐は残り、コメントが「公開の型から来たときの守り」と書かれる。[Q3]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 前の intent(動きを変えない整理)で、引数の表は別の intent にすると決めていた。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: `crates/cli/src/args.rs` の表化、`--to` の誤りの対象、`enter_flow` のコメント。`CliCommand` の型の形は変えない。[desc] [Q1] [Q2] [Q3]
- 範囲の外: `CliCommand` の型の形 [Q3]、誤りの報告の順 [Q2]、`--to` 以外の誤りの対象 [Q1]、凍結した 5 つの仕様書(go、python、typescript、csharp、powershell。その `04-cli.md` にも「対象は値」があるが直さない。前の intent の記録 `aidlc/spaces/default/intents/261122-internal-cleanup/ideation/intent-capture/intent-capture-questions.md` の Q1 の答え A で、凍結の明記だけにして中身は変えないと決めている)。
- 進め方: 小さなコミットに分ける。[memory:M1]

## Assumptions & Open Questions

None.
