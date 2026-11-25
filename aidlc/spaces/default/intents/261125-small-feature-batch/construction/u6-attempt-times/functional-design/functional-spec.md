# 機能の仕様: u6-attempt-times(試行ごとの時刻と所要時間)

出典: 要件(`requirements.md` の FR6.1〜FR6.9、FR8.1)、ユニットの定義(`unit-of-work.md` の U6)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C4、C5)、規則(`rules.md` の BR1.1〜BR5.2)、エンティティ(`entities.md`)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q3)。

## 1. 部品と変更

| 部品 | 変更 |
|---|---|
| CoreTypes | `Attempt` と `JobState.attempts` |
| CoreFlow(新しい純粋な関数) | `start_attempt(state, job_id, at)`(BR1.1、BR1.4)、`end_attempt(state, job_id, at)`(BR1.2)。`apply_outcome` と `apply_skip` は `attempts` をそのまま保つ |
| CoreReport の計算(core か cli) | `job_duration(job, now) -> JobDuration`、`format_duration(秒)`(BR4.1、BR4.2) |
| RuntimeState | 書き出しに `[[job.attempt]]`、読み込みに検査(BR2.1、BR2.2、BR3.1、BR3.2) |
| CliFlows(`step.rs`) | 試行ごとに、始める前に書く、終わったら閉じる |
| CliReport(`watch.rs`、`messages.rs`) | 所要時間の列(英日の見出し)。`watch` の流れが `Runtime::now()` を渡す |
| 文書とデモ | README、デモの `watch` の場面 |

## 2. 1 回の試行の流れ(`run_job_with_retry`)

```text
試行ごと:
  started_at = now()                       (U5 と同じ値。ログのファイル名にも使う)
  state = start_attempt(state, job, started_at)   → Fast で書く(BR1.1)
  起動 → 終わりを待つ(または起動の失敗)
  ended_at = now()
  state = end_attempt(state, job, ended_at)       (BR1.2)
  リトライなら: Fast で書く → 知らせ → 待つ
     待ちの中断: 試行を足さずに抜ける(BR1.3)
  リトライでなければ: 抜ける
抜けた後: 今の apply_outcome → Durable で書く(attempts を含む)
```

- 初めの書き込み(空のステートの初期化)は、最初の `start_attempt` の書き込みと一緒にしてよい(書く回数を増やさない)。
- 中断の合図を受けてジョブが終わったときも、ジョブの終わりを待つので `ended_at` が入る。`ended_at` が残らないのは、動いている最中か、conductor が強制終了されたときだけ(Q1)。

## 3. 読み込みの検査(どのコマンドでも同じ。BR3.1、BR3.2)

1. `[[job.attempt]]` の各表: キーは `started_at`(必須、文字列)と `ended_at`(任意、文字列)。ほかは `UNKNOWN_KEY`、欠けは `MISSING_KEY`、型の誤りは `WRONG_TYPE`、書式の誤りは `BAD_TIME`。
2. 最後でない試行に `ended_at` が無ければ `UNFINISHED_ATTEMPT`。
3. `success`・`warning`・`error` のジョブに試行が無ければ `NO_ATTEMPT`。
4. どれも `Corrupt`(65)。表示は今の壊れたステートと同じで、理由の文と `reset`(run ID があれば `reset --run-id <ID>`)の案内を出す。ステートの値は出さない(FR6.8)。

v0.9.9 までのステートは、動いたジョブがあれば `NO_ATTEMPT` で 65 になる(FR6.4。古い版から新しい版へのステートの引き継ぎは求めない)。

## 4. `watch` の表(BR4.1、BR4.2)

```text
パイプライン: nightly
ジョブ    状態      所要時間  補足
build     success   17s       終了コード 0
test      pending   1m05s
deploy    pending
```

- 列は「ジョブ」「状態」「所要時間」「補足」。英語の見出しは `duration`。
- `test` は最後の試行に終わりが無く、動いている最中とみなして今までの経過時間を出している。強制終了の後も同じに見えるため、README に「`conductor reset` か再実行で直る」と書く。
- 時刻は表示の時点で 1 回だけ取る。

## 5. 変わらないこと(BR5.1)

- 終了コード、ほかのコマンドの表示、`--dry-run` と `graph --state` の表示、ステートのほかのキー。
- `nfr_content.rs` は、`started_at` と `ended_at` のキーと時刻の値だけを許すように直し、コマンド・環境変数・出力が書かれないことは今までどおり確かめる。
