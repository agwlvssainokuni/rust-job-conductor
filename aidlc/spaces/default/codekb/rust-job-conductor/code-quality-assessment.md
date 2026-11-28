# コードの品質の評価

## テスト

| 項目 | 状態 |
|------|------|
| 枠組み | Rust 標準の `#[test]` だけ。テスト名は日本語 |
| `#[test]` の数 | core 472、runtime 259、cli 625、converter 116(合計 1,472)。内訳は下の「テストの数」 |
| 統合テスト | core 36、runtime 16、cli 27、converter 2 のファイル。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル) |
| 流れのテスト | `cli/tests/flow_*.rs` が、偽の `Runtime`/`Sink`/`Env`/`Signals` を注入して、終了コード・表示・ステートの書き込みの順序を確かめる |
| 実バイナリのテスト | `cli/tests/binary_*.rs` が、`conductor` とテスト専用の補助バイナリを動かす |
| 非機能のテスト | 性能(`*_perf.rs`、`nfr_performance.rs`、`nfr_perf.rs`)、安全性、頑健さ、ソースの検査(`nfr_source.rs`)、内容(`nfr_content.rs`) |
| 網羅のテスト | `CommandKind::ALL` と、core のテスト側の写し `tests/common/kinds.rs`(定義の誤り 34、`.env` の誤り 14) |
| 文書との一致 | README(`src/tests/docs.rs`、`msrv_consistency.rs`)、仕様書の数(`tests/spec_numbers.rs`。v0.10.3) |
| カバレッジ | CI の `coverage` で行 80% 以上を強制(テスト専用の補助バイナリは除外)。チームの方針(test-after、行カバレッジ 80%)と一致する |
| OS | CI の `test` で ubuntu・macos・windows。`msrv-test` で 1.100.0 |

評価: 良い。純粋な中核と差し替え口(時計を含む)の分離で、流れの失敗の経路まで偽の実装で試せる。この intent の 3 つのファイルは、どれも別のファイルのテスト(`args_grammar.rs` 69、`report_messages.rs` 45、`escape_table.rs` 3、`docs.rs` 22、core の定義のテスト)で、crate の中の入口か公開の入口から試されている。ファイルを分けても、テストのパス(`crate::args::parse_args`、`crate::report::…`)が保たれれば、テストは変わらない。

### テストの数

数え方(2026-11-28、`9376cb7`。読むだけのコマンドで数えた。テストは走らせていない):

- 統合テストのファイル: `crates/<crate>/tests/` の直下の `.rs`。補助のディレクトリは含めない。
- `src/tests/` のファイル: `common/` と `mod.rs` を含む全部の `.rs`。
- `#[test]`: `grep -rE '^\s*#\[test\]' crates --include='*.rs'` の行の数。

| クレート | `tests/` のファイル | `src/tests/` のファイル | `#[test]` |
|----------|--------------------|--------------------------|-----------|
| core | 36 | — | 472 |
| runtime | 16 | — | 259 |
| cli | 27 | 12 | 625 |
| converter | 2 | 4 | 116 |
| 合計 | 81 | 16 | 1,472 |

前回の知識ベース(v0.10.3 の intent、`8c5c21e`)からの差は、cli の `tests/spec_numbers.rs`(1 ファイル、12 テスト)だけ。`team.md` の Testing Posture の数(core 36、runtime 16、cli 27、converter 2、`src/tests/` 12・4、`#[test]` 1,472)は今の値と一致する。

## パスに依存するテスト

この intent でファイルを分けるときに気をつけるテスト。

| テスト | 読むもの・見るもの | この intent との関わり |
|--------|--------------------|------------------------|
| `crates/cli/tests/spec_numbers.rs`(12 テスト) | `crates/core/src/definition/walk/mod.rs`(`WALK_SRC`、`:440`。`const PIPELINE_KEYS`・`const JOB_KEYS` を含む最初の行から `];` までの `"` を数える)、`crates/runtime/src/state/reader.rs`、`crates/runtime/src/error.rs`、`crates/cli/src/report/messages.rs`(行の先頭が `pub const HEADER_` の行の数)、`crates/cli/src/types.rs`(`--interval` の定数、`GraphFormat`)、`crates/core/src/error.rs`、`crates/core/src/envfile/mod.rs`、`demo/run-demo.sh`、`demo/windows/run-demo.cmd`、仕様書 2 版 | `PIPELINE_KEYS`・`JOB_KEYS` を `walk/mod.rs` から動かすと、`WALK_SRC` を直す必要がある。`report/mod.rs` と `args.rs` は読まない。`messages.rs`・`types.rs` は動かさない前提 |
| `crates/cli/src/tests/docs.rs`(22 テスト) | `README.md` だけ。モジュールのパス `crate::report::{init_template, messages, graph_text, plan_text}`、`crate::types::Language` | `report` の子モジュールのパスを保つこと。ソースのファイルは読まない |
| `crates/cli/tests/nfr_source.rs` | cli の `src/` の全 `.rs`(再帰)。コメントを除いた行で、OS の分岐(`cfg(unix)`、`cfg(windows)`、`cfg!(…)`、`target_os`、`cfg(not(…))`)はパスの部品に `platform` があるファイルと `src/tests/` だけに許す。`unsafe`・`low_level`・ネットワークの語は全ファイルで禁止。`panic!(` は `bin/` と `src/tests/` の外で禁止。`#[path`・`include!`・`include_str!`・`include_bytes!` は `src/tests/` の外で禁止。`lib.rs` の最後が `#[cfg(test)]\nmod tests;` | 新しいファイル(`report/` や `args/` の下)も検査の対象。`#[path]` で別のファイルを読む分け方はできない |
| `crates/runtime/tests/nfr_source.rs` | runtime の `src/`(`bin/` を除く) | この intent の対象の外 |
| `crates/cli/tests/msrv_consistency.rs`(3 テスト) | `Cargo.toml`、`README.md`、`.github/workflows/ci.yml` | 関わらない(版の更新でも `rust-version` は変わらない) |
| `crates/cli/tests/binary_basic.rs:267` | `--version` の出力 `conductor 0.10.3\n` | 版を 0.10.4 にするとき直す |
| `crates/core/tests/definition_errors.rs`・`definition_valid.rs` | `tests/fixtures/` の定義と、誤りの行番号・並び | `walk` の呼ぶ順を変えると、同じ位置の誤りの並びが変わりうる |

`line!()`・`file!()` を使うコードは無く、ソースの行番号に依存するテストは無い。

## ファイルのパスを書く文書

ファイルを分けると古くなりうる記述(どれもテストでは検査されない)。

- `docs/spec/rust/02-architecture.md:335-337`(`definition/walk/{mod,retry,env}.rs`)、`:395`(`args.rs`)、`:408-414`(`report/mod.rs` と子モジュール)。
- `docs/spec/rust/08-output-and-i18n.md:3`、`:604`、`:606`、`:645`(エスケープは `report/mod.rs`)。
- `docs/spec/rust/07-process-and-signals.md:44`(`definition/walk/mod.rs`)。
- `docs/spec/rust/12-rebuild-guide.md:76・105・117・127・137・150・159・169・179・201・211`。
- `crates/cli/src/lib.rs:47-59`(モジュールの表)、`crates/core/src/lib.rs:57`、`crates/cli/src/report/mod.rs:15-31`、`crates/core/src/definition/walk/mod.rs:22-23`。
- `aidlc/spaces/default/memory/team.md` の Code Style(「使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える」)。
- language-agnostic 版はファイルのパスを書かない。

## 静的検査と書式

- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が CI の `lint` で関門になる。設定ファイルは無く既定。
- 本体の `#[allow]` は `runtime/src/platform/unix.rs:20` の `#![allow(unsafe_code)]` 1 つだけ。
- 分けたあとに使われなくなった `use` や、`pub(super)` の付け忘れは clippy・コンパイルで止まる。`pub` を広げすぎても警告は出ない(cli は private のモジュールの中なので、外には漏れない)。

## CI/CD

- マージの関門(`ci.yml`): `lint`、`build-check`、`test`(3 OS)、`msrv-test`、`coverage`、`docs-check`。`startup-bench` は警告のみ。
- リリース(`release.yml`): タグが `main` の上にあり `crates/cli/Cargo.toml` の版と一致すること → `ci.yml` の再利用 → 4 対象のビルドと smoke test → 下書きのリリース。
- 履歴: 1,110 コミット、マージ 0、作者 1 人。タグは `v0.1.0`〜`v0.10.3` の 22 個(`v0.10.3` は `9c54b0c`)。リモートのブランチは `origin/main` だけ。
- 前例: `walk.rs` を `walk/{mod,retry,env}.rs` に分けた `c34d3f7` は 1 コミット(+400 −341)。

## 文書

- `README.md`(955 行)。必須の語句は `docs-check` と `docs.rs` が検査。
- 各モジュールの冒頭の `//!` が、責務、契約、規則の番号を日本語で書く。`args.rs:15-55` の `//!` は文法の表(8 コマンド × 14 列)を持ち、使い方の 3 か所の 1 つ。
- `crates/core/src/error.rs:176` は、cli の包みを古い名前 `report::escape` で書く。
- `docs/spec/` に 7 版の仕様。`rust/` と `language-agnostic/` は v0.10.3。v0.10.3 で、根拠の列のテスト名の書き方、AC-430 の前提、性能の表の「参照実装の強制」の注記、段 10 の版の注記、テスト名の「34 種」を直した(コミット `2d25845`、`0d44e81`、`d93720a`、`c06fccf`、`d646030`、`7b7db42`)。

## 簡易評価(良い / 普通 / 弱い)

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | 良い | Rust 標準の命名。型・関数の名前が責務を表し、文書が規則の番号と結ぶ |
| 関数の大きさ | 普通 | 多くは小さい。`Reporter::render`(188 行、35 の枝)、`build_command`(132 行)、`impl Walker` の `cross_check`(58 行)、`pipeline_head`(47 行)、`follow_watch`、`run_job_with_retry` は長い |
| ファイルの大きさ | 普通 | 500 行を超える本体のファイルは `messages.rs`(780。文言の表)、`report/mod.rs`(746)、`runner.rs`(709)、`walk/mod.rs`(695)、`args.rs`(679)、`state/reader.rs`(637)、`types.rs`(core、630)、`resolve.rs`(630)、`flow/record.rs`(590)。この intent は 3 つを分ける |
| エラーの扱い | 良い | 失敗はすべて戻り値。固定の文。`walk` はすべて集め、`args` は最初の 1 つ |
| テストの存在 | 良い | 主要な経路にすべてテストがある |
| 重複 | 普通 | 使い方の表が 3 か所(`args.rs`、`messages.rs`、README)。`render` の枝に、接頭辞と `messages` の文をつなぐ似た `format!` が多い(E・F・G の形) |
| 不要なコード | 良い | `TODO`/`FIXME`/`HACK` は 0 |
| 網羅の守り | 普通 | `DisplayEvent::level` と `render` は網羅の `match`(変種の追加漏れはコンパイルで止まる)。テストの `with_all_events`(`report_messages.rs:99`)は手の一覧で、`DefinitionValid` と `ResetAllKept` を含まない |

## 守る性質(コードで確認したもの)

- `unsafe` は runtime の `platform/unix.rs` の `libc::kill` 1 か所。core・converter・cli は `#![forbid(unsafe_code)]`。
- 本体の `cfg(unix)`/`cfg(windows)` は `runtime/src/platform/` と `cli/src/platform/` の中だけ。
- ネットワークのコードとライブラリは無い。
- エラーの表示に入力の値を含めない(`args` の誤りの対象はオプションの綴りか固定の語、`walk` の理由は固定の文とエスケープしたキー名)。ステートに書くのは開始と終了の時刻だけ。
- 表示の言語と色は必要になったときだけ判定する(`report_messages.rs:694`・`:726` が確かめる)。

## 技術的負債と、この intent への影響

| 項目 | 場所 | 影響 |
|------|------|------|
| 大きなファイル | `crates/cli/src/report/mod.rs`、`crates/core/src/definition/walk/mod.rs`、`crates/cli/src/args.rs` | この intent の対象。まとまりと依存は `code-structure.md`、外からの参照は `component-inventory.md`、`render` の枝の分類は intent の記録 `inception/reverse-engineering/developer-scan.md` の 1.3 |
| `render` の枝の数 | `report/mod.rs:376-564` | 35 の枝(37 変種)。依頼の「40」とは数が違う |
| 呼ぶ順に依存する誤りの並び | `walk/mod.rs:641`(安定の並べ替え)、`args.rs` の `build_command`(最初の誤り) | 分けても呼ぶ順を保つこと |
| `spec_numbers.rs` のパス | `spec_numbers.rs:440`(`WALK_SRC`)、`:478`(`messages.rs`) | `PIPELINE_KEYS`・`JOB_KEYS` と `messages.rs` を動かさなければ、テストは変わらない |
| 文書のパス | 上の「ファイルのパスを書く文書」 | 直すかを要件で決める |
| 古い名前 `report::escape` | `crates/core/src/error.rs:176` | 名前だけの古さ |
| 版を 0.10.4 にする | 4 つの `Cargo.toml`、`Cargo.lock`(4 か所)、`binary_basic.rs:267`、仕様書の 2 版(R00:3・10、R03:1、R04:1・268・274、R09:1、R11:23-28・562、R12:13・362、L00:3・10、L01:3、L02:3、L04:306・310、L05:3、L06:3、L07:3、L08:111、L11:3・781、L12:3) | 版の更新は「版を X にする」の 1 コミット(`team.md`)。整理の intent なので 3 桁目を上げる |
| `watch --follow` の「止まった」の判定 | `crates/core/src/flow/activity.rs` | 判定は変えず、README の注意で補っている |
