## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T20:04:44Z
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
| R-07 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C3 run_job notes と C2 classify_and_decide | 呼び出しの前に合図が立っていると、run_job は起動せずに interrupted の JobExecution を返す。この実行はシグナルでは終了しておらず、terminatedBySignal は定義上「無し」になりうる。一方 classify_and_decide は、中断された実行をエラー(128 + シグナル番号)に分類する。components.md の Flow は、ジョブ間の中断では、ジョブの結果をエラーとして記録せず、まだ起動していないジョブを再開位置にするとしている。このため、起動前に中断された未起動のジョブが、エラーとして記録される恐れがある。また、128 + シグナル番号のシグナル番号の出どころが不明確である(合図が運ぶ番号と terminatedBySignal の関係)。 | 起動前の中断の扱いを一つに決める。たとえば、合図が事前に立っている場合は、Cli が run_job を呼ばず resume_point_on_interrupt の経路に回す。または、起動しなかった JobExecution を区別できるようにし、classify_and_decide がエラーとして記録しないと明記する。あわせて、128 + シグナル番号の算出元を、合図が運ぶ番号に一本化する。 | Unresolved |
| R-08 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 decide_before_run の Decision と E1 exit_codes | reject の場合の Cli の挙動が決まっていない。E1 の exit_codes には reject に対応する行がなく、conductor_own_error に入るかどうかも明記されていない。Decision は C1 の型の一覧にも components.md のエンティティにも無い。 | reject は conductor_own_error の非0で終了し、ジョブは実行しないと E1 に書く。Decision を C1 の型に加えるか、Flow が所有する戻り値として扱うかを書く。 | Resolved (reviewer) |
| R-09 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C2 resume_point_on_interrupt | 出力が resumeJobId のみで、@end への到達済みの状態(endReached)や完了の扱いは、返さない。最後のジョブの直後は null(完了)とあるが、@end 到達後に中断された場合の PipelineState の更新方法が読み取れない。 | 戻り値に endReached と completed を含めるか、decide_before_run の skip 経路に任せると明記する。 | Resolved (reviewer) |
| R-10 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/contract-design/contract-summary.md > C1 JobExecution と C2 classify_and_decide | ジョブを起動して中断された実行(started が true で interrupted)について、「128 + terminatedBySignal の番号」で分類すると書かれている。しかし、転送されたシグナルをジョブが捕捉し、自分の終了コードで終わった場合(例: SIGTERM を受けて 0 や 1 で終了)、terminatedBySignal は無しになる。このとき exitCode と interrupted が両方立つので、分類とスケジューラへ返す終了コードが決まらない。 | classify_and_decide が、中断された実行を exitCode ではなく受け取ったシグナルの番号で分類する旨を決める。あわせて、その番号の渡し方(JobExecution に受信したシグナル番号の項目を足す、または引数に足す)を C1・C2 で決める。この修正は R-07 の残りも同時に解消する。 | New |
| R-11 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/inception/domain-design/components.md > JobRunner の entity JobExecution、Flow の behaviour | 契約は JobExecution に started を加えたが、components.md の JobExecution の属性(jobId、command、exitCode、terminatedBySignal、startFailure、interrupted)には started がない。Flow の behaviour も「中断された実行はエラーに分類する」としか書かず、未起動の中断を区別していない。契約と上流の記述が食い違ったままである。 | Functional Design で、components.md の JobExecution と Flow の記述を契約に合わせて更新する。更新するまでは、契約を優先すると明記する。 | New |

> R-07 Reviewer note: 大半は解消した。JobExecution に started が加わり、未起動の中断はエラーにせず再開位置にする扱いが C2 classify_and_decide に書かれた。ただし 128 + シグナル番号の出所は、まだ C2 の入力だけでは追えない。未起動の場合、C1 は terminatedBySignal を「無し」と定めている。classify_and_decide の入力は [jobDefinition, jobExecution, jobStates, pipelineDefinition] で、シグナル番号を運ぶ引数はない。C3 の notes は、合図がシグナルの番号を運び、それを terminatedBySignal の算出に使う、と書くだけである。このため、未起動の中断で E1 の「128 + 受け取ったシグナルの番号」を返す手段が、契約に表れていない。

> R-08 Reviewer note: C1 に Decision(run、skip、reject)が加わり、E1 に reject の動き(ジョブを実行せず、理由を常に表示し、非0で終了し、ステートは変えない)が加わった。

> R-09 Reviewer note: resume_point_on_interrupt の出力が [resumeJobId, endReached, completed] になり、最後のジョブの直後と @end 到達済みの扱いも書かれた。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (none) | 実行していない | この確認では、ステージ定義が検証ツールを指定していない。契約の本文と components.md を手で突き合わせた。 |

### Summary

R-08 と R-09 は解消し、R-07 の中心(未起動の中断をエラーにしない)も解消した。残りは、中断時のシグナル番号を classify_and_decide が受け取る経路が契約に無い点と、ジョブが捕捉して自分の終了コードで終わる場合の扱いで、どちらも Minor である。Critical はなく、Major は 0 件のため、READY とした。
