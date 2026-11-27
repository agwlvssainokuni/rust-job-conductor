# 配り方: v0.10.2

出典: `deployment-pipeline-questions.md`(Q1〜Q5、まとめの確かめ済み)、`cd-config.md`、team.md の Deployment・Way of Working、project.md の Mandated。

## やり方

- **一度に全部(下書きを経る)**: 単一バイナリの CLI で、利用者が自分で取ってくるため、段階的な切り替えは当てはまらない。「下書きのリリース」が人の確かめの段になる(Q3)。
- 環境は 1 つ(利用者の手元)(Q1)。

## 手順

1. `main` が最新で、CI がすべて通っていることを確かめる(46079873817 で確認済み。記録のコミットの後の CI も確かめる)。
2. `main` の上にタグ `v0.10.2` を付けて push する(人が行うか、人の頼みで AI が行う)。
3. `release.yml` の 4 つのジョブが通るのを見る。
4. 下書きのリリースの成果物、`SHA256SUMS`、自動の注記を確かめ、v0.10.2 の注記を足す(Q5):
   - `conductor` の動きは v0.10.1 と同じ(コードの変更は版とドキュメントコメントだけ)。
   - 仕様書(`docs/spec/rust/` と `docs/spec/language-agnostic/`)を、v0.10.0〜v0.10.1 の実装に合わせた。`watch --follow`、`log_dir`、`unset_env`、既定のリトライ、無くてもよい `.env`、DOT、試行の時刻と所要時間を書き足し、実装と逆の記述と誤りの数を直した。language-agnostic の受け入れの条件に AC-519〜573 を足した。
5. 公開は人が確かめてから行う。

## smoke test

- `build` のジョブが、展開したバイナリの `--version` を 3 対象で確かめる(Intel macOS を除く)。
- 公開の後、利用者の手元では `conductor --version` が `conductor 0.10.2` を出す。

## 中止の条件

- `verify-tag` か `checks` か `build` が落ちたら、下書きは作られない。直してから、タグを消して付け直す(`rollback-runbook.md`)。
