# 規模の設計: u5-log-dir

出典: 規模の要件(`scalability-requirements.md` の NFR1.6、NFR1.7)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C6)。

## 設計

- ログのファイルは試行ごとに 1 つ。conductor は消さない(NFR1.6)。README に、利用者が消すこと(例: 古いファイルを消すコマンド)を書く。
- 名前の衝突は `-2` から `-1000` まで試す。それでも作れなければ、作れないとして 127(NFR1.7)。上限の定数は 1 か所に置く。
- 確かめ方: 衝突の番号の単体テスト(同じ名前を 1 つ、2 つ置いたとき)と、上限を超えたときの扱いのテスト(上限の定数を使い、全部を置くのではなく、名前を作る関数の境界を確かめる)。
