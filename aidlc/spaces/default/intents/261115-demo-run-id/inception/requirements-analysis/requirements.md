# Requirements(demo-run-id)

## Sources

- Initial description: デモ(UNIX 版 demo/run-demo.sh と Windows 版 demo/windows/run-demo.cmd、案内 demo/README.md)に、run ID(--run-id と環境変数、run ごとのステートの分離、同じ run ID での再開、watch の Run ID の行、reset --all)の場面を足す。あわせて validate の場面を足すかは、要件の整理で確認する。conductor 本体(crates/*/src)は変えない。Windows 版の自動テスト(crates/cli/tests/windows_demo.rs)も、足した場面に合わせて更新する。リリースはしない(版も上げない)。 [desc]
- Workflow-selected scope: express [scope]
- 既存コードの調査(`aidlc/spaces/default/codekb/rust-job-conductor/`、この intent の focused scan): 両デモの構造(場面 0〜6、補助関数、環境変数の限定の仕方、一時ディレクトリ)、`windows_demo.rs` が `run-demo.cmd` を実行せず `conductor` を直接呼ぶこと、実装済みの run ID の振る舞いと実際の表示(英・日)、並行実行の出力が混ざること、利用者の `CONDUCTOR_RUN_ID` がデモに影響すること。 [memory:M1]
- 質問への回答: Q1=A(`validate` の場面も足す)、Q2=A(run ID の場面は 2 つ)、Q3=A(デモの定義に `run_id_env = "DEMO_RUN_ID"` を書く) [Q1] [Q2] [Q3]

## Intent analysis

デモは、`conductor` の主な動きを、場面 0〜6 で見せている。v0.4.0 の `validate` と、v0.5.0 の run ID は、まだデモにない。UNIX 版(`sh`)と Windows 版(`cmd`)の両方に、場面を足して、新しい機能を、動くものとして見せる。`conductor` 本体は変えない。

## Functional requirements

- **FR1 新しい場面(UNIX 版 `demo/run-demo.sh`)** 既存の場面 0〜6 の後ろに、次の 3 場面を足す(番号は 7〜9。0〜6 の番号と内容は変えない)。出力は、ほかの場面と同じ形(見出し、`$ <コマンド>`、`→ 終了コード N`)で出す。
  - **FR1.1 場面 7「validate」** 定義を実行せずに検査する。正しい定義に `validate -v` を実行して、終了コード 0 と、`definition is valid`/`定義は正しいです` の 1 行を見せる。次に、誤りのある定義(一時ディレクトリで、`command` の引用符を閉じていない定義を作る)に `validate` を実行して、終了コード 64 と、誤りの文を見せる。実行していないこと(ジョブの出力が出ない、ステートファイルがない)を、示す。
    - 受け入れ基準: 場面 7 が最後まで動き、終了コードが 0 と 64 で表示される。ジョブのスクリプトの出力(`[extract] 実行中...` など)が出ない。ステートファイルの一覧が空である。
  - **FR1.2 場面 8「run ID でステートを分ける」** `build-1` で `load` を失敗(`DEMO_LOAD_EXIT=2`)させて止め(終了コード 2)、`build-2` は別のステートで最後まで成功する(0)。止まった run のステートが、`conductor.toml.build-1.state` として別のファイルになっていること(ファイルの一覧)と、`watch --run-id build-1` が `Run ID: build-1` の行と、止まった位置を出すことを見せる。同じ run ID(`build-1`)で呼び直すと、`load` から再開し、`extract` と `transform` は再実行されず(`skipped (already executed)`)、完了する。run ID の渡し方は、`--run-id` と、環境変数 `DEMO_RUN_ID` の両方を、使い分けて見せる(`build-1` はオプション、`build-2` は環境変数)。
    - 受け入れ基準: `build-2` の完走が `build-1` の止まった位置を変えない(`watch` で確かめる)。再開の出力に `extract` と `transform` のジョブの実行が出ない。
  - **FR1.3 場面 9「環境変数と後始末」** 環境変数 `DEMO_RUN_ID` と `--run-id` の優先を見せる(両方があれば `--run-id` が勝つ)。`--run-id ""` は、環境変数があっても run ID なし(既定のステート)に固定されることを見せる。止まった run を 2 つ作り(`DEMO_LOAD_EXIT=2`)、`reset --run-id` で 1 つを消し、`reset --all` で、残りと、run ID なしのステートを、すべて消す。最後に、ステートファイルがないこと(ファイルの一覧)と、`watch` が `no state` を出すことを見せる。
    - 受け入れ基準: 場面の終わりに、`conductor.toml*.state` が 1 つもない。`reset --all` の終了コードが 0。
  - **FR1.4 環境変数の見せ方** 環境変数を使うコマンドは、その直前に、変数の指定を 1 行で表示する(例 `DEMO_RUN_ID=build-2`)。実際のコマンドの行(`$ conductor …`)は、これまでどおりのまま。
- **FR2 新しい場面(Windows 版 `demo/windows/run-demo.cmd`)** FR1 と同じ 3 場面(7〜9)を、`cmd` で足す。既存の流儀(`setlocal` / `set "X=1"` / `endlocal`、`call :runcmd`、コードページ 65001、`ping` による待ち、CRLF)に従う。
  - 受け入れ基準: Windows の実機(または CI の `windows-2022`)で、場面 0〜9 が最後まで動き、各場面の終了コードが、UNIX 版と同じになる。
- **FR3 定義** 両デモの定義(`demo/conductor.toml`、`demo/windows/conductor.toml`)の `[pipeline]` に、`run_id_env = "DEMO_RUN_ID"` を書く。ジョブの定義と、場面 0〜6 の動作は、変えない(場面 0 の定義の表示は、1 行増える)。
- **FR4 利用者の環境の影響を受けない** 両デモは、冒頭で、`DEMO_RUN_ID` を消す(利用者の環境にあっても、場面 1〜6 が影響を受けない)。デモの定義が `run_id_env = "DEMO_RUN_ID"` なので、利用者の `CONDUCTOR_RUN_ID` は、効かない。UNIX 版は、Windows 版と同じに、`LC_ALL` と `LC_MESSAGES` を空にして、`DEMO_LANG` が、利用者の `LC_ALL` に負けないようにする。
  - 受け入れ基準: `CONDUCTOR_RUN_ID=zzz DEMO_RUN_ID=zzz LC_ALL=C sh demo/run-demo.sh` が、既存の場面 1〜6 で、ステートファイルを run ID 付きにせず、出力が日本語のまま、完走する。
- **FR5 案内(`demo/README.md`)** 場面の表に、場面 7〜9 の行を足し、環境変数 `DEMO_RUN_ID` と、定義の `run_id_env` を説明する。Windows 版の節の「場面 0〜6」を直す。UNIX 版と Windows 版の使い方を並べる構成は、変えない。
- **FR6 自動テスト**
  - **FR6.1 Windows 版(`crates/cli/tests/windows_demo.rs`、`cfg(windows)`)** 場面 7〜9 に当たる検証を足す: `validate -v` の標準エラーの 1 行と終了コード 0、誤りの定義の `validate` が 64、`build-1` が 2 で止まり `build-2` が 0 で完走する、`watch --run-id build-1` の標準出力に `Run ID: build-1`、`build-1` の再開で `extract`・`transform` が再実行されず 0、`--run-id` が `DEMO_RUN_ID` に勝つ、`--run-id ""` が既定のステートに固定する、`reset --run-id` と `reset --all` のあとにステートファイルがない。`Demo::run` は、`CONDUCTOR_RUN_ID` と `DEMO_RUN_ID` を環境から外す(開発機・CI の環境の影響を受けない)。このテストは、`run-demo.cmd` を実行しない(従来どおり、定義とスクリプトを写して、`conductor` を直接呼ぶ)。
  - **FR6.2 UNIX 版(`crates/cli/tests/unix_demo.rs`、`cfg(unix)`)** 新しいテストで、`sh demo/run-demo.sh` を、`CONDUCTOR`(`conductor` のバイナリ)と `DEMO_SLEEP=0` を指定して実行し、終了コード 0 と、各場面の見出し(0〜9)と、要点の行(`Run ID: build-1`、`definition is valid`、`→ 終了コード 64`)が出ることを確かめる。利用者の環境(`CONDUCTOR_RUN_ID`、`DEMO_RUN_ID`、`LC_ALL`)に左右されないよう、これらを外し、`DEMO_LANG` を英語(`en_US.UTF-8`)にする。
    - 受け入れ基準: Linux と macOS の CI で、このテストが成功する。`cargo test` の通常の実行時間が、数秒以内に収まる。

## Non-functional requirements

- **NFR1 本体は変えない** `conductor` の本体(`crates/*/src`)、`README.md`(ルート)、`Cargo.toml`、版、`.github/` は、変えない。リリースはしない。
- **NFR2 既存の動作** 場面 0〜6 の動作と終了コードは、変わらない。既存のテスト(`windows_demo.rs` の既存の検証を含む)は、成功のまま。
- **NFR3 品質** `cargo fmt --check`、`cargo clippy -D warnings`、Rust 1.85 でのビルドとテスト、行カバレッジ 80% 以上、3 OS の CI が成功する。CI の `docs-check`(ルートの README の語句)を壊さない。
- **NFR4 互換** UNIX 版は、POSIX の `sh` で動く(`bash` 固有の機能を使わない)。Windows 版は、`cmd.exe` だけで動く(PowerShell、WSL、Git Bash を前提にしない)。新しいファイルと変更したファイルは、Apache のライセンスヘッダ(`#`/`REM`/`//`)を保つ。

## Constraints

- 2 つの run ID を同時に走らせる見せ方は、採らない(出力が混ざり、`cmd` のサブルーチンは `start` に渡せないため)。順に 1 つずつ見せる。
- `.cmd` は UTF-8、CRLF のまま。ファイルの書き込みは、LF だけを出すツールのため、書き換えたあとに CRLF を保つ。
- `reset --all` の「別の定義のステートは残す」「壊れたものは警告」は、デモでは、見せない(手で作ったファイルが要るため。`README` と自動テストに、任せる)。

## Assumptions

- 場面の番号は、既存の 0〜6 を保ち、7(validate)、8(run ID で分ける)、9(環境変数と後始末)とする。[assumption]
- UNIX 版デモの自動テスト(FR6.2)を、新しく足す。いまは UNIX 版のデモに自動テストがなく、足した場面が壊れても気づけないため。[assumption]
- UNIX 版の `LC_ALL`/`LC_MESSAGES` の扱いを、Windows 版に揃える(FR4)。既存の小さな不整合の修正で、この変更の範囲に含める。[assumption]
- `run-demo.cmd` の実行は、CI で行われない(`windows_demo.rs` は、スクリプトを実行しない)。Windows 版の新しい場面の `cmd` スクリプトは、Windows の実機での実行が、この環境では確かめられない。[assumption]

## Out of scope

- `conductor` 本体の変更。版の引き上げとリリース。ルートの `README.md` の変更。
- 2 つの run ID の同時実行の見せ方。
- 別の定義のステートと壊れたステートを使う `reset --all` の場面。
- PowerShell 版のデモ。CI への、デモの実行(`run-demo.sh`/`run-demo.cmd` そのもの)の追加。

## Open questions

None.

## Assumptions & Open Questions

None.
