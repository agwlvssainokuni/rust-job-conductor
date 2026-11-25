# フェーズの確認: Inception → Construction

**判定: 合格(Pass)**。未解決の GAP・ORPHAN・不正な target・抜けた要件の ID は無い。

ユーザーストーリーは省いたため、`user-stories/traceability.json` は無い。要件の ID(FR1.1〜FR8.2 の 41 件)で確かめた。契約の設計は要件の網羅を持たないため、対象にしない。

| 段階 | 上流の ID | OK | Deferred | N/A | GAP | 抜け |
|---|---|---|---|---|---|---|
| domain-design | 41 | 39 | 1(FR7.5 → functional-design) | 1(FR8.2: 版の更新は部品の責務でない) | 0 | 0 |
| units-generation | 41 | 41 | 0 | 0 | 0 | 0 |

## 確かめたこと

- 要件の 41 件は、すべて部品(または機能の設計への送り)と、ユニットに割り当てられている。
- domain-design の Deferred(FR7.5)と N/A(FR8.2)は、units-generation でそれぞれ U7 に割り当てられている。
- Bolt の並び(Bolt 1〜7)は、ユニットの依存の形の順序に反しない(`bolt-plan.md`)。

## 引き継ぐ未決の事項(機能の設計で決める)

- ログのファイル名に使うジョブの ID の文字、同じ名前の扱い、権限、作れないときの扱い(u5)。
- 完了したジョブの最後の試行の `ended_at` が必須か、動いている最中と中断の見分け方(u6)。
- DOT での状態の表し方(u4)。
- `--interval` の範囲、`--follow` の終了コード、ステートが無いときの扱い(u7)。
