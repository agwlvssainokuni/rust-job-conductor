# 論理の部品: u6-attempt-times

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C4)。

| 部品 | 場所 | 変化 |
|---|---|---|
| CoreTypes | `crates/core/src/types.rs` | `Attempt`、`JobState.attempts` |
| CoreFlow | `crates/core/src/flow/attempt.rs`(新規) | `start_attempt`、`end_attempt`、`cancel_attempt` |
| CoreTime | `crates/core/src/time.rs` | `job_duration(job, now)`、`format_duration(秒)` |
| RuntimeState | `crates/runtime/src/state/reader.rs`、`writer.rs`、`crates/runtime/src/error.rs` | `[[job.attempt]]` の読み書き、3 つの理由 |
| CliFlows | `crates/cli/src/step.rs`、`aux_commands.rs` | 試行ごとの書き込み、`watch` に今の時刻 |
| CliReport | `crates/cli/src/report/watch.rs`、`messages.rs` | 所要時間の列 |
| 文書とデモ | `README.md`、`demo/README.md`、デモの `watch` の場面 | ステートの試行、所要時間、強制終了の後の経過時間、古いステートの 65 |

## 失敗の影響の範囲

- 開始の書き込みに失敗すれば、ジョブを起動せず 65(今の初期化の失敗と同じ)。
- 古い形・壊れたステートは、読む前に 65 で止まる。

## 共有の資源

- 時刻の整形と読み取り、現在の時刻は U5 と共有する。所要時間の関数は U7 も使う。
