# 部品の設計: 質問

今ある部品(core・runtime・cli と、その中のモジュール)への機能の割り当てを決めます。DOT の描画(cli の `report` に足す)、出力の複製(runtime がジョブの出力を受けて端末とファイルへ書く)、`watch` の繰り返し(cli の `watch` の流れに足し、待つのは今の `wait` を使う)、`unset_env`(core の環境の統合に足し、runtime がジョブの環境から消す)は、現実的な形が 1 つしか無いため、質問せずに決定の記録に書きます。

## Q1. [設計] B6 時刻: 整形と時計の置き場所

UTC の RFC 3339 の整形と読み取り、現在の時刻の取得を、どこに置きますか。core は入出力を持たない純粋な部品です(team-practices.md の Code Style)。

A. 整形と読み取りは core に置く(純粋。日付の計算も自前)。現在の時刻は runtime から取り、cli の差し替え口(`Runtime`)を通す(テストで偽の時計に替えられる)
B. 整形・読み取り・現在の時刻を、すべて runtime に置く
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q2. [設計] B6 時刻: 試行の時刻を測る場所

各試行の開始と終了の時刻を、どこで測りますか。

A. runtime のジョブの起動(`run_job`)が、開始と終了の時刻を結果に入れて返す
B. cli の流れが、`run_job` を呼ぶ前と後に、差し替え口から現在の時刻を取る
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: B

## Q3. [設計] B1 リトライの既定: 既定を当てはめる場所

`[pipeline.retry]` を、各ジョブにどの段階で当てはめますか。今のリトライの判断(`retry_decision`)は、ジョブの `retry` だけを見ます。

A. 定義を読んで検証するとき(core)に、各ジョブの実効の `retry` を決めておく。以後の流れと判断は、今のまま、ジョブの `retry` だけを見る
B. 定義にはパイプラインとジョブの `retry` を別々に持ち、判断のたびに両方を見る
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] B1 リトライの既定: 打ち消しの書き方

特定のジョブで既定のリトライを打ち消す書き方を決めます(要件 FR1.4)。

A. `[[job]]` に `retry = false` と書く(`retry` は表か `false` のどちらか)
B. `[[job]]` に別のキー `no_retry = true` を足す
C. `[job.retry]` に `max = 0` と書く(今の `max` の範囲 1〜100 を 0〜100 に広げる)
D. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 時刻の置き場所(Q1): RFC 3339 の整形と読み取りは core(純粋)。現在の時刻は runtime から取り、cli の差し替え口(`Runtime`)を通す
- 時刻を測る場所(Q2): cli の流れが `run_job` の前後で差し替え口から取る。開始の時刻はジョブを動かす前にステートへ書く
- リトライの既定(Q3): 定義を読んで検証するとき(core)に、各ジョブの実効の `retry` を決める。リトライの判断と流れは今のまま
- 打ち消しの書き方(Q4): `[[job]]` に `retry = false`
- 質問せずに決めるもの: DOT の描画は cli の `report` に足す。出力の複製は runtime がジョブの出力を受けて端末とファイルへ書く。`watch` の繰り返しは cli の `watch` の流れに足し、待つのは今の `wait` を使う。`unset_env` は core の環境の統合に足し、runtime がジョブの環境から消す

- Looks correct
- Request changes

[Answer]: Looks correct
