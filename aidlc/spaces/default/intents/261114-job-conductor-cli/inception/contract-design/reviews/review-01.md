## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T19:59:55Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 operations (`classify_and_decide`) | C2 には「ジョブの実行後」の判断(`classify_and_decide`)しかない。この入力は `jobExecution` を必須とするため、実行しないケースが表せない。たとえば、FR2.4(スキップ対象のジョブが呼ばれたら実処理をせず終了コード 0)、ジョブとジョブのあいだでの中断(components.md の Flow は、次に実行するジョブを `resumeJobId` に残す)、`@end` 後の個別実行(FR4.2)がそうである。「呼ばれたジョブを実行するか、スキップするか、順序外か」を誰が判断するかも、どの契約にも書かれていない。Open Questions の C2 行は、順序外・実行済みの振る舞いだけを扱っている。 | 実行前の判断を C2 に足す。たとえば `decide_before_run(jobId, jobStates, pipelineDefinition) -> run \| skip(理由, 終了コード 0) \| 順序外`。実行を伴わない中断(ジョブ間)の扱いも、C2 の入力か出力で表す。または、これらを Cli が持つ判断だと明記する。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > E2 rules、E3 rules、契約の所有と変更 | 書式の版の扱いに穴が残っている。(1) 定義ファイルに版のキーがない場合に、エラーか、版 1 とみなすか、が決まっていない。(2) 古い版の定義ファイルは、Q2 の「新しすぎるときはエラー」だけで、古い版のときの扱いがない。(3) ステートの規則は「古い版」だけで、新しすぎる版のステートの扱いがない。read_state は「版が合わない」と書くだけである。(4) 新しい版でキーを足した定義ファイルを古い `conductor` が読むと、「版が新しすぎる」エラーと「知らないキー」エラーのどちらが先に出るかが不定である。利用者向けのエラーが、バイナリごとに変わりうる。 | 次を明示する。版のキーを必須にするか(必須なら欠落はエラー)。版の比較の規則(同一のみ受理か、古い版も受理か)。ステートの新しすぎる版の扱い。版の検査を、知らないキーの検査より先に行うこと。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C1 `JobExecution`、C3 `run_job` | 中断のシグナルの契約が弱い。(1) `JobExecution` に `terminatedBySignal` と `interrupted` はあるが、シグナル番号を持つかが不明である。Flow の `128 + シグナル番号` の元になる値の出どころが決まっていない。転送したシグナルとは別のシグナルでジョブが終わる場合や、シグナルを無視して通常の終了コードで終わる場合も、扱いが不明である。(2) `run_job` の呼び出し前に合図がすでに立っていた場合に、ジョブを起動しないのか、が書かれていない。(3) 2回目のシグナルの扱いも書かれていない。 | `JobExecution` が持つシグナル番号の有無と、`interrupted` と終了コードの組み合わせの意味を書く。合図が立った後に `run_job` を呼ぶ場合の振る舞いを書く。2回目のシグナルの扱いを決める。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 `load_definition` | FR1.5 は `conductor pipeline <pipeline-name>` の名前が定義の `name` と一致することを求め、DefinitionLoader も「パイプライン名の不一致」を検証すると記している。しかし `load_definition` の入力は `path` だけで、検査の置き場所が決まっていない。Cli が比較するなら、その失敗は `DefinitionError` の種類にならない。 | `load_definition` に任意の期待名を足す。または、名前の比較を Cli の責務とし、どの失敗の種類で報告するかを書く。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C3 `reset_state`、C4 `convert`、契約の所有と変更(エラーとタイムアウト) | 失敗の分担にいくつか欠けがある。`reset_state` はファイルを書き換えるのに、失敗の種類が書かれていない(`StateError` のはずである)。`convert` は文字列を返すだけで、出力先(標準出力か、ファイルか)と、その書き込みの失敗の扱いが定まっていない。components.md は Converter が「出力の書き出し」を行うとしている。また、`check_consistency` の `mismatch` はエラーではなく値であり、誰がどのエラー終了として報告するか(E1 の「conductor 自身のエラー」)が書かれていない。 | `reset_state` の失敗の種類を書く。`convert` の出力先と書き込みの失敗の扱いを書く。`mismatch` を Cli が報告し、E1 の「conductor 自身のエラー」に含めることを明記する。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C1 末尾、E1 の `-v` 直後 | 参照の語句が、文の途中に残っている。「unit-of-work-dependency」(C1 の最後から2つ目の項目)と「constraint-register の C2」(E1 の最後から2つ目の項目)である。後者は、このまとめの契約 ID の C2(U1 の呼び出し)と紛らわしい。 | 参照を、出典の形式(括弧書きなど)に整える。契約 ID と紛らわしい表記は避ける。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (stage lists no validation tools) | Not run | The DAG edges were checked by hand. u2→u1 is C1 and C2 (types). u3→u1 is C1 and C2. u3→u2 is C3. u3→u4 is C4. The shared-type point is C1. All of them are present. The contracts table has the required columns. The three external boundaries are fixed per Q1. Unknown keys are errors (Q3). Q2 is partly reflected; see the versioning finding. |

### Summary

The coverage is sound. Every DAG edge and shared-type point has a contract. The external-versus-internal compatibility rules are stated coherently: the "ignore unknown fields" rule applies to internal types only, and the unknown-key error is reserved for E2. The main gaps are that the C2 decision has no pre-run counterpart, so the FR2.4 skip case and the between-job interruption have no contract to carry them, and that the format-version rules for the definition file and the state file are incomplete. Both can be settled in Functional Design, but they are worth deciding or recording before approval.
