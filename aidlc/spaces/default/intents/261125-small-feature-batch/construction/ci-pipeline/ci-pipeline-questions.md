# CI Pipeline の質問: v0.10.0

出典: team.md の Way of Working・Testing Posture・Deployment・Code Style(practices-discovery で人が確かめた内容)、project.md の Mandated、`.github/workflows/ci.yml`・`release.yml`・`audit.yml`。

どの問いも、チームの確かめた慣行と今の CI の設定で答えが決まっているため、人に新しく聞く問いは無い。答えと出典を残す。

## Q1. CI の道具は何か

A. GitHub Actions
B. CodePipeline / CodeBuild
C. Jenkins
X. Other (please specify)

[Answer]: A(`.github/workflows/ci.yml`。team.md の Testing Posture)

## Q2. ブランチの使い方は何か

A. `main` に直接コミットし、push の CI で事後に関門を効かせる(PR を使わない)
B. PR を経てから `main` に入れる
X. Other (please specify)

[Answer]: A(team.md の Way of Working。この版でも人が `main` を push し、CI を見た)

## Q3. マージの前の関門は何か

A. 今の `ci.yml` の関門(`lint`、`build-check`、`test` の 3 つの OS、`msrv-test`、`coverage` の 80%、`docs-check`)。`startup-bench` と `audit` は警告のみ
B. 関門を足す・強める
X. Other (please specify)

[Answer]: A(team.md の Testing Posture・Code Style・Deployment。署名・公証、`audit` と `startup-bench` の強制は v0.10.0 でも見送る)

## Q4. 成果物の置き場所は何か

A. GitHub の Releases(タグ `v*` で `release.yml` が下書きを作り、人が確かめて公開する)
B. パッケージのレジストリ(crates.io など)
X. Other (please specify)

[Answer]: A(team.md の Deployment、project.md の Mandated)
