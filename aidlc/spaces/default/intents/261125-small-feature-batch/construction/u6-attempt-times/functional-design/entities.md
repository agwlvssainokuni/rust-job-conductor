# エンティティ: u6-attempt-times(試行ごとの時刻と所要時間)

出典: 要件(`requirements.md` の FR6.1〜FR6.9)、ユニットの定義(`unit-of-work.md` の U6)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C4、C5)、設計の決定(ADR-004)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q3)。

## Attempt(新規。core の `types.rs`)

1 回の試行(ジョブの 1 回の起動)の時刻。

| 項目 | 型 | 説明 |
|---|---|---|
| `started_at` | `Timestamp` | 起動の直前に取った時刻。ログのファイル名の時刻と同じ値(U5) |
| `ended_at` | `Option<Timestamp>` | 試行が終わった後に取った時刻。動いている最中、または強制終了で止まったときは `None` |

- 時刻のほかの値(コマンド、環境変数、出力、終了コード)は持たない(FR6.3)。

## JobState(変更)

| 項目 | 変化 |
|---|---|
| `attempts: Vec<Attempt>` | 新規。古いものから順。再開しても前の試行を残し、後ろに足す(Q2) |

- `JobState::pending` は空の `attempts` を持つ。

## ステートファイル(変更。C4)

```toml
[[job]]
id = "build"
status = "success"
exit_code = 0

[[job.attempt]]
started_at = "2026-11-25T09:12:48Z"
ended_at = "2026-11-25T09:13:05Z"
```

- `[[job.attempt]]` のキーは `started_at`(必須)と `ended_at`(任意)だけ。値は UTC の RFC 3339(秒まで、`Z`)。
- `format_version` は 1 のまま。

## 読み取りの誤り(変更。runtime の `error.rs` の `reason`)

壊れたステート(`StateErrorKind::Corrupt`、65)の理由の文を 3 つ足す。どれも固定の文で、値を含めない。

| 理由 | 文 |
|---|---|
| `NO_ATTEMPT` | 動いたジョブに試行の記録がありません(古い形式の可能性があります) |
| `BAD_TIME` | 時刻の書式が誤っています |
| `UNFINISHED_ATTEMPT` | 終わりの時刻の無い試行が、最後の試行ではありません |

## 所要時間(新規。表示だけの値)

| 値 | 説明 |
|---|---|
| `JobDuration` | `Finished(秒)`(最後の試行が終わっている)、`Running(秒)`(最後の試行に終わりが無い。今の時刻までの経過)、`None`(試行が無い) |

- 表示の書き方は `17s`、`1m05s`、`2h03m04s`(Q3)。
