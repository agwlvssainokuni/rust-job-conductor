# Deployment Pipeline の質問: v0.10.4

出典:
- team.md の Deployment と Way of Working(practices-discovery で人が確かめた内容)
- project.md の Mandated と Forbidden
- `.github/workflows/release.yml`(v0.10.3 から変わっていない)
- 要件(`requirements.md` の FR5.1、NFR1)
- v0.10.3 の Deployment Pipeline の記録

どの問いも、チームの確かめた慣行、今の `release.yml`、承認済みの要件で答えが決まる。答えと出典を残し、まとめを人に確かめてもらう。

## Q1. 配る先(環境)はどこか

A. 1 つ(利用者の手元)。GitHub の Releases にバイナリを置く。ステージングは無い
B. ステージングと本番の 2 つ以上
X. Other (please specify)

[Answer]: A(team.md の Deployment)

## Q2. 配る手順は何で始まるか

A. `main` の上にタグ `v0.10.4` を付けて push すると、`release.yml` が動く(v0.10.1〜v0.10.3 では、人の頼みで AI が行った)
B. `main` への push のたびに、自動で配る
X. Other (please specify)

[Answer]: A(team.md の Way of Working と Deployment、project.md の Mandated)

## Q3. 公開の前に通る関門は何か

A. 次の順に通り、公開は人が確かめてから行う
   1. `verify-tag`(main 上にあり、版 0.10.4 と一致する)
   2. `checks`(`ci.yml` を再利用する)
   3. `build`(4 対象と smoke test)
   4. 下書きのリリース
B. 自動で公開する
X. Other (please specify)

[Answer]: A(team.md の Deployment、project.md の Mandated)

## Q4. 戻し方は何か

A. 次のとおりにする
   - 公開の前は、下書きとタグを消して付け直す。
   - 公開の後は、v0.10.3 のリリースが残っているので、そちらに戻せる。
   - v0.10.4 はコードの動きを変えていない(ファイルの分け方、文書、版だけ)。そのため、v0.10.3 との間でステートと定義はそのまま使える。
B. 自動で戻す
X. Other (please specify)

[Answer]: A(team.md の Deployment、要件の NFR1)

## Q5. リリースの注記に何を書くか

A. 自動の注記に加えて、次の 2 つを短く書く
   - コードの動きは v0.10.3 と同じであること
   - 大きな 3 つのソースのファイルを、責務ごとに分けたこと
B. 自動の注記だけにする
X. Other (please specify)

[Answer]: A(v0.10.0〜v0.10.3 の慣行)

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 配る先は、利用者の手元の 1 つ。GitHub の Releases にバイナリを置く(Q1)。
- `main` の上にタグ `v0.10.4` を付けて push すると、`release.yml` が動く。`release.yml` は変えない(Q2)。
- 版の検査、CI の再実行、4 対象のビルドと smoke test、下書きのリリースの順に進む。公開は人が確かめてから行う(Q3)。
- 戻すとき、公開の前は下書きとタグを消して付け直す。公開の後は v0.10.3 のバイナリに戻す。動きは同じなので、行き来できる(Q4)。
- リリースの注記には、動きが v0.10.3 と同じであることと、ソースのファイルを分けたことを短く書く(Q5)。

- Looks correct
- Request changes

[Answer]: Looks correct
