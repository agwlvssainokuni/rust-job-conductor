# Delivery Planning — 質問

このステージでは、Construction(実装)で、4つの Unit をどの順に作るかを決めます。ここで「Bolt」とは、作業の1回のまとまり(1つ以上の Unit を作り、動くものに到達するまでの1回の作業)のことです。Unit の依存(u2-runtime は u1-core に、u3-cli はその他すべてに依存する)は、すでに決まっています。ここで決めるのは、依存の許す範囲での、順序と進め方です。

チームの実践(practices-discovery)では、薄いエンドツーエンドの縦切り(ウォーキングスケルトン)は作らず、機能を1つずつ順に作ること、1人で進めること、テストは実装のあとに書くことが決まっています。

## Q1. [プロダクト] 何から作るか(順序)

依存の上では、u1-core と u4-converter は、どちらも先に作れます。u2-runtime は u1-core のあと、u3-cli は最後です。

- A. リスク優先: u1-core → u2-runtime → u4-converter → u3-cli。OS の違いや Rust への不慣れが集中する u2-runtime を、早めに確かめる
- B. 小さく慣れてから: u1-core → u4-converter → u2-runtime → u3-cli。小さい u4-converter で Rust と進め方に慣れてから、難しい u2-runtime に進む(推奨)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q2. [運用] 外部の事情による待ち

作業を止めうる、チームの外の事情(外部の API、データ、承認、他チームの引き渡しなど)はありますか。

- A. 特になし。GitHub Actions(CI)の設定だけ、自分で行う
- B. ある(内容は Other に記入)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 一番心配なこと(複数選択可 / select all that apply)

早めに取り組みたい心配事を選んでください。

- A. 複数の OS(特に Windows)での、プロセス起動とシグナルの扱いの違い
- B. Rust に不慣れな開発者が、実装を進められるか
- C. ステートの書き込みの安全性(原子的な更新、破損の検知)
- D. 起動オーバーヘッド(100ms 以内)を3つの OS で満たせるか
- E. 特になし
- X. Other (please specify)

[Answer]: A, C

## Q4. [開発] Unit ごとの確認コマンドを、今決めるか

Construction では、Unit ごとの完了のたびに、実際のプロジェクトの確認(テストなど)を、同じコマンドで実行します。この確認コマンドは、プロジェクトのひな形ができてから決めるほうが、正確になります(現時点では、Rust のプロジェクトは作られていません)。

- A. 最初の Unit のチェックポイントで、確認コマンドを提案して決める(今は決めない)(推奨)
- B. 今、仮のコマンド(例: `cargo test` に、`cargo fmt --check` と `cargo clippy` を加えたもの)を決める
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
