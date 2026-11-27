# Deployment Pipeline の質問: v0.10.2

出典: team.md の Deployment・Way of Working(practices-discovery で人が確かめた内容)、project.md の Mandated・Forbidden、`.github/workflows/release.yml`(v0.10.1 から変わっていない)、要件(`requirements.md` の FR5.1、NFR1)、v0.10.1 の Deployment Pipeline の記録(`261126-v0101-followups/operation/deployment-pipeline/`)。

どの問いも、チームの確かめた慣行、今の `release.yml`、承認済みの要件で答えが決まる。答えと出典を残し、まとめを人に確かめてもらう。

## Q1. 配る先(環境)はどこか

A. 1 つ(利用者の手元)。GitHub の Releases にバイナリを置く。ステージングは無い
B. ステージングと本番の 2 つ以上
X. Other (please specify)

[Answer]: A(team.md の Deployment)

## Q2. 配る手順の起動は何か

A. `main` の上にタグ `v0.10.2` を付けて push すると、`release.yml` が動く(v0.10.0・v0.10.1 では人の頼みで AI が行った)
B. `main` への push のたびに自動で配る
X. Other (please specify)

[Answer]: A(team.md の Way of Working・Deployment、project.md の Mandated)

## Q3. 公開の関門は何か

A. `verify-tag`(main 上、版 0.10.2 と一致)→ `checks`(`ci.yml` の再利用)→ `build`(4 対象、smoke test)→ 下書きのリリース。公開は人が確かめてから
B. 自動で公開する
X. Other (please specify)

[Answer]: A(team.md の Deployment、project.md の Mandated)

## Q4. 戻し方は何か

A. 公開前は下書きとタグを消して付け直す。公開後は v0.10.1 のリリースが残るので戻せる。v0.10.2 はコードの動きを変えていない(文書と版だけ)ので、v0.10.1 との間でステートと定義はそのまま使える
B. 自動の戻し
X. Other (please specify)

[Answer]: A(team.md の Deployment、要件の NFR1)

## Q5. リリースの注記に何を書くか

A. 自動の注記に加えて、「コードの動きは v0.10.1 と同じ。仕様書(rust 版と language-agnostic 版)を v0.10.0〜v0.10.1 の実装に合わせた」ことを短く書く
B. 自動の注記だけにする
X. Other (please specify)

[Answer]: A(v0.10.0・v0.10.1 の慣行)

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 配る先は利用者の手元の 1 つ。GitHub の Releases にバイナリを置く(Q1)。
- `main` の上にタグ `v0.10.2` を付けて push すると `release.yml` が動く。`release.yml` は変えない(Q2)。
- 版の検査、CI の再実行、4 対象のビルドと smoke test、下書きのリリースまで進み、公開は人が確かめてから(Q3)。
- 戻すときは、公開前は下書きとタグを消して付け直し、公開後は v0.10.1 のバイナリに戻す。動きは同じなので行き来できる(Q4)。
- リリースの注記には、動きは v0.10.1 と同じで、仕様書を実装に合わせたことを短く書く(Q5)。

- Looks correct
- Request changes

[Answer]: Looks correct
