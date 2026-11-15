# Code Summary(readme-reorg)

## 変更したファイル

変更は `README.md` と `demo/README.md` だけ(`git diff --stat HEAD~2 -- . ':!aidlc'`: README.md 203 行の増減、demo/README.md 15 行の増減)。コード、デモのスクリプト、`.github/`、`Cargo.*`、版は変えていない。

### README.md

- 節の順序を、計画の順に直した: 概要 → 目次 → インストール → クイックスタート → 使い方 → 定義ファイルの書き方 → ステートファイルの場所(run ID の小節を含む) → 終了コードの表 → 保証の範囲(at-least-once) → シグナルの扱い → convert の使い方 → ロケールと言語 → ライセンス。`終了コードの表` は、定義ファイルの後ろからステートの後ろへ動いた。見出しの語(docs-check の語を含むもの)は変えていない。
- 追加: `## 目次`(`##` の節と、主要な `###` 2 つ(ソースからのビルド、run ID の小節)へのリンク)、`## クイックスタート`、`### ソースからのビルド`の手順の拡張、`### コマンドの詳しい動き`、`<対象>` と `<版>` の説明。
- 表の短縮: コマンド表(`watch`・`reset`・`validate`)とオプション表(`-v`・`--run-id`・`--all`)の各セルを、短くした。移した詳細は、`### コマンドの詳しい動き`(`watch` の `Run ID:` 行、`reset` の動き、`validate` の詳細と `-v` の 1 行)にある。
- run ID の集約: 規則の本体(与え方、優先順位、`--run-id ""`、`run_id_env` の形、値の規則、中身の `run_id`、`watch`・`reset`、`reset --all`、大文字小文字、同時起動)は、`### run ID でステートを分ける(v0.5.0 以降)` にだけある。ほかの場所は、構文、1 文、小節へのリンク、終了コードの表の意味だけ。
- 版の注記: 配列の `command`(v0.2.0 以降。見出しに既存)、`validate`(v0.4.0 以降。コマンド表と詳しい動き)、run ID と `reset --all`(v0.5.0 以降。見出し、`--all` のセル、`run_id_env` のセル)。互換の注意(v0.1.x、`[[jobs]]`)は、その節に残した。

### demo/README.md

- 場面の表に場面 0(定義ファイルの表示)を追加(FR4.6)。場面 7 の表の誤りの文は、日本語と英語の両方を本文に書いた(既定の日本語では `command の引用符が閉じていません`)。
- `DEMO_<ジョブ名>_EXIT` の既定(0)、`DEMO_PAUSE` は値が `1` のときだけ、UNIX の `DEMO_SLEEP` は小数も可(既定 0.5)、Windows は整数で既定 1、を明記。ルートの README の run ID の小節へのリンクを追加。構成(UNIX と Windows を並べる)は保った。

## FR4 の 6 項目(前 → 後)

1. 展開の手順: `install -m 755 conductor /usr/local/bin/conductor`(アーカイブの最上位のディレクトリを言わない) → 展開すると `conductor-<版>-<対象>/` ができ(zip も同じ)、その中に `conductor`・`LICENSE` があると書き、`install -m 755 conductor-<版>-<対象>/conductor ...` に直した。実際の v0.6.0 の aarch64-apple-darwin のアーカイブで、この構造と `install` を確かめた。
2. シグナルの節: 「`watch`・`reset`・`convert` は既定の動作」 → `validate` を加えた。
3. `reset --all`: 「中身の `run_id` が一致するものを消す」と読めた → run ID なしの `<名>.state` は中身を読まず通常のファイルなら消す(シンボリックリンクは消さない)、run ID つきだけが中身の照合を受ける、消すのに失敗したら 65(終了コードの表の 65 にも追記)。
4. OS の表: Intel macOS に「クロスビルドで作ったもので、動作の確認はしていません」を追記。
5. 終了コードの書き方: `--all` の「(68)」の唐突な書き方を、「使い方の誤り(68)」に統一(`validate`・`--all` の併用・`validate` の `--run-id` も同じ形)。
6. demo/README.md の場面 0 を追加。

## 検証の結果

- docs-check と同じ 7 語句の `grep`: すべて見つかった(MISSING なし)。整理前(Step 1)も通っていた。
- 目次・リンク: `](#...)` のリンク 17 件、切れ 0 件。`demo/README.md`・`LICENSE` へのリンクは実在(GitHub のアンカー規則で見出しの slug を作る Python スクリプトで確認)。
- run ID の規則の集約: `A-Za-z0-9._-`、`1〜64 文字`、`--run-id` があれば環境変数は見ない、`--run-id ""` の固定、`reset --all` の「中身は読まずに」「シンボリックリンクは消しません」が、いずれも README に 1 回だけ。
- 整理前後の突き合わせ(`git show HEAD~2:README.md`): 整理前のバッククォートの語句 110 個がすべて残っている。文単位では、意図して書き換えた 24 件(FR3 の表の短縮で小節へ移したもの、FR4 の修正)だけが一致しない。終了コード 0/ジョブの値/1/64/65/66/67/68/70/127/128+番号の行、成功・警告の規則、at-least-once の箇条書き 5 つ、シグナルの動作、convert の制限(1 MiB など)、ロケールの規則は、残っている。
- クイックスタートの実行: リリースビルドの `conductor` で、`target/quickstart-check/` の定義を `LANG=C` で実行し、validate 0 → `FAIL=3 pipeline -v hello` 3 → `watch`(step2 が error) → 再開 0 → `watch`(no state)。出力は README に実物のまま載せた。
- ソースからのビルド: `cargo build --release --locked --bin conductor` 成功、`cargo test --workspace --locked` 666 件成功(1 件 ignored)、`cargo install --path crates/cli --locked --bin conductor --root target/install-check2` 成功(`conductor` だけが入る)、`conductor --version` は `conductor 0.6.0`。`git ls-remote https://github.com/agwlvssainokuni/rust-job-conductor.git HEAD` が成功し、URL は実在する。
- デモ: `DEMO_SLEEP=0 sh demo/run-demo.sh` を実行し、場面 0 の表示と、場面 7 の終了コード 64・日本語の文を確かめた。スクリプトの env 変数と既定値(`DEMO_SLEEP` UNIX 0.5、Windows 1)を読んで突き合わせた。
- `cargo fmt --all -- --check` 成功、`cargo clippy --workspace --all-targets --locked -- -D warnings` 警告なし、`cargo test --workspace --locked` 666 件成功・1 件 ignored(ベースラインと同じ)。
- 変更の範囲: `git diff --stat` は、`README.md` と `demo/README.md` と、この intent の記録だけ。

## 計画からのずれ・未確認

- `cargo install` の行に `--bin conductor` を足した。計画の `cargo install --path crates/cli --locked` は、テスト専用の補助 `conductor-cli-test-helper` まで入れてしまうため(実行して確認した)。
- Windows の zip の構造(最上位のディレクトリ)と、Windows でのクイックスタート(`cmd /c` への読み替え)は、このマシンでは実行していない。zip の構造は `release.yml` のパッケージの手順(`mkdir "$name"`、`7z a`)から読んだ。Windows の例は `["cmd", "/c", "echo step1"]` と `set FAIL=3` の 1 行にとどめた。
- 展開の実機確認は、aarch64-apple-darwin の tar.gz のみ(SHA256SUMS の照合も成功)。Linux・Intel macOS・Windows の実物は未確認。
- `shasum` と Gatekeeper の手順は、調査(照合表の ⑤)のとおり、コードの外で、実行していない。
- Build and Test の `test-results.md` は、このステージの範囲外のため書いていない(上の結果を引き継ぐ)。
