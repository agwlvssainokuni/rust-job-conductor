## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T20:02:19Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 operations (`classify_and_decide`) | C2 には「ジョブの実行後」の判断(`classify_and_decide`)しかない。この入力は `jobExecution` を必須とするため、実行しないケースが表せない。たとえば、FR2.4(スキップ対象のジョブが呼ばれたら実処理をせず終了コード 0)、ジョブとジョブのあいだでの中断(components.md の Flow は、次に実行するジョブを `resumeJobId` に残す)、`@end` 後の個別実行(FR4.2)がそうである。「呼ばれたジョブを実行するか、スキップするか、順序外か」を誰が判断するかも、どの契約にも書かれていない。Open Questions の C2 行は、順序外・実行済みの振る舞いだけを扱っている。 | 実行前の判断を C2 に足す。たとえば `decide_before_run(jobId, jobStates, pipelineDefinition) -> run \| skip(理由, 終了コード 0) \| 順序外`。実行を伴わない中断(ジョブ間)の扱いも、C2 の入力か出力で表す。または、これらを Cli が持つ判断だと明記する。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > E2 rules、E3 rules、契約の所有と変更 | 書式の版の扱いに穴が残っている。(1) 定義ファイルに版のキーがない場合に、エラーか、版 1 とみなすか、が決まっていない。(2) 古い版の定義ファイルは、Q2 の「新しすぎるときはエラー」だけで、古い版のときの扱いがない。(3) ステートの規則は「古い版」だけで、新しすぎる版のステートの扱いがない。read_state は「版が合わない」と書くだけである。(4) 新しい版でキーを足した定義ファイルを古い `conductor` が読むと、「版が新しすぎる」エラーと「知らないキー」エラーのどちらが先に出るかが不定である。利用者向けのエラーが、バイナリごとに変わりうる。 | 次を明示する。版のキーを必須にするか(必須なら欠落はエラー)。版の比較の規則(同一のみ受理か、古い版も受理か)。ステートの新しすぎる版の扱い。版の検査を、知らないキーの検査より先に行うこと。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C1 `JobExecution`、C3 `run_job` | 中断のシグナルの契約が弱い。(1) `JobExecution` に `terminatedBySignal` と `interrupted` はあるが、シグナル番号を持つかが不明である。Flow の `128 + シグナル番号` の元になる値の出どころが決まっていない。転送したシグナルとは別のシグナルでジョブが終わる場合や、シグナルを無視して通常の終了コードで終わる場合も、扱いが不明である。(2) `run_job` の呼び出し前に合図がすでに立っていた場合に、ジョブを起動しないのか、が書かれていない。(3) 2回目のシグナルの扱いも書かれていない。 | `JobExecution` が持つシグナル番号の有無と、`interrupted` と終了コードの組み合わせの意味を書く。合図が立った後に `run_job` を呼ぶ場合の振る舞いを書く。2回目のシグナルの扱いを決める。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 `load_definition` | FR1.5 は `conductor pipeline <pipeline-name>` の名前が定義の `name` と一致することを求め、DefinitionLoader も「パイプライン名の不一致」を検証すると記している。しかし `load_definition` の入力は `path` だけで、検査の置き場所が決まっていない。Cli が比較するなら、その失敗は `DefinitionError` の種類にならない。 | `load_definition` に任意の期待名を足す。または、名前の比較を Cli の責務とし、どの失敗の種類で報告するかを書く。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C3 `reset_state`、C4 `convert`、契約の所有と変更(エラーとタイムアウト) | 失敗の分担にいくつか欠けがある。`reset_state` はファイルを書き換えるのに、失敗の種類が書かれていない(`StateError` のはずである)。`convert` は文字列を返すだけで、出力先(標準出力か、ファイルか)と、その書き込みの失敗の扱いが定まっていない。components.md は Converter が「出力の書き出し」を行うとしている。また、`check_consistency` の `mismatch` はエラーではなく値であり、誰がどのエラー終了として報告するか(E1 の「conductor 自身のエラー」)が書かれていない。 | `reset_state` の失敗の種類を書く。`convert` の出力先と書き込みの失敗の扱いを書く。`mismatch` を Cli が報告し、E1 の「conductor 自身のエラー」に含めることを明記する。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C1 末尾、E1 の `-v` 直後 | 参照の語句が、文の途中に残っている。「unit-of-work-dependency」(C1 の最後から2つ目の項目)と「constraint-register の C2」(E1 の最後から2つ目の項目)である。後者は、このまとめの契約 ID の C2(U1 の呼び出し)と紛らわしい。 | 参照を、出典の形式(括弧書きなど)に整える。契約 ID と紛らわしい表記は避ける。 | Resolved (reviewer) |
| R-07 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C3 run_job notes と C2 classify_and_decide | 呼び出しの前に合図が立っていると、run_job は起動せずに interrupted の JobExecution を返す。この実行はシグナルでは終了しておらず、terminatedBySignal は定義上「無し」になりうる。一方 classify_and_decide は、中断された実行をエラー(128 + シグナル番号)に分類する。components.md の Flow は、ジョブ間の中断では、ジョブの結果をエラーとして記録せず、まだ起動していないジョブを再開位置にするとしている。このため、起動前に中断された未起動のジョブが、エラーとして記録される恐れがある。また、128 + シグナル番号のシグナル番号の出どころが不明確である(合図が運ぶ番号と terminatedBySignal の関係)。 | 起動前の中断の扱いを一つに決める。たとえば、合図が事前に立っている場合は、Cli が run_job を呼ばず resume_point_on_interrupt の経路に回す。または、起動しなかった JobExecution を区別できるようにし、classify_and_decide がエラーとして記録しないと明記する。あわせて、128 + シグナル番号の算出元を、合図が運ぶ番号に一本化する。 | New |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 decide_before_run の Decision と E1 exit_codes | reject の場合の Cli の挙動が決まっていない。E1 の exit_codes には reject に対応する行がなく、conductor_own_error に入るかどうかも明記されていない。Decision は C1 の型の一覧にも components.md のエンティティにも無い。 | reject は conductor_own_error の非0で終了し、ジョブは実行しないと E1 に書く。Decision を C1 の型に加えるか、Flow が所有する戻り値として扱うかを書く。 | New |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 resume_point_on_interrupt | 出力が resumeJobId のみで、@end への到達済みの状態(endReached)や完了の扱いは、返さない。最後のジョブの直後は null(完了)とあるが、@end 到達後に中断された場合の PipelineState の更新方法が読み取れない。 | 戻り値に endReached と completed を含めるか、decide_before_run の skip 経路に任せると明記する。 | New |

> R-01 Reviewer note: decide_before_run (run/skip/reject) と resume_point_on_interrupt が C2 に追加され、呼び出し順序も明記された。skip・@end 後・ジョブ間の中断を表せる。reject の具体条件は未解決事項に残るが、結果の種類は確定している。

> R-02 Reviewer note: E2 の rules で、版のキーは必須、版が新しい定義はエラー、対応する版は 1 のみ、検査順序(版を先に)が定義された。E3 も古い版と新しすぎる版の両方をエラーにすると明記した。

> R-03 Reviewer note: terminatedBySignal をシグナル番号と定義し、合図の事前設定、2回目以降のシグナルの扱いを C3 に記述した。ただし、事前設定の場合の分類の整合は新しい指摘を参照。

> R-04 Reviewer note: load_definition に expectedPipelineName を追加し、名称不一致を DefinitionError で返すと明記した。

> R-05 Reviewer note: reset_state の失敗を StateError に限定した。StateError と mismatch の報告は Cli、ConversionError と書き出し先の責務も Cli と明記した。

> R-06 Reviewer note: E1 の注記で、constraint-register の制約 C2 と契約 C2 を区別した。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (stage 定義に検証ツールの指定なし。契約表と Open Questions の表を目視確認) | 契約表(C1〜C4、E1〜E3)と Open Questions の表は、列数・区切りとも整っている。components.md のエンティティ・責務と矛盾する箇所は、上の新しい指摘のみ。 | 構造上の不備なし |

### Summary

前回の R-01〜R-06 は、いずれも解消している。残る主な懸念は、起動前に中断の合図が立っていたときの、未起動のジョブの記録(エラーか、再開位置か)と、128 + シグナル番号の算出元の曖昧さである。Major は1件で、READY の基準(Major 2件以下、Critical なし)を満たすため、承認時に判断材料として扱ってほしい。
