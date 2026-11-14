# Constraint Register

上流: intent-statement(特に Initiative Trigger と Initial Scope Signal)。

| ID | Type | Constraint | Source |
|----|------|------------|--------|
| C1 | Technical | 既存のジョブ実行環境そのものは変更せず、上位レイヤーで制約を補う。 | intent-statement の Initiative Trigger |
| C2 | Technical | スケジューラが受け付ける終了コードは 0 / 非0 の2値のみである。 | [Q2] |
| C3 | Technical | Linux・macOS・Windows のすべてで動作すること。 | [Q1] |
| C4 | Technical | 実装言語は Rust で決定済み。 | [Q3] |
| C5 | Organizational | 開発者は Rust に不慣れである。 | [Q3] |
| C6 | Scope | 通知機能そのものの組み込み、複数ホストにまたがる分散実行・ステート共有、Web UI / GUI は初版の対象外。 | intent-statement の Initial Scope Signal |
| C7 | Organizational | 期限・予算の制約はない。 | [Q4] |
| C8 | Organizational | 進行を妨げる組織上の事情はない。 | [Q7] |
| C9 | Regulatory | 規制・機密データに関する要件はない。 | [Q5] |

## Assumptions & Open Questions

None.
