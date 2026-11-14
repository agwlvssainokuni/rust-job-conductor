## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T20:22:15Z
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Critical | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/functional-spec.md > W3 step 8; rules.md > BR3.6, BR3.3 | W3 step 8 sets `completed` to true as soon as `@end` is reached. BR3.6 splits the cases into 一連の実行 and 個別実行, but `classify_and_decide` has no input saying which mode it runs in (contract-summary C2 inputs: jobDefinition, jobExecution, jobStates, pipelineDefinition). In individual runs the state would therefore be reset right after the job that points to `@end`. AC4.2.3 says the later job must still be called and skipped, and that the reset happens at that skip. With the state reset, the later job would be called against an empty state, W2 step 4 would return `reject(outOfOrder)` and the exit code would be 66 instead of 0. That also contradicts AC2.2.3 and AC2.3.3, where the jobs after `@end` are recorded as skipped and return 0. The two BR3.6 clauses for individual runs ("@end のあとのジョブが呼ばれ、最後のジョブがスキップされた") and for pipeline runs cannot be decided by the function as specified. | Decide when `completed` becomes true after `@end`. Either record the skips and mark completion only when the last job is executed or skipped (the same rule for both modes), or add an explicit mode input and say so in the contract. Rewrite BR3.6 and W3 step 8 to match. Add a worked sequence for AC4.2.3 and AC3.1.3. | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/entities.md > PipelineDefinition.jobs, JobDefinition.id, JobDefinition.transitions, PipelineDefinition.name, JobDefinition.command; rules.md > BR1.x; DefinitionError.kind | The entities set constraints that no rule checks and no `DefinitionError.kind` can report: duplicate job IDs, an empty or `@end`-prefixed ID, zero jobs, an empty name or command, and more than one transition for the same `on`. BR1.3 and BR3.2 depend on a job's position by ID, so a duplicate ID makes "前のジョブ" and "間のジョブ" ambiguous. Today such a definition is accepted by BR1.11. | Add rules (BR1.x) and `DefinitionError.kind` values for these constraints, or remove the constraints from entities.md. Add them to traceability.json `reverse` with their origin. Make the W1 order cover them (duplicate IDs before BR1.1 and BR1.3). | Resolved (reviewer) |
| R-03 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/rules.md > BR2.3; functional-spec.md > W3 steps 2-3; entities.md > JobExecution.signalNumber; contract-summary.md > C1 JobExecution, C2 classify_and_decide | The two documents order the checks differently. W3 checks "中断で started が true" (step 2) before "シグナル終了" (step 3). BR2.3 lists `startFailure`, `terminatedBySignal` and `interrupted` in that order. When a job is interrupted and also ended by a signal different from the one received (for example, killed after the forwarded signal), the first document returns 128 + `signalNumber` and the second returns 128 + `terminatedBySignal`. The contract (C2, E1) says "128 + 受け取ったシグナルの番号" for an interrupt, and C2 also says "128 + terminatedBySignal の番号". `signalNumber` is a new field that is not in C1, and the contract text was not updated. | Define a single precedence in BR2.3 and W3 (interrupt first, to match E1). State which field gives the number. Record `signalNumber` as an addition to C1/C2, or fix the contract wording so both documents agree. | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/rules.md > BR3.5 | The BR3.5 logic has no `run` branch for the normal case of "all previous jobs done, this one is next". That case exists only in W2 step 5, even though rules.md is the rule source. "記録されていない" does not say whether a missing job and a `pending` JobState are the same. When a job recorded as skipped by `byEnd` is called, the Decision reason (`skipTarget` or `afterEnd`) is not tied to the recorded `skipReason` (`byTransition` or `byEnd`). | Add the normal `run` branch to BR3.5. Define how a missing record and `pending` are treated. Add a mapping table from the recorded skip reason to the Decision reason. | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/rules.md > BR5.1; functional-design-questions.md > Q3 | Q3 asked for conductor's own exit codes to be "区別できる値" from job codes. The values 64-67 are ordinary values a job can return and pass through unchanged (BR2.2), so the scheduler cannot tell a job's 65 from a state error. The decision was made by the human, but the artifacts do not mention this limit. | Record the overlap as a known limitation in Assumptions & Open Questions, or ask the human to confirm it. | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/traceability.json > AC2.1.6, AC4.3.1 | AC2.1.6 requires that "ステートにエラーの位置が残る" but maps only to BR2.2. The resume position comes from BR3.4. AC4.3.1 maps to BR3.4 and BR3.5 and omits BR3.7 (and the `resumeJobId` clearing after a successful re-run is not specified anywhere). | Add BR3.4 to AC2.1.6. State in BR3.4/W3 when `resumeJobId` is cleared. | Resolved (reviewer) |
| R-07 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/functional-spec.md > W2 / W3 手順 8 / rules.md > BR3.6 | individual モードでは、@end 後に最後のジョブが W2 で skip された時点で completed を true にするとしているが、W2 の出力は Decision(run/skip/reject)だけで completed を持たない。W3 は実行後にしか呼ばれず、W4 は中断時だけである。このため、最後のジョブがスキップで終わる経路で、completed(と resumeJobId の解除)を決めて U2 に渡す関数が存在しない。AC4.2.3 の「完了後は初期化される」が実装できない。 | skip の判断のときに completed を返す仕組みを決める。たとえば W2 の出力に completed を足すか、スキップの記録後に完了を判定する関数を W5 として追加する。契約 C2 の更新の一覧にも入れる。 | New |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/functional-spec.md > W1 手順 / rules.md > BR1.11 | W1 の手順に BR1.13(ジョブID重複・空、ジョブ0件、空の name・command、遷移の重複)の検査がない。BR1.11 の受理条件も「BR1.1〜BR1.9」としか書いておらず、BR1.13 が抜けている。ワークフローを正とする方針のため、実装者が検査を落とすおそれがある。ID の重複や 0 件は、BR1.1/BR1.3 の遷移先の検査より先に行う必要もある。 | W1 に BR1.13 の検査の手順を足し(遷移の検査より前)、BR1.11 の条件に BR1.13 を含める。rules.md の要約表の BR1.1〜BR1.12 も BR1.13 まで直す。 | New |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u1-core/functional-design/functional-spec.md > W3 手順 5 / rules.md > BR2.4 と JobOutcome | W3 の手順番号が 5 で 2 回出る(分類と返す終了コード)。また、起動しなかった中断では分類を行わないのに、JobOutcome.classification は必須の enum で、その場合の値が定義されていない。 | 手順番号を振り直す。起動しなかった中断のときの JobOutcome の扱い(classification を省略可にする、または JobOutcome を返さない)を明記する。 | New |

> R-01 Reviewer note: classify_and_decide が runMode を受け取り、individual では @end で endReached のみ true、最後のジョブのスキップ時に completed が true になる。BR3.6 と W3 の 8 が一致している。ただし「スキップ時の completed」を誰が出力するかが未定義(新規指摘を参照)。

> R-02 Reviewer note: BR1.13 と DefinitionError の新しい kind(duplicateJobId、invalidJobId、noJobs、emptyField、duplicateTransition)が追加され、traceability にも逆引きの項目がある。W1 への反映漏れは新規指摘とする。

> R-03 Reviewer note: BR2.3 と W3 の 1〜5 の優先順位が一致し、signalNumber も entities.md と契約更新の一覧に入っている。

> R-04 Reviewer note: BR3.5 と W2 が同じ 7 段の順序規則になり、理由の対応も明記された。

> R-05 Reviewer note: entities.md の Assumptions と JobOutcome.reportedExitCode の制約に、既知の制限として記録された。

> R-06 Reviewer note: traceability の AC2.1.6 に BR3.4、AC4.3.1 に BR3.7 が入り、BR3.4 に resumeJobId を解除する条件が書かれた。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| traceability.json の JSON 検証 | PASS | 構文は正しく、AC2.1.6、AC4.3.1、BR1.13 の項目を確認した。 |

### Summary

前回の Critical を含む 6 件の指摘は、すべて解消している。新規は Major 1 件(individual モードでスキップ時の completed の出力元が未定義)と Minor 2 件で、Major は 2 件以下のため READY とするが、Major は承認前に直すことを勧める。
