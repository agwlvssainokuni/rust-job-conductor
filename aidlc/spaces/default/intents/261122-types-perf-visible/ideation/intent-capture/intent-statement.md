# 意図の記述(Intent Statement): 型と構造の整理・計算量・利用者から見える変更

## Sources

- [desc] Initial description: "型と構造の整理・計算量・利用者から見える変更(調査の 3・4・5)を 1 つの intent で: (3) 型と構造: ResumeInfo・PlanEntry.result・GraphNode(is_end)・RetryPolicy の backoff・NextStepNote の真偽値を列挙型に、RetryDecision::Retry に最後の終了コードと回数を持たせる、読まれないフィールド(JobExecution.job_id と command、JobOutcome.job_id、NextStep.job_id)、EnvFileError.reason、cli の Checked と CheckedDefinition の重複、準備の入口 3 つ、record_skip の真偽値の組、messages.rs の関数の形の統一と state の誤りの日本語の二重、外から使われない pub を狭める。(4) 計算量: 計画を作るときのステートの複製と find_mut、graph の overlay の線形の探索、遷移先の解決の job_index、.env の重複キーの位置の探索と削除、計画の名前の要約が値まで複製する点。(5) 利用者から見える変更: converter の文法の誤りの表示が「文法の誤りです: 文法の誤りです」と重なる点、.env の読み込みを開いてから大きさを見る順にそろえる。段はこの順で、利用者から見える変更を最後に置き、最後に版を 0.9.7 に上げる"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(承認は 1 回、段の順に進め、結果は最後にまとめる)
- [Q2] `intent-capture-questions.md` Q2: A(公開の型の変更に伴う既存のテストの変更は、組み立て、import、項目の読み方だけ認める。確かめる中身と期待する出力の文字列は変えない。一覧は検索で網羅して設計で承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(上限に近い入力の性能のテストを足し、直す前と後の時間を記録する。合格の時間の上限は設計の段で決める)
- [Q4] `intent-capture-questions.md` Q4: A(変換の文法の誤りの理由の文を形式ごとに具体的にする。例「TOML として読めません」「YAML として読めません」)
- [Q5] `intent-capture-questions.md` Q5: A(`.env` の読み込みを定義ファイルと同じ「種類を先に見る → 開く → 開いたものから大きさ」の順にそろえる)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)"

## Problem Statement

- 計画・図・リトライなどの公開の型に、真偽値や Option の組み合わせで状態を表し、あり得ない組み合わせを許す所がある。読まれないフィールドや、毎回写されるだけの値、同じ項目を二重に持つ型、似た準備の入口の重複もある。[desc]
- 計画を作るとき、図を作るとき、遷移先を解決するとき、`.env` の重複キーを扱うときに、入力の大きさの二乗の時間がかかる所がある。[desc]
- 変換の文法の誤りの表示は、種類と理由が同じ文で重なり、利用者に何も伝えない。`.env` の読み込みの順序は、定義ファイルとそろっていない。[desc]
- 解決したいのは、前の 2 つを利用者から見える動きを変えずに直し、最後の 1 つを利用者に分かる変更として直して、版を上げることである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]
- 段 3 については、変換と `.env` を使う利用者(誤りの表示が変わる)。[desc] [Q4] [Q5]

## Success Metrics

- 段 1(型と構造): 初期の記述の (3) の項目のそれぞれについて、あり得る場合だけを表す型になり、読まれないフィールドと二重の項目がなくなり、重複した入口がまとまる。利用者から見える動きは変わらない。[desc]
  - 確かめる手段: 設計の段で、項目ごとに今の場所(ファイルと行)と目標の形、完了の判定(たとえば「外から使われない pub を狭める」は対象の一覧と、狭めた後に外のクレートとテストから参照が無いこと。「messages.rs の関数の形の統一」は統一後の形と、対象の関数の一覧)を書き、実装の後にその場所を 1 つずつ確かめる。[desc] [Q1]
- 段 2(計算量): 初期の記述の (4) の項目が、入力の大きさに比例する時間(またはそれに近い時間)で終わる。上限に近い入力の性能のテストを足し、直す前と後の時間を記録する。合格の時間の上限は、設計の段で直す前の測定をもとに数で決める。[desc] [Q3]
- 段 3(利用者から見える変更): 初期の記述の (5) の 2 つを、次のとおり直す。[desc] [Q4] [Q5]
  - 変換の文法の誤りの理由の文が、形式ごとに具体的な文になる(例「TOML として読めません」)。種類の表示(「文法の誤りです」、英語は `syntax error`)は変えない。[Q4]
  - `.env` の読み込みが、定義ファイルと同じ「種類を先に見る → 開く → 開いたものから大きさ」の順になる。読む権限のない大きな `.env` は「読めない」と報告される。ほかの場合の誤りの種類は変わらない。[Q5]
  - どちらの変更もテストで確かめる。誤りの表示に入力の値を含めない性質は保つ(`aidlc/spaces/default/memory/project.md` の Forbidden の決まり)。[Q4] [Q5]
  - 前の intent(v0.9.6 の読み込みの順序)と同じく、変更を仕様書に書き、「v0.9.6 までは…」と注記する。[Q5]
- 版: 最後のコミットで v0.9.7 に上げる。[desc]
  - タグは `main` の上に付け、リリースは下書きとして作って人が確認してから公開する(チームの決まり)。この intent ではタグとリリースはせず、終わった後に頼まれたら行う。[memory:M6]
- 既存のテストの扱い: 段 1・2 の公開の型の変更で、組み立て、import、項目の読み方を直す必要があるテストは変えてよい。確かめる中身と期待する出力の文字列は変えない。変えるテストの一覧は、該当する行を検索で網羅して設計の段で示し、承認を取る。[Q2]
  - 段 3 で期待が変わるテスト(変換の文法の誤りの理由の文、版の表示)も、設計の段で一覧に挙げて承認を取る。[Q4] [desc]
  - 確かめる手段: Q1 の「各段の終わりにテストが通ることを確かめる」に従い、段の前と後でテストの件数も記録する。合格は、失敗 0 で、件数が「基準 + 足したテスト」に一致すること。基準は、実装を始める前の HEAD で `cargo test --locked --workspace` を macOS で実行した通過の件数とする。[Q1] [Q3]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- リファクタリングの候補の調査で挙がったもののうち、初期の記述の「調査の 3・4・5」を片付ける。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (3)、(4)、(5)。それぞれ段 1(型と構造)、段 2(計算量)、段 3(利用者から見える変更)にあたる。段はこの順で、利用者から見える変更の段を最後に置く。[desc]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は段の順に、小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 初期の記述に無いもの。[desc]
  - 直す仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` とする。凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は、前の intent `261122-internal-cleanup` の記録の決定に従い変えない(初期の記述には無い、前の決定の引き継ぎ)。[Q2]

## Assumptions & Open Questions

None.
