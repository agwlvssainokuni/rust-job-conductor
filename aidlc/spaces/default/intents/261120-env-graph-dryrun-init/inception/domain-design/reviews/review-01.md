## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-21T10:55:49Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/domain-design/components.md > component CorePlan (behaviour, entity PlanEntry.command_words, depends_on) | CorePlan は「分割後のコマンド」(command_words)を返すとされ、FR3.3 も dry-run でコマンドの誤り(引用符の閉じ忘れ)を見つけることを求める。しかしコマンドの分割は runtime::split(component-inventory.md)にあり、core は runtime に依存できない(core ← runtime ← cli)。CorePlan の depends_on に分割の部品は無く、分割をどこで行うか(CliFlows が分割して渡すのか、split を core へ移すのか)が書かれていない。部品が増えない前提(「split は載せない」)と矛盾しており、実装者が推測することになる。 | 分割の置き場所と呼び出しの向きを決めて書く。例: CliFlows が RuntimeRunner 側の split を呼んで分割結果と誤りを CorePlan に渡す、または split を core に移して CorePlan の depends_on に加える。決めた内容を ADR にし、FR3.3 の誤りがどこで出るかも明記する。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/domain-design/components.md > entity FlowEdge (identifier: from_job) | 1 つのジョブから出る辺は、成功と警告の行き先が異なれば 2 本以上になる(attributes に kind がある)。それなのに identifier が from_job だけで、一意にならない。 | identifier を (from_job, kind) などの複合にするか、辺を JobDefinition ごとの集合として表す形に直す。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/domain-design/components.md > component CoreTypes dependents (RuntimeRunner) と RuntimeRunner behaviour | RuntimeRunner は「統合済みの環境変数」を受け取るが、その型(EffectiveEnv)は CoreEnvFile が所有する。RuntimeRunner の depends_on は CoreTypes だけで、CoreTypes は EffectiveEnv を持たない。どの型で環境を受け渡すか不明である。また EffectiveEnv の識別子 job_id は、リトライごとに変わる CONDUCTOR_ATTEMPT を表せない。 | 受け渡しの型を決める(単純な名前と値の表なら CoreTypes に置く、または RuntimeRunner が CoreEnvFile に依存すると宣言する)。ATTEMPT を統合の入力にするかも書く。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/domain-design/components.md > CoreDefinition と CoreEnvFile (予約名 CONDUCTOR_ の拒否) | 予約名の拒否(FR1.11)が CoreDefinition(env の表)と CoreEnvFile(統合、.env 由来の名前)の両方に書かれており、責務が重なる。.env 由来の名前は定義の検証時には読まれていないので、後者が必要だが、境界が不明。 | 定義の env の表は CoreDefinition、.env 由来の名前は CoreEnvFile と、検査の分担を明記する。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/domain-design/decisions.md > ADR-001 から ADR-005 | 各 ADR に、セキュリティの影響(環境変数の値を出力・ステート・エラーに出さない、init の排他的作成など)の記述が無い。次の段階の指針は components.md の behaviour にあるが、決定の記録としては欠けている。 | ADR-001(値の秘匿)と ADR-004(排他的作成と上書き)に、セキュリティ上の影響を 1 行ずつ足す。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 手作業の整形式の検査(components.md の YAML) | PASS | 部品名は一意。depends_on と dependents は対称(CliFlows の 11 個の依存を含む)。自己依存なし。依存グラフは非循環。全エンティティは 1 つの部品が所有し identifier を持つ。references の 3 件(EffectiveEnv、FlowEdge、ExecutionPlan)は、owned_by の CoreTypes が所有する JobDefinition と PipelineState を指す。インフラは部品に含まれない。 |
| traceability.json と requirements.md の照合 | PASS | FR1.1 から FR5.2 までの 31 件が upstream_ids に揃い、すべて宣言済みの部品またはエンティティに対応する。 |

### Summary

構造の検査はすべて通り、4 件の ADR には 2 つ以上の代替案がある。主な懸念は、core の CorePlan が runtime にある split の結果(command_words)を必要とし、依存の向きと置き場所が未定な点である。これは Major 1 件で、READY の範囲内だが、承認の前に決めておくのがよい。
