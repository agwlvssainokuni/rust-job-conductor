# Deployment Execution の質問: v0.10.4

出典:
- `operation/deployment-pipeline/` の `deployment-strategy.md`、`cd-config.md`、`rollback-runbook.md`
- team.md の Deployment と Way of Working
- project.md の Mandated
- v0.10.3 の Deployment Execution の記録

今の `main` には、`origin/main` より先に記録のコミットがある。どれも `aidlc/` の中だけの変更。

`verify-tag` があるので、タグは `main` の上にある、push 済みのコミットに付ける必要がある。

## Q1. `main` の push と、タグ `v0.10.4` を付けて push することを、誰が行うか

A. 人が `main` を push する。CI が通るのを待って、人がタグを付けて push する
B. 人が `main` を push する。CI が通ったことを AI が確かめてから、人の頼みで AI がタグを付けて push する(v0.10.0〜v0.10.3 と同じ)
X. Other (please specify)

[Answer]: B

## Q2. 下書きの注記に変更点を書き足すか

A. 書き足す(中身は `deployment-strategy.md` の手順 4)。公開はしない
B. 書き足さない(自動の注記のまま)
X. Other (please specify)

[Answer]: A

## Q3. 公開するのは誰か

A. 人が下書きを確かめてから公開する
B. AI が公開する
X. Other (please specify)

[Answer]: A(project.md の Mandated、team.md の Deployment)

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 人が `main` を push する。AI は `main` の CI が通ったことを確かめてから、人の頼みでタグ `v0.10.4` を `main` の最新のコミットに付けて push する(Q1 B)。
- AI は `release.yml` の 4 つのジョブの成功と成果物(4 つと `SHA256SUMS`)を確かめ、下書きの注記に「動きは v0.10.3 と同じ。ソースのファイルを分けた」ことを書き足す。公開はしない(Q2 A)。
- 公開は人が下書きを確かめてから行う(Q3 A)。

- Looks correct
- Request changes

[Answer]: Looks correct
