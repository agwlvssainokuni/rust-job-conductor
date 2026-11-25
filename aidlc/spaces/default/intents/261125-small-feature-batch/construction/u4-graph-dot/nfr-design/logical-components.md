# 論理の部品: u4-graph-dot

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C5)。

| 部品 | 場所 | 変化 | 当てはまる非機能の設計 |
|---|---|---|---|
| CliArgs | `crates/cli/src/args.rs`、`types.rs` | `GraphFormat::Dot`、`--format` の値と誤りの文 | 信頼性(使い方の表) |
| CliReport(DOT) | `crates/cli/src/report/graph_dot.rs`(新規)、`report/mod.rs` | `render_dot` と `escape_dot_label` | セキュリティ(escape)、性能(比例する手間) |
| CliFlows | `crates/cli/src/aux_commands.rs` | 描画の分岐に `Dot` を足す | 観測(同じ経路の誤り) |
| CliReport(文言) | `crates/cli/src/report/messages.rs` | 使い方の文に `dot` | 信頼性 |
| 文書とデモ | `README.md`、`demo/run-demo.sh`、`demo/windows/run-demo.cmd`、`demo/README.md` | `--format dot` の説明と場面 | 信頼性(デモのテスト) |

## 失敗の影響の範囲

- 描画は純粋で失敗しない。影響は DOT の出力だけで、定義やステートを書かない。

## 共有の資源

- `graph_text.rs` の補助(`state_symbol`、`kind_label`、`is_main_edge`)を text・mermaid と共有する。振る舞いは変えない。
