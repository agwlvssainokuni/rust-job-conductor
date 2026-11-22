# 意図の把握の質問(Intent Capture Questions): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

## Sources

- [desc] Initial description: "残りの整理を 1 つの intent で: (1) JobExecution の列挙型化(動きを変えない。ステートファイルの書式は変えない)、(2) F: run_job(crates/runtime/src/runner.rs)に分割済みのコマンドを渡す(動きを変えない。Runtime トレイトと仕様書の 4 つの章)、(3) B3: core の定義ファイルの読み込みの順序(crates/core/src/definition/intake.rs の read_definition。利用者から見える変更で、仕様書と版の更新を伴う)。段はこの順で、B3 を最後に置く"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は 3 つの段(`JobExecution` の型、`run_job` の入力、定義ファイルの読み込みの順序)です。前回と同じく、計画の承認を 1 回にするかを決めます。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる(前回と同じ)
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 公開の型とトレイトを変えるときの、既存のテストの扱い

背景: 段 1 は `JobExecution`(core の公開の型)の 3 つの真偽値(起動したか、起動に失敗したか、中断されたか)を、あり得る組み合わせだけを表す型に変えます。段 2 は cli の `Runtime` トレイトの `run_job` の引数を変えます。runtime と core のテストの多くが `JobExecution` の項目を直接読んでいて(`execution.start_failure` など)、cli のテストの偽物(`crates/cli/tests/common/mod.rs`)は `Runtime` を実装しています。前回の答えは「組み立ての値と import だけ」でしたが、今回は項目の読み方も変わります。

- A. 認める。変えるのは組み立て、import、項目の読み方(同じ意味の新しい書き方への置き換え)、偽物の `run_job` の引数だけで、確かめる中身と期待する出力の文字列は変えない。設計の段で一覧と理由を示し、承認を取ってから進める
- B. テストの変更を減らすため、今の項目と同じ名前の読み出しの関数(`started()` など)を新しい型に残し、テストは組み立てと import だけ直す
- X. Other (please specify)

[Answer]: A(推奨を聞いたうえで A。項目を読んでいるのは 13 のテストのファイルで約 50 か所。B は古い見方を公開の関数として残し、段 1 の狙いを弱めるため)

## Q3. [プロダクト] 定義ファイルの読み込みの順序をどちらに揃えるか

背景: 定義ファイル(core)は「情報を見る → 大きさを確かめる → 開く」の順、変換の入力(converter)は「開く → 開いたものから大きさを見る」の順で、2 つが違います。どちらかに揃えると、まれな場合の誤りの種類が変わります。たとえば、読む権限のない大きなファイルのとき、今の core は「大きすぎる」、converter は「読めない」と報告します。

- A. core を converter の順(先に開く)に揃える。読む権限のない大きなファイルは「読めない」になる。開いたものから大きさを見るので、確かめてから開くまでの間にファイルが差し替わる心配もない
- B. converter を core の順(先に情報を見る)に揃える。読む権限のない大きなファイルは、変換でも「大きすぎる」になる
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 版の更新とリリース

背景: 段 3 は利用者から見える変更なので、版を上げる候補です(今は v0.9.5)。前回(`--to` の変更)は、intent の後に別に頼まれて v0.9.5 にしました。

- A. 前回と同じく、この intent では版を変えない。終わった後に、頼まれたら v0.9.6 にしてリリースする
- B. この intent の最後のコミットで v0.9.6 に上げる(タグとリリースは、終わった後に頼まれたら行う)
- X. Other (please specify)

[Answer]: B
