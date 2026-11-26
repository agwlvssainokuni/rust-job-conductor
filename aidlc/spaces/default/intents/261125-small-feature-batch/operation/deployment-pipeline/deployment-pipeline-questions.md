# Deployment Pipeline の質問: v0.10.0

出典: team.md の Deployment・Way of Working(practices-discovery で人が確かめた内容)、project.md の Mandated・Forbidden、`.github/workflows/release.yml`、`construction/ci-pipeline/ci-config.md`。

どの問いも、チームの確かめた慣行と今の `release.yml` で答えが決まる。答えと出典を残し、まとめを人に確かめてもらう。

## Q1. 配る先(環境)はどこか

A. 1 つ(利用者の手元)。GitHub の Releases にバイナリを置き、利用者が取ってくる。ステージングの環境は無い
B. ステージングと本番の 2 つ以上
X. Other (please specify)

[Answer]: A(team.md の Deployment)

## Q2. 配る手順の起動は何か

A. `main` の上にタグ `v0.10.0` を人が付けて push すると、`release.yml` が動く
B. `main` への push のたびに自動で配る
X. Other (please specify)

[Answer]: A(team.md の Way of Working・Deployment、project.md の Mandated)

## Q3. 公開の関門は何か

A. `verify-tag`(main 上、`crates/cli/Cargo.toml` の版と一致)→ `checks`(`ci.yml` の再利用)→ `build`(4 対象、smoke test)→ 下書きのリリース。公開は人が確かめてから
B. 自動で公開する
X. Other (please specify)

[Answer]: A(team.md の Deployment、project.md の Mandated)

## Q4. 戻し方は何か

A. 公開前は下書きを消し、タグを消して付け直す。公開後は前の版(v0.9.9)のリリースが残るので、利用者は前の版のバイナリに戻せる。必要なら v0.10.1 を出す
B. 自動の戻し
X. Other (please specify)

[Answer]: A(team.md の Deployment。人の確認を経ていない推論とされていたため、このまとめで確かめる)

## Consolidated Summary Confirmation

まとめ: 配る先は利用者の手元の 1 つ。人が `main` の上にタグ `v0.10.0` を付けて push すると `release.yml` が動き、版の検査、CI の再実行、4 対象のビルドと smoke test、下書きのリリースまで進む。公開は人が確かめてから。戻すときは、公開前は下書きとタグを消して付け直し、公開後は前の版のバイナリに戻すか v0.10.1 を出す。

- Looks correct
- Request changes

[Answer]: Looks correct
