# Code Summary(demo-run-id)

## 変更したファイル

- `demo/run-demo.sh`: 冒頭で `DEMO_RUN_ID`・`CONDUCTOR_RUN_ID` を消し、`LC_ALL`・`LC_MESSAGES` も消す(`DEMO_LANG` が効く)。補助関数 `states()`(`ls conductor.toml*.state`。無ければ `(なし)`)と `setenv()`(`$ export DEMO_RUN_ID=…` を 1 行で表示)を追加。場面 7(validate)、8(run ID でステートを分ける)、9(環境変数と後始末)を追加。
- `demo/windows/run-demo.cmd`: 同じ場面 7〜9、`:states`(`dir /b`)、冒頭で `DEMO_RUN_ID`・`CONDUCTOR_RUN_ID` を消す。UTF-8・CRLF(230 行すべて CRLF。`file` と `grep -c` で確認)。
- `demo/conductor.toml`、`demo/windows/conductor.toml`: `[pipeline]` に `run_id_env = "DEMO_RUN_ID"`。
- `demo/README.md`: 場面 7〜9 の表の行、`run_id_env` と環境変数の箇条書き、Windows 版の節の「場面 0〜9」。
- `crates/cli/tests/unix_demo.rs`(新規、3 件)。
- `crates/cli/tests/windows_demo.rs`: `Demo::run` に `env_remove("CONDUCTOR_RUN_ID")`・`env_remove("DEMO_RUN_ID")`、場面 7〜9 の検証 6 件を追加。

変えていないもの: `crates/*/src`、ルートの `README.md`、`Cargo.toml`・`Cargo.lock`、`.github/`(`git diff` で確認)。

## 場面の実際の出力(UNIX 版を実行して確認。日本語・英語・環境汚染の 3 通り、すべて終了コード 0)

- 場面 7: `validate -v` は 0(`definition is valid (4 jobs)`)。`bad.toml`(`command = "sh -c 'echo hi"`。TOML の文字列は閉じていて、中のシェル風の `'` が閉じていない)の `validate --config` は 64(`command has an unclosed quote` / `command の引用符が閉じていません`)。ステートの一覧は `(なし)`。
- 場面 8: `--run-id build-1` は 2 で止まる。`DEMO_RUN_ID=build-2` は 0 で完走。一覧は `conductor.toml.build-1.state` だけ。`watch --run-id build-1` は 0(`Run ID: build-1`)。同じ run ID の再実行は 0(`extract`・`transform` は `skipped (already executed)`)。一覧は `(なし)`。
- 場面 9: `DEMO_RUN_ID=env-1` のみは 2(`env-1.state`)。`DEMO_RUN_ID=env-1` と `--run-id opt-1` は 2(`opt-1.state`。オプションが勝つ)。`--run-id ""` は 2(`conductor.toml.state`)。`--run-id opt-2` は 2。一覧は 4 個(`env-1`、`opt-1`、`opt-2`、run ID なし)。`reset --run-id opt-1` は 0(3 個になる)。`reset --all` は 0(`(なし)`)。`watch` は 0(`no state (nothing has run yet)`)。
- `CONDUCTOR_RUN_ID=zzz DEMO_RUN_ID=zzz LC_ALL=C` を付けても完走し、日本語のまま、`zzz` は使われない。

## 追加したテスト(合計 9 件)

- `unix_demo.rs` 3 件: 完走と場面 0〜9・要点の行、利用者の環境(`CONDUCTOR_RUN_ID`・`DEMO_RUN_ID`・`LC_ALL`・`LC_MESSAGES`・`NO_COLOR`)に左右されない、一時ディレクトリが残らない(`TMPDIR` を専用の一時ディレクトリにして確認)。
- `windows_demo.rs` 6 件(場面 7、8 の 2 件、9 の 2 件、環境を外すことの確認 1 件): macOS では 0 件として動く。型検査は通る。実行は CI の windows-2022。

## 検証の結果

- `cargo fmt --all -- --check`: 成功。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。
- `cargo test --workspace --locked`: 666 件成功、失敗 0、無視 1(基準 663 + 新しい 3 件)。
- `cargo +1.85 build --workspace --locked --all-targets`: 成功。
- `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`: 成功。
- `cargo llvm-cov`(`--summary-only`): 行カバレッジ 97.74%(基準 97.72%、下限 80%)。
- ルートの README の docs-check の語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert): すべて存在する。

## 基準からの逸脱

- `unix_demo.rs` の 3 件目は、`windows_demo.rs` の 6 件目と同様に、小さな確認を足した(合計は計画の「約 10 件」の範囲内)。
- `run-demo.cmd` を書いたあと、そのファイルだけ、`perl -pi -e 's/\r?\n/\r\n/'` で行末を CRLF にそろえた(計画の Step 4 が求める手当て)。
- 手順の不手際: `windows_demo.rs` の追記の本文を、ファイル書き込みツールではなく、シェルのヒアドキュメントで書いた(その後の編集は Edit)。内容は同じで、再実行した検査はすべて成功。計画のチェック欄の更新の一部に `sed -i` を使った(Step 1、2。以降は Edit)。

## 未実施・残るギャップ

- **`demo/windows/run-demo.cmd` は実行していない**(この環境は macOS)。CI でも実行されない。書き方は、既存の場面(`setlocal`/`endlocal`、`set "X=1"`、`call :runcmd`、`%errorlevel%`)に合わせ、読み直して確かめたのみ。誤りの定義の作成(`echo` の並び)、`--run-id ""` の引数の受け渡し、`dir /b` が一致なしで `errorlevel 1` を返すことへの依存は、Windows の実機での確認が要る(Build and Test で、あなたの確認の項目)。`windows_demo.rs` も、この環境では実行していない(型検査のみ。実行は CI の windows-2022)。
- UNIX 版のコマンド表示で、`--run-id ""` は、空の引数が見えず `--run-id  nightly` と出る(動作は正しい)。
- コミット(6 件): UNIX 版デモ、UNIX 版テスト、Windows 版デモ、Windows 版テスト、README、Step 7 の記録(このファイルと追跡の JSON)。
