# Deployment Execution の質問: v0.10.2

出典: `operation/deployment-pipeline/deployment-strategy.md`・`cd-config.md`・`rollback-runbook.md`、team.md の Deployment・Way of Working、project.md の Mandated、v0.10.1 の Deployment Execution の記録。

いまの `main` は、`origin/main` より記録のコミットが先にある(`aidlc/` の中だけ)。タグは `main` の上(push 済みのコミット)に付ける必要がある(`verify-tag`)。

## Q1. `main` の push と、タグ `v0.10.2` の付与と push は誰が行うか

A. 人が `main` を push し、CI の成功を待って、人がタグを付けて push する
B. 人が `main` を push し、CI の成功を AI が確かめてから、人の頼みで AI がタグを付けて push する(v0.10.0・v0.10.1 と同じ)
X. Other (please specify)

[Answer]: B

## Q2. 下書きの注記に変更点を足すか

A. 足す(`deployment-strategy.md` の手順 4 の内容)。公開はしない
B. 足さない(自動の注記のまま)
X. Other (please specify)

[Answer]: A

## Q3. 公開するのは誰か

A. 人が下書きを確かめてから公開する
B. AI が公開する
X. Other (please specify)

[Answer]: A(project.md の Mandated、team.md の Deployment)

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 人が `main` を push する。AI が `main` の CI の成功を確かめてから、人の頼みで `main` の最新のコミットにタグ `v0.10.2` を付けて push する(Q1 B)。
- `release.yml` の 4 つのジョブの成功と成果物(4 つと `SHA256SUMS`)を確かめ、下書きの注記に「動きは v0.10.1 と同じ。仕様書を実装に合わせた」ことを書き足す。公開はしない(Q2 A)。
- 公開は人が下書きを確かめてから行う(Q3 A)。

- Looks correct
- Request changes

[Answer]: Looks correct
