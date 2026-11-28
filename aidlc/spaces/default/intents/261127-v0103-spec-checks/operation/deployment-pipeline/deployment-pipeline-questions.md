# Deployment Pipeline の質問: v0.10.3

出典:
- team.md の Deployment・Way of Working(practices-discovery で人が確かめた内容)
- project.md の Mandated・Forbidden
- `.github/workflows/release.yml`(v0.10.2 から変わっていない)
- 要件(`requirements.md` の FR4.1、NFR1)
- v0.10.2 の Deployment Pipeline の記録(`261127-v0102-spec-docs/operation/deployment-pipeline/`)

どの問いも、次の 3 つで答えが決まる。答えと出典を残し、まとめを人に確かめてもらう。
- チームの確かめた慣行
- 今の `release.yml`
- 承認済みの要件

## Q1. 配る先(環境)はどこか

A. 1 つ(利用者の手元)。GitHub の Releases にバイナリを置く。ステージングは無い
B. ステージングと本番の 2 つ以上
X. Other (please specify)

[Answer]: A(team.md の Deployment)

## Q2. 配る手順の起動は何か

A. `main` の上にタグ `v0.10.3` を付けて push すると、`release.yml` が動く(v0.10.1・v0.10.2 では人の頼みで AI が行った)
B. `main` への push のたびに自動で配る
X. Other (please specify)

[Answer]: A(team.md の Way of Working・Deployment、project.md の Mandated)

## Q3. 公開の関門は何か

A. 次の順に進み、公開は人が確かめてから行う
   1. `verify-tag`: main の上にあり、版 0.10.3 と一致する
   2. `checks`: `ci.yml` を再利用する
   3. `build`: 4 対象をビルドし、smoke test を行う
   4. 下書きのリリース
B. 自動で公開する
X. Other (please specify)

[Answer]: A(team.md の Deployment、project.md の Mandated)

## Q4. 戻し方は何か

A. 次のとおりにする
   - 公開前: 下書きとタグを消して付け直す。
   - 公開後: v0.10.2 のリリースが残るので、それに戻せる。
   - v0.10.3 はコードの動きを変えていない(仕様書、team.md、テスト、版だけ)。そのため、v0.10.2 との間でステートと定義はそのまま使える。
B. 自動の戻し
X. Other (please specify)

[Answer]: A(team.md の Deployment、要件の NFR1)

## Q5. リリースの注記に何を書くか

A. 自動の注記に加えて、次の 2 つを短く書く
   - コードの動きは v0.10.2 と同じであること。
   - 仕様書の小さな粗を直し、仕様書の数を実装と突き合わせるテストを足したこと。
B. 自動の注記だけにする
X. Other (please specify)

[Answer]: A(v0.10.0〜v0.10.2 の慣行)

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 配る先は利用者の手元の 1 つ。GitHub の Releases にバイナリを置く(Q1)。
- `main` の上にタグ `v0.10.3` を付けて push すると `release.yml` が動く。`release.yml` は変えない(Q2)。
- 進む順は次のとおり。公開は人が確かめてから行う(Q3)。
  1. 版の検査
  2. CI の再実行
  3. 4 対象のビルドと smoke test
  4. 下書きのリリース
- 戻し方は次のとおり。動きは同じなので、v0.10.2 との間で行き来できる(Q4)。
  - 公開前: 下書きとタグを消して付け直す。
  - 公開後: v0.10.2 のバイナリに戻す。
- リリースの注記には、次の 2 つを短く書く(Q5)。
  - 動きは v0.10.2 と同じであること。
  - 仕様書の粗の直しと、数の検査のテストを足したこと。

- Looks correct
- Request changes

[Answer]: Looks correct
