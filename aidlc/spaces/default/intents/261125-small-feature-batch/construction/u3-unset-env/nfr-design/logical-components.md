# 論理の部品: u3-unset-env

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C1)。

| 部品 | 場所 | 変化 |
|---|---|---|
| CoreTypes | `crates/core/src/types.rs` | `unset_env` |
| CoreDefinition | `crates/core/src/definition/walk/env.rs`、`mod.rs`、`error.rs` | 読み取り、名前の規則、`env` との重なり(`UnsetEnvConflict`) |
| CoreEnvFile | `crates/core/src/envfile/mod.rs`、`merge.rs`、`parse.rs` | `removals`、`EnvEntry.line`・`EnvFileContent.file`、`check_unset_conflicts`、`UnsetConflict` |
| RuntimeRunner | `crates/runtime/src/runner.rs` | 消す名前を受け取り、`env_remove` してから足す |
| CliDeps / CliFlows | `crates/cli/src/deps.rs`、`ctx.rs`、`step.rs` | `run_job` に消す名前、`.env` の後に重なりの検査 |
| CliReport | `crates/cli/src/report/plan_text.rs`、`messages.rs` | `--dry-run` の消す名前 |
| 文書とデモ | `README.md`、`demo/env.toml`、`demo/windows/env.toml`、`demo/README.md` | `unset_env` の説明と例 |

## 失敗の影響の範囲

- 重なりの誤りは実行の前に 64 で止まり、ジョブを動かさない。

## 共有の資源

- 環境の統合(C1)を U2 と共有する。足す変数の形は変えない。
