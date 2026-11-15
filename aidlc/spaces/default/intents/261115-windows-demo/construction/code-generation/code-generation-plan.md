# コード生成の計画: Windows 向けデモ

## Summary

- Builds: Windows 向けのデモ一式(cmd で書いた実行スクリプトとジョブ、定義ファイル)と、その案内
- Touches: `demo/windows/`(新規)、`demo/README.md`(Windows の案内に書き換え)、`crates/cli/tests/windows_demo.rs`(新規のテスト)
- Tests: 5 件(Windows 専用の統合テスト。定義ファイルの内容と、場面 1〜5 の終了コードを確かめる)。あわせて、このマシン(Windows 11)で `run-demo.cmd` を実際に最後まで動かして確かめる

## 前提(要件の `前提 A1〜A3` を、そのまま使う)

- 待ち時間は整数の秒(`DEMO_SLEEP`、既定 1 秒)。`ping -n <秒+1> 127.0.0.1 >nul` で待つ(`timeout` は、標準入力が端末でないと失敗するため使わない)。
- 色(ANSI)は使わない。
- `conductor` 本体(`crates/*/src`)は変えない。

## 実装の手順

### Step 1: 構成と下ごしらえ [ ]

- [ ] `demo/windows/` と `demo/windows/scripts/` を作る(FR1.1)。
- [ ] UNIX 版のファイル(`demo/run-demo.sh`、`demo/conductor.toml`、`demo/scripts/*.sh`)は触らない(FR1.1)。
- [ ] 改行は CRLF で書く(cmd は LF だけの `.cmd` で、ラベルや `goto` が壊れることがあるため)。

### Step 2: ジョブスクリプト(4 本)を実装する [ ]

- [ ] `demo/windows/scripts/extract.cmd`、`transform.cmd`、`load.cmd`、`notify.cmd` を作る(FR1.2、FR2.2、FR2.3)。
- [ ] 各ジョブは、`DEMO_<ジョブ名>_EXIT`(既定 0)で終了コードを決め、`DEMO_SLEEP`(既定 1 秒)だけ待ち、UNIX 版と同じ文面(`  [extract] 実行中...`、`  [extract] 終了コード N で終わります`)を出す。
- [ ] 先頭に Apache のライセンスヘッダ(`REM`)と、日本語のコメントを付ける(NFR2)。

### Step 3: 定義ファイル `demo/windows/conductor.toml` を実装する [ ]

- [ ] UNIX 版と同じパイプライン `nightly`(`extract`、`transform`、`load`、`notify`)を書く(FR1.3)。
- [ ] `extract` は `warning_exit_codes = [1]` と、警告のときの遷移(`next = "notify"`)を持つ。
- [ ] 文字列の `command` は `scripts\extract.cmd` の形(バックスラッシュは、そのままの文字として扱われる)。TOML の書き方は、リテラル文字列(`'...'`)にして、バックスラッシュの重ねを避ける。
- [ ] `notify` は、配列の `command`(`["cmd", "/c", "scripts\\notify.cmd"]`)で書き、UNIX 版と同じ「配列でも書ける」の見せ場を残す(FR1.3)。

### Step 4: 実行スクリプト `demo/windows/run-demo.cmd` を実装する [ ]

- [ ] 一時ディレクトリ(`%TEMP%` の下)に、`conductor.toml` と `scripts\` を写して、そこで動かし、終わりに後片付けをする(FR2.6)。
- [ ] `CONDUCTOR` が未指定のとき、`target\release\conductor.exe` を使う。無ければ `cargo build --release --locked --bin conductor` でビルドする(FR2.5)。
- [ ] 言語: 既定は日本語。`DEMO_LANG`(例 `en_US`)で切り替える。切り替えの方法(`LANG` などの環境変数)は、Windows の `conductor` の言語判定を確かめてから決める(FR2.7)。
- [ ] 場面 0〜6 を、UNIX 版と同じ順に、同じ見出しで見せる(FR2.1)。各コマンドの前に `$ <コマンド>`、後に `→ 終了コード N` を出す。
- [ ] 場面 2・3 は、環境変数を、その場面のコマンドの中だけに効かせる(`setlocal` と `endlocal`)。
- [ ] `DEMO_PAUSE=1` のとき、各場面の後で Enter を待つ(`pause` ではなく `set /p`。メッセージを UNIX 版に合わせる)(FR2.4)。

### Step 5: 統合テストを書いて実行する(実装の後)[ ]

- [ ] `crates/cli/tests/windows_demo.rs` を新しく作る。`#[cfg(windows)]` で、Windows のときだけ動かす。既存の `bin_support` を使う。
- [ ] テスト 1(FR1.3): `demo/windows/conductor.toml` が、`convert`(または読み込み)で、4 つのジョブと、`extract` の警告の遷移を持つ。
- [ ] テスト 2(FR2.1 場面 1): 全部成功 → 終了コード 0。
- [ ] テスト 3(FR2.1・FR2.2 場面 2): `DEMO_EXTRACT_EXIT=1` → 警告で `notify` へ進み、終了コード 0。
- [ ] テスト 4(FR2.1・FR2.2 場面 3・4): `DEMO_LOAD_EXIT=2` → 終了コード 2。そのあと、環境変数なしでもう一度 → `load` から再開して完了。
- [ ] テスト 5(FR2.1 場面 5・6): `run` と `reset`、`convert --to yaml` が、終了コード 0。
- [ ] 既存の `cargo test` が通ることを確かめる(NFR4)。単体の実行コマンドは `unit-test-instructions.md` に書く。

### Step 6: Windows 実機で、デモを最後まで動かす [ ]

- [ ] このマシン(Windows 11)で、`demo\windows\run-demo.cmd` を最後まで動かし、場面 0〜6 の表示と終了コードが、README の説明と合うことを確かめる(NFR3)。`DEMO_PAUSE=1` と `DEMO_LANG` も、1 回ずつ確かめる。
- [ ] 前提 A2(Windows の `watch` は色なし)と A1(整数の秒)が、実際にその通りか確かめ、違えば README の書き方に反映する。

### Step 7: 案内を更新する(`demo/README.md` だけ)[ ]

- [ ] `demo/README.md` の見出しの文と、使い方に、Windows 版(`demo\windows\run-demo.cmd`)を並べて書く(FR3.1)。
- [ ] 「Windows では、`sh` のスクリプトは動きません。…書き換えてください」の一文は、Windows 版の案内に書き換える(FR3.1)。
- [ ] Windows 版の違い(`DEMO_SLEEP` は整数の秒、`watch` の色なし、`DEMO_LANG` の書き方)を、短く書く。
- [ ] ルートの `README.md` は、変えない(FR3.2)。

### Step 8: 整形・lint と、既存テスト [ ]

- [ ] `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` が通る(NFR4)。

### Step 9: 記録 [ ]

- [ ] `code-summary.md` と `traceability.json` を書く。

## 要件との対応

| 要件 | 計画の手順 |
|---|---|
| FR1.1 | Step 1 |
| FR1.2 | Step 2、Step 4 |
| FR1.3 | Step 3、Step 5(テスト 1) |
| FR2.1 | Step 4、Step 5(テスト 2〜5)、Step 6 |
| FR2.2 | Step 2、Step 5(テスト 3・4) |
| FR2.3 | Step 2 |
| FR2.4 | Step 4、Step 6 |
| FR2.5 | Step 4 |
| FR2.6 | Step 4 |
| FR2.7 | Step 4、Step 6 |
| FR3.1 | Step 7 |
| FR3.2 | Step 7 |
| NFR1 | Step 2〜4(cmd だけで書く)、Step 6 |
| NFR2 | Step 2〜5(ライセンスヘッダと日本語のコメント) |
| NFR3 | Step 6 |
| NFR4 | Step 8 |

## 決めておくこと(確認のうえで、実装の中で決める)

- `demo/README.md` は、作業の前から、プロジェクトにあるファイルである。この作業では、要件 FR3.1 のとおり、Windows の案内に必要な部分だけを書き換える。
