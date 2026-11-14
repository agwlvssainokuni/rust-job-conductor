# Team Allocation(Bolt の担当の割り当て)

上流: bolt-plan、unit-of-work、team-practices。**Bolt** とは、Construction での作業の1回のまとまり(1つ以上の Unit を作り、動くものに到達するまで)である。**mob** とは、1つの Bolt を一緒に作る担当の組を指す。

## 前提

- チーム編成のステージは、この計画(mvp)に含まれていない。そのため、すべての Bolt を、開発担当の AI(aidlc-developer-agent)に割り当てる。 delivery-planning のステージ定義
- 依頼者本人が、スコープ・優先度の唯一の決定者であり、各ステージの承認を行う。 stakeholder-map、[Q5 of intent-capture]
- 1人で、直列に進める。同時に複数の Bolt を作ることはしない。 team-practices

## 割り当て

| Bolt | 含む Unit | 担当の mob | 承認 |
|------|-----------|------------|------|
| B1 | u1-core | 開発担当の AI(aidlc-developer-agent) | 依頼者 |
| B2 | u4-converter | 開発担当の AI(aidlc-developer-agent) | 依頼者 |
| B3 | u2-runtime | 開発担当の AI(aidlc-developer-agent) | 依頼者 |
| B4 | u3-cli | 開発担当の AI(aidlc-developer-agent) | 依頼者 |

- チームの数は1つであり、複数チームの調整(Program Board)は不要である。
- 設計・テスト・セキュリティなどの、各ステージの専門の観点は、そのステージを進める担当が持つ(Construction のステージ側の取り決め)。

## 開発者の事情

- 開発者(依頼者)は Rust に不慣れである。AI が実装を進めるが、依頼者が、各 Unit の設計と、チェックポイントで確認する。変更は小さく保ち、小さな単位でレビューする。 team-practices

## Assumptions & Open Questions

None.
