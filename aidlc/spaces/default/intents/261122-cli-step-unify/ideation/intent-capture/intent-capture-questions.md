# 意図の把握の質問(Intent Capture Questions): run と pipeline の重複をなくすリファクタリング

## Sources

- [desc] Initial description: "run と pipeline の重複をなくすリファクタリング: crates/cli/src/run.rs と pipeline.rs で、1 つのジョブを実行する手順(ステートの取り出し、スキップの記録、初期ステートの書き込み、リトライつきの実行、結果の分類、記録)がほぼ同じ文で 2 か所にある。1 つの関数にまとめ、pipeline にはループと中断の扱いだけを残す。動きは変えない。"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 何が出来上がれば「できた」と言えるか

背景: `run.rs` と `pipeline.rs` には、1 つのジョブを処理する手順(ステートの取り出し、スキップの記録、初期ステートの書き込み、リトライつきの実行、結果の分類と報告、記録、完了の判断)が、ほぼ同じ文で 2 か所にあります。

- A. 1 つのジョブを処理する手順が、1 つの関数にまとまる。`run` と `pipeline` は、その関数を呼ぶだけになる(`pipeline` には、ジョブのループと、ジョブのあいだの中断の扱いだけが残る)。利用者から見える動き(出力、終了コード、ステートの書き込みの順序と内容)は 1 文字も変わらない
- B. A に加えて、仕様書(`docs/spec/rust` の実行の流れの章)の、run と pipeline の手順の記述も、共通の手順として書き直す
- X. Other (please specify)

[Answer]: B

## Q2. [プロダクト] 範囲

背景: 調査では、ほかにも重複が見つかっています(`--dry-run` の模擬のステートの更新と本番の更新、run ID の解決とステートの一致の検査の 3 か所など)。チームの慣行は「変更は小さな単位」です。

- A. run と pipeline の重複だけ。ほかの重複は、別の intent にする
- B. run と pipeline の重複に加えて、run ID の解決とステートの一致の検査(`ctx.rs`、`graph.rs`、watch)も、この intent でまとめる
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 確かめ方

背景: 書き込みの順序と終了コードを細かく見る既存のテスト(`flow_run.rs`、`flow_pipeline.rs`、`flow_retry.rs`、`flow_run_id.rs`、`signals_order.rs` など)があります。

- A. 既存のテストを変えずに、すべて通ることで確かめる。テストは、共通の関数の単体のテストを最小限足すだけ
- B. A に加えて、run と pipeline で同じ手順になることを並べて確かめるテストを足す
- X. Other (please specify)

[Answer]: A
