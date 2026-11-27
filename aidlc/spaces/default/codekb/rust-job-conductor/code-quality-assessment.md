# コードの品質の評価

## テスト

| 項目 | 状態 |
|------|------|
| 枠組み | Rust 標準の `#[test]` だけ。テスト名は日本語 |
| `#[test]` の数 | core 472、runtime 259、cli 613、converter 116(合計 1,460)。内訳は下の「テストの数」 |
| 統合テスト | core 36、runtime 16、cli 26、converter 2 のファイル(`tests/*.rs`)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル) |
| 流れのテスト | `cli/tests/flow_*.rs` が、偽の `Runtime`/`Sink`/`Env`/`Signals` を注入して、終了コード・表示・ステートの書き込みの順序を確かめる。`flow_aux.rs` に `watch --follow` のテスト |
| 実バイナリのテスト | `cli/tests/binary_*.rs` が、`conductor` とテスト専用の補助バイナリを動かす。シグナルは `binary_signal.rs`、`signals_real.rs`、`signals_order.rs` |
| 非機能のテスト | 性能(`*_perf.rs`、`nfr_performance.rs`、`nfr_perf.rs`)、安全性(`nfr_security.rs`)、頑健さ(`nfr_robust.rs`)、ソースの検査(`nfr_source.rs`)、内容(`nfr_content.rs`: ステートに許すのは開始と終了の時刻だけ) |
| 性能のテストの上限 | 「debug の見積もりの約 5 倍」で決めた固定の時間を `assert!` で強制する。5 回の中央値で比べる(下の「性能のテストの上限の一覧」) |
| 網羅のテスト | `CommandKind::ALL`(本番の公開の配列)と、core のテスト側の写し `tests/common/kinds.rs` の `DEFINITION_ERROR_KINDS: [D; 34]`・`ENV_FILE_ERROR_KINDS: [E; 14]`(`_` を使わない網羅の `match` と `const _: ()` の確かめで、変種の追加漏れをコンパイルで止める) |
| 参照出力との一致 | `core/tests/graph_layout_oracle.rs` が、`fixtures/graph_layout_oracle*.md` と配置を比べる |
| 文書との一致 | 下の「文書のテスト」。`docs/spec/` を読むテストは無い |
| カバレッジ | CI の `coverage` で行 80% 以上を強制(テスト専用の補助バイナリは除外)。チームの方針(test-after、行カバレッジ 80%)と一致する |
| OS | CI の `test` で ubuntu・macos・windows。`msrv-test` で 1.100.0 |

評価: 良い。純粋な中核と差し替え口(時計を含む)の分離で、流れの失敗の経路(書き込みの失敗、シグナル、出力の失敗、時刻)まで偽の実装で試せる。揺れの源は、実時計と固定の時間の上限に頼るテスト。

### テストの数

数え方(2026-11-28、`8c5c21e`。読むだけのコマンドで数えた。テストは走らせていない):

- 統合テストのファイル: `crates/<crate>/tests/` の直下の `.rs`。補助のディレクトリ(`common/`、`graph_common/`、`bin_support/`、`signal_support/`、`fixtures/`)は含めない。
- `src/tests/` のファイル: `common/` と `mod.rs` を含む全部の `.rs`。
- `#[test]`: 行に `#[test]` だけがあるもの(`grep '^[[:space:]]*#\[test\][[:space:]]*$'`)。`grep -rE '^\s*#\[test\]'` でも同じ 1,460。`#[ignore]` つき 1 件(`converter/tests/nfr_security.rs:190`)を含む。

| クレート | `tests/` のファイル | `#[test]` |
|----------|--------------------|-----------|
| core | 36 | 472 |
| runtime | 16 | 259 |
| cli | 26(ほかに `src/tests/` 12) | 613 |
| converter | 2(ほかに `src/tests/` 4) | 116 |

前回の知識ベース(v0.10.2 の intent、`55958fd`)から、テストの数は変わっていない(`55958fd..8c5c21e` のコードの変更は `crates/runtime/src/lib.rs` の文書と版だけ)。

### 文書のテスト

| ファイル | 読むもの | 確かめること |
|----------|----------|--------------|
| `crates/cli/src/tests/docs.rs`(cli の単体テスト。`crate::` の非公開の項目を使える) | `README.md`(`CARGO_MANIFEST_DIR/../../README.md`。CRLF を LF に) | 必須の語句(`assert_readme_has`)と `DOCS_CHECK_WORDS` の 7 語(`ci.yml` の `docs-check` と同じ語と順)、版の表記の見出し、例の定義と `.env` を `load_definition`/`parse_definition`/`parse_env_file` に通した値、`graph` の text の図 7 例・`--dry-run` の計画・`init` のひな形と実際の出力の 1 文字ずつの一致、ログの 3 つの文(`messages::LOG_*`)、`watch` の使い方の行が `messages::USAGE`(英日)と README で揃うこと |
| `crates/cli/tests/msrv_consistency.rs` | `Cargo.toml`、`README.md`、`.github/workflows/ci.yml`(補助 `repo_file`) | `rust-version` の値が、README の `Rust({msrv} 以降)`、`ci.yml` の `dtolnay/rust-toolchain@{msrv}.0`、`ci.yml` のコメント 2 つと一致する(3 関数) |
| `crates/cli/tests/unix_demo.rs`、`windows_demo.rs` | デモのスクリプトを実行した出力 | 場面 0〜15 の見出しと要点の行、終了コード。場面の数はテストの中のリテラル(`0..=15`) |
| `crates/*/tests/nfr_source.rs`、`crates/converter/tests/nfr_security.rs` | 各クレートの `src/` の `.rs` | ソースの文字列の検査(`unsafe`、OS の分岐の位置、ネットワーク、`forbid(unsafe_code)`)。ソースを文字列として読む前例 |
| CI `docs-check` | `README.md` | 7 語の `grep -q` |

`docs/spec` の文字列は、`crates/` のどの `.rs` にも現れない。CI にも仕様書を見るジョブは無い。

### 性能のテストの上限の一覧

| テスト | 上限(5 回の中央値。debug) | 仕様書の扱い(rust 11 章 3.1 / language-agnostic 11 章 2.1) |
|--------|---------------------------|----------------------------------------------------------------|
| `core/tests/plan_simulate_perf.rs:39`(P1・P2) | 150 ms(`:105`・`:136` で `assert!`)。1,000 ジョブの計画の生成は `:189` で 1 秒 | 強制 / **目安**(AC-573)。1 秒は 強制 / 強制(AC-501) |
| `core/tests/graph_lookup_perf.rs:33・35` | 350 ms・600 ms | 強制 / 目安(「参照実装の基準(同)」、AC なし) |
| `core/tests/env_parse_perf.rs:33` | 100 ms | 強制 / 目安(同) |
| `core/tests/definition_perf.rs:35・37・42` | 350 ms・1,600 ms・700 ms(P3) | P1・P2 は 強制 / 目安(同)。P3 はどちらの表にも無い |
| `core/tests/env_refs_perf.rs:31` | 40 ms | 強制 / 目安(同) |
| `core/tests/flow_perf.rs:34` | 50 ms | 強制 / 表に無い |
| `core/tests/env_merge_perf.rs` | 50 ms・100 ms | 強制 / 表に無い |
| `core/tests/graph_perf.rs` | 1 秒(6 つの形) | 強制 / 強制(AC-486) |

language-agnostic 版の 2.1 は、参照実装の debug の版の時間を「参照実装の基準」として「目安」に置き、時間の上限が実装と機械に依らない項目(1 秒、CPU 時間、中身の一致など)だけを「強制」にしている。

## 仕様書の数と実装の取り出し口

行は `8c5c21e` の各ファイルの行。`R` = `docs/spec/rust/`、`L` = `docs/spec/language-agnostic/`。仕様書は v0.10.2 で v0.10.0〜v0.10.1 の機能を書き足し、今回の照合では下の数はどれも実装と一致した(表の行の数も数えた)。

| 数 | 実装の取り出し口(見え方) | 仕様書の主な記述 |
|----|------------------------------|------------------|
| 定義の誤り 34 | `core/src/error.rs:34-83` の列挙(公開。配列なし)。写し `core/tests/common/kinds.rs:26`(core の統合テストだけ) | R03:644「9.3 種類の一覧(34 種)」(表 34 行)、R08:126-167(表 34 行と「(34 種。…)」)、R10:121・129(識別子 34 個)、R00:23、R02:113・322、R11:219、R12:22・24・169・413。L03:513・814(表 34 行)、L08:128-169、L10:140・142、L00:24、L12:46・47・193・267・349 |
| `.env` の誤り 14 | `core/src/envfile/mod.rs:74-90`。写し `kinds.rs:114` | R03:415(表 14 行)、R08:347、R10:122(識別子 14 個)、R00:23、R02:116・323、R11:219・303、R12:169・244・413。L03:582(表 14 行)、L08:365、L10:148、L00:24、L12:193・267・349 |
| `[pipeline]` 9・`[[job]]` 9 | `core/src/definition/walk/mod.rs:48`・`:59`(private) | R03:144・184「上の 9 つ」(各表 9 行)。L03:312・348 |
| `[job.retry]` 5 | `walk/retry.rs:34`(private) | R03:239「次の 5 つ」(表 5 行)。L03:405 |
| ステート 10・5・2 | `runtime/src/state/reader.rs:192`・`:205`・`:208`(private) | R06:34-47(表 10 行。数の語なし)、R06:61「上の 5 つ」、R06:70「上の 2 つ」、R12:200。L06:36-49(表 10 行)、L06:63・72、L12:224 |
| 試行の誤りの理由 3 | `runtime/src/error.rs:124-128`(`pub(crate) mod reason`) | R10:168-170(3 行)、R12:200。L12:224「試行の 3 つの理由」 |
| `watch` の列 4 | `cli/src/report/watch.rs` | R08:425「4 列」、R08:470・481。L00:29、L08:491 |
| `--interval` 1〜3600・既定 2 | `cli/src/types.rs:215-219`(`pub(crate)`) | R04:47・82・186、R08:554・592、R10:236。L04:48・88 |
| `--format` 3 値 | `cli/src/types.rs` の `GraphFormat`、`args.rs:608` | R04:112、L04:130(数の語なし) |
| コマンド 8 | `cli/src/types.rs:68` の `CommandKind::ALL`(公開) | R04:107、L04:125 |
| 使い方の誤り 8・ステートの誤り 6・command の誤り 2・変換の誤り 6・作成の誤り 5・食い違い 3 | 各列挙(`UsageErrorKind`、`StateErrorKind`、`CommandErrorKind`、`ConversionErrorKind`、`CreateError`、`MismatchKind`) | R10:222・139・137・181・142・141、R02:157、R09:303。L10:232・156・154・194・150・158、L09:400、L06:616、L11:784 |
| デモの場面 0〜15 | `demo/run-demo.sh:69-277`(`scene` 16 個)、`demo/windows/run-demo.cmd:67-330`(16 個) | R00:31、R11:509-524(表)、R11:533、R12:216。L00:32、L11:11・1226・1245、L12:355 |
| 段 0〜21 | 実装に対応は無い(R12・L12 の `### 段 N.` の見出し 22 個) | R00:32、R12:101・146。L00:33、L12:32・125・170 |

突き合わせで気をつけること:

- 版の注記つきで古い数を残す箇所がある: R12:105・L12:129「33 種」(段 10 の時点)、R08:425「3 列」(v0.9.x まで)、R04:107・L04:125「6 コマンド」(v0.8.0 以前)、R11:533「1〜14」・L11:1226「`=== 14. `」(v0.9.x まで)、R12:60「理由文は…(22 種)」(段 5 の時点。今の状態の理由は 25)。
- 数の語が無く、表の行の数だけで数を表す箇所がある(ステートの最上位 10、試行の理由 3、`--format` の 3)。
- 数の取り出し口の多くが private の `const` で、テストから値を取れない。取り出し方は、ソースの文字列を読む(`nfr_source.rs` の前例)、動きで数える、公開の口を足す(`project.md` の NEVER「テストの都合で本番を曲げない」に当たりうる)のどれか。

## 仕様書の粗(A-1)

| 粗 | 場所 | 事実 |
|----|------|------|
| 根拠の列のテスト名の書き方 | L11:235 の決まり「日本語の名前の `_` は空白に当たる」 | `…::…` の名前 743 個のうち、空白で書いたもの 588、日本語の語を `_` でつないだもの 14(AC-097a と、4.7 `convert` の AC-206・207・209・212a・216・217・221・229〜234)。空白にする形は、`br3_1`・`log_dir` のような ASCII の識別子の `_` を残すため、元の関数の名前へ一意に戻らない。末尾を `…` で切った名前も混ざる。rust 版の 11 章はファイル名だけを書く |
| AC-430 の前提 | L11:1232 | 仕様は `LC_ALL=ja_JP.UTF-8`、`LC_MESSAGES=ja_JP.UTF-8`。テスト(`cli/tests/unix_demo.rs:236-251`)は `LC_ALL=C`、`LC_MESSAGES=C`。`C` は英語なので、デモが `LC_ALL` を消すこと(`demo/run-demo.sh:35-36`)をこのテストは見分けられない |
| AC-573 の扱い | L11:54(2.1 の表)、L11:913、L05:949、L02:198、L12:247 | language-agnostic 版は「目安」(AC-573 の本文は「目安。参照実装は開発用の版で強制として検査する」)。rust 版 R11:118 と参照実装は強制。同じ表の「参照実装の基準(同)」の 6 行も、参照実装は強制で、仕様は目安(上の「性能のテストの上限の一覧」) |

## 静的検査と書式

- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が CI の `lint` で関門になる。設定ファイルは無く既定。
- 本体の `#[allow]` は `runtime/src/platform/unix.rs:20` の `#![allow(unsafe_code)]` 1 つだけ。テストの共通の補助(`core/tests/common/mod.rs`、`core/tests/graph_common/mod.rs`、`runtime/tests/common/mod.rs`、`cli/tests/common/mod.rs`、`cli/tests/bin_support/mod.rs`、`cli/tests/signal_support/mod.rs`)に `#![allow(dead_code)]` がある。

## CI/CD

- マージの関門(`ci.yml`): `lint`、`build-check`(MSRV、C のビルドの不在)、`test`(3 OS)、`msrv-test`、`coverage`、`docs-check`。`startup-bench` は警告のみ。
- リリース(`release.yml`): タグが `main` の上にあり `crates/cli/Cargo.toml` の版と一致すること → `ci.yml` の再利用 → 4 対象のビルドと smoke test → 下書きのリリース。
- `audit.yml`: `cargo audit`(PR と毎週。警告のみ)。
- 変更は PR を使わず `main` に直接コミットする運用(`team.md`)のため、関門は `push` の CI で事後に効く。
- 履歴: 1,088 コミット、マージ 0、作者 1 人。タグは `v0.1.0`〜`v0.10.2` の 21 個(`v0.10.0`〜`v0.10.2` は `main` の上)。リモートのブランチは `origin/main` だけ(`8c5c21e` の時点)。
- `docs/spec/` を検査するジョブは無い。

## 文書

- `README.md`(955 行)は、インストール、クイックスタート、使い方(`watch --follow` の節を含む)、定義の書き方(retry、env、`unset_env`、`log_dir`)、ステート(試行の時刻)、終了コード、保証、シグナル、graph、dry-run、init、convert、ロケールを書く。必須の語句は `docs-check` と `docs.rs` が検査。
- 各モジュールの冒頭の `//!` が、責務、契約、規則の番号(BR/NFR/FR、v0.10.0 の Unit、v0.10.1 の BR)、守る性質を日本語で書く。`crates/runtime/src/lib.rs` の呼び出しの順序は v0.10.2 で今の流れ(試行を始めて `Fast` で書く)に直った。
- `docs/spec/` に 7 版の再構築用の仕様(各 13 文書)。`rust/` と `language-agnostic/` は v0.10.2 で v0.10.0〜v0.10.1 の機能を書き足した(差の一覧は解消)。ほかの 5 版は v0.8.0・v0.9.0 のまま。

## 簡易評価(良い / 普通 / 弱い)

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | 良い | Rust 標準の命名。型・関数の名前が責務を表し、文書が規則の番号と結ぶ。ただし `core/tests/env_types.rs:55` のテスト名が `定義の誤りの種類は_33_種で表示が空でない` のまま(中のコメントは 34 種) |
| 関数の大きさ | 普通 | 多くは小さい。`walk`、`args` の解釈、`follow_watch`、`run_job_with_retry`、文言の表は長いが、単一の責務 |
| エラーの扱い | 良い | 失敗はすべて戻り値(パニックしない)。固定の文。ログの失敗も値(`LogOutcome`)で返す |
| テストの存在 | 良い | 主要な経路にすべてテストがある |
| 重複 | 普通 | 使い方の表が 3 か所(`args.rs`、`messages.rs`、README)、仕様書の `--help` の全文を含めると 4 か所。数が仕様書の 2 版の多くの章に重ねて書かれている(上の表) |
| 不要なコード | 良い | `TODO`/`FIXME`/`HACK` は無い |
| 仕様書の追従 | 普通 | v0.10.2 で書き足し、数は今は一致する。ただし検査が無く、次の変更で再びずれうる |

## 守る性質(コードで確認したもの)

- `unsafe` は runtime の `platform/unix.rs` の `libc::kill` 1 か所。core・converter・cli は `#![forbid(unsafe_code)]`。
- 本体の `cfg(unix)`/`cfg(windows)` は `runtime/src/platform/` と `cli/src/platform/` の中だけ(`clear_screen_supported` と `color_supported` は `cfg!(unix)`)。
- ネットワークのコードとライブラリは無い。
- エラーの表示に入力の値を含めない。ステートに書くのは開始と終了の時刻だけで、command・環境変数・出力は書かない。ログのファイルにはジョブの出力だけを書く。
- 入力の大きさの上限(定義・`.env`・ステート・変換の入力は 1 MiB、入れ子 32 段、`.env` は 10,000 項目、変換の展開は 100,000 ノードと 16 MiB、ログの名前の stem は 100 バイト)。

## 技術的負債と、この intent への影響

| 項目 | 場所 | 影響 |
|------|------|------|
| 仕様書の数を確かめるテストが無い | 上の「仕様書の数と実装の取り出し口」 | この intent の A-3。置き場所の候補は、`docs.rs`・`msrv_consistency.rs` と同じくリポジトリのファイルを読む形。誤りの種類の数は core の統合テストに置けば `kinds.rs` の配列が使える。キーの表は private |
| 古い数の注記 | R12:105・L12:129 ほか(上) | 数の検査が版の注記つきの古い数を拾わない書き方が要る |
| 根拠の列の書き方の不揃い | L11 の 14 の AC | A-1。決まり(L11:235)に合わせるか、決まりを直すか |
| AC-430 の前提とテストの差 | L11:1232、`unix_demo.rs:243-244` | A-1。仕様を `C` にするか、テストを `ja_JP.UTF-8` にして強くするか(どちらも本番のコードは変わらない) |
| AC-573 の目安と強制 | L11:54・913 ほか | A-1。強制にするなら、2.1 の「参照実装の基準(同)」の 6 行との揃いも決める |
| テスト名の「33 種」 | `core/tests/env_types.rs:55` | 名前だけの古さ。直すかは要件で決める |
| `team.md` の古い記述 | `aidlc/spaces/default/memory/team.md` | A-2。コミットの数、タグの範囲、版の上げ方の例、v0.10.0 の intent だけの決まり、「v0.10.0 でも」の限定(テストの数は今の値と一致) |
| 版を 0.10.3 にする | 4 つの `Cargo.toml`、`Cargo.lock`(4 か所)、`crates/cli/tests/binary_basic.rs:267`、R00:3・10、R03:1、R04:1・268・274、R09:1、R11:23-28・562、R12:13・362、L00:3・10、L01:3、L02:3、L04:306・310、L05:3、L06:3、L07:3、L08:111、L11:3・781、L12:3 | 版の更新は「版を X にする」の 1 コミット(`team.md`) |
| `watch --follow` の「止まった」の判定 | `crates/core/src/flow/activity.rs` | 判定は変えず、README の注意で補っている |
