# 意図の把握の質問(Intent Capture Questions): 型と構造の整理・計算量・利用者から見える変更

## Sources

- [desc] Initial description: "型と構造の整理・計算量・利用者から見える変更(調査の 3・4・5)を 1 つの intent で: (3) 型と構造: ResumeInfo・PlanEntry.result・GraphNode(is_end)・RetryPolicy の backoff・NextStepNote の真偽値を列挙型に、RetryDecision::Retry に最後の終了コードと回数を持たせる、読まれないフィールド(JobExecution.job_id と command、JobOutcome.job_id、NextStep.job_id)、EnvFileError.reason、cli の Checked と CheckedDefinition の重複、準備の入口 3 つ、record_skip の真偽値の組、messages.rs の関数の形の統一と state の誤りの日本語の二重、外から使われない pub を狭める。(4) 計算量: 計画を作るときのステートの複製と find_mut、graph の overlay の線形の探索、遷移先の解決の job_index、.env の重複キーの位置の探索と削除、計画の名前の要約が値まで複製する点。(5) 利用者から見える変更: converter の文法の誤りの表示が「文法の誤りです: 文法の誤りです」と重なる点、.env の読み込みを開いてから大きさを見る順にそろえる。段はこの順で、利用者から見える変更を最後に置き、最後に版を 0.9.7 に上げる"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は 3 つの段(型と構造、計算量、利用者から見える変更)で、前回の intent A と同じくらいか少し小さい見込みです。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる(前回と同じ)
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 公開の型を変えるときの、既存のテストの扱い

背景: 段 1 は、計画・図・リトライ・実行の結果などの公開の型の形を変えます。既存のテストがそれらを組み立てたり項目を読んだりしているので、テストのファイルを直す必要が出ます。

- A. 前々回と同じく認める。変えるのは組み立て、import、項目の読み方(同じ意味の新しい書き方への置き換え)だけで、確かめる中身と期待する出力の文字列は変えない。変えるテストの一覧は検索で網羅して作り、設計の段で承認を取る
- B. 既存のテストを変える必要がある項目は、今回は外す
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 計算量の改善の確かめ方

背景: 段 2 では、入力の大きさの二乗の時間がかかる所(計画のステートの複製、図のステートの探索、遷移先の解決、`.env` の重複キー)を直します。今ある性能のテストは、上限の中に収まっています。

- A. 前々回と同じく、上限に近い入力の性能のテストを足し、直す前と後の時間を測って記録する(合格の時間の上限は設計の段で、直す前の測定をもとに決める)
- B. 性能のテストは足さず、既存のテストが通ることだけで確かめる
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 変換の文法の誤りの表示をどう直すか

背景: 変換(`convert`)の文法の誤りは、今は「<ファイル>:<行>: 文法の誤りです: 文法の誤りです」のように、種類の表示と理由の文が同じ文で重なります。

- A. 理由の文を形式ごとに具体的にする(例: 「TOML として読めません」「YAML として読めません」)。表示は「…: 文法の誤りです: TOML として読めません」になる
- B. 理由の文が種類の表示と同じときは、理由を出さない。表示は「…: 文法の誤りです」になる
- C. 今回は直さない(範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] `.env` の読み込みの順序

背景: 定義ファイルの読み込みは、前回(v0.9.6)に「種類を先に見る → 開く → 開いたものから大きさを見る」の順にしました。`.env` の読み込みは、まだ「大きさを見てから開く」の順です。そろえると、読む権限のない大きな `.env` は「大きすぎる」ではなく「読めない」と報告されるようになります(前回と同じ種類の変更)。

- A. 定義ファイルと同じ順にそろえる(読む権限のない大きな `.env` は「読めない」になる)
- B. 今回は直さない(範囲から外す)
- X. Other (please specify)

[Answer]: A
