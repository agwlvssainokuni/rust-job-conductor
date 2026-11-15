# Requirements Analysis — 質問

依頼: デモ(UNIX 版 `demo/run-demo.sh`、Windows 版 `demo/windows/run-demo.cmd`、案内 `demo/README.md`)に、run ID(`--run-id`・環境変数、run ごとのステートの分離、同じ run ID での再開、`watch` の `Run ID:` の行、`reset --all`)の場面を足す。`conductor` 本体は変えない。Windows 版の自動テスト(`crates/cli/tests/windows_demo.rs`)も、足した場面に合わせて更新する。

調査(codekb)で分かった、デモの作りに関わる点で、決める必要のあるものを 3 つ確認します。なお、2 つの run ID を同時に走らせる見せ方は、出力が混ざるため採らず、順に 1 つずつ見せます(質問しません)。

## Q1. [validate] `validate` の場面も足しますか

`validate`(v0.4.0 で追加)は、まだデモに入っていません。

- A. 足す。定義の誤りを、実行せずに見つける場面を、1 つ足す(推奨)
- B. 足さない(今回は run ID だけ)
- X. Other (please specify)

[Answer]: A

## Q2. [場面の構成] run ID の場面を、いくつに分けますか

- A. 2 場面(推奨)
  - 場面 A「run ID でステートを分ける」: run ID `build-1` で `load` を失敗させて止め、run ID `build-2` は別のステートで最後まで成功する。`watch --run-id build-1` で `Run ID:` の行と止まった位置を見せ、ステートのファイル名(`conductor.toml.build-1.state`)を見せ、同じ run ID で呼び直して `load` から再開する。
  - 場面 B「環境変数と後始末」: 環境変数で run ID を渡し、`--run-id` が環境変数に勝つこと(と、`--run-id ""` で、なしに固定できること)を見せる。止まった run を 2 つ作り、`reset --run-id` で 1 つ消し、`reset --all` で残りをすべて消す。
- B. 1 場面にまとめる(上の内容を、1 つの長い場面で)
- C. 3 場面に分ける(分ける・再開 / 環境変数と優先順位 / 後始末)
- X. Other (please specify)

[Answer]: A

## Q3. [定義] デモの定義に `run_id_env` を書きますか

調査で、利用者の環境に `CONDUCTOR_RUN_ID` があると、既存の場面 1〜5 にも run ID が効いてしまうことが分かりました。

- A. 書く。デモの定義の `[pipeline]` に `run_id_env = "DEMO_RUN_ID"` を書き、デモは `DEMO_RUN_ID` という名前で run ID を渡す。利用者の `CONDUCTOR_RUN_ID` の影響を受けず、定義ファイルの `run_id_env` の見せ場にもなる(推奨)
- B. 書かない。既定の `CONDUCTOR_RUN_ID` を使い、デモの冒頭で、利用者の `CONDUCTOR_RUN_ID` を消す
- X. Other (please specify)

[Answer]: A
