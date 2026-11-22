# 意図の把握の質問(Intent Capture Questions): 動きを変えない整理のリファクタリング

## Sources

- [desc] Initial description: "動きを変えない整理のリファクタリング(調査の候補 A、B、D、E、F をまとめる): (1) cli の整理: run ID の決定とステートの照合の 3 か所(ctx.rs、graph.rs、watch)の集約、コマンドの分割の使い回し(dry_run と report_execution の通らない枝)、check_definition_and_env の ctx への移動、run と pipeline の入口の重複。(2) 型の整理: 分類と遷移を文字列で表示に渡すのをやめる、JobStatus と SkipReason の文字列の対応を core の型に 1 か所、check_consistency を core へ、reader.rs の上限つきの読み取りの重複。(3) core の整理: walk.rs を retry と env に分割、既定の成功コード [0] の重複、exit_code_of、入れ子の上限 32 と表示の上限 256 の重複、空の NextStep、JobExecution を作る関数。(4) テストと文書: graph のテストを形ごとに組み直す、converter のテスト名を日本語に、テストの偽物の共通の補助、README の終了コード 68 の行、go・python・typescript・csharp・powershell の仕様書の扱い。利用者から見える動きは変えない。引数の表(args.rs)は別の intent。"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] go・python・typescript・csharp・powershell の 5 つの仕様書の扱い

背景: この 5 つは v0.8.0 の時点で止まっており、graph、`--dry-run`、`env_file`、`init` がほとんど書かれていません。最新は `docs/spec/rust` と `docs/spec/language-agnostic` です。

- A. それぞれの `00-README.md` の冒頭に「v0.8.0 時点の凍結版。最新は rust と language-agnostic」と明記するだけ(中身は変えない)
- B. 今回は何もしない(別に決める)
- C. language-agnostic をもとに、5 つを最新に作り直す(この intent の範囲が大きく広がる)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は 4 つの段(cli の整理、型の整理、core の整理、テストと文書)で、全体で 20〜30 個ほどの小さなコミットになる見込みです。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 確かめ方

背景: 段 1〜3 は製品のコードだけを変えます。段 4 はテストのファイル自体を組み直します(graph のテストの組み直し、converter のテスト名の改名、偽物の補助の共通化)。

- A. 段 1〜3 は、既存のテストを 1 行も変えずにすべて通ることで確かめる。段 4 は、テストの中身(確かめている入力と期待)を落とさずに組み直し、重複して同じ例を確かめているものだけをまとめる。まとめたものの一覧を記録する
- B. 段 4 でも、テストの件数を減らさない(重複もそのまま残し、ファイルの移動と名前の変更だけにする)
- X. Other (please specify)

[Answer]: A
