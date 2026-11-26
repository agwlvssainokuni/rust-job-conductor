# 機能の設計: 質問

要件(`requirements.md`、承認済み)の未決の事項のうち、人の判断が要るものだけを質問にします。F1(README の文)、F2(空白を除いて比べる・読む前に印を見る)、F4(手順の節)、F5(デモの待ち)、文書のずれ(R-07 の `crates/cli/src/tests/report_messages.rs` の 1225 行の注釈を加える)は、要件と今のコードで形が決まるので、設計の文書に直接書きます。

## Q1. [設計] F3 で直すとき、直す場所の広さ

コードを読んだ範囲では、計画の模擬は、ジョブごとに `PipelineDefinition::job_index`(`crates/core/src/types.rs:92`。一覧を先頭から探す)と、`record_of`(`crates/core/src/flow/mod.rs:46`。ステートの記録を先頭から探す)を呼びます。この 2 つは、計画の模擬だけでなく、実際の実行の流れ(`flow/decide.rs`、`flow/next_step.rs`)とも共有です。測って二乗が原因と確かめられたとき(FR3.3)、どこを直しますか。

A. 計画の模擬の側だけを直す(`crates/core/src/plan/` の中で、ジョブの位置を一覧の順から渡し、記録は位置で引く今の `status_at` の形を使う)。共有の関数と、実行の流れは変えない
B. 共有の関数を直す(`job_index` と `record_of` に、位置が合えば探さずに済む近道を入れる)。実行の流れも同じに速くなる
C. 測った結果を見てから、どちらにするかをコード生成の計画で決める
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- F3: 測って二乗が原因と確かめられたときは、計画の模擬の側(`crates/core/src/plan/`)だけを直す。ジョブの位置は一覧の順から渡し、記録は位置で引く今の `status_at` の形を使う。共有の `job_index`・`record_of` と、実行の流れ(`flow/decide.rs`・`flow/next_step.rs`)は変えない(Q1 A)。
- 他の項目(F1、F2、F4、F5、文書のずれ)は、承認済みの要件のとおりに設計の文書へ書く。文書のずれの直し先に、要件の承認のときに直すと決めた R-07 の `crates/cli/src/tests/report_messages.rs` の 1225 行の注釈を加える。

- Looks correct
- Request changes

[Answer]: Looks correct
