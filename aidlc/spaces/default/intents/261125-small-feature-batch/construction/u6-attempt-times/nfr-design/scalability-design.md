# 規模の設計: u6-attempt-times

出典: 規模の要件(`scalability-requirements.md` の NFR1.4)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 4)、契約(`contract-summary.md` の C4)。

## 設計

- 試行 1 つは、見出し `[[job.attempt]]` と 2 つのキーの 3 行(約 90 バイト。非機能の要件のレビューの指摘で「2 行」を直す)。ステートの大きさの上限(1 MiB)は変えず、超えれば今の `TooLarge`(65)と `reset` の案内。
- 試行の数に上限は設けない。`reset` か、完了の後の削除(`remove_on_complete`)で消える。
- 確かめ方: 試行を 1,000 個持つジョブのステートを読み書きできること(runtime のテスト)。
