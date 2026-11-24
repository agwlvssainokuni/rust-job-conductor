# 変える場所の一覧(Inventory): cli のライブラリの公開範囲を狭める

HEAD c35ada6(cli-public-surface の意図の把握のコミット。版 0.9.8)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。初期の記述と実物が違ったところは「(記述との違い)」と書く。設計の要点、テストの移動と変更の一覧(T…)、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent の判断と、この intent の Q1〜Q3): **テストのコードの都合で本番のコードを曲げない**。本番に、テストのための要素(`pub`、`#[doc(hidden)]`、テストのための引数や関数)を足さない。`#[cfg(test)]` の中は可。公開に残すのは、`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目を使うのに要る型だけ(Q3: A)。狭めた項目を直接使うテストは src の単体テストへ移し(Q1: A)、今の確かめ方と期待の値は変えない。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変えない。版は上げない。

決めてほしい点(`functional-spec.md` §10 の D1〜D7)は、ユーザーが決めた(すべて推奨のとおり。2026-11-24)。D1: `src/tests/` のディレクトリ。D2: 検査の範囲を製品のコードに限る(`src/tests/` を OS 分岐と `panic!(` の検査の外にし、`lib.rs` の末尾の宣言を確かめる。`unsafe`・ネットワークの検査は `src/tests/` も対象のまま。ユーザーは、規則「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」を製品のコードについての規則と読むことを認めた)。D3: 狭めた項目で確かめるテストだけを移す。D4: `flow_init.rs` の 2 件は統合テストに残し、原本の写しで書く。D5: `tests/common` にテストの言語 `Language` を足す。D6: 言語に依らない仕様書は直さない。D7: 補助は `src/tests/` に写す。この文書の「(D1)」〜「(D7)」は、その決定のとおりの形を書いている。T 表(T1〜T10)と移すテストの一覧は、設計の承認で認めてもらう(承認待ち)。

範囲の外(初期の記述): core・runtime・converter の公開範囲。依存の注入の継ぎ目そのものの形(`Runtime`・`Sink`・`Env`・`Signals` のメソッド、`Deps` の項目)。

## 0. 基準と、設計の検証

### 0.1 テストの件数(`cargo test --workspace --locked`、macOS、rustc 1.100.0、debug、HEAD c35ada6)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 112 | 270 | 382 |
| runtime | 69 | 128 | 197 |
| converter | 96 | 19 通過 + 1 無視 | 115 通過 + 1 無視 |
| cli | 2 | 456 | 458 |
| 全体 | | | **1,152 通過、1 無視、0 失敗** |

テスト名の一覧は `cargo test --workspace --locked -- --list` の `: test` の行(1,153 行。無視の 1 件を含む)。cli の単体テストは `panic_hook.rs` の 2 件だけ。

### 0.2 設計の検証(写しの木での試し)

T 表を検索だけに頼らないために、HEAD c35ada6 の写し(`git archive HEAD` を `target/cli-pub/work` に展開)に、この一覧の全項目(本番)と `functional-spec.md` §4 の T 表を当て、次を確かめた。写しは確かめたあと消した(`target/cli-pub` ごと)。リポジトリのコードは変えていない。仕様書(`docs/spec`)の書き換えはコンパイルとテストに関わらないので、写しには当てていない。

- **本番のコードだけを変えた段階**(§1 の `lib.rs` の形を当てた形)で、`cargo build -p conductor-cli --locked` と `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` は、警告 0 件で通る。**狭めた項目のうち、本番で使われなくなる(`dead_code` の警告が出る)ものは 0 件**(どの項目も、crate の中の流れか表示が使っている)。
- 同じ段階で、`cargo check --workspace --locked --all-targets --keep-going --message-format=short` のエラーは **62 件**で、出たファイルはすべて T 表にある: `args_grammar.rs` 5、`exit_codes.rs` 29(`exit::*` の各定数と関数)、`docs_graph.rs` 4、`graph_report.rs` 3、`docs_dry_run.rs`・`docs_init.rs`・`flow_init.rs`・`graph_long_id.rs`・`init_template.rs`・`init_types.rs`・`plan_report.rs`・`report_messages.rs` 各 2、`common/kinds.rs`・`common/mod.rs`・`escape_table.rs`・`flow_validate.rs`・`run_id_arg.rs` 各 1。
- 同じ段階で、`cargo doc --workspace --no-deps --locked` の cli の警告が 4 件増える(`lib.rs:30`、`:52`、`:54`、`:55` の、非公開になる `parse_args`・`run_id`・`exit`・`report` へのドキュメントのリンク)。§1 の説明の直しで 0 件に戻る。
- T 表(と D1〜D5 の推奨)を当てたあと、`cargo fmt --all` が 3 か所を整える(`tests/common/mod.rs` の import が 1 行に収まる、`tests/flow_init.rs` の `replacen` の呼び出し、`src/tests/docs_graph.rs` の `render_text` の呼び出しが 1 行に収まる)。整えたあと `cargo fmt --all -- --check` が通る。**実装では、各コミットで `cargo fmt --all` をかけてから確かめる**。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc` の両方で通る。`cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(テストを含まない本番の形)も通る。
- `cargo test --workspace --locked` が **1,152 通過、1 無視、0 失敗**(core 112 + 270、runtime 69 + 128、converter 96 + 19 + 1 無視、cli **164** + **294**。`functional-spec.md` §6 の見込みと一致)。
- テスト名の一覧の差(名前の最後の部分で比べた。どちらも並べ替えて比べる): **差は 0 件**(消えた名前・増えた名前・名前の変わったテストは無い。★ なし)。パスで比べた差は 324 行 = 移した 162 件 × 2(統合テストのバイナリの中の `X` から、単体テストの `tests::<元のファイル名>::X` へ)。単体テストのパスの内訳は `tests::args_grammar` 60、`report_messages` 36、`graph_report` 28、`exit_codes` 7、`init_template` 7、`plan_report` 7、`docs_graph` 6、`escape_table` 3、`init_types` 3、`graph_long_id` 2、`docs_dry_run` 1、`docs_init` 1、`run_id_arg` 1。
- ソースの検査(`tests/nfr_source.rs` の 6 件)は、T 表を当てただけでは **2 件が失敗する**: `os_ごとの分岐は_platform_の中だけ`(移した `args_grammar.rs` の `#[cfg(unix)]` 4 か所と `report_messages.rs` の `cfg!(unix)` 2 か所)と `テスト専用の経路を実行ファイルに持たない`(移した `args_grammar.rs` の `panic!(` 5 か所と `init_template.rs` の 2 か所)。cli の検査は `src/` の下を**再帰で**読み、`#[cfg(test)]` で切らずに**ファイル全体**を検査するため(converter の検査とは違う)。§3 の `nfr_source.rs` の直し(D2)を当てると 6 件すべてが通る。`unsafe`・`low_level`・ネットワークの語の検査は `src/tests/` も対象のまま通る(移すコードにそれらの語は無い)。
- `cargo doc --workspace --no-deps --locked` の警告は、基準と同じ 5 件(core 1、runtime 4。この intent と無関係)。cli は 0 件。
- 行カバレッジ(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`): 全体の行 **98.06%**。cli の src の各ファイルは 92.86%〜100%。`src/tests/` の下のファイルは計測の表に出ない(テストのコード)。
- 写しで変わったファイル: 本番 1(`src/lib.rs`)、単体テストの新しいファイル 17(`src/tests/mod.rs`、`common/mod.rs`、`common/kinds.rs`、`docs_support.rs` と、元の統合テストと同じ名前の 13)、統合テスト 7(`tests/common/mod.rs`、`docs_dry_run.rs`、`docs_graph.rs`、`docs_init.rs`、`flow_init.rs`、`flow_validate.rs`、`nfr_source.rs`)、消えた統合テスト 11(移した 10 ファイルと `tests/common/kinds.rs`)。core、runtime、converter は変わらない。`src/report/escape.rs` の説明(§2)と `tests/common/mod.rs` のコメント(§4)の直しは、文だけなので写しでは当てていない。

---

## 1. `crates/cli/src/lib.rs` の公開の項目と判定(Q3)

「継ぎ目を使うのに要る型」の基準(Q3): 公開に残すトレイトと関数(`run_cli_with`、`Deps`、`Runtime`・`Sink`・`Env`・`Signals`)の引数・戻り値・項目に出てくる **cli の**型で、偽物を書くのに要るもの。core・runtime・converter の型(`PipelineDefinition`、`InterruptSignal`、`ConversionError` など)は、それぞれのクレートが公開しているので、この判定の対象の外。

### 1.1 今の公開(HEAD の `lib.rs:93-117`)と判定

| # | 項目(今の公開のしかた) | 判定 | 理由 |
|---|---|---|---|
| P1 | `run_cli`(関数) | **残す** | `main` が使う(`main.rs:21`) |
| P2 | `StdSink`(`pub use deps::…`) | **残す** | `main` が使う |
| P3 | `SystemEnv`(同) | **残す** | `main` が使う |
| P4 | `install_panic_hook`(`pub use panic_hook::…`) | **残す** | `main` が使う |
| P5 | `run_cli_with`(関数) | **残す** | 依存の注入の継ぎ目(流れのテストが偽物を渡す入口) |
| P6 | `Deps`(`pub use deps::…`。`pub` の項目 `runtime`・`sink`・`env`・`signals`) | **残す** | 継ぎ目 |
| P7 | `Runtime`(trait) | **残す** | 継ぎ目 |
| P8 | `Sink`(trait) | **残す** | 継ぎ目 |
| P9 | `Env`(trait) | **残す** | 継ぎ目 |
| P10 | `Signals`(trait) | **残す** | 継ぎ目 |
| P11 | `SystemRuntime` | **残す** | 継ぎ目の本番の実装(`run_cli` の中と、統合テストの `ScriptedRuntime` が `inner` に持って委ねる) |
| P12 | `SystemSignals` | **残す** | 継ぎ目の本番の実装(`signals_real.rs`・`signals_order.rs` が本物の監視を確かめる) |
| P13 | `SignalError` | **残す** | 継ぎ目に要る型: `Signals::start` の戻り値 `Result<(), SignalError>`。偽物(`FakeSignals` の `Fail`)が `SignalError::Failed` を返す |
| N1 | `parse_args`(`pub use args::…`) | **狭める** | 継ぎ目の引数・戻り値に出ない。crate の中の `run_cli_with` だけが呼ぶ |
| N2 | `CliCommand`(`pub use types::…`) | **狭める** | 同(解析の結果。継ぎ目に出ない) |
| N3 | `CommandKind` | **狭める** | 同 |
| N4 | `GraphFormat` | **狭める** | 同 |
| N5 | `InitOptions` | **狭める** | 同 |
| N6 | `InitTarget` | **狭める** | 同 |
| N7 | `Language` | **狭める** | 同(`Env` は文字列を返すだけで、言語の判定は crate の中) |
| N8 | `Level` | **狭める** | 同 |
| N9 | `ParsedArgs` | **狭める** | 同 |
| N10 | `RunIdArg` | **狭める** | 同 |
| N11 | `UsageError` | **狭める** | 同 |
| N12 | `UsageErrorKind` | **狭める** | 同 |
| N13 | `pub mod exit`(定数 `EXIT_OK`〜`EXIT_INTERNAL` の 8 つ、`exit_code_for`、`outcome_exit_code`、`signal_exit_code`) | **狭める**(`mod exit;`) | 継ぎ目に出ない。終了コードは `run_cli_with` の戻り値 `i32` で、名前の定数は要らない(Q3: A。B「`exit` を残す」は選ばれなかった) |
| N14 | `pub mod report`(`DisplayEvent`、`Reporter`、`ResultDetail`、`StateContext`、`push_line` と、公開の下位モジュール `escape`、`graph_mermaid`、`graph_text`、`init_template`、`plan_text`) | **狭める**(`mod report;`) | 継ぎ目に出ない。表示は `Sink` に書かれたバイト列として外から見える |
| N15 | `pub mod run_id`(`resolve_run_id`) | **狭める**(`mod run_id;`) | 継ぎ目に出ない |

- 残す: **13 項目**(関数 3、trait 4、型 6)。狭める: **15 項目**(再公開の関数 1、再公開の型 11、公開モジュール 3)。
- (記述との違い)初期の記述は「継ぎ目に要る型」を例として挙げていなかった。基準に当たる cli の型は `SignalError` の 1 つだけ(`Runtime` のメソッドの引数・戻り値は core・runtime・converter の型だけ。`Env`・`Sink` は `str`・`String`・`bool`・`io::Result` だけ)。
- 下位モジュールの中の `pub` の項目(`report::graph_text::render_text` など)は、`pub` のまま残す(親のモジュールが非公開になるので、crate の外からは見えない。`pub(crate)` への書き換えはしない。差を小さくするため)。写しで、`unreachable_pub`(既定で無効)のほかに警告が出ないことを確かめた。
- 本番で使われなくなる項目は 0 件(§0.2)。消す項目は無い。

### 1.2 目標の形(`lib.rs:88-121`)

```rust
mod args;
mod aux_commands;
mod ctx;
mod deps;
mod dry_run;
mod exit;
mod graph;
mod init;
mod panic_hook;
mod pipeline;
mod platform;
mod report;
mod retry;
mod run;
mod run_id;
mod step;
mod types;

use std::ffi::OsString;

pub use deps::{
    Deps, Env, Runtime, SignalError, Signals, Sink, StdSink, SystemEnv, SystemRuntime,
    SystemSignals,
};
pub use panic_hook::install_panic_hook;

use args::parse_args;
use types::{CommandKind, ParsedArgs};

use ctx::Ctx;
use exit::{EXIT_OK, EXIT_USAGE};
use report::{DisplayEvent, Reporter};

// (run_cli と run_cli_with は今のまま)

// クレートの内部を直接確かめる単体テスト(`src/tests/`)。
#[cfg(test)]
mod tests;
```

- `run_cli_with` の本体は `parse_args`、`ParsedArgs::…`、`CommandKind::…` を名前で使うので、非公開の `use` で取り込む(今は `pub use` が兼ねている)。
- 最後の `#[cfg(test)] mod tests;` は、§3 の単体テストの宣言(コミット 1 で足す。D1)。

### 1.3 説明(ドキュメントコメント)の直し

| 場所 | 今 | 直した後 |
|---|---|---|
| `lib.rs:30` | 入口の関数の表の `[`parse_args`]` の行 | 行を消す(公開の入口ではない。引数の解釈は「モジュールの構成」の表の `args` の行が述べている) |
| `lib.rs:24` の節の後(表の前か後) | (なし) | 1 文を足す:「公開するのは、この表の関数と、下の差し替え口(trait、本番の実装、`Deps`、`SignalError`)だけ。ほかはすべて crate の中。」 |
| `lib.rs:52` | `[`run_id`]` | `` `run_id` ``(リンクにしない) |
| `lib.rs:54` | `[`exit`]` | `` `exit` `` |
| `lib.rs:55` | `[`report`]` | `` `report` `` |
| `src/report/escape.rs:18` | 「`tests/escape_table.rs` と」 | 「単体テストの `src/tests/escape_table.rs` と」(コミット 2) |

- 完了の判定: `cargo doc --workspace --no-deps --locked` の cli の警告が 0 件。`grep -n "\[\`\(parse_args\|run_id\|exit\|report\)\`\]" crates/cli/src/lib.rs` が 0 件。

## 2. 狭めた項目を使うテスト(移す・変える。詳細は `functional-spec.md` §4。承認待ち)

検索(§7)と写しのコンパイルのエラー(§0.2)で網羅した。`conductor_cli::` の参照のある 19 ファイルのうち、公開に残す項目だけを使う `signals_order.rs`・`signals_real.rs` は変えない。

| ファイル(HEAD) | 件数 | 狭めた項目の使い方 | 扱い |
|---|---|---|---|
| `tests/args_grammar.rs` | 60 | `parse_args`、型 9 つ、`run_id::resolve_run_id`、`common::FakeEnv::with` | 60 件すべてを `src/tests/args_grammar.rs` へ移す(T1) |
| `tests/report_messages.rs` | 36 | `report::{DisplayEvent, Reporter, ResultDetail, StateContext}`、`Language`、`Level`、`UsageError`、`UsageErrorKind`、`common::{FakeEnv, FakeSink, state_error}`、`common::kinds` | 36 件すべてを移す(T2) |
| `tests/graph_report.rs` | 28 | `render_text`、`render_mermaid`、`escape_label`、`Language` | 28 件すべてを移す(T3) |
| `tests/graph_long_id.rs` | 2 | `render_text`、`Language` | 2 件すべてを移す(T3) |
| `tests/plan_report.rs` | 7 | `render_plan`、`Language` | 7 件すべてを移す(T4) |
| `tests/init_template.rs` | 7 | `render_template`、`Language` | 7 件すべてを移す(T5) |
| `tests/exit_codes.rs` | 7 | `exit::*` | 7 件すべてを移す(T6) |
| `tests/escape_table.rs` | 3 | `report::escape::{escape_full, escape_name}` | 3 件すべてを移す(T6) |
| `tests/run_id_arg.rs` | 1 | `RunIdArg`、`UsageErrorKind` | 1 件を移す(T6) |
| `tests/init_types.rs` | 3 | `CliCommand`、`CommandKind`、`InitOptions`、`InitTarget` | 3 件すべてを移す(T6) |
| `tests/docs_graph.rs` | 8 | 6 件が補助 `render`・本体で `render_text`、`Language` | 6 件を `src/tests/docs_graph.rs` へ移す。2 件は残る(T7。D3) |
| `tests/docs_dry_run.rs` | 2 | 1 件が `render_plan`、`Language` | 1 件を移す。1 件は残る(T7。D3) |
| `tests/docs_init.rs` | 2 | 1 件が `render_template`、`Language` | 1 件を移す。1 件は残る(T7。D3) |
| `tests/flow_init.rs` | 9 | 全件が補助の引数に `Language`、2 件が期待の値に `render_template` | **統合テストに残す**。`Language` はテストの写し(`common::Language`)に、`render_template(…)` の 4 か所は原本の写しから作る期待のひな形に(T8。D4、D5) |
| `tests/flow_validate.rs` | 13 | 1 件が `FakeEnv::for_language(conductor_cli::Language::En)` | 統合テストに残す。`common::Language::En` に(T8。D5) |
| `tests/common/mod.rs` | — | `Language`(`FakeEnv::for_language` の引数) | テストの言語 `Language`(写しの enum)を足し、`pub mod kinds;` を消す(T9。D5) |
| `tests/common/kinds.rs` | — | `UsageErrorKind` | `src/tests/common/kinds.rs` へ移す(使うのは `report_messages.rs` だけ。T9) |
| `tests/nfr_source.rs` | 6 | (狭めた項目は使わない) | `src/tests/` を OS 分岐と `panic!(` の検査の外にし、`lib.rs` の最後の宣言と、製品のファイル(`src/tests/` の外)に `#[path` が無いことを、同じテストで確かめる(テストの数は変えない。T10。D2) |
| `tests/signals_order.rs`、`signals_real.rs` | 1 + 1 | `Signals`、`SystemSignals`(残す) | 変えない |

- 移す: **162 件**(13 ファイル。うち 3 ファイルは一部)。統合テストに残して書き方を変える: `flow_init.rs` の 2 件(期待のひな形の書き方)と、`flow_init.rs` の 9 件・`flow_validate.rs` の 1 件の import の名前。消す: 0 件。足す: 0 件。名前を変える: 0 件(★ なし)。
- (記述との違い)初期の記述の `docs_*.rs` のうち、狭めた項目を使うのは `docs_graph.rs`・`docs_dry_run.rs`・`docs_init.rs` の一部(計 8 件)だけで、`docs_env.rs`・`docs_retry.rs` は使わない(変えない)。初期の記述に無かった `flow_init.rs`、`flow_validate.rs`、`graph_long_id.rs`、`tests/common/mod.rs`、`tests/common/kinds.rs` も狭めた項目を使う。
- (記述との違い)cli のソースの検査 `nfr_source.rs` は、src を再帰で読み、ファイル全体を製品のコードとみなす。移すテストの `#[cfg(unix)]`・`cfg!(unix)`(6 か所)と `panic!(`(7 か所)に当たるため、検査の範囲の直しが要る(D2)。

## 3. 単体テストの置き場所と、ソースの検査(D1、D2)

- 置き場所(D1。決定: 推奨のとおり。ユーザー、2026-11-24): `lib.rs` の**最後**に `#[cfg(test)] mod tests;` を置き、`src/tests/` のディレクトリに置く(converter と同じ形)。下位のファイル名は元の統合テストと同じにする。補助は `src/tests/common/`(`mod.rs`、`kinds.rs`)と `src/tests/docs_support.rs`(`functional-spec.md` §2.2)。
- ソースの検査(`tests/nfr_source.rs`。HEAD の `:23-70`、`:117-144`)の今の範囲: `src/` の下のすべての `.rs` を再帰で読み、各行の `//` 以降を除いて検査する。`unsafe`・`low_level`(`:80`)とネットワークの語(`:92`)は全ファイル、OS 分岐(`:117`)は `platform` を除く全ファイル、`panic!(`(`:136`)は `bin` を除く全ファイル。
- 目標(D2。決定: 検査の範囲を製品のコードに限る。推奨のとおり。ユーザー、2026-11-24。ユーザーは、規則「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」を製品のコードについての規則と読むことを認めた): 補助 `is_unit_test(path)`(`src/tests/` の下か)を足し、OS 分岐と `panic!(` の 2 つの検査で、`src/tests/` の下のファイルを除く。`unsafe`・`low_level`・ネットワークの語の検査は、`src/tests/` も今のまま対象にする。除いた範囲が本番に入らないことを確かめるため、`テスト専用の経路を実行ファイルに持たない` に、`lib.rs` の末尾が `#[cfg(test)]` と `mod tests;` の 2 行であること(改行は `\n` にそろえて比べる)と、製品のファイル(`src/tests/` の外)に `#[path` が無いこと(`src/tests/` を本番から読む道が無いこと。HEAD では `crates/cli/src` に `#[path` は 0 件)を確かめる 2 つの確かめを足す。テストの数は変えない。形は `functional-spec.md` §2.3。
- 説明: `nfr_source.rs:15-18`(モジュールの説明)に「単体テスト(`src/tests/`)は、OS 分岐と `panic!(` の検査の外」を足す。

## 4. 統合テストの補助(`tests/common/`、`tests/docs_support/`、`tests/bin_support/`)

| 補助 | 使う統合テスト(移したあと) | 扱い |
|---|---|---|
| `tests/common/mod.rs` の `FakeEnv`・`FakeSink`・`state_error` | 流れのテスト(`flow_*.rs`、`nfr_robust.rs`、`nfr_perf.rs`)が使い続ける | **残す**。移す `report_messages.rs`・`args_grammar.rs` のために、`src/tests/common/mod.rs` に写しを置く(D7)。写しの説明に「統合テストの `tests/common/mod.rs` の同じ名前のものの写し」と書く |
| `tests/common/mod.rs` の `FakeEnv::for_language(Language)` | `flow_init.rs`、`flow_validate.rs` | 残す。引数の型を、`common` に足すテストの言語 `Language { En, Ja }`(写しの enum)にする(D5) |
| `tests/common/mod.rs:289` のコメント(「`report_messages.rs` とも共有する」) | — | 「単体テストの `src/tests/report_messages.rs` も同じ写しを使う」に直す |
| `tests/common/kinds.rs` | `report_messages.rs` だけ | **移す**(`src/tests/common/kinds.rs`。`git mv`)。`tests/common/mod.rs` の `pub mod kinds;` を消す |
| `tests/docs_support/mod.rs` | `docs_env.rs`、`docs_retry.rs`、残る `docs_graph.rs`・`docs_dry_run.rs`・`docs_init.rs` | **残す**(変えない)。移す 8 件が使うのは `readme()` だけなので、`src/tests/docs_support.rs` に `readme()` の写しだけを置く(`panic!(` を含む `block_after_heading` は写さない) |
| `tests/bin_support/mod.rs`、`tests/signal_support/mod.rs` | 実バイナリ・シグナルのテスト | 変えない(移すテストは使わない) |

- `tests/common/mod.rs` の `#![allow(dead_code)]` は残す(流れのテストのファイルごとに使う補助が違う)。`src/tests/common/mod.rs` には付けない(写しで、すべての項目が使われ、警告が出ないことを確かめた。途中のコミットでは、使う項目だけを置く。`functional-spec.md` §8)。

## 5. 仕様書(`docs/spec/rust`)

直す節と語句の一覧(各コミットで直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない。版を上げないので、「v0.9.8 までは…」などの版の注記は足さず、今の形を書く)。

| # | ファイル:行(HEAD) | 節 | 今の語句 | 直した後 | コミット |
|---|---|---|---|---|---|
| S1 | rust `02:137` | 2.4 `conductor-cli` | 「再公開(`lib.rs`): `parse_args`、`Deps`、…、`UsageErrorKind`。公開モジュール: `exit`、`report`(…)、`run_id`(`args`、`deps`、`panic_hook`、`types` は非公開で、中の公開の項目は上の再公開で出る)。」 | 「再公開(`lib.rs`): `Deps`、`Env`、`Runtime`、`SignalError`、`Signals`、`Sink`、`StdSink`、`SystemEnv`、`SystemRuntime`、`SystemSignals`、`install_panic_hook`。公開の関数は `run_cli`、`run_cli_with`。公開するのは、`main` が使うもの(`run_cli`、`StdSink`、`SystemEnv`、`install_panic_hook`)と、依存の注入の継ぎ目(4 章)と、継ぎ目に出る型 `SignalError` だけで、公開モジュールは無い(`args`、`exit`、`report`、`run_id`、`types` などは、すべて crate の中)。」。続く「v0.9.6 までは …(外から使われないので crate の中に狭めた)。」の文は残す | 12 |
| S2 | rust `02:143` | 2.4 の関数の表 | `` `parse_args` `` | `` `parse_args`(`args`。crate の中) `` | 12 |
| S3 | rust `02:144-145` | 同 | `` `exit_code_for` ``、`` `signal_exit_code` `` | `` `exit_code_for`(`exit`。crate の中) ``、`` `signal_exit_code`(同) `` | 12 |
| S4 | rust `02:146` | 同 | `` `resolve_run_id`(`run_id`) `` | `` `resolve_run_id`(`run_id`。crate の中) `` | 12 |
| S5 | rust `02:148` | 同 | `` `escape_name` / `escape_full`(`report::escape`) `` | `` `escape_name` / `escape_full`(`report::escape`。crate の中) `` | 12 |
| S6 | rust `02:150` | 同 | 「終了コードの定数(`exit`):」 | 「終了コードの定数(`exit`。crate の中):」 | 12 |
| S7 | rust `02:257` | 4 章 | 「テストの偽の実装(`crates/cli/tests/common/mod.rs`):」 | 「テストの偽の実装(`crates/cli/tests/common/mod.rs`。単体テストの `crates/cli/src/tests/common/mod.rs` は、`FakeEnv`・`FakeSink` の写しを持つ):」 | 9(`FakeSink` の写しが入るコミット) |
| S8 | rust `02:348` | 5.4 `crates/cli/src` | `lib.rs` の責務「公開の再公開、`run_cli`、`run_cli_with`」 | 「差し替え口と `install_panic_hook` の再公開、`run_cli`、`run_cli_with`。最後に単体テストの宣言 `#[cfg(test)] mod tests;`」 | 1 |
| S9 | rust `02:372` の後 | 5.4 の表 | (行なし) | 行を足す。ファイルの列は `tests/`(`mod.rs`、`common/`、`docs_support.rs` と、元の統合テストと同じ名前のファイル)、責務の列は「クレートの内部(引数の解析、表示、run ID、終了コード)を直接確かめる単体テスト(`lib.rs` の最後の `#[cfg(test)] mod tests;`)」 | 1 |
| S10 | rust `07:28-29` | 1 章の末尾 | 「`platform` モジュールの外に置かない(ソース検索のテスト `nfr_source.rs` が確かめる。8 章)。」 | 「`platform` モジュールの外に置かない(cli のテストのときだけの単体テスト `src/tests/` を除く。ソース検索のテスト `nfr_source.rs` が確かめる。8 章)。」 | 1 |
| S11 | rust `07:400-401` | 8.2 | 「`conductor-cli`: … は、`src/platform/` の外にない(`color_supported` の `cfg!(unix)` も `platform/mod.rs` の中)。」 | 文の後ろに「単体テスト `src/tests/`(`lib.rs` の最後の `#[cfg(test)] mod tests;` からだけ読み、本番の成果物に入らない)は、この検査と `panic!(` の検査の外(`unsafe`・ネットワークの検査は対象)。」を足す | 1 |
| S12 | rust `08:585` | 9 章 | 「期待値の表(`tests/escape_table.rs`。…)」 | 「期待値の表(単体テストの `src/tests/escape_table.rs`。…)」 | 2 |
| S13 | rust `08:605` | 9 章 | 「`tests/escape_table.rs` と converter の単体テストの同じ表で揃える」 | 「cli の単体テスト `src/tests/escape_table.rs` と converter の単体テストの同じ表で揃える」 | 2 |
| S14 | rust `11:162` | 3.3 の表 | 「表示のエスケープの表」のテストの列 `cli/tests/escape_table.rs` | `cli/src/tests/escape_table.rs` | 2 |
| S15 | rust `11:166` | 3.3 の表 | 「`src/bin/` 以外に `panic!(` がない」 | 「`src/bin/` と単体テスト `src/tests/` の外に `panic!(` がない(`src/tests/` は `lib.rs` の最後の `#[cfg(test)] mod tests;` からだけ読む)」 | 1 |
| S16 | rust `11:173` | 3.4 | 「runtime と cli のそれぞれの `src/platform/` の中にだけ置く」 | 「runtime と cli のそれぞれの `src/platform/` の中にだけ置く(cli の単体テスト `src/tests/` を除く)」 | 1 |
| S17 | rust `11:178` | 3.5 | `cli/tests/nfr_source.rs`(…、OS 分岐の場所、テスト専用の経路なし、…) | 「…、OS 分岐の場所、テスト専用の経路なし(この 2 つは単体テスト `src/tests/` を除き、`src/tests/` が `#[cfg(test)]` の宣言からだけ読まれることを確かめる)、…」 | 1 |
| S18 | rust `11:188` | 4.1 方針 | 「(各 `src/*.rs` の `#[cfg(test)]`。converter は、途中の表現を比べる単体テストを `src/tests/` にも置く)」 | 「(各 `src/*.rs` の `#[cfg(test)]`。converter は、途中の表現を比べる単体テストを `src/tests/` にも置く。cli は、クレートの内部を直接確かめる単体テストを `src/tests/` に置く)」 | 1 |
| S19 | rust `11:266-274` | 4.2 cli の表 | `args_grammar.rs`、`exit_codes.rs`、`run_id_arg.rs`、`escape_table.rs`、`report_messages.rs`、`graph_report.rs`、`graph_long_id.rs`、`plan_report.rs`、`init_template.rs`・`init_types.rs` の行 | 行を消す(移したコミットで 1 行ずつ。行の文は S24 の単体テストの表へそのまま移す) | 1〜9 |
| S20 | rust `11:279` | 4.2 cli の表 | `docs_env.rs`、`docs_graph.rs`、`docs_dry_run.rs`、`docs_init.rs` の目的「README の説明の検査(必須の語句、版の表記 `v0.9.0 以降`、例が実際の読み取り・出力と一致)」 | 「README の説明の検査(必須の語句、版の表記 `v0.9.0 以降`、`.env` の例が実際の読み取りと一致)。`graph`・計画・ひな形の例と実際の出力を比べるテストは単体テストの同じ名前のファイル」 | 10 |
| S21 | rust `11:287` | 4.2 cli の表 | `common/mod.rs` の「`FakeEnv::for_language`」 | 「`FakeEnv::for_language`(引数はテストの言語 `Language`。ロケールを選ぶためだけの写しの enum)」 | 11 |
| S22 | rust `11:288` | 4.2 cli の表 | `common/kinds.rs` の行 | 行を消す(S24 の表へ移す) | 9 |
| S23 | rust `11:293` | 4.2 の表の後 | 「cli の src の単体テストは、`panic_hook.rs` の 2 件(…)だけ。」 | 「cli の src の単体テストは、`panic_hook.rs` の 2 件(…)と、下の `src/tests/` の単体テスト。」(フックの設定の文は残す) | 1 |
| S24 | rust `11:293` の後 | 4.2 | (なし) | 「cli の単体テスト(`crates/cli/src/tests/`。`#[cfg(test)]`)」の表を足す: `mod.rs`(下位のモジュールの宣言)、`common/mod.rs`(`FakeEnv`・`FakeSink`・`state_error` の写し)、`common/kinds.rs`(S22 の文)、`docs_support.rs`(`readme` の写し)と、S19 の各行(文はそのまま)、`docs_graph.rs`・`docs_dry_run.rs`・`docs_init.rs`(README の図・計画・ひな形の例が実際の出力と一致)。注記:「このディレクトリは、`tests/nfr_source.rs` の OS 分岐と `panic!(` の検査の外(3.3、3.5 節)」。コミット 1 で `mod.rs` と `exit_codes.rs` の行を作り、2〜10 で行を足す | 1〜10 |

- 件数: rust 11 の 4.2 は、ファイルごとのテストの件数を書かない。仕様書にテストの件数(`1,152` など)の記述は無い(`grep -rn "1,15[0-9]" docs/spec/rust docs/spec/language-agnostic` が 0 件)。直す件数は無い。
- 変えない記述: rust `04:218`(`parse_args` が引数の誤りを返す、という実装の説明。公開の記述ではない)、rust `08:624`(「テスト(`graph_report.rs`、`plan_report.rs`)の期待値のとおり」。ファイル名は同じで、場所を書いていない)、rust `02:246`(`run_cli_with` に偽物を渡せる。継ぎ目は残る)、rust `02:370`(`report/init_template.rs` の責務。src のファイル)、rust `10:112`(`panic_hook.rs` の単体テスト)、rust `07:5`、`07:432-433`(`signals_*`、`binary_signal.rs`。変えないテスト)。
- language-agnostic: 公開範囲の記述は無い。11 章の受け入れテストの「根拠」の欄は、参照実装のテストを `ファイル名::テスト名` で書く(`args_grammar::…`、`report_messages::…` など。場所(`tests/` か `src/tests/` か)は書かない)。移す先のファイル名とテスト名を元と同じにする(D1)ので、根拠の欄の名前はそのまま正しく、**直さない**(D6。決定: 直さない。推奨のとおり。ユーザー、2026-11-24)。区分「単体」は受け入れテストの確かめ方の区分で、参照実装のテストの置き場所の区分ではない(converter の intent と同じ判断)。
- 完了の判定(直した後): `functional-spec.md` §7 の検索。

## 6. 範囲の外で、触らないもの(確かめた)

- 依存の注入の継ぎ目の形(`Runtime`・`Sink`・`Env`・`Signals` のメソッドと引数、`Deps` の項目、`SignalError` の変種)。`run_cli`・`run_cli_with` のシグネチャと本体の処理。
- cli のほかの src(`args.rs`、`ctx.rs`、`deps.rs`、`report/*.rs` など)。可視性の書き換え(`pub` → `pub(crate)`)もしない(§1.1)。`src/report/escape.rs` の説明の 1 行だけ直す(§1.3)。
- `src/bin/conductor-cli-test-helper.rs`(cli のライブラリを使わない)、`src/main.rs`(公開に残す 4 つだけを使う)。
- 統合テストの補助 `tests/bin_support/`、`tests/signal_support/`、`tests/docs_support/`。流れ・実バイナリ・デモ・非機能のテスト(§2 の表の外のファイル)。
- core、runtime、converter のコードとテスト。`Cargo.toml`、`Cargo.lock`、CI の設定(`.github/workflows/*`)。README。

## 7. 網羅に使った検索のコマンド

```sh
# 今の公開(lib.rs)と main の使用
grep -n "^pub \|^    [A-Z].*,$" crates/cli/src/lib.rs
grep -n "conductor_cli" crates/cli/src/main.rs crates/cli/src/bin/*.rs
# 継ぎ目の引数・戻り値・項目に出る型(cli の型は SignalError だけ)
grep -n "fn \|pub " crates/cli/src/deps.rs
# 狭めた項目をテストから使う所(複数行の use を含む)
grep -rn -A4 "use conductor_cli" crates/cli/tests
grep -rn "conductor_cli::" crates/cli/tests
# 補助の使用
grep -rn "mod common\|mod docs_support\|common::\|docs_support::\|kinds::\|for_language\|state_error(" crates/cli/tests
# 移すテストの中の、ソースの検査に当たる書き方
grep -n "panic!(\|unsafe\|low_level\|std::net\|curl\|hyper\|tokio\|cfg" crates/cli/tests/{args_grammar,report_messages,graph_report,graph_long_id,plan_report,init_template,exit_codes,escape_table,run_id_arg,init_types,docs_graph,docs_dry_run,docs_init}.rs crates/cli/tests/common/kinds.rs
# テストの件数と名前
grep -c "#\[test\]" crates/cli/tests/*.rs
cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$"
# 写しの木(本番だけを変えた段階)で、エラーの場所を出す
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
# 仕様書
grep -rn "parse_args\|pub mod\|公開\|args_grammar\|report_messages\|graph_report\|plan_report\|init_template\|exit_codes\|escape_table\|run_id_arg\|init_types\|graph_long_id\|docs_graph\|docs_dry_run\|docs_init\|kinds\.rs\|FakeEnv\|for_language\|nfr_source\|cli/tests\|src/tests" docs/spec/rust
grep -rn "args_grammar\|report_messages\|graph_report\|plan_report\|init_template\|escape_table\|run_id_arg\|init_types\|graph_long_id\|docs_graph\|docs_dry_run\|docs_init\|flow_init\|conductor_cli\|tests/" docs/spec/language-agnostic
git grep -n "conductor_cli\|parse_args\|escape_table\|args_grammar\|report_messages" -- ':!aidlc' ':!crates' ':!docs/spec'   # 0 件(README・CI に参照なし)
```
