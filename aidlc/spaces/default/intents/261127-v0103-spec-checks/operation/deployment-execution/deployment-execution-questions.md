# Deployment Execution の質問: v0.10.3

出典:
- `operation/deployment-pipeline/` の `deployment-strategy.md`、`cd-config.md`、`rollback-runbook.md`
- team.md の Deployment・Way of Working
- project.md の Mandated
- v0.10.2 の Deployment Execution の記録

今の `main` には、`origin/main` にまだ無い記録のコミットがある。記録のコミットは `aidlc/` の中のものだけ。

`verify-tag` があるので、タグは `main` の上の、push 済みのコミットに付ける必要がある。

## Q1. `main` の push と、タグ `v0.10.3` を付けて push するのは誰か

A. 人が `main` を push し、CI の成功を待ってから、人がタグを付けて push する
B. 人が `main` を push する。AI が CI の成功を確かめてから、人の頼みでタグを付けて push する(v0.10.0〜v0.10.2 と同じ)
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

- 人が `main` を push する。AI は `main` の CI が通ったことを確かめてから、人の頼みで `main` の最新のコミットにタグ `v0.10.3` を付けて push する(Q1 B)。
- AI は次のことを行う。公開はしない(Q2 A)。
  - `release.yml` の 4 つのジョブが通ったことを確かめる。
  - 成果物(4 つのファイルと `SHA256SUMS`)がそろっていることを確かめる。
  - 下書きの注記に、次の 2 つを書き足す。
    - 動きは v0.10.2 と同じであること。
    - 仕様書の粗を直し、数の検査のテストを足したこと。
- 公開は、人が下書きを確かめてから行う(Q3 A)。

- Looks correct
- Request changes

[Answer]: Looks correct
