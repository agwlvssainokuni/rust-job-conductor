# Business Rules — u2-retry-execution

リトライの実行の規則。根拠は、`requirements.md` の要件(FR、NFR)と、`functional-design-questions.md` の回答(Q)。

```yaml
rules:
  - id: BR2.1
    statement: ジョブの 1 回の実行結果が、次のすべてを満たすときだけ、リトライの対象にする。(1) ジョブにリトライの設定がある (2) ジョブが起動した (3) conductor の中断ではない (4) 起動の失敗ではない (5) シグナルで終了していない (6) 終了コードがあり、設定の終了コードの集合に含まれる。
    category: policy
    applies_to: ジョブの実行結果
    trigger: ジョブの 1 回の実行が終わったとき
    logic: IF 上記の 6 つがすべて成り立つ THEN RetryDecision の候補は Retry、そうでなければ Stop
    violation_behaviour: なし(対象外は、これまでと同じ動き)
    source: FR1.2, FR3.1, FR3.6
  - id: BR2.2
    statement: 実行し直した回数(retries_done)が最大回数 max に達していなければ、実行し直す。達していれば、実行し直さない。合計の実行は、最大で max + 1 回になる。
    category: constraint
    applies_to: RetryState
    trigger: BR2.1 が Retry のとき
    logic: IF retries_done < max THEN Retry、ELSE Stop
    violation_behaviour: なし
    source: FR3.2
  - id: BR2.3
    statement: k 回目(k は 1 から)のリトライの前の待ち時間は、固定の間隔のときは interval_secs である。倍率と上限があるときは、1 回目は interval_secs、2 回目以降は「直前の待ち時間 × multiplier」で、max_interval_secs を超えない。計算は、桁あふれしないように、頭打ちで行う。
    category: calculation
    applies_to: RetryDecision.wait_secs
    trigger: Retry と決まったとき
    logic: IF multiplier と max_interval_secs がない THEN wait = interval_secs; ELSE IF k = 1 THEN wait = min(interval_secs, max_interval_secs); ELSE wait = min(last_wait × multiplier, max_interval_secs)
    violation_behaviour: なし
    source: FR2.1, FR2.2, Q2
  - id: BR2.4
    statement: 待機の前に、リトライの知らせ(job_id、直前の終了コード、何回目か、最大回数、待ち時間)を、標準エラーに必ず出す。`-v` がなくても出す。
    category: policy
    applies_to: RetryNotice
    trigger: Retry と決まり、待機に入る前
    logic: IF Retry THEN RetryNotice を、「常に標準エラー」の出力に出す
    violation_behaviour: 出力の書き込みの失敗は、既存の表示と同じ扱い(パニックにしない)
    source: FR6.1, Q3
  - id: BR2.5
    statement: 待機中に中断のシグナルを受けたら、すぐに待機をやめる。その結果は、起動前に中断されたときと同じ扱いにする。終了コードは 128 + シグナル番号で、エラーとしては記録せず、再開位置はそのジョブになる。以降の実行はしない。
    category: policy
    applies_to: WaitOutcome
    trigger: 待機中、または待機の開始時に、シグナルが立っている
    logic: IF WaitOutcome が Interrupted THEN 「起動していない、中断された」実行結果(signal_number つき)を作り、既存の分類と記録の流れに渡す
    violation_behaviour: なし
    source: FR4.1, Q7(要件)
  - id: BR2.6
    statement: ジョブの実行中のシグナルの扱い(子への転送、128 + 番号での終了、再開位置の記録)は、これまでと変えない。
    category: constraint
    applies_to: ジョブの実行
    trigger: ジョブの実行中
    logic: 既存の実行(`run_job`)をそのまま使う
    violation_behaviour: なし
    source: FR4.2
  - id: BR2.7
    statement: 最大回数まで実行し直しても、まだ対象の終了コードで終わるときは、最後の実行の結果を、通常のエラーと同じに扱う。終了コードは最後の実行のもの、再開位置はそのジョブ。
    category: policy
    applies_to: 最後の実行結果
    trigger: BR2.2 が Stop(上限に達した)のとき
    logic: IF retries_done = max かつ 対象の終了コード THEN 通常の分類と記録の流れをそのまま使う
    violation_behaviour: なし
    source: FR3.4, Q6(要件)
  - id: BR2.8
    statement: 実行し直した結果が、成功または警告の終了コードのときは、通常と同じ遷移で先に進む。
    category: policy
    applies_to: 実行結果
    trigger: 実行し直した結果が出たとき
    logic: IF 成功または警告と分類される THEN 既存の遷移の規則に従う(リトライしない)
    violation_behaviour: なし
    source: FR3.3
  - id: BR2.9
    statement: リトライの設定がないジョブ、または BR2.1 の対象外の結果は、これまでと完全に同じ動きにする。待機も、通知も、追加の実行もない。
    category: constraint
    applies_to: 実行の流れ
    trigger: ジョブの実行が終わったとき
    logic: IF BR2.1 を満たさない THEN 既存の流れ(分類、表示、記録、終了コード)のまま
    violation_behaviour: なし
    source: FR1.2, FR1.6, NFR1
  - id: BR2.10
    statement: リトライの回数と、リトライの途中の状態は、ステートファイルに記録しない。ステートは、最終の結果が出たときに、これまでと同じ形で 1 回だけ書く。途中で止まったあとの再開では、そのジョブを最初の実行から数え直す。
    category: constraint
    applies_to: ステートファイル
    trigger: リトライの途中、および再開
    logic: IF リトライの途中 THEN ステートを書かない; IF 再開 THEN retries_done = 0 から始める
    violation_behaviour: なし
    source: FR5.1, FR5.2
  - id: BR2.11
    statement: `run`(ジョブ 1 つ)と `pipeline`(パイプライン全体)は、同じリトライの流れ(1 つの共通の処理)を使い、同じ結果になる。
    category: policy
    applies_to: 実行の流れ
    trigger: ジョブの実行
    logic: 共通の処理「ジョブを、リトライを含めて実行する」を、`run` と `pipeline` の両方が呼ぶ
    violation_behaviour: なし
    source: FR3.5
  - id: BR2.12
    statement: ジョブの結果の表示(既存の `report_execution`)は、リトライを含めた最終の結果について、1 回だけ行う。途中の失敗の終了コードは、リトライの知らせ(BR2.4)に含めて示す。
    category: policy
    applies_to: 表示
    trigger: 最終の結果が出たとき
    logic: IF リトライがあった THEN 途中の結果の表示は、リトライの知らせだけにする
    violation_behaviour: なし
    source: FR6.1
  - id: BR2.13
    statement: 待機は、既存の差し込み口 `Runtime` に足す「待つ」を通して行う。本番は、シグナルが立っていないかを短い間隔(100 ミリ秒以内)で確かめながら待つ。テストは、待たずに記録だけをする偽の実装を使える。
    category: constraint
    applies_to: 待機
    trigger: 待機に入るとき
    logic: Runtime::wait(待ち時間, 合図) -> WaitOutcome
    violation_behaviour: なし
    source: NFR3, NFR7, Q1
  - id: BR2.14
    statement: 待機の打ち切りは、間隔の長さにかかわらず、シグナルを受けてから 1 秒以内に完了する。
    category: constraint
    applies_to: 待機
    trigger: 待機中のシグナル
    logic: 確かめる間隔を 100 ミリ秒以内にする
    violation_behaviour: なし
    source: NFR7
  - id: BR2.15
    statement: リトライの知らせは、固定の文と、整数と、ジョブ ID(既存のエスケープを通したもの)だけで組み立てる。入力の値そのものは、入れない。英語と日本語の両方を用意する(最初の文言は、この Unit で書く)。
    category: policy
    applies_to: RetryNotice
    trigger: 表示
    logic: 既存の表示の方針(固定の文、`escape_name`)に従う
    violation_behaviour: なし
    source: FR6.1, FR6.2
  - id: BR2.16
    statement: 起動できなかった場合(conductor が付ける終了コード 127)は、リトライの対象にしない。ジョブ自身が終了コード 127 で終わった場合は、設定の終了コードの一覧に 127 があれば、対象にする。
    category: policy
    applies_to: ジョブの実行結果
    trigger: BR2.1 の判定
    logic: IF start_failure THEN Stop; ELSE IF exit_code = 127 かつ 一覧に 127 THEN Retry の候補
    violation_behaviour: なし
    source: FR3.6, Q6(u1)
```

## 規則の要約

| ID | 内容 | カテゴリ | 要件 |
|---|---|---|---|
| BR2.1 | リトライの対象の判定(6 つの条件) | policy | FR1.2、FR3.1、FR3.6 |
| BR2.2 | 回数の上限(合計で最大 max + 1 回) | constraint | FR3.2 |
| BR2.3 | 待ち時間の計算(固定、倍率と上限、頭打ち) | calculation | FR2.1、FR2.2 |
| BR2.4 | 待機の前に、常に標準エラーへ知らせる | policy | FR6.1 |
| BR2.5 | 待機中のシグナルで、すぐ待機をやめて中断として扱う | policy | FR4.1 |
| BR2.6 | 実行中のシグナルは、これまでと同じ | constraint | FR4.2 |
| BR2.7 | 使い切ったら、通常のエラーと同じ | policy | FR3.4 |
| BR2.8 | 実行し直して成功・警告なら、通常の遷移 | policy | FR3.3 |
| BR2.9 | 対象外は、これまでと完全に同じ | constraint | FR1.2、FR1.6、NFR1 |
| BR2.10 | 回数を記録しない。再開は数え直し | constraint | FR5.1、FR5.2 |
| BR2.11 | `run` と `pipeline` は共通の処理 | policy | FR3.5 |
| BR2.12 | 結果の表示は最終の 1 回 | policy | FR6.1 |
| BR2.13 | 待機は `Runtime` の差し込み口を通す | constraint | NFR3、NFR7 |
| BR2.14 | 待機の打ち切りは 1 秒以内 | constraint | NFR7 |
| BR2.15 | 知らせは固定の文と整数だけ(英語と日本語) | policy | FR6.1、FR6.2 |
| BR2.16 | 127 の扱い | policy | FR3.6 |
