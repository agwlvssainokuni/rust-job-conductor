# 配り方: v0.10.0

出典: `deployment-pipeline-questions.md`(Q1〜Q4、まとめの確かめ済み)、`cd-config.md`、team.md の Deployment・Way of Working、project.md の Mandated。

## やり方

- **一度に全部(下書きを経る)**: 単一バイナリの CLI で、利用者が自分で取ってくるため、段階的な切り替え(blue/green、canary)は当てはまらない。代わりに「下書きのリリース」が人の確かめの段になる。
- 環境は 1 つ(利用者の手元)。ステージングは無い(team.md)。

## 手順(人が行う)

1. `main` が最新で、CI がすべて通っていることを確かめる(45726504409 で確認済み)。
2. `main` の上にタグ `v0.10.0` を付けて push する。
3. `release.yml` の 4 つのジョブが通るのを見る。
4. 下書きのリリースの成果物、`SHA256SUMS`、自動の注記を確かめ、必要なら v0.10.0 の変更(B1〜B7)を注記に足す。
5. 公開する。

## smoke test

- `build` のジョブが、展開したバイナリの `--version` を 3 対象で確かめる(Intel macOS はクロスビルドのため除き、注記に書く)。
- 公開の後、利用者の手元では `conductor --version` が `conductor 0.10.0` を出すことで確かめられる。

## 中止の条件

- `verify-tag` か `checks` か `build` が落ちたら、下書きは作られない。原因を直してから、タグを消して付け直す(`rollback-runbook.md`)。
