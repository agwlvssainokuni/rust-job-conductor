# Deployment Pipeline の質問: v0.10.1

出典: team.md の Deployment・Way of Working(practices-discovery で人が確かめた内容)、project.md の Mandated・Forbidden、`.github/workflows/release.yml`(v0.10.0 から変わっていない。`git diff v0.10.0..HEAD -- .github` が空)、要件(`requirements.md` の FR2.2、FR7.1)、v0.10.0 の Deployment Pipeline の記録(`261125-small-feature-batch/operation/deployment-pipeline/`)。

どの問いも、チームの確かめた慣行、今の `release.yml`、承認済みの要件で答えが決まる。答えと出典を残し、まとめを人に確かめてもらう。

## Q1. 配る先(環境)はどこか

A. 1 つ(利用者の手元)。GitHub の Releases にバイナリを置き、利用者が取ってくる。ステージングの環境は無い
B. ステージングと本番の 2 つ以上
X. Other (please specify)

[Answer]: A(team.md の Deployment)

## Q2. 配る手順の起動は何か

A. `main` の上にタグ `v0.10.1` を付けて push すると、`release.yml` が動く(タグを付けて push するのは人。v0.10.0 では人の頼みで AI が行った)
B. `main` への push のたびに自動で配る
X. Other (please specify)

[Answer]: A(team.md の Way of Working・Deployment、project.md の Mandated)

## Q3. 公開の関門は何か

A. `verify-tag`(main 上、`crates/cli/Cargo.toml` の版 0.10.1 と一致)→ `checks`(`ci.yml` の再利用)→ `build`(4 対象、smoke test)→ 下書きのリリース。公開は人が確かめてから
B. 自動で公開する
X. Other (please specify)

[Answer]: A(team.md の Deployment、project.md の Mandated)

## Q4. 戻し方は何か

A. 公開前は下書きを消し、タグを消して付け直す。公開後は前の版(v0.10.0)のリリースが残るので、利用者は前の版のバイナリに戻せる。必要なら v0.10.2 を出す。v0.10.1 はステートと定義の形を変えていないので、v0.10.0 との間で行き来できる
B. 自動の戻し
X. Other (please specify)

[Answer]: A(team.md の Deployment、v0.10.0 の `rollback-runbook.md`、要件の NFR1)

## Q5. リリースの注記に何を書くか

A. 自動の注記に加えて、F1〜F5 と文書のずれの直しを短く書く。F2 で、空白を除くと Windows の予約名になるジョブ ID(例: `con `)のログの名前が v0.10.0 と変わること(`con_` → `_con_`)を必ず書く
B. 自動の注記だけにする
X. Other (please specify)

[Answer]: A(要件の FR2.2「リリースの注記に書く」、v0.10.0 の慣行)

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 配る先は利用者の手元の 1 つ。GitHub の Releases にバイナリを置く(Q1)。
- `main` の上にタグ `v0.10.1` を付けて push すると `release.yml` が動く。`release.yml` は v0.10.0 から変えない(Q2)。
- 版の検査、CI の再実行、4 対象のビルドと smoke test、下書きのリリースまで進み、公開は人が確かめてから(Q3)。
- 戻すときは、公開前は下書きとタグを消して付け直し、公開後は v0.10.0 のバイナリに戻すか v0.10.2 を出す。ステートと定義の形は変えていないので行き来できる(Q4)。
- リリースの注記には、F1〜F5 と文書のずれを短く書き、F2 のログの名前の変化を必ず書く(Q5)。

- Looks correct
- Request changes

[Answer]: Looks correct
