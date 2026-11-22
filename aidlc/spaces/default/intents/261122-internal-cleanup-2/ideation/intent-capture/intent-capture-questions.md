# 意図の把握の質問(Intent Capture Questions): 動きを変えない整理の第 2 弾

## Sources

- [desc] Initial description: "動きを変えない整理の第 2 弾(調査の候補のうち B3・F・JobExecution の列挙型化を除く全部): (1) 説明のずれ(apply_skip の説明、Plain、escape、lib.rs の表、config_path・65・read_state_raw、definition_key の仮、仕様書 11 章のテストの件数)と制約(core・converter の forbid(unsafe_code)、上限の数と定数の const assert)、(2) 小さな重複と計算量(status_at の共有と pending_ids の O(n²)、merge_env の O(E²)、既定の定義ファイル名、@end と Transition の target の型、<file>:<line>、graph の text と Mermaid の共通、使い方の誤りと出力の失敗の補助、終了コードの補助、ステートの削除、escape を core の sanitize に)、(3) cli の ctx の整理(report_execution の移動、Checked と Session、prepare_run と prepare_checked、resolve_run_id の置き場所)、(4) 型とテスト(run ID の型、スキップと拒否の理由の型、PlanMode/RunMode・EdgeKind/TransitionOn の重複、flow_with_state.rs を今の流れに)。利用者から見える動きは変えない"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は 4 つの段(説明と制約、小さな重複と計算量、cli の ctx、型とテスト)で、前回の整理(32 コミット)と同じくらいの大きさになる見込みです。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる(前回と同じ)
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 公開の型を変えるときの、既存のテストの扱い

背景: 段 2 と段 4 のいくつか(`DisplayEvent` の行き先を型に、スキップと拒否の理由の型の分割、run ID の型、`PlanMode`/`RunMode` などの重複)は、公開の型の形を変えます。既存のテストが、その型を直接組み立てていると、組み立ての値を直す必要が出ます(前回の `report_messages.rs` と同じ)。

- A. 前回と同じく認める。ただし変えるのは組み立ての値と import だけで、期待する出力の文字列は変えない。設計の段で、変えるテストの一覧と理由を示し、承認を取ってから進める
- B. 既存のテストを変える必要がある項目は、今回は外す(公開の型を変えない範囲だけにする)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] runtime の `flow_with_state.rs` の扱い

背景: このテストは「cli の流れを小さな関数で再現する」ものですが、今の cli と違う古い流れ(完了で常にステートを消す、`next_job_id` でたどる、変化がなくてもスキップを書く)になっていて、テスト名も古い動きのままです。本物の流れは、cli の `flow_run.rs`・`flow_pipeline.rs` が偽の Runtime で確かめています。

- A. runtime の関数を組み合わせる部分(読み・書き・照合・少なくとも 1 回の記録)は残し、再現する流れ、テスト名、期待を今の cli に合わせる
- B. cli の流れのテストと重なる部分は削り、runtime の関数の組み合わせを確かめる最小限だけにする(件数が減る。削るテストの一覧を記録する)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 計算量の改善の確かめ方

背景: 段 2 で、二乗の時間がかかる 2 か所(`@end` に着いたときの残りのジョブの列挙、`.env` の合成)を直します。今の性能のテストは、`.env` を 20 項目でしか測っていません。

- A. 上限に近い入力(たとえば `.env` 10,000 項目、多くのジョブ)の性能のテストを足し、直す前は遅く、直した後は速いことを測って記録する
- B. 性能のテストは足さず、既存のテストが通ることだけで確かめる
- X. Other (please specify)

[Answer]: A
