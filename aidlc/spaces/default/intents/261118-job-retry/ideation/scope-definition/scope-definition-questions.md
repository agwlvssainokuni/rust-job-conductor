# Scope Definition — 確認事項

意図の整理(`intent-statement.md`)で決めたこと: 一時的な失敗で止まるジョブを、所定の終了コードのときに最大 N 回、設定した間隔で自動リトライする。出力の内容での判定と、外部への通知は、最初の版でやらない。

## Q1. [プロダクト] 最初の版に、必ず入れる設定項目はどれですか?(複数選択可)

最初の版で価値を出すのに必要な、最小の範囲を決める質問です。

- A. リトライする終了コードの指定(どの終了コードのときだけ再試行するか)
- B. 最大の回数 N
- C. リトライの間隔(固定の秒数)
- D. None(まだ決めていない)
- X. Other (please specify)

[Answer]: A, B, C

## Q2. [プロダクト] リトライの設定は、どの単位で書けるようにしますか?

- A. ジョブごと(ジョブの定義の中に書く)
- B. ジョブごとと、全ジョブ共通の既定の設定(ジョブの設定が優先)
- C. パイプライン全体だけ
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] リトライの途中の様子(何回目か、次まで何秒か)の表示は、どの扱いにしますか?

- A. 必須(最初の版に入れる)
- B. あると良い(最初の版に入らなくてもよい)
- C. 不要
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] リトライの間隔の方式は、最初の版でどうしますか?

意図の整理で、指数バックオフ(回ごとに間隔を伸ばす方式)は「やらない」と決めておらず、含めるかどうかが未定でした。

- A. 固定の間隔だけにする(回ごとに伸ばす方式は、最初の版に入れない)
- B. 固定の間隔と、回ごとに伸ばす方式の両方を入れる
- C. Not yet defined
- X. Other (please specify)

[Answer]: B

## Q5. [プロダクト] 期限や、進め方の優先はありますか?

- A. 期限はない。必須の項目を先に作って出す
- B. 期限がある(Other で日付と理由を書いてください)
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

- Looks correct
- Request changes

[Answer]: Looks correct
