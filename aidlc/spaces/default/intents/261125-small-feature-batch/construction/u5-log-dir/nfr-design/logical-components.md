# 論理の部品: u5-log-dir

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C2、C6)。

| 部品 | 場所 | 変化 | 当てはまる非機能の設計 |
|---|---|---|---|
| CoreTime(新規) | `crates/core/src/time.rs` | Timestamp、整形・詰めた形・読み取り・差 | 信頼性(境界の表) |
| CoreTypes・CoreDefinition | `crates/core/src/types.rs`、`definition/walk/mod.rs` | `log_dir` | セキュリティ(入力の検証) |
| RuntimeClock(新規) | `crates/runtime/src/clock.rs` | 現在の時刻 | 信頼性 |
| RuntimeLog(新規) | `crates/runtime/src/log.rs`、`platform/unix.rs`・`platform/windows.rs` | 安全な名前、衝突、ディレクトリ、0600 | セキュリティ、規模 |
| RuntimeRunner | `crates/runtime/src/runner.rs` | ログの指定、出力の複製、待ち切り、LogOutcome | 性能、信頼性、観測 |
| CliDeps | `crates/cli/src/deps.rs` | `now()`、`run_job` のログの指定と戻り値 | 信頼性(差し替え) |
| CliFlows | `crates/cli/src/step.rs` ほか | 試行ごとの時刻とログの指定、127 の表示、警告 | 観測 |
| CliReport | `crates/cli/src/report/mod.rs`、`messages.rs` | 3 つの文 | 観測 |
| 文書とデモ | `README.md`、`demo/` | `log_dir` の説明と場面 | 信頼性 |

## 失敗の影響の範囲

- ログを作れないとき、その試行だけが 127。書けないときは警告だけで、ジョブの結果は変わらない。待ち切れのときは、その試行の残りの出力だけが失われる。

## 共有の資源

- `log_dir` のディレクトリは、同じ定義の別の run ID の実行と共有しうる。「無いときだけ作る」と番号で、互いのファイルを壊さない。
- CoreTime と `Runtime::now()` は U6 と共有する(契約 C2)。
