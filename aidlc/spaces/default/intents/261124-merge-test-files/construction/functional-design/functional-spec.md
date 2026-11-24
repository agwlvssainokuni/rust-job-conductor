# 機能の仕様(Functional Spec): テストのファイルの統合

意図: `ideation/intent-capture/intent-statement.md`(Q1〜Q5 すべて A)。**移す場所の正確な一覧(項目ごとの元と移す先、名前の衝突、`cfg` の扱い、仕様書の S1〜S16・L1〜L10、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD 16ac9d6 で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの移動(§4 の T 表)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

決めてほしい点(§10 の D1〜D8)は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。この文書の D の箇所は、その決定のとおりの形を書いている。§4 の T 表(T1〜T13)は、ユーザーが設計の承認で認めた(2026-11-25)。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない**(ユーザーの判断)。本番のコード(`crates/*/src` の `src/tests/` の外)は 1 行も変えない。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。版は上げない(`0.9.8` のまま)[desc]。
- **テストの名前・確かめの行・期待の値は変えない** [Q1]。変わるのは、ファイルとパス、import、補助の読み方だけ。本体で変わる行は、補助の名前の衝突を避ける 5 行だけ(D1、D2。確かめの行ではない)と、子のモジュールへ移す 4 件の字下げ(D3)。消すテスト 0 件、足すテスト 0 件、名前を変えるテスト 0 件(★ なし)。
- cli の README のテストは単体テスト `crates/cli/src/tests/docs.rs` 1 つにし、補助の `readme` を 1 つにする [Q2]。
- cli のテストの補助(`FakeEnv`、`FakeSink`、`state_error`)は `crates/cli/tests/common/fakes.rs` 1 つにし、統合テストは `mod`、単体テストは `#[path]` で読む [Q3]。
- init のひな形の原本はデータのファイル(`crates/cli/tests/fixtures/init_{ja,en}.toml`)にし、両方のテストが `include_str!` で読む [Q4]。
- `run_id_arg.rs` は `args_grammar.rs` の `--run-id` の節へ移し、紛れ込んだ 2 件を正しい節へ並べ直す [Q5]。
- 製品のコードの検査(cli の `nfr_source.rs`、runtime の `nfr_source.rs`、converter の `nfr_security.rs`)は、テストの数と中身を変えずに通る。CI が名前で指すテストのファイル(`nfr_perf`)は変えない。
- 仕様書は `docs/spec/rust` と、言語に依らない仕様書の 11 章の根拠の欄を直す(D6)。凍結した仕様書は触らない。
- 小さなコミットの並び(1 項目 1 コミット程度。12 コミット)で、どのコミットでもビルド、`clippy -D warnings`、テストが通る [memory:M1] [memory:M4]。ファイルの移動は、移す先が新しいファイルなら `git mv`、既にあるファイルへの統合なら中身を足して元を `git rm` する(§8.1)。承認は、この設計とコードの計画の 1 回。結果は最後にまとめて見せる [Q1]。
- 新しい依存を足さない(dev-dependencies も)[memory:M5]。`unsafe` と製品のコードの `cfg(unix)`/`cfg(windows)` を足さない。MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

| 項目 | 元 → 移す先 | 移るテスト | 本番の形の変わり方 |
|------|-------------|------------|--------------------|
| 1 cli の README のテスト | `tests/docs_{retry,env,graph,dry_run,init}.rs`、`tests/docs_support/mod.rs`、`src/tests/docs_{graph,dry_run,init}.rs`、`src/tests/docs_support.rs` → `src/tests/docs.rs` | 18(統合 10、単体 8) | なし |
| 2 converter | `tests/convert_roundtrip.rs` → `tests/convert_ac.rs`。`src/tests/convert_ac.rs` → `src/tests/convert_roundtrip.rs`。`tests/common/{mod,kinds}.rs` → `tests/nfr_security.rs` | 3 | なし |
| 3 cli の小さな単体テスト | `src/tests/{init_types,run_id_arg}.rs` → `src/tests/args_grammar.rs`(と、同じファイルの 2 件の並べ直し)。`src/tests/graph_long_id.rs` → `src/tests/graph_report.rs` | 4(+ 並べ直し 2) | なし |
| 4 runtime | `tests/runner_env.rs` → `tests/runner_basic.rs`。`tests/state_read_raw.rs` → `tests/state_read.rs`(子のモジュール `raw`) | 9 | なし |
| 5 cli の補助の写し | `tests/common/mod.rs` と `src/tests/common/mod.rs` の写し → `tests/common/fakes.rs` | 0 | なし |
| 6 init のひな形の原本 | `tests/flow_init.rs` と `src/tests/init_template.rs` の定数 → `tests/fixtures/init_{ja,en}.toml`(と、D5 の `.gitattributes`) | 0 | なし |

ファイルの数: 無くなる 19(cli 13、converter 4、runtime 2)、新しく作る 4(`src/tests/docs.rs`、`tests/common/fakes.rs`、`tests/fixtures/init_ja.toml`、`init_en.toml`)と、D5 の推奨で `.gitattributes`。cli の統合テストの実行ファイルが 5 つ、converter が 1 つ、runtime が 2 つ減る。

### 2.1 項目 1 の形(`crates/cli/src/tests/docs.rs`。D1 の推奨: 節で分けた平らな 1 ファイル)

```rust
//! README の検査。README を読むだけで、書き込まない。
//!
//! 節は README の並び(リトライ、環境変数、`graph`、`--dry-run`、`init`)。各節は、必須の語句と
//! 版の表記の検査、載っている例を実際の読み取り・解析に通すか、実際の出力と比べるテストを持つ。
//! 最初に、各節が使う補助(README の読み取り、必須語句の検査、見出しのあとの例の取り出し)を置く。

use std::path::{Path, PathBuf};

use crate::report::init_template::render_template;
use crate::types::Language;

/* 補助: DOCS_CHECK_WORDS、readme、assert_readme_has、assert_docs_check_words、block_after_heading
   (tests/docs_support/mod.rs の本体のまま。#![allow(dead_code)] は外す) */

// ---- リトライの説明(…元のファイルの説明の括弧の中…) ----
const RETRY_HEADING: &str = "### リトライの設定(`[job.retry]`)";   // 元の名前は HEADING
/* tests/docs_retry.rs の 4 件 */

// ---- 環境変数の説明(…) ----
const ENV_HEADING: &str = "### 環境変数の設定(`env`、`env_file`)(v0.9.0 以降)";   // 元の名前は HEADING
/* tests/docs_env.rs の 2 件 */

// ---- `graph` の説明(…) ----
/* tests/docs_graph.rs の 2 件 → src/tests/docs_graph.rs の 6 件と補助 graph_example・render・jobs */

// ---- `--dry-run` の説明(…) ----
/* tests/docs_dry_run.rs の 1 件 → src/tests/docs_dry_run.rs の 1 件 */

// ---- `init` の説明(…) ----
/* tests/docs_init.rs の 1 件 → src/tests/docs_init.rs の 1 件 */
```

- 各節の見出しの括弧の中は、元のファイルのモジュールの説明の括弧の中をそのまま写す。
- 本体で変わるのは、`HEADING` を使う 3 行(`docs_env` の 2 行を `ENV_HEADING` に、`docs_retry` の 1 行を `RETRY_HEADING` に)だけ。
- 統合テストだった 10 件は `tempfile::tempdir()` と `conductor_core::{load_definition, parse_env_file}` を、元と同じ書き方(パスつき)で使う。`tempfile` は cli の dev-dependencies にあり、単体テストからも使える(足す依存は無い)。
- `src/tests/mod.rs`: `mod docs_support;`、`mod docs_dry_run;`、`mod docs_graph;`、`mod docs_init;` を消し、`mod docs;` を足す。モジュールの説明の「クレートの内部(…)を直接確かめる単体テスト」を「クレートの内部(…)と README を直接確かめる単体テスト」にする。
- 完了の判定: `test ! -e crates/cli/tests/docs_support && ls crates/cli/tests/docs_*.rs crates/cli/src/tests/docs_*.rs 2>/dev/null | wc -l` が 0。`grep -c "#\[test\]" crates/cli/src/tests/docs.rs` が 18。`grep -c "fn readme()" -r crates/cli` が 1。

### 2.2 項目 2 の形(converter)

- `tests/convert_ac.rs`: `request` の後に定数 `PIPELINE_TOML`・`PIPELINE_YAML`(`include_str!("fixtures/…")`)、`convert_text_はファイルに触れず_ラベルを使う` の後に `出力は毎回同じ`。モジュールの説明を「受け入れ基準と、`convert`(ファイルを読む入口)、変換の出力が毎回同じであること。変換の結果を読んだ木を入力の木と比べる 2 件と、往復の木を比べるテストは、単体テストの `src/tests/convert_roundtrip.rs` にある。」にする。
- `src/tests/convert_roundtrip.rs`: import に `std::path::{Path, PathBuf}`、`ConversionRequest`、`convert` を足し、ファイルの最後に節「受け入れ基準。ファイルを読む入口 `convert` の結果の木」と、補助 `fixture`・`request`、2 件を置く。モジュールの説明に「あわせて、受け入れ基準のうち、変換の結果を読んだ木を入力の木と比べるもの。ほかの受け入れ基準は、統合テストの `tests/convert_ac.rs` にある。」を足す。
- `tests/nfr_security.rs`: `mod common;` と `use common::kinds::CONVERSION_ERROR_KINDS;` を消し、表示の節の先頭に、一覧の定数・網羅の `match`・コンパイル時の確かめを置く(別名はファイルの `K` にそろえ、`pub` を外す。説明は `kinds.rs` のものを写す)。`tests/common/` を消す。
- `src/tests/mod.rs`: `mod convert_ac;` を消し、説明の「下位のモジュールのファイル名は、元の統合テスト(`tests/` の下)と同じにしている」を、下位のモジュールの中身を並べる文(往復と受け入れ基準の木の比較 `convert_roundtrip`、ランダムな入力と構造 `nfr_security`、文書の確認と展開 `read_resolve`)にする。

### 2.3 項目 3 の形(cli の `args_grammar.rs`、`graph_report.rs`)

`args_grammar.rs` の節の並び(直した後):

```text
// ---- init(…) ----              … 今の 7 件、+ ハイフンだけが標準出力でそれ以外はファイル(元 init_types.rs)
// ---- --run-id と --all(…) ---- … 今の 5 件、+ run_id_arg_は空を固定_…(元 run_id_arg.rs)
// ---- resolve_run_id(…) ----     … 補助 resolved と 2 件(resolve_run_id_の… の 2 件が続けて並ぶ)
// ---- --remove-on-complete(…) ---- … fr2_5_… の 2 件(resolve_run_id の節から移す。D4)
// ---- graph(…) ----              … 今のまま
```

- 移す 2 件の前に、元のファイルのモジュールの説明の 1 行目を `///` で付ける(どの型を確かめるテストかが分かるように)。
- 新しい節の見出しの括弧の中は、テストの名前の先頭 `fr2_5_` が指す番号を、ほかの節と同じ書き方で書く。
- モジュールの説明に 2 行を足す:「あわせて、引数から作る型(`init` の出力先 `InitTarget::from_path`、`--run-id` の指定 `RunIdArg::parse`)と、run ID の決定 `resolve_run_id` も確かめる。」
- import は変えない(移す 2 件が使う `OsString`、`InitTarget`、`RunIdArg`、`UsageErrorKind`、`RunId` は既にある)。

`graph_report.rs` の最後(D2 の推奨):

```rust
// ---- 65 文字以上のジョブ ID(…元のファイルの説明の括弧の中…) ----
//
// (元のファイルのモジュールの説明の 2〜4 行目を、ここにコメントで写す)

/// `a` の成功が、長い ID のジョブ(3 番目)へ飛ぶ定義。
fn long_id_definition(long: &str) -> PipelineDefinition { /* 元の definition の本体のまま */ }

fn long_id() -> String { /* 元のまま */ }

#[test]
fn レーンの図では_65_文字以上のジョブ_id_を切らずに出す() { /* 元のまま。`definition(&long)` だけ `long_id_definition(&long)` */ }

#[test]
fn 辺の一覧では_65_文字以上の行き先のジョブ_id_を切らずに出す() { /* 同上 */ }
```

- モジュールの説明に「最後の節は、65 文字以上のジョブ ID を切らずに出すこと」の 1 行を足す。import は変えない。
- `src/tests/mod.rs`: `mod init_types;`、`mod run_id_arg;`、`mod graph_long_id;` を消す。

### 2.4 項目 4 の形(runtime)

`runner_basic.rs`(直した後の並び):

```text
//! (今の説明 1 行)
//!
//! (元 runner_env.rs の説明 2 行。`env` モードで子の環境を確かめる、環境変数はプロセス全体に設定しない)
… 今のテスト 9 件(終了の観測の直後に合図が立っていれば_interrupted_を足す まで)
// ---- 環境変数の引数(…) ----
fn child_env(…)            … 元のまま
… 元 runner_env.rs の 5 件
// ---- Windows の cmd(Windows だけ) ----
#[cfg(windows)] … 今の 3 件(ファイルの最後のまま)
```

`state_read.rs` の最後(D3 の推奨):

```rust
/// `read_state_raw`(完了のステートも返す読み取り)と `read_state` の差のテスト(…元の括弧の中…)。
///
/// run ID つきの `read_state`(引数 2 つ)を使うので、run ID なしの `read_state`(上のテストが使う
/// `common::without_run_id::read_state`)と名前が重ならないよう、子のモジュールに置く。
mod raw {
    use super::common::{def, put};
    use conductor_core::RunId;
    use conductor_runtime::{StateErrorKind, read_state, read_state_raw};

    const COMPLETED: &str = "…元のまま…";

    /* 元 state_read_raw.rs の 4 件。字下げだけが変わる(cargo fmt が 1 行を折り返す) */
}
```

- `mod raw` に `#[cfg]` は付けない。`state_read.rs` の `#[cfg(unix)]` の 3 件と `within_5s` は動かさない。
- `runner_env.rs`、`state_read_raw.rs` を `git rm` する。

### 2.5 項目 5 の形(`crates/cli/tests/common/fakes.rs`)

```rust
//! 統合テストと単体テストが共有する偽の `Env`・`Sink` と、ステートの誤りの組み立て。
//!
//! 統合テストは `tests/common/mod.rs` の `mod fakes;` で、単体テストは `src/tests/common/mod.rs` の
//! `#[path = "../../../tests/common/fakes.rs"] mod fakes;` で、同じこのファイルを読む。`Env`・`Sink`
//! の trait と、言語 `Language` は、読む側(親のモジュール)が用意する(統合テストは `conductor_cli`
//! の公開の trait とテスト用の写しの `Language`、単体テストは `crate::deps` と `crate::types::Language`)。
//! 注意: `use super::Language` は、統合側の写しの enum と単体側の `crate::types::Language` の両方を読む。
//! 両者の変種(今は `En`・`Ja`)を食い違わせないこと。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::path::Path;

use conductor_runtime::{StateError, StateErrorKind};

use super::{Env, Language, Sink};

/* FakeEnv、FakeSink、state_error: 今の写しの本体のまま */
```

```rust
// crates/cli/tests/common/mod.rs(統合テスト。写しの部分を消した後)
// 偽の `Env`・`Sink` と `state_error` は、単体テストと共有するファイル(`fakes.rs`)にある。
// `fakes.rs` は、この親のモジュールの `Env`・`Sink`・`Language` を使う。
mod fakes;

pub use fakes::{FakeEnv, FakeSink, state_error};
// `enum Language`、`use conductor_cli::{…, Env, …, Sink, …}` は今のまま。`use std::io;` は消す。

// crates/cli/src/tests/common/mod.rs(単体テスト。全体)
pub mod kinds;

#[path = "../../../tests/common/fakes.rs"]
mod fakes;

pub use fakes::{FakeEnv, FakeSink, state_error};

use crate::deps::{Env, Sink};
use crate::types::Language;
```

- `Language` の型の違いは、`fakes.rs` が `super::Language` を使い、各側が自分の `Language` を親のモジュールに置くことで扱う(統合テストはテスト用の写しの `enum`、単体テストは `crate::types::Language`。どちらも `En`・`Ja` の 2 つ)。`fakes.rs` の `for_language` の網羅の `match` は、両方でコンパイルされる。`fakes.rs` の先頭のモジュールの説明に、注意書き「`use super::Language` は、統合側の写しの enum と単体側の `crate::types::Language` の両方を読む。両者の変種(今は `En`・`Ja`)を食い違わせないこと」を置く(上の形の最後の 2 行。本番の `Language` に変種を足すと、単体側は `for_language` の網羅の `match` がコンパイルで止まるが、統合側の写しの enum は止まらないので、人が気づくための書き置き)。
- `dead_code`: 統合テストは親の `#![allow(dead_code)]` が子にも効く(使われない `color_read` は警告にならない)。単体テストは今と同じく全項目を使う。写しで、macOS と Windows の対象の clippy が通る。
- `state_error` の説明のコメントは 1 つにする(「英語の表示を ASCII と確かめる単体テスト(`src/tests/report_messages.rs`)も使うので、ASCII の文にする。」)。
- cli の `nfr_source.rs`: `#[path` は `src/tests/` の中なので検査の外。`fakes.rs` は `src/` の外。写しで 6 件が通る(`inventory.md` §5.2)。

### 2.6 項目 6 の形(ひな形の原本)

```rust
// crates/cli/tests/flow_init.rs
/// 日本語のひな形の原本(名前は既定の `my-pipeline`)。単体テストの `src/tests/init_template.rs` も
/// 同じファイルを読む。
const ORIGINAL_JA: &str = include_str!("fixtures/init_ja.toml");
/// 英語のひな形の原本(…同じ…)。
const ORIGINAL_EN: &str = include_str!("fixtures/init_en.toml");

// crates/cli/src/tests/init_template.rs
/// 日本語のひな形の原本(名前は既定の `my-pipeline`)。統合テストの `tests/flow_init.rs` も
/// 同じファイルを読む。
const ORIGINAL_JA: &str = include_str!("../../tests/fixtures/init_ja.toml");
const ORIGINAL_EN: &str = include_str!("../../tests/fixtures/init_en.toml");
```

- データのファイルの中身は、今の生の文字列とバイト単位で同じにする(末尾の改行 1 つ)。コミットの前に、HEAD の文字列を取り出したものと `cmp` で比べる。
- 改行(D5 の推奨): リポジトリの根に `.gitattributes` を作り、1 行 `crates/cli/tests/fixtures/*.toml text eol=lf` を置く(同じコミット)。Windows の取り出しでも LF になり、全文の一致のテストが落ちない。

## 3. 実物で確かめて初期の記述と違った点・足した点

- 項目 1: 統合テストの 5 つは README・core・`tempfile` だけを使い、単体テストへ依存を足さずに移せる(記述のとおり)。定数 `HEADING` が 2 つのファイルで衝突する(D1)。
- 項目 2: converter の `src/tests/mod.rs` の説明「ファイル名は元の統合テストと同じ」が成り立たなくなる(説明を直す)。統合と単体で `convert_ac`・`convert_roundtrip` の中身が入れ替わる形になるので、言語に依らない仕様書の根拠の欄 3 か所を直す(L1〜L3)。
- 項目 3: 紛れ込んだ 2 件は `fr2_5_remove_on_complete_…` の 2 件(`--remove-on-complete` の文法)。正しい節は無いので作る(D4)。`graph_long_id.rs` の補助 `definition` が `graph_report.rs` の補助と衝突する(D2)。
- 項目 4: `state_read_raw.rs` が使う `read_state`(引数 2 つ)と、`state_read.rs` が使う `read_state`(引数 1 つの包み)が衝突する(D3)。`runner_env.rs` の 5 件は OS に依らず、`runner_basic.rs` の `#[cfg(windows)]` の 3 件の前に置ける。
- 項目 5: 統合テストの `use std::io;` が使われなくなる(消す)。
- 項目 6: **データのファイルにすると、Windows の CRLF の取り出しで全文の一致が落ちる見込み**(初期の記述に無い。D5)。
- 仕様書: 言語に依らない仕様書の 772 行目の根拠の `tests/common/kinds.rs` は、前からの食い違い(D7)。

## 4. 移すテストの一覧(T1〜T13。ユーザーが設計の承認で認めた、2026-11-25)

Q1: A に従い、移すテストの確かめる中身(呼ぶ関数、入力、確かめの行、期待の値)は変えない。消すテストは無い。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §9。

```sh
grep -c "#\[test\]" crates/cli/tests/docs_*.rs crates/cli/src/tests/docs_*.rs crates/cli/src/tests/{init_types,run_id_arg,graph_long_id}.rs crates/converter/tests/convert_roundtrip.rs crates/converter/src/tests/convert_ac.rs crates/runtime/tests/{runner_env,state_read_raw}.rs
# 4、2、2、1、1 / 6、1、1 / 1、1、2 / 1 / 2 / 5、4(計 34)
cargo test --workspace --locked -- --list 2>&1 | grep -E "Running|: test$"
```

### 4.1 T 表

| ID | 元(HEAD) | 移す先 | 件数 | パスの変わり方(`--list`) | 本体で変わる行 |
|----|-----------|--------|------|---------------------------|----------------|
| T1 | converter `tests/convert_roundtrip.rs` | `tests/convert_ac.rs` | 1(`出力は毎回同じ`) | 変わらない(統合テスト。実行ファイルが `convert_ac` に) | なし |
| T2 | converter `src/tests/convert_ac.rs` | `src/tests/convert_roundtrip.rs` | 2(`ac_7_1_1_toml_から_yaml`、`ac_7_1_2_アンカーつきの_yaml_から_toml`) | `tests::convert_ac::` → `tests::convert_roundtrip::` | なし |
| T3 | converter `tests/common/{mod,kinds}.rs` | `tests/nfr_security.rs` | 0(補助の一覧だけ) | — | —(一覧の別名 `C` → `K`) |
| T4 | runtime `tests/runner_env.rs` | `tests/runner_basic.rs` | 5 | 変わらない(実行ファイルが `runner_basic` に) | なし |
| T5 | runtime `tests/state_read_raw.rs` | `tests/state_read.rs` の `mod raw` | 4(`fr3_1_…` 2、`fr3_2_…`、`fr3_4_…`) | `<名前>` → `raw::<名前>`(実行ファイルが `state_read` に) | 字下げだけ(`fr3_4_…` の 1 行を `cargo fmt` が折り返す) |
| T6 | cli `src/tests/init_types.rs` | `src/tests/args_grammar.rs` の init の節 | 1(`ハイフンだけが標準出力でそれ以外はファイル`) | `tests::init_types::` → `tests::args_grammar::` | なし |
| T7 | cli `src/tests/run_id_arg.rs` | `src/tests/args_grammar.rs` の `--run-id` の節 | 1(`run_id_arg_は空を固定_…`) | `tests::run_id_arg::` → `tests::args_grammar::` | なし |
| T8 | cli `src/tests/args_grammar.rs` の `resolve_run_id` の節 | 同じファイルの新しい `--remove-on-complete` の節 | 2(`fr2_5_remove_on_complete_…`) | 変わらない | なし(並べ替えだけ) |
| T9 | cli `src/tests/graph_long_id.rs` | `src/tests/graph_report.rs` の最後 | 2 | `tests::graph_long_id::` → `tests::graph_report::` | 2 行(`definition(&long)` → `long_id_definition(&long)`。D2) |
| T10 | cli `src/tests/docs_{graph,dry_run,init}.rs`(と `docs_support.rs`) | `src/tests/docs.rs` | 8 | `tests::docs_<話題>::` → `tests::docs::` | なし |
| T11 | cli `tests/docs_{retry,env,graph,dry_run,init}.rs`(と `docs_support/mod.rs`) | `src/tests/docs.rs` | 10 | `<名前>`(統合テスト)→ `tests::docs::<名前>`(単体テスト。階層が変わる) | 3 行(`HEADING` → `ENV_HEADING` 2 行、`RETRY_HEADING` 1 行。D1) |
| T12 | cli `tests/common/mod.rs` と `src/tests/common/mod.rs` の写し | `tests/common/fakes.rs` | 0(補助だけ) | — | —(`fakes.rs` の先頭に、`use super::Language` が統合側の写しの enum と単体側の `crate::types::Language` の両方を読むので、両者の変種(今は `En`・`Ja`)を食い違わせないことの注意書きを置く。§2.5) |
| T13 | cli `tests/flow_init.rs` と `src/tests/init_template.rs` の定数 | `tests/fixtures/init_{ja,en}.toml` | 0(定数の中身だけ) | — | —(定数の名前と型は同じ) |

- 移るテストは計 34 件(パスが変わるもの 28 件、実行ファイルだけが変わるもの 6 件)と、同じファイルの中で並べ替える 2 件。
- 統合から単体へ階層が変わる 10 件(T11)は、README・core・`tempfile` だけを使い、cli の公開の入口を使わないので、確かめる中身は変わらない [Q2]。

### 4.5 残すテスト(変えない)

- T 表の外のテストの本体。`flow_init.rs` の 9 件と `init_template.rs` の 4 件は、定数の宣言だけが変わり、本体は変わらない。統合テストの `common` を使うテストは、再公開の名前が同じなので変わらない。

### 4.6 消すテストと、同じことを確かめて残るテスト

**なし**。

### 4.7 名前を変えるテスト(★)

**なし**。名前の最後の部分で比べた前後の差は 0(写しで確かめた)。パスが変わる 28 件は §4.1 のとおり。

## 5. 足すテスト

なし。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 112 | 269 | 381 | 0 |
| runtime | 69 | 128 | 197 | 0 |
| converter | 96 | 19 + 1 無視 | 115 + 1 無視 | 0 |
| cli | **172**(162 + 10) | **284**(294 − 10) | 456 | 0(T11 の 10 件が統合から単体へ) |
| 全体 | | | **1,149 通過、1 無視、0 失敗** | **0** |

実行ファイルごとの見込み: cli の lib 172、cli の `docs_*` 5 つは無くなる。converter の `convert_ac` 9、統合の `convert_roundtrip` は無くなる、converter の lib 96(`tests::convert_roundtrip` が 15 件)。runtime の `runner_basic` 17、`state_read` 19、`runner_env`・`state_read_raw` は無くなる。基準と見込みは macOS の件数。写しの木での試しで、この件数になることを確かめた。Windows でも、移すテストは OS に依らないので、差は同じ 0(Windows だけの 3 件は `runner_basic.rs` に残る)。

## 7. 確かめ方

- 実装の前と後で、`cargo test --workspace --locked` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,149 − 消したテスト 0 + 足したテスト 0 = 1,149**(と無視 1)」に一致すること。
- テスト名の一覧の差: 前後で `cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sed 's/: test$//; s/.*:://' | sort` を比べ、**差が 0** であること。パスつきの差(`sed 's/: test$//' | sort`)は 56 行(28 件の前と後)で、§4.1 の「パスの変わり方」のとおりであること。T1・T4 の 6 件は、実行ファイルごとの件数(`convert_ac` 9、`runner_basic` 17)で確かめる。
- 移したテストの本体が変わっていないこと: 元のファイル(HEAD)と移す先で、各テストの関数を取り出し、空白を除いて比べる。違うのは T9 の 2 行と T11 の 3 行だけ(補助の名前)。確かめの行(`assert…`)は 1 行も違わない。
- ひな形のデータのファイル: `git show <基準>:crates/cli/src/tests/init_template.rs` から取り出した原本と `cmp` で一致。`git check-attr text eol -- crates/cli/tests/fixtures/init_ja.toml` が `text: set`・`eol: lf`(D5)。
- **完了の判定**。最後のコミットのあとで、次をすべて行う:

```sh
# 無くなるファイル(19)
for f in crates/cli/tests/docs_{retry,env,graph,dry_run,init}.rs crates/cli/tests/docs_support \
         crates/cli/src/tests/docs_{graph,dry_run,init,support}.rs \
         crates/cli/src/tests/{init_types,run_id_arg,graph_long_id}.rs \
         crates/converter/tests/convert_roundtrip.rs crates/converter/src/tests/convert_ac.rs crates/converter/tests/common \
         crates/runtime/tests/{runner_env,state_read_raw}.rs; do test ! -e "$f" || echo "残る: $f"; done   # 出力なし
# 新しいファイル(4 と .gitattributes)
ls crates/cli/src/tests/docs.rs crates/cli/tests/common/fakes.rs crates/cli/tests/fixtures/init_{ja,en}.toml .gitattributes
# 項目ごとの数
grep -c "#\[test\]" crates/cli/src/tests/docs.rs                  # 18
grep -rn "fn readme()" crates/cli | wc -l                         # 1
grep -rln "struct FakeEnv\|struct FakeSink\|pub fn state_error(path" crates/cli   # crates/cli/tests/common/fakes.rs だけ(本番の ctx.rs と report_messages.rs の別の state_error は形が違うので当たらない)
grep -rn "r##\"# conductor の定義ファイルのひな形\|r##\"# A starter conductor" crates/cli | wc -l   # 0
grep -rn 'include_str!("\(\.\./\.\./tests/\)\?fixtures/init_' crates/cli | wc -l   # 4
grep -n "^// ---- " crates/cli/src/tests/args_grammar.rs           # init → --run-id と --all → resolve_run_id → --remove-on-complete → graph の順
grep -n "^mod raw" crates/runtime/tests/state_read.rs              # 1 行
# 製品のコードの検査(数を変えずに通る)
cargo test -p conductor-cli --locked --test nfr_source             # 6 通過
cargo test -p conductor-runtime --locked --test nfr_source         # 5 通過
cargo test -p conductor-converter --locked --test nfr_security     # 10 通過、1 無視
grep -rn "include\|#\[path" crates/cli/src | grep -v "^crates/cli/src/tests/"   # 0 件
# 仕様書: 無くなったファイルの名前が残らない
grep -rn -E "docs_(dry_run|env|graph|init|retry|support)|state_read_raw|runner_env\.rs|init_types|run_id_arg\.rs|graph_long_id|src/tests/convert_ac" docs/spec/rust docs/spec/language-agnostic   # 0 件
grep -n '^| `convert_roundtrip.rs`' docs/spec/rust/11-quality-and-build.md   # 1 行(単体テストの表だけ)
grep -n '^| `convert_ac.rs`' docs/spec/rust/11-quality-and-build.md          # 1 行(統合テストの表だけ)
grep -n '^| `common/kinds.rs`' docs/spec/rust/11-quality-and-build.md        # 2 行(core の統合テストと cli の単体テスト。converter の行は無い)
grep -n "tests/common/kinds.rs" docs/spec/language-agnostic/11-quality-and-build.md   # 1 行(772 行目。D7 で直さない)
grep -n -o "convert_ac::[^\`]*\|convert_roundtrip::[^\`]*" docs/spec/language-agnostic/11-quality-and-build.md   # ac_7_1_1・ac_7_1_2 は convert_roundtrip、出力は毎回同じ は convert_ac
grep -n "state_read::raw::fr3_4\|fixtures/init_\|common/fakes.rs\|^| \`docs.rs\`" docs/spec/rust/*.md docs/spec/language-agnostic/11-quality-and-build.md
# 範囲の外が変わっていない
git diff --stat 16ac9d6 -- Cargo.lock '*Cargo.toml' .github README.md                 # 空
git diff --stat 16ac9d6 -- $(git ls-files 'crates/*/src/*.rs' 'crates/*/src/**/*.rs' | grep -v '/src/tests/')   # 空(本番のコード)
git diff --stat 16ac9d6 -- crates/core crates/cli/tests/nfr_perf.rs crates/cli/tests/msrv_consistency.rs crates/cli/tests/nfr_source.rs crates/runtime/tests/nfr_source.rs   # 空
git diff --stat 16ac9d6 -- docs/spec/go docs/spec/python docs/spec/typescript docs/spec/csharp docs/spec/powershell   # 空
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4)のまま。
- Windows の対象のコンパイル: `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` をローカルの macOS で走らせる(写しで通ることを確かめた)。実行(とくに D5 の改行)は CI の `test`(windows)で確かめる。
- 本番の形: `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` が通る。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(写しで 98.06%)、MSRV 1.100 でのテスト(ローカルの rustc が 1.100.0)[memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` と `Cargo.toml` の差分が無い)[memory:M5]。版の表記(`0.9.8`)は変えない。
- 利用者から見える動き: 本番のコードが変わらない(上の `git diff` が空)ことと、全テストが期待を変えずに通ることで確かめる。
- **D5 の最終確認(Windows の CI)**: `.gitattributes` の 1 行が効くか(Windows の取り出しでひな形のデータのファイルが LF のままか)は、ローカルの macOS では確かめられず、CI の `test`(windows)でしか確かめられない。コード生成の記録(code-summary)に、次の項目を設けて書く: (1) コミット 12 の後の CI の `test`(windows)の結果(`flow_init` と `init_template` の件数と成否)。(2) 失敗したときの戻し先: テストの側で `\r\n` を `\n` に直す案(D5 の代案。`include_str!` の定数を、改行を直した `String` を返す関数か `LazyLock` にし、使う側の数行を合わせる。テストの名前・確かめの行・期待の値は変えない。変える行は記録し、承認を取る)。CI の結果が出るまで、この項目は「未確認」と書く。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

各項目は互いに依らないので、どの順でもビルドとテストは通る。converter → runtime → cli の小さなもの → README のテスト → 補助 → ひな形の順にする(小さく、影響の狭いものから)。各コミットで、そのコミットで動かしたファイルを指す仕様書の行も直す。

**各コミットの確かめ**: `cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、変えたクレートのテスト(`cargo test -p <クレート> --locked`。実行ファイルごとの件数がコミットごとの見込みと合うこと)、そのコミットの仕様書の行の検索。最後の確かめ(13)で全テストと §7 を行う。

| # | コミット | 変えるもの | 件数の見込み(全体 / 変わる実行ファイル) |
|---|----------|------------|------------------------------------------|
| 1 | converter: 出力が毎回同じことのテストを、受け入れ基準の統合テストのファイルへ移す | T1。仕様書 S7(`convert_roundtrip.rs` の行を消し、`convert_ac.rs` の行に「出力が毎回同じ」を足す)、L3 | 1,149 / `convert_ac` 9 |
| 2 | converter: 変換の結果の木を比べる受け入れ基準の単体テストを、往復の単体テストのファイルへ移す | T2、`src/tests/mod.rs` の説明。仕様書 S2、S7(`convert_ac.rs` の行の単体テストのファイル名)、S9、L1、L2 | 1,149 / converter の lib 96 |
| 3 | converter: 変換の失敗の種類の一覧を、表示の検査のファイルへ移す | T3。仕様書 S8 | 1,149 / `nfr_security` 10 + 1 無視 |
| 4 | runtime: 子の環境変数のテストを、実行の統合テストのファイルへ移す | T4。仕様書 S4、S5 | 1,149 / `runner_basic` 17 |
| 5 | runtime: 完了のステートも返す読み取りのテストを、ステートの読み取りのテストのファイルへ移す | T5。仕様書 S6、L4、L5 | 1,149 / `state_read` 19 |
| 6 | cli: init の出力先の型のテストを、引数の文法のテストの init の節へ移す | T6。仕様書 S13(`init_types.rs` の行)、S14(`InitTarget::from_path`) | 1,149 / cli の lib 162 |
| 7 | cli: run ID の指定の型のテストを、引数の文法のテストの --run-id の節へ移し、--remove-on-complete の 2 件を自分の節に並べ直す | T7、T8。仕様書 S13(`run_id_arg.rs` の行)、S14(`RunIdArg::parse` と `resolve_run_id`) | 1,149 / cli の lib 162 |
| 8 | cli: 長いジョブ ID の図のテストを、図の整形のテストのファイルへ移す | T9。仕様書 S13(`graph_long_id.rs` の行)、S15 | 1,149 / cli の lib 162 |
| 9 | cli: README の例を比べる単体テストを、1 つのファイルにまとめる | T10(`git mv src/tests/docs_graph.rs src/tests/docs.rs` のあと、補助 `readme` と dry-run・init の 1 件ずつを足し、`docs_support.rs`・`docs_dry_run.rs`・`docs_init.rs` を `git rm`)。仕様書 S3、S13(`docs_support.rs` と単体の `docs_*` の行を `docs.rs` の行に)、`11:270` の行の末尾「単体テストの同じ名前のファイル」を「単体テストの `src/tests/docs.rs`」に。L8a(言語に依らない仕様書 946 行目。分岐の図の根拠の `docs_graph` は、README の分岐の例を実際の出力と比べる単体テストを指すので、このコミットで `docs` に) | 1,149 / cli の lib 162 |
| 10 | cli: README の語句と例の読み取りの統合テストを、README の単体テストのファイルへ移す | T11(補助をファイルの先頭へ、リトライと環境変数の節を `graph` の節の前へ、残りを各節へ。`tests/docs_*.rs` と `tests/docs_support/` を `git rm`)。仕様書 S10、S13(`docs.rs` の行を最後の形に)、L6、L7、L8b(958 行目)、L9、L10 | 1,149 / cli の lib 172、統合 284 |
| 11 | cli: 偽の Env と Sink とステートの誤りの写しを、統合テストと単体テストで共有する 1 つのファイルにする | T12(`fakes.rs` を作り、2 つの `common/mod.rs` の写しを消す。統合テストの `use std::io;` も消す)。仕様書 S1、S11(`common/mod.rs` の行と `common/fakes.rs` の行)、S12 | 1,149 |
| 12 | cli: init のひな形の原本をデータのファイルにし、統合テストと単体テストの両方から読む | T13、`.gitattributes`(D5)。仕様書 S11(`fixtures/init_{ja,en}.toml` の行)、S16 | 1,149 |
| 13 | (確かめ)§7 の検索、件数、テスト名の差、本体の比べ、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジの記録 | コミットなし。記録はコード生成の記録に書く | — |

合計 **12 コミット**(13 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

### 8.1 ファイルの移動と履歴

- 移す先が新しいファイルのときは `git mv` を使う: コミット 9 で `git mv crates/cli/src/tests/docs_graph.rs crates/cli/src/tests/docs.rs`(10 ファイルの中で最も大きい 146 行。同じコミットで足す行は約 90 行で、元の行は変えないので、git は名前の変更として見る)。`git log --follow crates/cli/src/tests/docs.rs` で `docs_graph.rs` の履歴まで追える。
- 移す先が既にあるファイル(T1〜T9、T11)と、写しを 1 つにするもの(T12、T13)は、中身を足し、元を `git rm` する(同じコミット)。元の履歴は `git log -- <元のパス>`、行の出どころは `git blame -w -C -C <移す先>` で追える。コミットの本文に「元: <元のパス>」を書く。
- 1 つのコミットで、移す先へ足すことと元を消すことを両方行う(途中の状態で同じテストが 2 つあったり、無かったりしない)。

## 9. 範囲の外(書かない)

- 本番の小さなモジュールの統合(別の intent)。core の補助と性能のテストの統合。`nfr_perf.rs`・`msrv_consistency.rs`・`platform`・converter の src の直下のファイル。
- 本番のコード、`Cargo.toml`、`Cargo.lock`、CI の設定、README。版の更新、タグとリリース。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。言語に依らない仕様書は、11 章の根拠の欄 L1〜L10(D6)のほかは変えない。772 行目の前からの食い違いは直さない(D7)。
- 製品のコードの検査のテストの中身。T 表の外のテスト。

## 10. 決めてほしい点

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | `docs.rs` の形と、定数 `HEADING` の衝突(`docs_retry.rs` と `docs_env.rs` で値が違う) | **節で分けた平らな 1 ファイル**。定数を `RETRY_HEADING`・`ENV_HEADING` にする(本体の 3 行が変わる。確かめの行ではない)。パスは `tests::docs::<名前>` | 話題ごとの子のモジュール(`mod retry { … }` など 5 つ)。本体は 1 文字も変わらないが、全体の字下げが変わり、`cargo fmt` が長い行を折り返す。パスは `tests::docs::<話題>::<名前>`。`git mv` の名前の変更も検出されにくい | 平らな形は、差分が小さく、`git mv` で `docs_graph.rs` の履歴を追え、Rust に不慣れなレビューでも読みやすい。代案は本体の文字を守れるが、差分が大きい。どちらも名前の最後の部分は同じ | 決定: 節で分けた平らな 1 ファイルにし、定数を `RETRY_HEADING`・`ENV_HEADING` にする(推奨のとおり。ユーザー、2026-11-25) |
| D2 | `graph_long_id.rs` の補助 `definition(long)` と、`graph_report.rs` の補助 `definition(ids, skips, extra)` の衝突 | **移す側を `long_id_definition` にする**(呼ぶ 2 行が変わる) | 子のモジュール `mod long_id { … }` に包む(本体は変わらない。パスが `tests::graph_report::long_id::<名前>`) | D1 と同じ考え(補助の名前で解き、ファイルを平らに保つ)。衝突するのは補助だけ | 決定: 移す側を `long_id_definition` にする(推奨のとおり。ユーザー、2026-11-25) |
| D3 | `state_read_raw.rs` の 4 件が使う `conductor_runtime::read_state`(引数 2 つ)と、`state_read.rs` の `common::without_run_id::read_state`(引数 1 つ)の衝突 | **子のモジュール `mod raw { … }` に包む**(本体は字下げだけ。パスが `raw::<名前>`。言語に依らない仕様書の根拠も `state_read::raw::…` にする) | 移す 4 件の中の `read_state` を別名(`read_state as read_state_with_run_id` など)にする(本体の約 10 行が変わる)/ `state_read_raw.rs` を残す(範囲から外す) | 衝突するのが外のクレートの関数の名前で、補助の名前を変えて解けない。子のモジュールなら本体の文字を変えずに済む。`raw` の名前は、元のファイル名の後ろの部分 | 決定: 子のモジュール `mod raw` に包む(推奨のとおり。ユーザー、2026-11-25) |
| D4 | 紛れ込んだ `fr2_5_remove_on_complete_…` の 2 件の行き先 | **新しい節 `--remove-on-complete` を、`resolve_run_id` の節の後(`graph` の節の前)に作る** | `--run-id と --all` の節の最後に置く / `init` の節の前(オプションの節の先頭)に置く | 今の `args_grammar.rs` に `--remove-on-complete` の節は無い。`--run-id`・`resolve_run_id` の直後に置けば、移す距離が短く(2 件の並べ替えだけ)、オプションの節が続く | 決定: `resolve_run_id` の節の後に `--remove-on-complete` の節を作る(推奨のとおり。ユーザー、2026-11-25) |
| D5 | ひな形のデータのファイルの改行(Windows の取り出しで CRLF になる見込み) | **リポジトリの根に `.gitattributes` を作り、`crates/cli/tests/fixtures/*.toml text eol=lf` の 1 行を置く**(コミット 12) | テストの側で `\r\n` を `\n` に直す(定数を関数や `LazyLock` にするので、使う側の 3 行ほどが変わる)/ 何もしない(Windows の CI で落ちうる) | 写しの使い捨ての git で、`core.autocrlf=true` の取り出しで CR が 29 個入り、`.gitattributes` があると 0 個になることを確かめた。今の原本は `.rs` の中なので、この問題は無かった。設定は 1 行で、テストと本番のコードを変えない。Windows で実際に落ちるかは CI でしか確かめられない | 決定: リポジトリの根に `.gitattributes`(`crates/cli/tests/fixtures/*.toml text eol=lf`)を置く(コミット 12。推奨のとおり。ユーザー、2026-11-25) |
| D6 | 言語に依らない仕様書 11 章の根拠の欄(`convert_ac::…`、`convert_roundtrip::…`、`state_read_raw`、`docs_*`。L1〜L10) | **新しいモジュールの名前に直す**(`docs_*` は `docs`、`state_read_raw` は `state_read`(`::raw::` つき)、converter の 3 か所は入れ替え) | 直さない(言語に依らない仕様書は触らない) | 根拠の欄が、無くなるファイルや、別のファイルへ移ったテストを指すことになる。前の intent でも、ファイル名が変わった根拠の欄は直した。凍結した仕様書ではない | 決定: L1〜L10 を新しいモジュールの名前に直す(推奨のとおり。ユーザー、2026-11-25) |
| D7 | 言語に依らない仕様書 11 章 772 行目(使い方の誤りの全種類)の根拠の `tests/common/kinds.rs`。実物の一覧は cli の `src/tests/common/kinds.rs` | **今回は直さない**(この intent の移動と無関係な、前からの食い違い。core の同じ名前のファイルは残る) | cli の単体テストの `src/tests/common/kinds.rs` に直す(コミット 3 か 11 に 1 行) | 直すなら 1 行で、範囲は小さい。ただし、この intent は「ファイルの統合」で、移したファイルを指す記述ではない | 決定: 今回は直さない(推奨のとおり。ユーザー、2026-11-25) |
| D8 | ひな形のデータのファイルの名前 | **`crates/cli/tests/fixtures/init_ja.toml`、`init_en.toml`**(Q4 の例のとおり) | `init_template_ja.toml`・`init_template_en.toml`(本番の `report/init_template.rs` と名前をそろえる) | converter・core の `tests/fixtures/` と同じ置き場所。cli に `tests/fixtures/` は無いので新しく作る。名前は、`init` が書き出すひな形であることが分かれば足りる | 決定: `init_ja.toml`・`init_en.toml` にする(推奨のとおり。ユーザー、2026-11-25) |

D1〜D8 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-25)**: §4 の T 表(T1〜T13。消すテスト・名前を変えるテストは無い。本体で変わる 5 行と、字下げが変わる 4 件)、§2 の項目ごとの形、§8 のコミットの並び。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた(確かめる手段の小さな項目は、親の項目に含める)。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | cli の README のテストは `src/tests/docs.rs` 1 つになり、補助の `readme` の写しは 1 つ。元の 10 ファイルは無くなる。統合テストの 5 つは単体テストへ階層が変わる | BR1.1、BR1.2、BR1.3 |
| SM2 | converter・cli・runtime の (2)〜(4) のファイルは、まとめ先へ移って無くなる。`run_id_arg.rs` は `--run-id` の節へ移り、紛れ込んだ 2 件は正しい節へ並ぶ | BR2.1、BR2.2、BR2.3、BR3.1、BR3.2、BR3.3、BR4.1、BR4.2 |
| SM3 | cli のテストの補助の中身は 1 つのファイルにだけあり、統合テストと単体テストの両方がそれを読む | BR5.1、BR5.2、BR5.3 |
| SM4 | init のひな形の原本はテスト用のデータのファイルにだけあり、両方のテストがそれを読む | BR6.1、BR6.2 |
| SM5 | テストの名前・確かめの行・期待の値は変えない。名前の最後の部分で比べた差 0。件数が「基準 − 消したテスト + 足したテスト」に一致 | BR9.2、BR9.3 |
| SM6 | 製品のコードの検査のテストの数と中身を変えずに通る | BR9.6 |
| SM7 | 仕様書のテストのファイルの表と根拠の欄が実装と合い、無くなったファイルの名前が残らないことを検索で確かめる | BR7.1、BR7.2、BR7.3 |
| SM8 | 利用者から見える動きは変わらない。本番のコードは変えない。版は上げない | BR9.1、BR9.5 |
| SM9 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
