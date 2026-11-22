# 関係者の地図(Stakeholder Map): args.rs のオプションの表化

## Sources

- [desc] Initial description: "args.rs のオプションの読み取りを表にまとめる refactor(誤りの報告の順を保つ。convert --to の誤りが入力の値を含む件と、enter_flow の到達しない分岐も扱う)"
- [Q1] `intent-capture-questions.md` Q1: B
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | オプションを足しやすくなること。誤りの報告の順が変わらないこと | 決める人(範囲、`--to` の扱い) | [desc] [Q1] [Q2] [Q3] |
| `conductor convert` を使う人 | `--to` の誤りの対象が値からオプションの名前に変わる | 影響する人 | [Q1] |
| 仕様書を読んで実装する人 | `--to` の誤りの対象の記述が新しい動きと合うこと | 影響する人 | [Q1] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 利用者から見える変更(`--to` の誤りの対象)を、版の更新の候補として記録する | [Q1] |

## Assumptions & Open Questions

None.
