# エンティティ: u7-watch-follow(`watch` の継続表示)

出典: 要件(`requirements.md` の FR7.1〜FR7.5、FR8.2)、ユニットの定義(`unit-of-work.md` の U7)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C5)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q4)。

## CliCommand(変更。cli の `types.rs`)

| 項目 | 型 | 説明 |
|---|---|---|
| `follow` | `bool` | `--follow` があるか。`watch` だけで使える |
| `interval` | `Option<u32>` | `--interval <秒>`。1〜3600 の整数(Q1)。`--follow` と一緒のときだけ。無ければ 2 |

## FollowEnd(新規。cli の中の値)

継続表示が終わった理由。

| 値 | 意味 | 終了コード |
|---|---|---|
| `Completed` | ステートが完了になった | 0 |
| `Removed` | 一度見えたステートが消えた | 0 |
| `Stopped` | エラーか中断で止まった(下の `PipelineActivity::Stopped`) | 0 |
| `Interrupted` | Ctrl-C などの中断の合図 | 0(Q3) |
| `StateError` | ステートが読めない | 65(Q4) |
| `OutputFailed` | 標準出力に書けない | 65(今の `watch` と同じ) |

合図の準備に失敗したときは、表示を始める前に今の `run`・`pipeline` と同じ表示と 70 で終わる(`FollowEnd` には入れない)。

## PipelineActivity(新規。core の純粋な判定)

ステートから読み取る、パイプラインの動きの様子(Q2)。

| 値 | 条件 |
|---|---|
| `Completed` | `completed` が真 |
| `Running` | 終わりの時刻の無い試行を持つジョブがある |
| `Stopped` | 上のどちらでもなく、`error` のジョブがあるか、再開の位置のジョブが試行を持っている |
| `Between` | 上のどれでもない(まだ始まっていない、ジョブとジョブの間) |

## 版

- `crates/cli/Cargo.toml` の `version` を `0.10.0` にする(FR8.2)。ほかのクレートの版の扱いは今の版の更新のやり方に合わせる。
