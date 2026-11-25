# 規模の設計: u3-unset-env

出典: 規模の要件(`scalability-requirements.md` の NFR1.4)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C1)。

## 設計

- 名前の数の上限は設けず、定義の大きさの上限に従う。
- `.env` との重なりの誤りは「ファイル名と行」で重複を除き、今の `.env` の誤りの表示の上限(20 件と残りの件数)に従う(`reliability-design.md` の 1)。
- 確かめ方: 同じ `.env` を 2 つのジョブが読む定義で、`pipeline` の誤りが 1 つだけになることを cli の流れのテストで確かめる。
