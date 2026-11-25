# 論理の部品: u7-watch-follow

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C5)。

| 部品 | 場所 | 変化 |
|---|---|---|
| CoreFlow | `crates/core/src/flow/activity.rs`(新規)、`flow/mod.rs`、`lib.rs` | `PipelineActivity`、`pipeline_activity` |
| CliArgs | `crates/cli/src/args.rs`、`types.rs` | `--follow`、`--interval` |
| CliFlows | `crates/cli/src/aux_commands.rs` | 継続表示の繰り返し |
| CliFlows(U6 の直し) | `crates/cli/src/step.rs` | リトライの待ちの間は試行を開いたまま |
| CliPlatform | `crates/cli/src/platform/mod.rs` と OS ごとのファイル | 画面を消せるかの判定 |
| CliReport | `crates/cli/src/report/mod.rs`、`messages.rs` | 画面を消す文字列と区切り、使い方の文 |
| 文書とデモ | `README.md`、`demo/README.md`、デモのスクリプト | 継続表示の説明と場面 |
| 版 | `crates/cli/Cargo.toml`(と `Cargo.lock` の自分のクレートの版) | 0.10.0 |

## 失敗の影響の範囲

- 継続表示はステートを読むだけなので、失敗しても実行中のパイプラインに影響しない。
- U6 の流れの直しは、書く時刻を変えずに書く時だけを変える。

## 共有の資源

- `watch` の表と所要時間(U6)、待ちと合図(既存)を共有する。
