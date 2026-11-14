## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T15:08:10Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Cli behaviour / responsibilities (シグナルの受け取り) and JobRunner behaviour | シグナルの扱いが部品の境界で閉じていない。Cli は JobRunner を同期呼び出し(style: sync)で呼ぶため、ジョブの待機中に Cli がシグナルを受け取って JobRunner に伝える経路が、どちらの部品の記述にも書かれていない(コールバック、共有フラグ、JobRunner 側での受け取りなど、どの形かが未定)。また、ジョブが動いていない間(一連の実行でジョブとジョブのあいだ、ステートの書き込み中)にシグナルを受けたときの扱いも、FR2.9 が定めるジョブ実行中の場合以外は書かれていない。実装者が推測で決めることになる。 | シグナルの流れを1つに決めて、Cli と JobRunner の双方の interaction に書く(誰が受け取り、どの手段で JobRunner に届き、JobRunner は何を返すか)。ジョブ実行中でないときのシグナルの扱い(ステートの書き込み中を含む)も、方針を1行で書く。具体の仕組みは Functional Design に回してよいが、責務の分担はここで確定する。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Flow(entities: [])、Reporter、StateStore の behaviour | Flow は「入出力を持たず、必要な情報はすべて引数で受け取る」とされるが、受け取る情報の型(JobDefinition の遷移ルール、JobState の状態、終了コード)と返す結果(分類、次のジョブ、スキップ集合、完了、再開位置)が、どの部品のどのエンティティでもなく、依存の辺としても現れない。同様に Reporter は他部品の JobExecution や JobState を表示の元データとして受け取り、StateStore は PipelineDefinition 由来のジョブ構成(IDと順序)を受け取るが、これも宣言されていない。部品間は Cli 経由でも、データの型の上では Flow・Reporter・StateStore が DefinitionLoader・JobRunner・StateStore のエンティティに依存している。FR4.2 の「@end 到達後の個別実行」や FR4.3 の「再開位置」を Flow が純粋関数で実現できるかは、JobState に「@end でスキップされた」ことと「エラーの位置」を表す属性があるかに依存するが、属性は status / skipReason のみで、その表現が決まっていない。 | components.md に、Flow が受け取る入力と返す結果のおおまかな形(名前と属性の範囲)と、Reporter が受け取る表示データの種類を、所有部品つきで書き加える。あわせて JobState の status と skipReason で「@end によるスキップ」と「エラーの位置」を表せることを確認し、足りなければ属性を追加する。型を共有することによる依存は、辺を増やさずに「データ型のみの依存」として明記する。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/decisions.md > ADR-003 / ADR-004 / ADR-005 / ADR-006 の Alternatives Rejected | 各 ADR は Context / Decision / Consequences / Alternatives Rejected と Security / Compliance を備えるが、ADR-003、004、005、006 の退けた案は1つずつしかない。Inception フェーズの規則は、アーキテクチャ上の決定ごとに2案以上の比較を求めている(ADR-001、002 は2案ある)。ADR-004 は Q3 の選択肢が実質 A と B のみで、StateStore と Cli のどちらが照合するかの案(Cli が StateStore の読み出しと定義を比べる)が検討されていない。 | ADR-003〜006 に、2つ目以上の退けた案と理由を加える。ADR-004 には、Cli が照合する案を加える。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Cli behaviour(終了コードの決定) | Cli が、警告は 0、エラーはジョブの終了コード、成功コードに含まれない 0 は 1、起動失敗は 127、シグナルは 128 + シグナル番号、という終了コードの規則を持つ。これは FR2.2 / FR2.5 の業務規則であり、入出力のない Flow に置けば Flow の単体テストで確かめられる。調整役の Cli に残すと、規則の確認が統合テストに寄り、Cli が ADR-001 の言う「大きくなりうる」方向に進む。個別実行と一連の実行の両方の流れの組み立ても Cli にあり、肥大化の歯止めが「大きくなったら分ける」の一文のみである。 | 終了コードの規則(分類結果から終了コードへの対応)を Flow の責務に移すか、Cli に残す理由を ADR に書く。Cli に残す場合は、流れの組み立てを個別実行と一連の実行に分けるなど、肥大化したときの分割の方針を1行で示す。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > StateStore(PipelineState の identifier: pipelineName)と Assumptions & Open Questions | FR4.6 は「定義ファイル1つにつきステート1つ」で、別パスの定義ファイルを指定したときの紐づけは未決とされている。PipelineState の識別子を pipelineName にすると、別パスに同名のパイプラインがあるときに衝突する。識別子の選び方が、未決の紐づけを先取りしている。また、reset と watch が、食い違いのあるステートや壊れたステートに対して動くべきか(FR4.6 は reset 後に実行できると定める)が、StateStore の behaviour に書かれていない。 | PipelineState の識別子を、紐づけの方式が決まるまで仮のものと明記するか、定義ファイルのパスを識別に含める。reset は食い違い・破損の検査を迂回して初期化できること、watch は食い違い・破損のときの扱いを、StateStore と Cli の責務に一行で追記する。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/traceability.json | 17件のストーリーはすべて対象があり、対象の名前も components.md に存在する。ただし NFR1(起動オーバーヘッド 100ms、3つの OS)と NFR4(型の誤りの事前検知)を担う部品が、どの記述にも示されていない。NFR1 は、定義の読み込み、ステートの読み込み、ロケール判定のすべてが起動経路に乗るため、部品をまたぐ制約である。 | NFR1 と NFR4 を担う部品(DefinitionLoader、StateStore、Cli の起動経路)を components.md または traceability に1行ずつ記し、Construction で測る対象にする。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 目視による整合の確認(depends_on と dependents) | PASS | Cli の6つの依存は、各部品の dependents(Cli)と対称。循環と自己依存はなく、部品名は一意 |
| 目視によるエンティティ所有の確認 | PASS | 5つのエンティティはそれぞれ1つの部品が所有し、識別子を持ち、参照は所有部品つきで宣言されている |
| 目視によるストーリー対応の確認 | PASS | US1.1〜US7.1 の17件すべてが traceability.json にあり、対象はすべて components.md に存在する |
| 目視による境界の確認 | PASS | インフラは部品に含まれず(外部依存として扱う)、配置の形や技術スタックの決定もない。mermaid は有効で、text fallback がある |

### Summary

部品の境界は整合しており、循環や所有の重複はない。承認前に判断してほしいのは、シグナルを受け取る Cli と転送する JobRunner のあいだの経路(同期呼び出しのため未定)と、Flow などの入出力のない部品が受け取る/返す情報の形が、どの部品にも所有されていない点である。いずれも Functional Design で補える範囲で、Major は2件のため READY とした。
