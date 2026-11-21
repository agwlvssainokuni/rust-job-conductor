# 納品計画: 質問

この段階では、4 つのユニットを、どの順で、どの大きさで作るかを決めます。「Bolt」は、作りの 1 回のまとまり(作る範囲を 1 つ決め、動くところまで作って終える 1 周)のことです。

前提(すでに決まっていること):

- ユニットの依存: `u3-dry-run` は `u1-env-injection` と `u2-graph` に依存する。`u4-init` は `u1-env-injection` に依存する。
- チームの方針: 薄い縦切りを最初に作ることはしない。機能を 1 つずつ順に作る(team.md の Walking Skeleton)。ユーザーが決めた順は、u1(環境変数)→ u2(graph)→ u3(dry-run)→ u4(init)。
- 今の設定: Construction は、ユニットごとに順に(unit-major)、直列(serial)。

## Q1. [デリバリー] 何から作るか

どの考え方で、作る順を決めますか。

- A. 決まっている順(u1 → u2 → u3 → u4)をそのまま使う。依存の図に合い(依存する側が後)、チームの「機能を 1 つずつ順に」の方針にも合う。薄い縦切り(walking skeleton)は作らない
- B. リスクの高いものを先にする(例: レーンの配置が難しい u2 を先にする)。依存の図には合う
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [デリバリー] 点数で順位をつけるか

WSJF(価値と緊急度とリスクの低減を、大きさで割った点数で、順位をつける方法)のような、点数のモデルを使いますか。

- A. 使わない。順は Q1 で決め、理由を文章で書く(4 つだけで、依存の図でほぼ決まるため)
- B. 使う(点数と重みを、この段階で決める)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [デリバリー] 1 つの Bolt の大きさ

1 つの Bolt に、どこまで入れますか。

- A. 1 ユニット = 1 つの Bolt(計 4 つ)。各ユニットは、設計から、コード生成とテストまで、通して終える。大きさ(L、M、M、S)が、ばらつくが、小さな単位でレビューするチームの慣行に合う
- B. 小さいユニットを束ねる(例: u3 と u4 を 1 つにする)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [デリバリー] 同時に作るか

複数の Bolt を、同時に作りますか。

- A. 1 つずつ順に作る(直列)。4 つのユニットが、同じファイル(`CliArgs`、`CliReport`、`CliFlows`、メッセージのカタログ、`CoreTypes`)を編集するため、同時に作ると衝突する。チームは、解釈の人数も 1 人(開発者と AI)
- B. 依存のないユニット(u1 と u2)は、同時に作る
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q5. [デリバリー] 外部の待ちはあるか

この作業を止める、チームの外のものはありますか(外部の API、データ、承認、ほかのチームからの引き渡し)。

- A. ない。すべて、このリポジトリの中で完結する。外部のサービスは、CI(GitHub Actions)だけで、新しい依存も足さない
- B. ある(具体的に書く: 持ち主、待つ時間、止まる Bolt、遅れたときの対応)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q6. [デリバリー] いちばん心配なこと

この作りで、いちばん心配なのはどれですか。早い段階で手を打つ対象にします。

- A. u1 が定義の型を変えるため、既存の多くのテストに影響が出ること(u1 を最初に作り、型の変更を最初に片づける)
- B. u2 の `git log --graph` 風のレーンの配置が、複雑な流れで崩れること(期待する出力を、設計の段階で先に決める)
- C. u3 のコマンドの分割(`core` から `runtime::split` を呼べない)の置き場所が決まらないこと(機能の設計で、最初に決める)
- X. Other (please specify)

[Answer]: A

## Bolt ごとの質問(回答を受けて、成果物の中で整理する)

- どのユニットを、この Bolt に入れるか: Q3 の回答に従う。
- 薄い縦切り(walking skeleton)か: チームの方針により、どの Bolt も該当しない。
- 終わりの条件: 各ユニットの要件(FR)の受け入れ条件を満たし、行カバレッジ 80% 以上で、`cargo fmt`、`clippy`、テストが通ること。
- 出荷して分かること: Bolt ごとに、成果物の中で書く。
- 誰が作るか: 開発者と AI(`aidlc-developer-agent`)。チームの編成(1.5)は、この intent では省略した。

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifacts?

- Looks correct
- Request changes

[Answer]: Looks correct
