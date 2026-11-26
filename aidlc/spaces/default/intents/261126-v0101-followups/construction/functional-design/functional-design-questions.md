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

- F3: 測って二乗が原因と確かめられたときは、`flow` の中に、ジョブの位置を受け取る内部の入口を足し、実行と模擬の両方が同じ判断の本体を呼ぶ形で直す。判断の規則は 1 か所のままで、plan に書き写さない。実行の動きと結果は変えない(Q2 で Q1 を見直した答え)。
- 他の項目(F1、F2、F4、F5、文書のずれ)は、承認済みの要件のとおりに設計の文書へ書く。文書のずれの直し先に、要件の承認のときに直すと決めた R-07 の `crates/cli/src/tests/report_messages.rs` の 1225 行の注釈を加える。
- 設計のレビューの R-02〜R-04 を直す: デモのテストは標準入力を空のまま(`read` は EOF ですぐ戻る)にする。測定の表は `code-summary.md`、直さないときの判定の理由は `build-and-test-summary.md` に置く。直しても基準に届かないときは、直しを戻して記録にする。

- Looks correct
- Request changes

[Answer]: Looks correct

## Requested Changes Feedback

設計の承認の場で、レビューの指摘 R-01〜R-04 を直すと決めた(R-01 は推奨のとおり、`flow` に位置を受け取る内部の入口を足す形)。

[Answer]: Request Changes(R-01〜R-04 を直す。R-01 は推奨の形)

## Q2. [設計] F3 で直すとき、直す場所の広さ(Q1 の見直し)

設計のレビューで、Q1 の A(計画の模擬の側だけを直す)は、そのままでは実現できないと分かりました。遅さの元になりうる所(`job_index`・`record_of` の探索と、規則 6 の `definition.job[..index]` の走査)は、実行と模擬が共有する判断の本体 `crates/core/src/flow/decide.rs` の `decide_before_run`(非公開)と `flow/next_step.rs` の中にあり、`plan/simulate.rs` はジョブ ID だけでこれを呼びます。plan の側だけで直すと、判断の規則 1〜7 を plan に書き写すことになり、「実行と同じ判断を使う」が崩れます。

A. `flow` の中に、ジョブの位置を受け取る内部の入口を足し、実行と模擬の両方が同じ判断の本体を呼ぶ形で直す。判断の規則は 1 か所のまま。実行の動きと結果は変えない(`decide.rs`・`next_step.rs` の変更を認める)
B. 直さない。測った結果と、直さない理由を記録するだけにする
C. 共有の公開の関数(`job_index`・`record_of`)そのものに近道を入れる(Q1 の B)
X. Other (please specify)

[Answer]: A
