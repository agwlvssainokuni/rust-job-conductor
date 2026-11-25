# 機能の仕様: u7-watch-follow(`watch` の継続表示)

出典: 要件(`requirements.md` の FR7.1〜FR7.5、FR8.1、FR8.2)、ユニットの定義(`unit-of-work.md` の U7)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C5)、規則(`rules.md` の BR1.1〜BR5.2)、エンティティ(`entities.md`)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q4)。

## 1. 部品と変更

| 部品 | 変更 |
|---|---|
| CliArgs(`args.rs`、`types.rs`) | `--follow`、`--interval <秒>`(BR1.2) |
| CoreFlow | `pipeline_activity(state) -> PipelineActivity`(純粋。BR3.1) |
| CliFlows(`aux_commands.rs`) | `run_watch` が `--follow` のとき繰り返す。合図の準備、間隔の待ち(`Runtime::wait`)、終わりの判定 |
| CliFlows(`step.rs`) | リトライの待ちの間は試行を開いたままにする(3.1) |
| CliReport(`watch.rs`、`mod.rs`、`messages.rs`、`platform`) | 画面を消す描き直し(UNIX 系の端末)と、追記の区切り |
| 文書とデモ | README の使い方と継続表示の説明。デモに `watch --follow` の場面 |
| 版 | `crates/cli/Cargo.toml` を 0.10.0 |

## 2. 流れ

```text
conductor watch --follow [--interval N]
  合図の準備(今の run・pipeline と同じ)
  seen = false
  繰り返し:
    合図が立っていれば → 0 で終わる(BR3.2)
    ステートを読む
      読めない → 誤りの表示、65(BR4.2)
      無い     → seen なら 0 で終わる(Removed)。でなければ「ステートが無い」を描く(BR4.1)
      ある     → seen = true。now() を取り、表を描く(BR2.1、BR2.2)
                 pipeline_activity が Completed か Stopped なら 0 で終わる(BR3.1)
    N 秒待つ(合図で打ち切れば 0 で終わる)
```

- 描く:UNIX 系の端末では、1 回目から毎回、画面を消す制御の文字の後に表を書く。ほかは、1 回目はそのまま、2 回目から空の 1 行を置いて表を書く。
- 標準出力に書けなければ、今の `watch` と同じく 65。
- 合図の準備に失敗したら、今の `run`・`pipeline` と同じ表示と終了コード(70)。

## 3. 動きの判定(`pipeline_activity`)

| 順 | 条件 | 値 |
|---|---|---|
| 1 | `completed` | `Completed` |
| 2 | 終わりの時刻の無い試行を持つジョブがある | `Running` |
| 3 | `error` のジョブがある、または `resume_job_id` のジョブが試行を持つ | `Stopped` |
| 4 | ほか | `Between` |

- `run` を 1 ジョブずつスケジューラから呼ぶ使い方では、ジョブとジョブの間は `Between` なので表示を続ける。
- 強制終了の後(最後の試行に終わりが無い)は `Running` のままなので、Ctrl-C で抜ける(U6 の README と同じ説明)。

### 3.1 リトライの待ちの間は `Running` に見えるようにする(設計のレビューの指摘)

今の U6 の流れは、リトライの前に「閉じた試行」を書いてから待つ。待ちの間のステートは、`status` が `pending`(初回)か前回の `error`(再開)のままで、`resume_job_id` がそのジョブを指すため、上の 3 の `Stopped` に当てはまってしまう。そこで U6 の流れ(`crates/cli/src/step.rs` の `run_job_with_retry`)を次のように直す。

| 場面 | 今(U6) | 直した後 |
|---|---|---|
| 試行が終わり、リトライする | `end_attempt` → Fast で書く → 待つ | 終わりの時刻をメモリに持ち、試行は開いたまま待つ(書かない) |
| 待ちの後、次の試行を始める | `start_attempt` → Fast | `end_attempt(持っていた終わりの時刻)` → `start_attempt(now)` → Fast で 1 回書く |
| 待ちの途中で中断 | (閉じた試行は書き済み)試行を足さずに抜ける | `end_attempt(持っていた終わりの時刻)` をしてから抜ける(最後の Durable の書き込みに入る) |

- ステートに書く時刻は今と同じ(試行の本当の終わりの時刻)。待ちの間だけ、ファイルの上では最後の試行が開いて見える。
- 待ちの間に conductor が強制終了されると、最後の試行は開いたまま残る(実行中の強制終了と同じ扱い。U6 の BR1.4 で次の開始のときに閉じる)。
- 書く回数は、リトライ 1 回で Fast 3 回から Fast 2 回に減る。U6 の流れのテストの期待(回数、待ちの間のステート)を合わせて直す。
- この直しで、`Stopped` は「動いている試行が無く、`error` のジョブがある、または `resume_job_id` のジョブが試行を持つ」のままで、リトライの待ちと区別できる。

## 4. 使い方の誤り(BR1.2)

| 入力 | 結果 |
|---|---|
| `--interval` に 0、3601、小数、負、文字 | 68 |
| `--follow` の無い `--interval` | 68 |
| `watch` 以外の `--follow`・`--interval` | 68 |

- 質問の Q1 の選択肢は使い方の誤りを「2」と書いたが、このプロジェクトの使い方の誤りの終了コードは 68(`EXIT_USAGE`)なので、68 とする。

## 5. 変わらないこと

- `--follow` の無い `watch` の表示と終了コード。ほかのコマンド。ステートは読むだけで書かない。
