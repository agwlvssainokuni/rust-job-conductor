# Team Allocation — 正常終了しなかったジョブの自動リトライ

ここでは、Bolt(作業の一部を 1 回通して作り、動くものとして終える単位)を、だれが作るかを示す。

チーム編成の段階は、この作業では省略した(小さな機能の追加で、作る人が決まっているため)。そのため、すべての Bolt を、AI の開発担当(`aidlc-developer-agent`)が作る。(`delivery-planning-questions.md` の前提、`bolt-plan.md`)

| Bolt | Unit | 担当 | 進め方 |
|---|---|---|---|
| B1 | u1-retry-config | AI の開発担当 | 1 つずつ順に |
| B2 | u2-retry-execution | AI の開発担当 | 1 つずつ順に |
| B3 | u3-retry-display-docs | AI の開発担当 | 1 つずつ順に |

- 作者(人)は、Plan Approval(コードの計画の承認)、各 Unit の完了の承認、設計の承認を行う。(`scope-document.md`: 決める人は作者ひとり)
- 同時に進める Bolt はない。(Units Generation の Q3: A)
- 組(mob)は作らない。担当は 1 つで、役割の分担を決める必要がない。
