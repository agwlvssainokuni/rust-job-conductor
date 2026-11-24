# 機能の仕様(Functional Spec): cli のライブラリの公開範囲を狭める

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 A)。**変える場所の正確な一覧(公開の項目ごとの判定 P1〜P13・N1〜N15、目標の形、説明の直し、テストの使用、補助の扱い、直す仕様書の節と語句 S1〜S24、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD c35ada6 で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの変更(§4 の T 表)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない。本番に、テストのための要素(`pub`、`#[doc(hidden)]`、テストのための引数や関数)を足さない**(ユーザーの判断)。`#[cfg(test)]` の中は可。本番で足すのは、`lib.rs` の最後の `#[cfg(test)] mod tests;`(単体テストの宣言。本番の成果物には入らない)だけ。
- 公開に残すのは、`main` が使う 4 つ(`run_cli`、`StdSink`、`SystemEnv`、`install_panic_hook`)、依存の注入の継ぎ目(`run_cli_with`、`Deps`、`Runtime`・`Sink`・`Env`・`Signals`、`SystemRuntime`・`SystemSignals`)、継ぎ目を使うのに要る型(`SignalError` の 1 つ)の **13 項目**だけ。ほかの **15 項目**(`parse_args`、型 11 個、`pub mod exit`・`report`・`run_id`)は crate の中に狭める [Q3]。
- 狭めた項目を直接使うテスト(**162 件**)は、**src の単体テストへ移す**(Q1: A)。**今の確かめ方と期待の値は変えない**。移すときに変わるのは、ファイルの先頭(`mod common;` などと import)、`conductor_cli::…` のパスを `crate::…` にすることだけ(§4)。
- **移すテストの名前は変えない**(★ なし)。パスは変わる(統合テストのバイナリ `args_grammar` の `X` から、単体テストの `tests::args_grammar::X` へ)。移す先のファイル名を元の統合テストと同じにし、言語に依らない仕様書の根拠の欄(`args_grammar::X` など)をそのまま正しく保つ(D1、D6)。
- 公開の入口 `run_cli_with` などで確かめているテストは `tests/` に残す。その中で、狭めた項目を**期待の値を作るためや、偽物の言語を選ぶためだけに**使うもの(`flow_init.rs`、`flow_validate.rs`)は、統合テストに残し、同じ値を写しで書く形に変える(D4、D5。Q2 の「書き方を変える」)。
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。版は上げない(`0.9.8` のまま)[desc]。
- 小さなコミットの並びで、どのコミットでもビルド、`clippy -D warnings`、テストが通る。**テストを先に移し、移し終えてから公開を狭める**(先に狭めると統合テストのコンパイルが止まる)[Q2] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は小さなコミットで進め、結果は最後にまとめて見せる [Q2]。
- 消すテストは 0 件。足すテストは 0 件。ソースの検査 `nfr_source.rs` の 2 件は、検査の範囲を直す(テストの数は変わらない。D2)。
- 新しい依存を足さない [memory:M5]。`unsafe` と src の `cfg(unix)`/`cfg(windows)` を本番に足さない(移すテストの `#[cfg(unix)]` は `src/tests/` の中だけ。D2)。cli は `#![forbid(unsafe_code)]` のまま。MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。
- **ソースの検査を弱めない**: cli の `tests/nfr_source.rs` は `src/` を再帰で読み、ファイル全体を検査する。`src/tests/` を、OS 分岐と `panic!(` の 2 つの検査の外にする代わりに、`src/tests/` が `lib.rs` の最後の `#[cfg(test)] mod tests;` からだけ読まれる(本番の成果物に入らない)ことを、末尾の宣言と、製品のファイルに `#[path` が無いことの 2 つで、同じテストで確かめる。`unsafe`・`low_level`・ネットワークの語の検査は、`src/tests/` も対象のまま(D2。BR3.2)。
- 仕様書は `docs/spec/rust` を直す。`docs/spec/language-agnostic` は直さない(公開範囲の記述が無く、根拠の欄の名前は D1 の形で正しいまま。D6)。凍結した仕様書は触らない。版の注記は足さない。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

| 項目 | 本番から消すもの・変えるもの | 本番の形の変わり方 | テストの代わり |
|------|------------------------------|--------------------|----------------|
| 1 | `lib.rs` の `pub mod exit`・`report`・`run_id` を `mod` に。`pub use args::parse_args` と `pub use types::{…}`(11 個)を消し、`run_cli_with` が使う `parse_args`・`CommandKind`・`ParsedArgs` を非公開の `use` で取り込む。説明の非公開の項目へのリンク 4 つ(`inventory.md` §1.3) | 公開は 13 項目。本番の処理は変わらない。最後に `#[cfg(test)] mod tests;` | 単体テスト(`src/tests/`)が `crate::args::parse_args`、`crate::types::…`、`crate::report::…`、`crate::exit::…`、`crate::run_id::…` を直接使う |
| 2 | `src/report/escape.rs` の説明の `tests/escape_table.rs` | 文だけ | — |
| 3 | (本番の変更なし)`tests/nfr_source.rs` の検査の範囲 | — | `src/tests/` を OS 分岐と `panic!(` の検査の外に。`lib.rs` の最後の宣言と、製品のファイルに `#[path` が無いことを確かめる |

### 2.1 項目 1 の形(`lib.rs`)

`inventory.md` §1.2。下位モジュールの中の `pub` の項目は `pub` のまま(親が非公開なので外から見えない。差を小さくするため `pub(crate)` に書き換えない)。写しで、本番で使われなくなる項目が 0 件(`dead_code` の警告なし)であることを確かめた。

### 2.2 単体テストの置き場所(D1。決定: `src/tests/` のディレクトリ。推奨のとおり。ユーザー、2026-11-24)

```text
crates/cli/src/lib.rs                 … 最後に `#[cfg(test)] mod tests;`(前に 1 行のコメント)
crates/cli/src/tests/mod.rs           … 下位のモジュールの宣言だけ
crates/cli/src/tests/common/mod.rs    … FakeEnv・FakeSink・state_error(統合テストの tests/common/mod.rs の写し)
crates/cli/src/tests/common/kinds.rs  … 元の tests/common/kinds.rs(git mv。import 1 行だけ変わる)
crates/cli/src/tests/docs_support.rs  … readme()(統合テストの tests/docs_support/mod.rs の写し)
crates/cli/src/tests/args_grammar.rs     … 元の tests/args_grammar.rs(60 件)
crates/cli/src/tests/report_messages.rs  … 元の tests/report_messages.rs(36 件)
crates/cli/src/tests/graph_report.rs     … 元の tests/graph_report.rs(28 件)
crates/cli/src/tests/graph_long_id.rs    … 元の tests/graph_long_id.rs(2 件)
crates/cli/src/tests/plan_report.rs      … 元の tests/plan_report.rs(7 件)
crates/cli/src/tests/init_template.rs    … 元の tests/init_template.rs(7 件)
crates/cli/src/tests/exit_codes.rs       … 元の tests/exit_codes.rs(7 件)
crates/cli/src/tests/escape_table.rs     … 元の tests/escape_table.rs(3 件)
crates/cli/src/tests/run_id_arg.rs       … 元の tests/run_id_arg.rs(1 件)
crates/cli/src/tests/init_types.rs       … 元の tests/init_types.rs(3 件)
crates/cli/src/tests/docs_graph.rs       … 元の tests/docs_graph.rs のうち 6 件と補助(graph_example、render、jobs)
crates/cli/src/tests/docs_dry_run.rs     … 元の tests/docs_dry_run.rs のうち 1 件
crates/cli/src/tests/docs_init.rs        … 元の tests/docs_init.rs のうち 1 件
```

`src/tests/mod.rs` の形(最後の形。途中のコミットでは、移したものだけを宣言する):

```rust
//! クレートの内部(引数の解析、表示、run ID、終了コード)を直接確かめる単体テスト。
//!
//! `lib.rs` の最後の宣言から読む。テストのときだけコンパイルし、本番の成果物には入らない。
//! 公開の入口(`run_cli_with` と差し替え口)だけで確かめるテストは、`tests/` の統合テストにある。

mod common;
mod docs_support;

mod args_grammar;
mod docs_dry_run;
mod docs_graph;
mod docs_init;
mod escape_table;
mod exit_codes;
mod graph_long_id;
mod graph_report;
mod init_template;
mod init_types;
mod plan_report;
mod report_messages;
mod run_id_arg;
```

- 下位のモジュールは `use super::common;`、`use super::common::{FakeEnv, FakeSink};`、`use super::docs_support::readme;`、`use crate::…;` で使う(親の非公開の項目は子から見える)。本番の項目の可視性は変えない。
- `src/tests/common/mod.rs` は、`crate::deps::{Env, Sink}` と `crate::types::Language` を使い、`FakeEnv`(項目、`locale_read`、`color_read`、`with`、`for_language`、`impl Env`)、`FakeSink`(項目、`out_text`、`err_text`、`impl Sink`)、`state_error` を、統合テストの `tests/common/mod.rs` から一字も変えずに写す。`#![allow(dead_code)]` は付けない(写しで、すべて使われることを確かめた)。
- `src/tests/docs_support.rs` は `readme()` だけ(`env!("CARGO_MANIFEST_DIR")` から `../../README.md` を読む。統合テストと同じ。マクロで `std::env` ではない)。
- `src/tests/` のファイルには `#[cfg(test)]` を書かない(宣言の側で付いている)。

### 2.3 項目 3 の形(`tests/nfr_source.rs`。D2。決定: 検査の範囲を製品のコードに限る。推奨のとおり。ユーザー、2026-11-24。ユーザーは、規則「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」を製品のコードについての規則と読むことを認めた)

```rust
/// 単体テスト(`src/tests/` の下)のファイルか。`lib.rs` の最後の `#[cfg(test)] mod tests;` から
/// 読むテストのコードで、本番の成果物には入らない。
fn is_unit_test(path: &Path) -> bool {
    path.starts_with(src_dir().join("tests"))
}

// os_ごとの分岐は_platform_の中だけ: 除く条件に `|| is_unit_test(path)` を足す
// テスト専用の経路を実行ファイルに持たない: 除く条件に `|| is_unit_test(path)` を足し、最後に次を足す
    // 検査の外にする src/tests/ は、テストのときだけ読む(lib.rs の最後の宣言)
    let lib = std::fs::read_to_string(src_dir().join("lib.rs"))
        .expect("読める")
        .replace("\r\n", "\n");
    assert!(
        lib.trim_end().ends_with("#[cfg(test)]\nmod tests;"),
        "src/tests/ は #[cfg(test)] の宣言からだけ読む"
    );
    // 製品のファイル(src/tests/ の外)から、#[path] で src/tests/ を読む道が無い
    let paths = find(&|line| line.contains("#[path"), &|path| is_unit_test(path));
    assert!(paths.is_empty(), "{paths:#?}");
```

- 足すのは、同じテストの中の 2 つの確かめ(`lib.rs` の末尾の宣言と、`src/tests/` の外に `#[path` が無いこと)だけ。テストの数は変えない(6 件)。2 つで、`src/tests/` が `#[cfg(test)]` の宣言からだけ読まれ、本番の成果物に入らないことを確かめる(`find` は `//` 以降を除いて読むので、コメントの中の文字には当たらない)。HEAD の src に `#[path` は 0 件(`grep -rn "#\[path" crates/cli/src`)。
- テスト名は変えない。`クレートは_forbid_unsafe_code_を宣言している`、`src_に_unsafe_と_low_level_がない`、`src_にネットワークのコードがない`、`依存に_c_のコンパイルを要するものとネットワークのライブラリがない` は変えない(`src/tests/` も対象のまま)。
- 写しで、この形で 6 件すべてが通り、`src/tests/` を除かないと 2 件が失敗する(OS 分岐 6 か所、`panic!(` 7 か所)ことを確かめた(`inventory.md` §0.2)。

## 3. 実物で確かめて初期の記述と違った点、範囲から外したもの

- **記述と違った点**(実物に従った):
  - 「継ぎ目を使うのに要る型」に当たる cli の型は `SignalError` の 1 つだけ(`inventory.md` §1.1)。
  - 狭めた項目を使うテストは、初期の記述の 9 ファイルと `docs_*.rs` の一部(`docs_graph.rs`・`docs_dry_run.rs`・`docs_init.rs` の 8 件。`docs_env.rs`・`docs_retry.rs` は使わない)のほか、`graph_long_id.rs`、`flow_init.rs`、`flow_validate.rs`、`tests/common/mod.rs`、`tests/common/kinds.rs`(`inventory.md` §2)。
  - cli のソースの検査は `src/` を再帰で読み、`#[cfg(test)]` で切らずにファイル全体を検査する(converter とは違う)。移すテストの `#[cfg(unix)]`・`cfg!(unix)`・`panic!(` が当たる(D2)。
  - 本番で使われなくなる項目は 0 件で、消す本番のコードは無い。
- **範囲から外したもの**: core・runtime・converter の公開範囲。継ぎ目の形。下位モジュールの中の `pub` を `pub(crate)` に書き換えること。言語に依らない仕様書(D6)。

## 4. 移す・変える・消すテストの一覧(T1〜T10。ユーザーが設計の承認で認めた、2026-11-24)

Q1: A に従い、**確かめる中身(呼ぶ関数、入力、期待の値、誤りの種類と対象)は変えない**。移すテストの本体の行は、`conductor_cli::` のパスを `crate::` にする所のほかは、一字も変えない(`cargo fmt` が 1 行に収める所を除く)。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §7。T 表は、`conductor_cli::` のすべての参照(複数行の `use` を含む)と、補助(`common::`、`docs_support::`、`kinds::`、`for_language`)の参照を `crates/cli` の全体で検索して作り、さらに **HEAD の写しの木で本番のコードだけを変えてコンパイルし、エラーの出たファイル(17 ファイル、62 件)がすべて T 表にあること**を確かめた(`inventory.md` §0.2)。

```sh
grep -rn -A4 "use conductor_cli" crates/cli/tests
grep -rn "conductor_cli::" crates/cli/tests
grep -rn "mod common\|mod docs_support\|common::\|docs_support::\|kinds::\|for_language" crates/cli/tests
grep -c "#\[test\]" crates/cli/tests/*.rs
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
```

core・runtime・converter のテストの参照は 0 件。

**移す・残すの規則(D3)**: 本体か、そのファイルの補助で、**狭めた関数を呼ぶか、狭めた型の値を作って確かめる**テストは移す。狭めた項目を使わないテストは、同じファイルにあっても統合テストに残す。偽物の言語を選ぶためだけに `Language` を使うテストと、期待のひな形を作るためだけに `render_template` を使うテスト(`flow_init.rs`、`flow_validate.rs`)は、公開の入口 `run_cli_with` で確かめているので統合テストに残し、写しを使う形に変える(D4、D5)。

### 4.1 T1〜T6: ファイルごと移す(10 ファイル、154 件。`git mv` で移してから直す)

| ID | 元(HEAD) | 件数 | 移す先 | 変える行(本体の確かめは変えない) |
|----|-----------|------|--------|----------------------------------|
| T1 | `tests/args_grammar.rs` | 60 | `src/tests/args_grammar.rs` | `:17` の `mod common;` を消す。`:21-24` の `use conductor_cli::{…, parse_args};` を `use super::common;`、`use crate::args::parse_args;`、`use crate::types::{CliCommand, CommandKind, GraphFormat, InitOptions, InitTarget, ParsedArgs, RunIdArg, UsageError, UsageErrorKind};` に。`:620` の `conductor_cli::run_id::resolve_run_id` を `crate::run_id::resolve_run_id` に(差 10 行) |
| T2 | `tests/report_messages.rs` | 36 | `src/tests/report_messages.rs` | `:17` の `mod common;` を消す。`:21-27` の import を `use super::common;`、`use super::common::kinds::{…};`、`use super::common::{FakeEnv, FakeSink};`、`use crate::report::{DisplayEvent, Reporter, ResultDetail, StateContext};`、`use crate::types::{Language, Level, UsageError, UsageErrorKind};` に(差 11 行) |
| T3 | `tests/graph_report.rs`、`tests/graph_long_id.rs` | 28 + 2 | `src/tests/graph_report.rs`、`graph_long_id.rs` | `use conductor_cli::Language;` と `use conductor_cli::report::…;` を `use crate::report::…;` と `use crate::types::Language;` に(差 6 行、4 行) |
| T4 | `tests/plan_report.rs` | 7 | `src/tests/plan_report.rs` | 同じく import の 2 行(差 4 行) |
| T5 | `tests/init_template.rs` | 7 | `src/tests/init_template.rs` | 同じく import の 2 行(差 4 行) |
| T6 | `tests/exit_codes.rs`、`escape_table.rs`、`run_id_arg.rs`、`init_types.rs` | 7 + 3 + 1 + 3 | `src/tests/` の同じ名前 | import の 1 行を `use crate::exit::*;`、`use crate::report::escape::{escape_full, escape_name};`、`use crate::types::{RunIdArg, UsageErrorKind};`、`use crate::types::{CliCommand, CommandKind, InitOptions, InitTarget};` に(各差 2 行) |

- テスト名は、各ファイルの `#[test]` の関数名のとおり(一覧は `grep -n "^fn " crates/cli/tests/<ファイル>`。154 件すべて変えない)。
- `args_grammar.rs` の `#[cfg(unix)]` の 4 か所(`:316`、`:347`、`:601`、`:1120`。中の `use std::os::unix::…` を含む)と `panic!(` の 5 か所、`report_messages.rs` の `cfg!(unix)` の 2 か所(`:701`、`:748`)、`init_template.rs` の `panic!(` の 2 か所(`:136`、`:141`)は、そのまま移す(D2 で検査の範囲を直す)。
- `init_template.rs` の原本 `ORIGINAL_JA`・`ORIGINAL_EN` とモジュールの説明(「期待値は、機能の仕様の原本から書いたもので、実装の出力から作っていない」)はそのまま。

### 4.2 T7: 一部を移す(3 ファイル、8 件。D3)

| 元(HEAD) | 行 | テスト名(変えない) | 扱い |
|-----------|----|--------------------|------|
| `tests/docs_graph.rs` | 41 | `readme_の図の例は_実際の出力と一致する` | 移す |
| 同 | 121 | `readme_の同じ行き先と両方の飛び越しの例は_実際の出力と一致する` | 移す |
| 同 | 133 | `readme_の分岐の例は_実際の出力と一致する` | 移す |
| 同 | 141 | `readme_の交差の例は_実際の出力と一致する` | 移す |
| 同 | 152 | `readme_の交差に同じ行き先が混ざる例は_実際の出力と一致する` | 移す |
| 同 | 186 | `readme_の両方の飛び越しと交差が混ざる例は_実際の出力と一致する` | 移す |
| 同 | 22、163 | `readme_に_graph_の必須の語句と版の表記がある`、`readme_の辺の一覧に切り替わる形は_4つの種類の外の形だけを述べる` | **統合テストに残す**(変えない) |
| `tests/docs_dry_run.rs` | 40 | `readme_の計画の例は_実際の出力と一致する` | 移す |
| 同 | 22 | `readme_に_dry_run_の必須の語句と版の表記がある` | **残す** |
| `tests/docs_init.rs` | 42 | `readme_のひな形の例は_実際の日本語のひな形と一致する` | 移す |
| 同 | 24 | `readme_に_init_の必須の語句と版の表記と終了コード_69_がある` | **残す** |

- 移す先(`src/tests/docs_graph.rs` など): モジュールの説明(元の 1 行)の後ろに「README の例と実際の出力を比べる単体テスト。必須の語句の検査は統合テストの同じ名前のファイル」を足す。`mod docs_support;` と `use docs_support::{…};` を `use super::docs_support::readme;` に。`conductor_cli::report::graph_text::render_text(conductor_cli::Language::En, …)` を `crate::report::graph_text::render_text(crate::types::Language::En, …)` に(`docs_graph.rs` の 2 か所。`render` の中の 1 か所は `cargo fmt` で 1 行になる)、`docs_dry_run.rs` の `conductor_cli::report::plan_text::render_plan(conductor_cli::Language::Ja, &plan)` を `crate::…` に、`docs_init.rs` の import の 2 行を `use crate::report::init_template::render_template;`、`use crate::types::Language;` に。補助 `graph_example`・`render`・`jobs` は `docs_graph.rs` の移す先へ一緒に移る(残る 2 件は使わない)。
- 残る統合テスト: 移すテストを消し、モジュールの説明に「図(計画・ひな形)の例と実際の出力を比べるテストは、単体テスト(`src/tests/<同じ名前>`)にある」を足す。`use docs_support::{assert_docs_check_words, assert_readme_has, readme};` は残るテストが使う。`conductor_cli` の import は無くなる。

### 4.3 T8: 統合テストに残し、書き方を変える(`flow_init.rs`、`flow_validate.rs`。D4、D5)

| ファイル | 変える行 | 変えた後 | 期待の値 |
|----------|----------|----------|----------|
| `tests/flow_init.rs` | `:22` の `use common::{FakeEnv, FakeSignals, FakeSink, ScriptedRuntime, SignalMode, run_with_fakes};`、`:23` `use conductor_cli::Language;`、`:24` `use conductor_cli::report::init_template::render_template;` | `use common::{FakeEnv, FakeSignals, FakeSink, Language, ScriptedRuntime, SignalMode, run_with_fakes};`(`cargo fmt` で折り返す)。`render_template` の import を消す | — |
| 同 | (なし。import の後) | ひな形の原本 `ORIGINAL_JA`・`ORIGINAL_EN`(`init_template.rs` の原本と同じ文字列。説明に「単体テストの `src/tests/init_template.rs` の原本と同じ」)と、補助 `expected_template(lang, name)`(原本の `name = "my-pipeline"` を `name = "<name>"` に 1 回だけ置き換える。説明に「`name` はエスケープの要らない名前だけ」)を足す | — |
| 同 | `:75`、`:87`(`既定の名前とパスで_ひな形が書かれ_標準エラーに作成の1行`)、`:187`、`:198`(`config_ハイフンは標準出力にひな形だけを出し_force_を無視し_ファイルを作らない`)の `render_template(Language::…, …)` | `expected_template(Language::…, …)` | **同じバイト列**(`render_template(En, "my-pipeline")` = `ORIGINAL_EN`、`(Ja, "my-pipeline")` = `ORIGINAL_JA`、`(Ja, "夜間")` = `ORIGINAL_JA` の名前を `夜間` にしたもの。「夜間」はエスケープされない)。写しで 9 件が通ることを確かめた |
| `tests/flow_validate.rs` | `:172` の `FakeEnv::for_language(conductor_cli::Language::En)` | `FakeEnv::for_language(common::Language::En)` | 同じ(`LANG=en_US.UTF-8` だけを持つ環境) |

- `flow_init.rs` のほかの 7 件と補助 `run_with`・`run`(引数 `lang: Language`)は、`Language` が `common::Language` になるだけで、本体は変わらない。
- 名前を変えるテストは無い。

### 4.4 T9: 補助の変更

| ID | ファイル | 変える内容 |
|----|----------|------------|
| T9a | `tests/common/mod.rs` | `:18` の `pub mod kinds;` を消す(コミット 9)。`:27-29` の import から `Language` を消し(`cargo fmt` で 1 行になる)、`// ---- Env ----` の後に、テストの言語 `#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Language { En, Ja }`(説明「偽の環境のロケールを選ぶ、テストの言語(`FakeEnv::for_language` の引数)」)を足す(コミット 11。D5)。`for_language` の本体は変えない。`:289` のコメントの `report_messages.rs` を「単体テストの `src/tests/report_messages.rs` も同じ写しを使う」に(コミット 9)。モジュールの説明は変えない |
| T9b | `tests/common/kinds.rs` → `src/tests/common/kinds.rs` | `git mv`。`:22` の `use conductor_cli::UsageErrorKind as U;` を `use crate::types::UsageErrorKind as U;` に(差 2 行)。一覧・網羅の `match`・コンパイル時の確かめは変えない |
| T9c | `src/tests/mod.rs`、`src/tests/common/mod.rs`、`src/tests/docs_support.rs`(新しい) | §2.2 の形。写しの本体は元と一字も変えない |

### 4.5 T10: ソースの検査の範囲(`tests/nfr_source.rs`。D2)

§2.3 の形(`is_unit_test` を足し、OS 分岐と `panic!(` の検査から `src/tests/` を除き、`テスト専用の経路を実行ファイルに持たない` に、`lib.rs` の末尾の宣言の確かめと、`src/tests/` の外に `#[path` が無いことの確かめを足す)。テストの数は変わらない(6 件)。モジュールの説明に「単体テスト(`src/tests/`)は、OS 分岐と `panic!(` の検査の外(`lib.rs` の最後の `#[cfg(test)]` の宣言からだけ読み、製品のファイルに `#[path` が無いことを確かめる)」を足す。

### 4.6 消すテスト

**なし**(0 件)。

### 4.7 名前を変えるテスト(★)

**なし**。移す 162 件の名前は、すべて変えない。パスは、統合テストのバイナリの中の名前から、`conductor_cli` の単体テストの `tests::<元のファイル名>::…` に変わる(名前の最後の部分は同じ)。写しで、名前の最後の部分で比べた差が 0 件であることを確かめた。

## 5. 足すテスト

なし。移すテストが、今の確かめをそのまま持つ。本番の動きは変わらないので、新しい確かめは要らない。`nfr_source.rs` の `テスト専用の経路を実行ファイルに持たない` に足す 2 つの確かめ(`lib.rs` の最後の宣言、製品のファイルに `#[path` が無いこと)は、検査の範囲を直す代わりの確かめで、テストの数は増えない。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 112 | 270 | 382 | 0 |
| runtime | 69 | 128 | 197 | 0 |
| converter | 96 | 19 + 1 無視 | 115 + 1 無視 | 0 |
| cli | **164**(2 + 162) | **294**(456 − 162) | 458 | 0 |
| 全体 | | | **1,152 通過、1 無視、0 失敗** | **0** |

基準と見込みは macOS の件数。写しの木での試しで、この件数になることを確かめた(`inventory.md` §0.2)。Windows では、`args_grammar.rs` の `#[cfg(unix)]` の 3 件(テスト単位の 3 か所。`:601` はテストの中のブロック)が単体テストでも数えられないのは今と同じで、全体の差は 0。

## 7. 確かめ方

- 実装の前と後で、`cargo test --workspace --locked` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,152 − 消したテスト 0 + 足したテスト 0 = 1,152**(と無視 1)」に一致すること。cli は単体 164、統合 294 [Q2]。
- テスト名の一覧の差: 前後で `cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sed 's/: test$//; s/.*:://' | sort` を比べ、差が 0 件であること(移したテストはパスが変わるので、名前の最後の部分で比べる)。あわせて、パスで比べた差が 324 行(移した 162 件の前後)であること。
- 移したテストの本体が変わっていないこと: `git show -M --color-moved` で、移動として表示されることを見る。T1〜T6 は名前の変更として検出される(差は §4.1 の import の行だけ)。
- テストのファイルの変更は §4 の T 表だけ。コミットごとに `git diff --stat -- crates/cli` を見る。
- **完了の判定**。最後のコミットのあとで、次をすべて行う [Q3] [Q2]:

```sh
# 公開の項目(lib.rs)
grep -n "^pub " crates/cli/src/lib.rs        # pub use deps::{…}(10 個)、pub use panic_hook::install_panic_hook、pub fn run_cli、pub fn run_cli_with だけ
grep -n "^pub mod\|pub use args\|pub use types" crates/cli/src/lib.rs   # 0 件
tail -2 crates/cli/src/lib.rs                # #[cfg(test)] と mod tests;
# 狭めた名前を tests/ から参照する所(SM1 の確かめる手段)
grep -rn "conductor_cli::\(parse_args\|CliCommand\|CommandKind\|GraphFormat\|InitOptions\|InitTarget\|Language\|Level\|ParsedArgs\|RunIdArg\|UsageError\|exit\|report\|run_id\)" crates/cli/tests    # 0 件
grep -rn -A3 "use conductor_cli::{" crates/cli/tests | grep -E "parse_args|CliCommand|Language|Level|ParsedArgs|RunIdArg|UsageError|GraphFormat|Init"   # 0 件
grep -rn "conductor_cli" crates/cli/src/tests     # 0 件
ls crates/cli/src/tests crates/cli/src/tests/common   # mod.rs、docs_support.rs、13 ファイル / mod.rs、kinds.rs
test ! -e crates/cli/tests/common/kinds.rs
git diff --stat c35ada6 -- crates/core crates/runtime crates/converter Cargo.lock '*Cargo.toml'   # 空
# src/tests/ は OS 分岐と panic!( の検査の外なので、本番の外にあることと、ほかの禁止の語が無いことを別に確かめる
grep -rn "unsafe\|low_level\|std::net\|TcpStream\|UdpSocket\|reqwest\|hyper\|ureq\|tokio\|curl\|openssl" crates/cli/src/tests   # 0 件
grep -rln "cfg(unix)\|cfg!(unix)" crates/cli/src/tests   # args_grammar.rs、report_messages.rs だけ(移したテストのまま)
grep -rn "#\[path" crates/cli/src | grep -v "^crates/cli/src/tests/"   # 0 件(製品のファイルから src/tests/ を読む道が無い。nfr_source.rs も同じことを確かめる)
grep -n "is_unit_test\|#\[path\|ends_with(\"#\[cfg(test)\]" crates/cli/tests/nfr_source.rs   # §2.3 の 3 つ(除く条件 2 か所、末尾の宣言、#[path の確かめ)
# 仕様書(S1〜S24)
grep -n "再公開\|公開モジュール" docs/spec/rust/02-architecture.md   # 2.4 は S1 の文(parse_args・型・exit・report・run_id を公開として書かない)
grep -rn "cli/tests/escape_table\|\`tests/escape_table" docs/spec/rust      # 0 件
grep -n "args_grammar.rs\|report_messages.rs\|graph_report.rs\|plan_report.rs\|init_template.rs\|exit_codes.rs\|escape_table.rs\|run_id_arg.rs\|init_types.rs\|graph_long_id.rs\|common/kinds.rs" docs/spec/rust/11-quality-and-build.md   # cli の単体テストの表(S24)の行だけ(core・converter の表の common/kinds.rs の行を除く)
grep -n "src/tests" docs/spec/rust/02-architecture.md docs/spec/rust/07-process-and-signals.md docs/spec/rust/08-output-and-i18n.md docs/spec/rust/11-quality-and-build.md   # S7〜S18、S23、S24 の行
git diff --stat c35ada6 -- docs/spec/language-agnostic docs/spec/go docs/spec/python docs/spec/typescript docs/spec/csharp docs/spec/powershell   # 空
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4)のまま増えないこと(cli は 0 件)。
- Windows の対象のコンパイル: `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` をローカルの macOS で走らせる(写しで通ることを確かめた)。実行は CI の `test`(windows)で確かめる。
- 本番の形: `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` が、警告なしで通る(狭めた項目に `dead_code` が出ない)。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(写しで 98.06%)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分が無い)[memory:M5]。`unsafe` と本番の src の `cfg` を足さない。版の表記(`0.9.8`)は変えない。
- 利用者から見える動き: 統合テスト(流れ・実バイナリ・デモ・非機能。294 件)と移した単体テスト(162 件)が、期待を変えずに通ることで確かめる。本番の処理のコードは変わらない(`lib.rs` の可視性と説明、`escape.rs` の説明だけ)。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

テストを先に移し(1〜10)、統合テストに残るテストの書き方を変え(11)、最後に公開を狭める(12)。各コミットで、その変更に当たる仕様書の行(S)も直す。移すコミットでは、まだ公開しているので、残る統合テストはそのまま通る。

**各コミットの確かめ**: `cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test -p conductor-cli --locked`(件数がコミットごとの見込みと合うこと)。写しの木では、全項目を当てた最後の形でだけ確かめたので、途中の形はこのコミットごとの確かめで見る。最後の確かめ(13)で全テストと §7 を行う。

| # | コミット | 変えるもの | cli の件数の見込み(単体 / 統合) |
|---|----------|------------|----------------------------------|
| 1 | cli: 単体テストの置き場所を作り、終了コードのテストを移す | `lib.rs` の最後に `#[cfg(test)] mod tests;`。`src/tests/mod.rs`(`mod exit_codes;` だけ)、`src/tests/exit_codes.rs`(T6 の一部。`git mv`)。`tests/nfr_source.rs`(T10。D2)。仕様書 S8、S9、S10、S11、S15、S16、S17、S18、S19(`exit_codes.rs` の行)、S23、S24(`mod.rs` と `exit_codes.rs` の行) | 9 / 449 |
| 2 | cli: エスケープの表のテストを単体テストへ移す | `src/tests/escape_table.rs`(T6)、`mod escape_table;`。`src/report/escape.rs` の説明(`inventory.md` §1.3)。仕様書 S12、S13、S14、S19、S24 | 12 / 446 |
| 3 | cli: run ID の指定の型のテストを単体テストへ移す | `src/tests/run_id_arg.rs`(T6)。仕様書 S19、S24 | 13 / 445 |
| 4 | cli: init の型のテストを単体テストへ移す | `src/tests/init_types.rs`(T6)。仕様書 S19、S24 | 16 / 442 |
| 5 | cli: 引数の文法のテストを単体テストへ移す | `src/tests/args_grammar.rs`(T1)。`src/tests/common/mod.rs`(`FakeEnv` の項目・`with`・`impl Env` だけ。`locale_read`・`color_read`・`for_language` はコミット 9 で足す)。仕様書 S19、S24 | 76 / 382 |
| 6 | cli: graph の表示のテストを単体テストへ移す | `src/tests/graph_report.rs`、`graph_long_id.rs`(T3)。仕様書 S19、S24 | 106 / 352 |
| 7 | cli: 計画の表示のテストを単体テストへ移す | `src/tests/plan_report.rs`(T4)。仕様書 S19、S24 | 113 / 345 |
| 8 | cli: ひな形のテストを単体テストへ移す | `src/tests/init_template.rs`(T5)。仕様書 S19、S24 | 120 / 338 |
| 9 | cli: 表示の文言のテストを単体テストへ移す | `src/tests/report_messages.rs`(T2)。`src/tests/common/mod.rs` に `FakeEnv` の残り(`locale_read`・`color_read`・`for_language`)、`FakeSink`、`state_error` を足す。`src/tests/common/kinds.rs`(T9b。`git mv`)。`tests/common/mod.rs` の `pub mod kinds;` とコメント(T9a の一部)。仕様書 S7、S19、S22、S24 | 156 / 302 |
| 10 | cli: README の例と実際の出力を比べるテストを単体テストへ移す | `src/tests/docs_support.rs`、`docs_graph.rs`、`docs_dry_run.rs`、`docs_init.rs`(T7)。統合テストの 3 ファイルから 8 件を外す。仕様書 S20、S24 | 164 / 294 |
| 11 | cli: init と validate の流れのテストを、公開の入口だけで書く | `tests/common/mod.rs` にテストの言語 `Language`(T9a の残り。D5)、`tests/flow_init.rs`、`tests/flow_validate.rs`(T8。D4)。仕様書 S21 | 164 / 294 |
| 12 | cli: ライブラリの公開を main と差し替え口に狭める | `lib.rs` の可視性と説明(項目 1。`inventory.md` §1.2、§1.3)。仕様書 S1〜S6 | 164 / 294 |
| 13 | (確かめ)§7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジの記録 | コミットなし。記録はコード生成の記録に書く | — |

合計 **12 コミット**(13 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

- コミット 1 で、ソースの検査の範囲の直し(T10)と、`lib.rs` の最後の宣言を同時に入れる(足す確かめが、その宣言と、製品のファイルに `#[path` が無いことを確かめるため)。`panic!(`・`#[cfg(unix)]` を含むテストが `src/tests/` に入るのはコミット 5 以降。
- コミット 5 の `args_grammar.rs` は 1,133 行の移動だが、差は import の 10 行だけ。コミット 9 の `report_messages.rs`(1,096 行)も差は 11 行。レビューは `git show -M --color-moved` で、移動と差を分けて見られる。
- `src/tests/common/mod.rs` には `#![allow(dead_code)]` を**付けない**(統合テストの `tests/common/mod.rs` とは違い、使われない補助を警告で見つけられるようにする)。その代わり、途中のコミット(5〜8)では、**そのコミットまでに移したテストが使う項目だけ**を置く。コミット 5〜8 で置くのは `FakeEnv` の項目(`vars`・`terminal`・`var_calls`・`terminal_calls`・`names`)、`with`、`impl Env for FakeEnv` だけで、`locale_read`・`color_read`・`for_language`・`FakeSink`・`state_error` は、それらを使う `report_messages.rs` を移すコミット 9 で足す(`kinds.rs` も 9)。**各コミットで `cargo clippy --workspace --all-targets --locked -- -D warnings` を通し**、使われない項目の `dead_code` の警告が出ないことを確かめる(写しの木では最後の形でだけ確かめたので、途中の形はこの確かめで見る)。警告が出たら、その項目を使うコミットへ移し、記録する。
- コミット 12 の前に、`git grep -n "conductor_cli::" -- crates/cli/tests` が、公開に残す名前(`Deps`、`Env`、`Runtime`、`SignalError`、`Signals`、`Sink`、`SystemRuntime`、`SystemSignals`、`run_cli_with`)だけであることを確かめる(移し忘れがあればビルドで止まる)。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」(core・runtime・converter の公開範囲、継ぎ目の形、下位モジュールの中の可視性の書き換え)。
- 言語に依らない仕様書と、凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- 版の更新、タグとリリース。README。CI の設定。
- 流れ・実バイナリ・デモ・非機能のテスト(T 表の外)と、統合テストの補助 `bin_support`・`signal_support`・`docs_support`。

## 10. 決めてほしい点

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | 移す先の形 | **`src/tests/` のディレクトリ**(`lib.rs` の最後に `#[cfg(test)] mod tests;`。下位のファイル名を元の統合テストと同じにする。補助は `src/tests/common/`、`src/tests/docs_support.rs`) | (B)各モジュールの既存・新しい `#[cfg(test)] mod tests` に分けて入れる(`args.rs` に引数の文法、`report/graph_text.rs` に graph の表示など)。(C)`lib.rs` の中の 1 つの `mod tests { … }` に全部を書く | 推奨は converter と同じ形で、元のファイルと 1 対 1、`git` が移動として追え、言語に依らない仕様書の根拠の欄(`args_grammar::…` など)がそのまま正しい。(B)はテストの意味でまとめ直す判断が要り、本番のファイルが長くなる(`args.rs` 626 行に 1,133 行)。どの形でも、cli のソースの検査はファイル全体を読むので D2 の直しは要る。(C)は `lib.rs` が約 4,000 行になる | 決定: `src/tests/` のディレクトリ。`lib.rs` の最後に `#[cfg(test)] mod tests;`、下位のファイル名は元の統合テストと同じ(推奨のとおり。ユーザー、2026-11-24) |
| D2 | cli のソースの検査(`nfr_source.rs`)と、移すテストの `#[cfg(unix)]`・`cfg!(unix)`(6 か所)・`panic!(`(7 か所) | **`src/tests/` の下を、OS 分岐と `panic!(` の 2 つの検査の外にし、`lib.rs` の末尾が `#[cfg(test)]` と `mod tests;` であることを同じテストで確かめる**(§2.3)。`unsafe`・`low_level`・ネットワークの検査は `src/tests/` も対象のまま | (B)移すテストの書き方を変えて検査に当たらないようにする: `panic!(` は `unreachable!(` などに書き換えられるが、`#[cfg(unix)]` の 3 件と `cfg!(unix)` の 2 件(UTF-8 でないバイト列の `OsString`、色の判定)は OS の分岐が要るので書き換えられない(統合テストにも残せない。狭めた `parse_args`・`Reporter` を直接呼ぶ)。(C)`src/tests/` を 5 つの検査すべての外にする | 検査の目的は「製品のコード」の性質で、`src/tests/` はテストのときだけコンパイルされ、本番の成果物に入らない。推奨は、その前提(宣言が `#[cfg(test)]` であること)を検査の中で確かめるので、範囲を外しても製品のコードの検査は弱まらない。(B)は書き換えられないものが残るので選べない。(C)は不要に広い(`unsafe`・ネットワークの語は移すテストに無く、対象に残せる) | 決定: 検査の範囲を製品のコードに限る。`src/tests/` を OS 分岐と `panic!(` の検査の外にし、`lib.rs` の末尾の `#[cfg(test)] mod tests;` を確かめる。`unsafe`・`low_level`・ネットワークの検査は `src/tests/` も対象のまま(推奨のとおり。ユーザー、2026-11-24)。ユーザーは、プロジェクトの規則「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」(`project.md` の Forbidden)を「製品のコードについて」の規則と読むことを認めた(テストのときだけコンパイルする `src/tests/` の `#[cfg(unix)]`・`cfg!(unix)` は、この規則に反しない) |
| D3 | 一部だけが狭めた項目を使うファイル(`docs_graph.rs`・`docs_dry_run.rs`・`docs_init.rs`)の移す単位 | **狭めた項目で確かめるテストだけを移す**(8 件。必須の語句の 4 件は統合テストに残す) | (B)ファイルごと移す(12 件。`docs_support` の `assert_readme_has`・`assert_docs_check_words`・`DOCS_CHECK_WORDS` も写す) | 推奨は Q1 の範囲(狭めた項目を直接使うテスト)だけを動かし、README の語句の検査は今の統合テストのまま。ファイル名が統合テストと単体テストの両方に残るが、言語に依らない仕様書の根拠の欄(`docs_graph::…` など)はどちらでも正しい。(B)は 1 つの README の検査が 1 か所にまとまるが、Q1 の範囲の外のテストまで動き、写す補助が増える | 決定: 狭めた項目で確かめるテストだけを移す(8 件。推奨のとおり。ユーザー、2026-11-24) |
| D4 | `flow_init.rs` の 2 件(`run_cli_with` で動かし、期待のひな形を `render_template` で作る) | **統合テストに残し、期待のひな形を原本の写し(`ORIGINAL_JA`・`ORIGINAL_EN` と名前の置き換え)で書く**(§4.3。期待のバイト列は同じ) | (B)2 件を単体テストへ移し、偽の `Runtime`(`ScriptedRuntime`)・`FakeSignals`・`run_with_fakes` など約 350 行を `src/tests/common/` に写す。(C)`flow_init.rs` の 9 件をファイルごと移す(写す補助は (B) と同じ) | 推奨は公開の入口で確かめるテストを統合テストに残し(初期の記述の「公開の入口で確かめる形」と同じ向き)、写すのはひな形の文字列(約 70 行。`init_template.rs` の原本と同じ)だけ。期待の値を実装の関数ではなく原本から書くので、`init_template.rs` の方針(「実装の出力から作っていない」)とも揃う。代償は、ひな形を変えるとき原本の写しを 2 か所(単体テストと `flow_init.rs`)直すこと。(B)(C)は Q1 の字句どおり(狭めた関数を呼ぶテストを移す)だが、大きな偽物の写しが 2 か所に残る | 決定: 統合テストに残し、期待のひな形を原本の写しで書く(推奨のとおり。ユーザー、2026-11-24) |
| D5 | `tests/common/mod.rs` の `FakeEnv::for_language(Language)` と、それを使う `flow_init.rs`(19 か所)・`flow_validate.rs`(1 か所) | **`tests/common/mod.rs` に、テストの言語 `Language { En, Ja }`(写しの enum)を足し、`for_language` の本体と呼ぶ所の本体は変えない**(import の名前だけ変わる) | (B)`for_language` を消し、呼ぶ所を `FakeEnv::with(&[("LANG", "en_US.UTF-8")])` などに書き換える(20 か所と、`flow_init.rs` の補助 `run_with`・`run` の引数の型) | 推奨は差が小さく(import と enum の 6 行)、テストの本体が変わらない。`kinds.rs` と同じ「テストの写し」の形。(B)は写しの型が無くなるが、20 か所の書き換えになる | 決定: `tests/common/mod.rs` にテストの言語 `Language`(写しの enum)を足す(推奨のとおり。ユーザー、2026-11-24) |
| D6 | 言語に依らない仕様書の 11 章の根拠の欄(`args_grammar::…`、`report_messages::…` など) | **直さない**(D1 の推奨の形で、ファイル名とテスト名が同じなので正しいまま) | 移したテストの根拠に「区分: 単体」を足す | 根拠の欄は「参照実装のテストの名前」を示すだけで、場所(`tests/` か `src/tests/` か)を書いていない。受け入れテストそのもの(外から実行して確かめる)は変わらない。区分は受け入れテストの確かめ方の区分で、参照実装のテストの置き場所の区分ではない(converter の intent と同じ判断) | 決定: 直さない(推奨のとおり。ユーザー、2026-11-24) |
| D7 | 単体テストが使う `FakeEnv`・`FakeSink`・`state_error`・`readme` の置き方 | **`src/tests/common/mod.rs` と `src/tests/docs_support.rs` に写す**(統合テストの補助は残す。写しの説明に元の場所を書く) | (B)`#[path = "../../tests/common/mod.rs"]` で統合テストの補助を単体テストからも読み、`#[cfg(test)] extern crate self as conductor_cli;` を `lib.rs` に置いて `conductor_cli::…` のパスを通す | 推奨は約 130 行の写しが 2 か所にあるが、どちらも単純で、写しであることが説明で分かる。(B)は写しが無くなるが、本番の `lib.rs` にテストのための名前の別名を足す(`cfg(test)` の中だが、Rust に不慣れな読み手に分かりにくい)。また統合テストの補助は `Language` を写しの enum にする(D5)ので、単体テストの `crate::types::Language` と型が合わず、そのままでは共有できない | 決定: `src/tests/common/mod.rs` と `src/tests/docs_support.rs` に写す(推奨のとおり。ユーザー、2026-11-24) |

D1〜D7 は、ユーザーが決めた(すべて推奨のとおり。2026-11-24)。

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-24)**: §4 の T 表(T1〜T10。移す 162 件、名前は変えない、消すテストは 0 件)、§4.3 の書き方を変えるテスト(`flow_init.rs` の 4 か所の期待の書き方と import、`flow_validate.rs` の 1 か所)、§2.3 のソースの検査の範囲の直し。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた(字下げした項目は、親の項目に含める)。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 公開範囲が、`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目を使うのに要る型だけになる(確かめる手段: 項目ごとの判定の一覧と、狭めた名前を `tests/` から参照する所が 0 件) | BR1.1、BR1.2、BR1.3、BR2.1 |
| SM2 | 狭めた項目を直接呼ぶテストは src の単体テストへ移り、確かめ方と期待の値を残す。移す先はソースの検査と矛盾しない(「継ぎ目を使うのに要る型」の基準と、型ごとの判定を含む) | BR2.1、BR3.1、BR3.2、BR3.3、BR3.4 |
| SM3 | 仕様書 rust の公開範囲とテストの場所の記述が実装と合う(検索で確かめる) | BR4.1、BR4.2 |
| SM4 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM5 | 既存のテストの扱い(移す・変える・消すを検索で網羅、消すものと名前を変えるものは承認)と件数(基準 − 消したテスト + 足したテスト) | BR3.3、BR8.1、BR9.2、BR9.3、BR9.6 |
| SM6 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR1.3、BR9.4 |
