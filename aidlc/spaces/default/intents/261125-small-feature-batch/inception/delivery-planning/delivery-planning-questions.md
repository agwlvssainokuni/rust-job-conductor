# 実装の計画: 質問

作る順序(B1 → B7 を 1 つずつ。スコープ Q4、チームの実践)、Bolt(1 回の作る周回で、終わると動くものができる単位)の大きさ(1 つのユニットで 1 つの Bolt)、直列に作ること、薄い縦切りを先に作らないこと、外部の待ち(無い)は、これまでの段階で決まっている。ここでは、まだ決めていない 2 点だけを確かめる。

## Q1. [リリース担当] いちばん気がかりなところ(当てはまるものをすべて選んでください)

早めに確かめたい、気がかりな部分はどれですか。答えは、各 Bolt の「何を確かめるか」と、リスクの記録に使います(作る順序は変えません)。

A. B5 の出力の複製(子の出力をパイプで受けて端末とファイルへ書く。3 つの OS で同じに動くか)
B. B6 のステートの形の変更(古いステートを 65 で止める。今のテストへの影響)
C. B7 の継続表示(端末の描き直し、Ctrl-C とパイプラインの終わりでの止まり方)
D. None(特に無い)
X. Other (please specify)

[Answer]: A, B, C

## Q2. [品質] 各 Bolt の完了の条件

各 Bolt を「完了」とする条件を決めます。

A. 機能のコードとテスト(test-after)がそろい、ローカルの macOS で `cargo test`・`cargo fmt --check`・`cargo clippy -D warnings` が通り、README とデモを更新し、`main` に push した後の CI(3 つの OS、MSRV、行カバレッジ 80%)が通ったら完了
B. A のうち、CI の確認は最後の Bolt(版の更新)の後にまとめて行う
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 気がかり(Q1): B5 の出力の複製(3 つの OS)、B6 のステートの形の変更、B7 の継続表示の 3 つ。各 Bolt で確かめることとリスクの記録に書く。作る順序は変えない
- 完了の条件(Q2): Bolt ごとに、コードとテスト、ローカルの macOS での test・fmt・clippy、README とデモの更新、push 後の CI(3 つの OS、MSRV、行カバレッジ 80%)まで通ったら完了
- これまでに決まっていること: B1 → B7 を 1 つずつ直列に作る。1 つのユニットで 1 つの Bolt。薄い縦切りを先に作らない。外部の待ちは無い。作るのは開発者ご自身と AI(aidlc-developer-agent)

- Looks correct
- Request changes

[Answer]: Looks correct
