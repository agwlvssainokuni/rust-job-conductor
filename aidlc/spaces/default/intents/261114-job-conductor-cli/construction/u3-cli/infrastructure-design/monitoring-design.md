# Monitoring Design(u3-cli)

上流: infrastructure-specification、nfr-design の observability-design、reliability-design、performance-design。`conductor` は、1回で終わるコマンドで、常駐しない。**稼働中の監視の対象(サービス)ではなく、メトリクスの収集・アラート・ダッシュボード・分散トレースの仕組みを持たない**。観測は、`conductor` 自身の標準出力・標準エラーと終了コード(既存のスケジューラのログが拾う)と、**CI が出す測定の記録**だけである。この文書は、observability-design の戦略を、プラットフォーム(GitHub Actions と、利用者のスケジューラ)で、どう実現するかを、表で示す。

## Metrics & KPIs

| Metric | Source | Threshold | Why it matters |
|--------|--------|-----------|----------------|
| `conductor` 自身のオーバーヘッド(起動前、ジョブのあと、合計) | CI の `startup-bench` ジョブ(3つの OS)。ジョブの要約欄(Job Summary)に、p95 を出力 | 合計 p95 が 100ms 以内(配分の確定のあと、超えたら CI を失敗にする。確定までは警告) | NFR1(AC2.5.3)を、リリースのたびに、守っていることの確認 |
| `pipeline` の、ジョブ1つあたりのオーバーヘッド | 同上 | p95 50ms 以内(警告として記録) | `pipeline` の、ジョブ数に対する費用 |
| ジョブ 1,000 個の定義での起動の時間 | 同上 | 1 秒以内(仮。警告) | 容量(NFR1.5) |
| 行カバレッジ | CI の `coverage` ジョブ(Linux) | 80% 以上(超えない場合は失敗) | NFR5.1 |
| 稼働中のメトリクス(リクエスト数、遅延、エラー率、CPU、メモリなど) | なし(該当しない) | — | `conductor` は、常駐せず、1回で終わる |

## Alerts

| Alert | Condition | Severity | Routes to |
|-------|-----------|----------|-----------|
| CI の失敗(`main` への取り込み前) | `lint`、`build-check`、`test`、`coverage` のいずれかが失敗 | マージを止める | GitHub のプルリクエストのチェック(必須のチェック) |
| リリースのワークフローの失敗 | `release.yml` のジョブが失敗 | リリースを止める(リリースが作られない) | GitHub の通知(リポジトリの、Actions の失敗の通知。担当者のメール) |
| 起動時間の基準超え(警告) | `startup-bench` が基準を超える | 警告(失敗にはしない。配分の確定のあと、失敗にする) | ジョブの要約欄(担当者が見る) |
| 稼働中のアラート(死活、遅延、エラー率) | 該当しない | — | — |

## SLIs / SLOs

| SLI | SLO target | Measurement window |
|-----|------------|--------------------|
| なし(該当しない) | — | — |

- 稼働率や遅延の SLO は、定めない(常駐しない、1回で終わるコマンドのため)。性能の目標は、`conductor` 自身のオーバーヘッド(NFR1)で、CI が、リリースごとに確かめる(上の Metrics)。

## Logs & Tracing

- **ログ:** `conductor` は、独自のログファイルを作らない。診断は、標準エラーの固定の文(`conductor: error: ...`、`-v` の `[conductor] ...`)と、終了コードである。**利用者の、既存のスケジューラのログが、これらを拾う**(ジョブ本体の出力と合わせて、1つのログに残る)。
- **トレース:** なし(分散の呼び出しがない)。
- **ダッシュボード:** なし。CI の結果は、GitHub の Actions の画面で見る。実行状態の確認は、`conductor watch`。
- **利用状況の収集(テレメトリ):** しない(NFR4.5)。

## Assumptions & Open Questions

- 起動時間の基準の確定(配分の確定のあと、CI を失敗にする)は、Code Generation の実測のあとに行う。 [assumption]
- リリースのワークフローの失敗の通知は、GitHub の既定の通知(リポジトリの管理者のメール)に任せる。専用の通知先(チャットなど)は、設けない。 [assumption]
