## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T15:35:38Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Cli behaviour / responsibilities (シグナルの受け取り) and JobRunner behaviour | シグナルの扱いが部品の境界で閉じていない。Cli は JobRunner を同期呼び出し(style: sync)で呼ぶため、ジョブの待機中に Cli がシグナルを受け取って JobRunner に伝える経路が、どちらの部品の記述にも書かれていない(コールバック、共有フラグ、JobRunner 側での受け取りなど、どの形かが未定)。また、ジョブが動いていない間(一連の実行でジョブとジョブのあいだ、ステートの書き込み中)にシグナルを受けたときの扱いも、FR2.9 が定めるジョブ実行中の場合以外は書かれていない。実装者が推測で決めることになる。 | シグナルの流れを1つに決めて、Cli と JobRunner の双方の interaction に書く(誰が受け取り、どの手段で JobRunner に届き、JobRunner は何を返すか)。ジョブ実行中でないときのシグナルの扱い(ステートの書き込み中を含む)も、方針を1行で書く。具体の仕組みは Functional Design に回してよいが、責務の分担はここで確定する。 | Resolved (reviewer) |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Flow(entities: [])、Reporter、StateStore の behaviour | Flow は「入出力を持たず、必要な情報はすべて引数で受け取る」とされるが、受け取る情報の型(JobDefinition の遷移ルール、JobState の状態、終了コード)と返す結果(分類、次のジョブ、スキップ集合、完了、再開位置)が、どの部品のどのエンティティでもなく、依存の辺としても現れない。同様に Reporter は他部品の JobExecution や JobState を表示の元データとして受け取り、StateStore は PipelineDefinition 由来のジョブ構成(IDと順序)を受け取るが、これも宣言されていない。部品間は Cli 経由でも、データの型の上では Flow・Reporter・StateStore が DefinitionLoader・JobRunner・StateStore のエンティティに依存している。FR4.2 の「@end 到達後の個別実行」や FR4.3 の「再開位置」を Flow が純粋関数で実現できるかは、JobState に「@end でスキップされた」ことと「エラーの位置」を表す属性があるかに依存するが、属性は status / skipReason のみで、その表現が決まっていない。 | components.md に、Flow が受け取る入力と返す結果のおおまかな形(名前と属性の範囲)と、Reporter が受け取る表示データの種類を、所有部品つきで書き加える。あわせて JobState の status と skipReason で「@end によるスキップ」と「エラーの位置」を表せることを確認し、足りなければ属性を追加する。型を共有することによる依存は、辺を増やさずに「データ型のみの依存」として明記する。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/decisions.md > ADR-003 / ADR-004 / ADR-005 / ADR-006 の Alternatives Rejected | 各 ADR は Context / Decision / Consequences / Alternatives Rejected と Security / Compliance を備えるが、ADR-003、004、005、006 の退けた案は1つずつしかない。Inception フェーズの規則は、アーキテクチャ上の決定ごとに2案以上の比較を求めている(ADR-001、002 は2案ある)。ADR-004 は Q3 の選択肢が実質 A と B のみで、StateStore と Cli のどちらが照合するかの案(Cli が StateStore の読み出しと定義を比べる)が検討されていない。 | ADR-003〜006 に、2つ目以上の退けた案と理由を加える。ADR-004 には、Cli が照合する案を加える。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Cli behaviour(終了コードの決定) | Cli が、警告は 0、エラーはジョブの終了コード、成功コードに含まれない 0 は 1、起動失敗は 127、シグナルは 128 + シグナル番号、という終了コードの規則を持つ。これは FR2.2 / FR2.5 の業務規則であり、入出力のない Flow に置けば Flow の単体テストで確かめられる。調整役の Cli に残すと、規則の確認が統合テストに寄り、Cli が ADR-001 の言う「大きくなりうる」方向に進む。個別実行と一連の実行の両方の流れの組み立ても Cli にあり、肥大化の歯止めが「大きくなったら分ける」の一文のみである。 | 終了コードの規則(分類結果から終了コードへの対応)を Flow の責務に移すか、Cli に残す理由を ADR に書く。Cli に残す場合は、流れの組み立てを個別実行と一連の実行に分けるなど、肥大化したときの分割の方針を1行で示す。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > StateStore(PipelineState の identifier: pipelineName)と Assumptions & Open Questions | FR4.6 は「定義ファイル1つにつきステート1つ」で、別パスの定義ファイルを指定したときの紐づけは未決とされている。PipelineState の識別子を pipelineName にすると、別パスに同名のパイプラインがあるときに衝突する。識別子の選び方が、未決の紐づけを先取りしている。また、reset と watch が、食い違いのあるステートや壊れたステートに対して動くべきか(FR4.6 は reset 後に実行できると定める)が、StateStore の behaviour に書かれていない。 | PipelineState の識別子を、紐づけの方式が決まるまで仮のものと明記するか、定義ファイルのパスを識別に含める。reset は食い違い・破損の検査を迂回して初期化できること、watch は食い違い・破損のときの扱いを、StateStore と Cli の責務に一行で追記する。 | Resolved (reviewer) |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/traceability.json | 17件のストーリーはすべて対象があり、対象の名前も components.md に存在する。ただし NFR1(起動オーバーヘッド 100ms、3つの OS)と NFR4(型の誤りの事前検知)を担う部品が、どの記述にも示されていない。NFR1 は、定義の読み込み、ステートの読み込み、ロケール判定のすべてが起動経路に乗るため、部品をまたぐ制約である。 | NFR1 と NFR4 を担う部品(DefinitionLoader、StateStore、Cli の起動経路)を components.md または traceability に1行ずつ記し、Construction で測る対象にする。 | Resolved (reviewer) |
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Flow behaviour、entity JobOutcome(classification)、entity NextStep(resumeJobId)、Cli behaviour(ジョブが動いていないときのシグナル) | 要件では、シグナルで終わったジョブをエラーとして扱い、ステートにエラーの位置を残すと定めている。しかし Flow の分類(成功・警告・エラー)には、「中断された」実行結果(JobExecution の `interrupted`)がどの分類になるか書かれていない。また、ジョブの合間にシグナルを受けたときに「次に実行するジョブの位置」がステートに残ると書かれているが、これを `resumeJobId` で表すのか別の値で表すのか決まっていない。 | Flow の behaviour に、中断された結果はエラーに分類し、`resumeJobId` をそのジョブにする、と1行足す。ジョブの合間の中断で残す位置を表す属性(`resumeJobId` を流用するか)を、components.md か Functional Design の持ち越し事項に記す。 | Resolved (reviewer) |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Part B「データ型のみの依存」の Flow の行、entity NextStep の references | データ型のみの依存の表は、Flow が JobState と JobExecution の形に依存すると書くが、Flow が所有するエンティティ(JobOutcome、NextStep)の references には JobState がなく、NextStep は JobExecution も参照していない。表と、エンティティの参照が一致しない。 | 表の行に合わせて、JobState への参照を NextStep に足す。足さない場合は、表の行から JobState を外す。 | Resolved (reviewer) |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > Cli behaviour と Flow behaviour(ジョブ間のシグナル) | ジョブ間で中断されたとき、Cli は「次のジョブは起動せず、ステートに次に実行するジョブの位置を残す」と書いている。一方、その位置を Flow に判断させるのか、Cli が直前の NextStep から書くのかは、書かれていない。Flow 側は「ジョブ間の中断では nextJobId を resumeJobId として残す」とだけ書いており、Flow を呼ぶ契機が示されていない。また、最後のジョブの直後に中断された場合の扱いも決まっていない。 | Functional Design で、ジョブ間の中断のときも、Cli が直前の NextStep から resumeJobId を書くことを明記する。最後のジョブの直後に中断された場合の扱い(完了とするか)も決める。 | New |

> R-07 Reviewer note: Flow の behaviour に、中断された実行(interrupted)はエラー(128 + シグナル番号)に分類し、実行中のジョブを resumeJobId にすると明記された。ジョブ間の中断では、起動前の次のジョブ(NextStep の nextJobId)を resumeJobId として残し、ジョブの結果はエラーとして記録しないと決められた。

> R-08 Reviewer note: NextStep の references に JobState(StateStore)が追加された。「データ型のみの依存」表の Flow の行(JobDefinition、JobState、JobExecution)と、Entity Ownership 表の NextStep の References 欄が、YAML と一致している。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 目視での構造確認(components.md) | PASS | すべての references は所有部品の下に宣言されている(JobOutcome から JobExecution、NextStep から JobDefinition と JobState、DisplayEvent から JobOutcome と JobState)。Cli の depends_on の6つと、各部品の dependents(Cli)は対称である。依存の辺は Cli から出る星型で、循環はない。 |

### Summary

R-07 と R-08 はどちらも解消されており、components.md は構造として整合している(宣言、対称性、非循環)。残るのはジョブ間の中断で resumeJobId を誰がいつ書くかという細部だけで、Functional Design で決められる。
