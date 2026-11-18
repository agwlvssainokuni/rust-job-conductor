# Delivery Planning — 確認事項

ここで決めるのは、3 つの作業単位(Unit)を、どんなまとまりで、どの順に作るかです。まとまりを「Bolt」と呼びます。Bolt は、作業の一部を 1 回通して作り、動くものとして終える単位です。

すでに決まっていること(質問しません):

- 依存は一列(u1-retry-config → u2-retry-execution → u3-retry-display-docs)なので、順番は u1、u2、u3 に決まります。(`unit-of-work-dependency.md`)
- 1 つずつ順に作ります。(Units Generation の Q3: A、`team.md` の「機能を 1 つずつ順に作る」)
- 最初に、薄い端から端までの動くもの(ウォーキングスケルトン)を作ることはしません。(`team.md`)
- 作るのは、AI の開発担当です(チーム編成の段階は省略しました)。

## Q1. [開発者] 1 つの Bolt の大きさは、どうしますか?

- A. Unit 1 つを、Bolt 1 つにする(Bolt は 3 つ)
- B. u1 と u2 をまとめて 1 つにする(Bolt は 2 つ。u3 だけ別)
- C. 全部を 1 つの Bolt にする
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q2. [開発者] この作業で、いちばん心配なことは何ですか?(早い Bolt で確かめておきたいこと)

- A. 間隔の待機中のシグナルの扱いと、時間の差し込み口(u2 の難所)
- B. 定義の検証の取りこぼし(省略、境界値、重なり)(u1)
- C. 設定を書かない定義ファイルの動作が、変わってしまうこと(後方互換)
- D. 特にない
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] この作業を止めうる、チームの外の待ちはありますか?(外部の API、データ、承認、ほかのチームの手渡し)

- A. ない
- B. ある(Other で、何を、いつまでか)
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

- Looks correct
- Request changes

[Answer]: Looks correct
