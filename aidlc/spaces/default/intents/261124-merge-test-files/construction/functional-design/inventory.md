# 変える場所の一覧(Inventory): テストのファイルの統合

HEAD 16ac9d6(merge-test-files の意図の把握のコミット。版 0.9.8)の時点で、実物を読んで作った一覧。リポジトリのコードは変えていない。行番号はこの時点のもの。初期の記述と実物が違ったところは「(記述との違い)」と書く。設計の要点、テストの移動の一覧(T…)、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent の判断と、この intent の Q1〜Q5): **テストのコードの都合で本番のコードを曲げない**。本番のコード(`crates/*/src` の `src/tests/` の外)は 1 行も変えない。利用者から見える動きは変えない。版は上げない。テストの名前・確かめの行・期待の値は変えない(変わるのはファイルとパス、import、補助の読み方)。消すテスト・名前を変えるテストは無い(★ なし)。

決めてほしい点(`functional-spec.md` §10 の D1〜D8)は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。D1: `docs.rs` は節で分けた平らな 1 ファイルにし、定数を `RETRY_HEADING`・`ENV_HEADING` にする。D2: 移す側の補助を `long_id_definition` にする。D3: `state_read_raw.rs` の 4 件を子のモジュール `mod raw` に包む。D4: `resolve_run_id` の節の後に `--remove-on-complete` の節を作る。D5: リポジトリの根に `.gitattributes`(`crates/cli/tests/fixtures/*.toml text eol=lf`)を置く。D6: 言語に依らない仕様書の根拠の欄 L1〜L10 を直す。D7: 772 行目は今回は直さない。D8: データのファイルの名前は `init_ja.toml`・`init_en.toml`。この文書の「(D1)」〜「(D8)」は、その決定のとおりの形を書いている(写しの木にも同じ形を当てた)。T 表(T1〜T13)は、設計の承認で認めてもらう(承認待ち)。

範囲の外(初期の記述): 本番の小さなモジュールの統合(別の intent)、core の補助と性能のテストの統合、`nfr_perf.rs`・`msrv_consistency.rs`・`platform`・converter の src の直下のファイル。

## 0. 基準と、設計の検証

### 0.1 テストの件数(`cargo test --workspace --locked`、macOS、rustc 1.100.0、debug、HEAD 16ac9d6)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 112 | 269 | 381 |
| runtime | 69 | 128 | 197 |
| converter | 96 | 19 通過 + 1 無視 | 115 通過 + 1 無視 |
| cli | 162 | 294 | 456 |
| 全体 | | | **1,149 通過、1 無視、0 失敗** |

- テスト名の一覧は `cargo test --workspace --locked -- --list` の `: test` の行(1,150 行。無視の 1 件を含む)。
- この intent に関わる実行ファイルの件数(HEAD): cli の `docs_dry_run` 1、`docs_env` 2、`docs_graph` 2、`docs_init` 1、`docs_retry` 4、`flow_init` 9、cli の lib の単体テスト 162(うち `tests::docs_dry_run` 1、`tests::docs_graph` 6、`tests::docs_init` 1、`tests::init_types` 1、`tests::run_id_arg` 1、`tests::graph_long_id` 2)。converter の `convert_ac` 8、`convert_roundtrip` 1、`nfr_security` 10 + 無視 1、converter の lib 96(うち `tests::convert_ac` 2)。runtime の `runner_basic` 12、`runner_env` 5、`state_read` 15、`state_read_raw` 4。
- `cargo doc --workspace --no-deps --locked` の警告は 5 件(core 1、runtime 4。この intent と無関係)。

### 0.2 設計の検証(写しの木での試し)

T 表を検索だけに頼らないために、HEAD 16ac9d6 の写し(`git archive HEAD` を `target/merge-tests/work` に展開。ビルドは `target/merge-tests/tgt`)に、この一覧の全項目と `functional-spec.md` §4 の T 表と §2 の形(D1〜D8 は推奨の形)を当て、次を確かめた。写しは確かめたあと消した(`target/merge-tests` ごと)。リポジトリのコードは変えていない。

- `cargo fmt --all -- --check` が通る(`cargo fmt` が整えたのは 1 か所だけ: `state_read.rs` の子のモジュールへ移した `fr3_4_…` の 1 行が、字下げで 100 文字を超え、2 行に折り返された。中身は同じ)。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc` の両方で通る。`cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(本番の形)も通る。途中で 1 回だけ止まった: 統合テストの `tests/common/mod.rs` で、偽の実装を `fakes.rs` へ出すと `use std::io;` が使われなくなる(`unused_imports`。`#![allow(dead_code)]` では止まらない)。設計(§5)に、この import を消すことを入れた。
- `cargo test --workspace --locked` が **1,149 通過、1 無視、0 失敗**(core 112 + 269、runtime 69 + 128、converter 96 + 19 + 1 無視、cli **172** + **284**)。実行ファイルごとの差: cli の lib 162 → 172、`docs_*` の 5 つが無くなる(計 10 件)、converter の `convert_ac` 8 → 9、`convert_roundtrip`(統合)が無くなる、runtime の `runner_basic` 12 → 17、`runner_env` が無くなる、`state_read` 15 → 19、`state_read_raw` が無くなる。ほかの実行ファイルの件数は同じ。
- テスト名の一覧の差(名前の最後の部分で、並べ替えて比べた): **差 0**(足した名前・消えた名前・変わった名前は無い。★ なし)。パスで比べた差は 56 行(移った 28 件の前と後。§4 の T 表の「パス」の列のとおり)。統合テストの間で移った 6 件(converter の `出力は毎回同じ`、runtime の `runner_env` の 5 件)は、`--list` が実行ファイルの名前を出さないのでパスの差に出ない。実行ファイルごとの件数で確かめた。
- 移したテストの本体: 空白を除いて元と比べ、28 件 + 6 件のうち、違うのは補助の名前を変えた 5 行だけ(`ENV_HEADING` 2 行、`RETRY_HEADING` 1 行、`long_id_definition` 2 行。D1、D2)。確かめの行(`assert…`)と期待の値は、すべて元と同じ。
- init のひな形のデータのファイル 2 つは、HEAD の `src/tests/init_template.rs` の原本の文字列と、バイト単位で一致する(`cmp`)。
- 製品のコードの検査: cli の `nfr_source` 6 件、runtime の `nfr_source` 5 件、converter の `nfr_security` 10 件(+ 無視 1)が、数を変えずに通る(検査のテストのコードは変えていない。converter の `nfr_security.rs` は、誤りの種類の一覧を受け入れるだけで、検査の部分は変えない)。
- 改行(D5): 写しの中の使い捨ての git の作業場所で、`core.autocrlf=true` でひな形のデータのファイルを取り出すと、CR が 29 個入る(`include_str!` は改行を変えないので、ひな形の全文の一致のテストが Windows で落ちる)。`.gitattributes` に `crates/cli/tests/fixtures/*.toml text eol=lf` を置くと、CR は 0 個(`git check-attr` で `text: set`、`eol: lf`)。使い捨ての場所は確かめたあと消した。
- `cargo doc --workspace --no-deps --locked` の警告は基準と同じ 5 件。
- 行カバレッジ(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`): 全体の行 **98.06%**(80% 以上)。
- 本番のコードの差: 写しと HEAD の `crates/*/src` を比べ、`src/tests/` の外の差は 0。
- 仕様書の書き換え(§7)も写しに当て、`functional-spec.md` §7 の検索が期待どおりに出ることを確かめた。凍結した仕様書(go、python、typescript、csharp、powershell)は HEAD と同じ。
- 使ったツールは rustc・cargo 1.100.0(MSRV と同じ版)。

---

## 1. cli の README のテスト(項目 1。Q2: A)

### 1.1 今のファイル(10 ファイル、計 613 行)

| ファイル(HEAD) | 行 | テスト | 中身 |
|---|---|---|---|
| `crates/cli/tests/docs_support/mod.rs` | 65 | — | 補助 `DOCS_CHECK_WORDS`、`readme`、`assert_readme_has`、`assert_docs_check_words`、`block_after_heading`。`#![allow(dead_code)]` |
| `crates/cli/tests/docs_retry.rs` | 79 | 4 | 定数 `HEADING`(リトライの見出し)。`readme_に_job_retry_と_5_つのキーが書かれている`、`古い一文はそのままの形では残っていない`、`readme_の例の定義を実際の読み取りに通すとリトライの設定が入っている`、`ci_の_docs_check_の_7_語が_readme_にある` |
| `crates/cli/tests/docs_env.rs` | 73 | 2 | 定数 `HEADING`(環境変数の見出し)。`readme_に必須の語句と版の表記がある`、`readme_の例の定義と_env_を実際の読み取りと解析に通せる` |
| `crates/cli/tests/docs_graph.rs` | 62 | 2 | `readme_に_graph_の必須の語句と版の表記がある`、`readme_の辺の一覧に切り替わる形は_4つの種類の外の形だけを述べる` |
| `crates/cli/tests/docs_dry_run.rs` | 38 | 1 | `readme_に_dry_run_の必須の語句と版の表記がある` |
| `crates/cli/tests/docs_init.rs` | 38 | 1 | `readme_に_init_の必須の語句と版の表記と終了コード_69_がある` |
| `crates/cli/src/tests/docs_support.rs` | 27 | — | `readme` の写し |
| `crates/cli/src/tests/docs_graph.rs` | 146 | 6 | 補助 `graph_example`、`render`、`jobs`。図の例 6 件 |
| `crates/cli/src/tests/docs_dry_run.rs` | 54 | 1 | `readme_の計画の例は_実際の出力と一致する` |
| `crates/cli/src/tests/docs_init.rs` | 31 | 1 | `readme_のひな形の例は_実際の日本語のひな形と一致する` |

### 1.2 確かめたこと

- 統合テストの 5 つは、cli の公開の入口を使わない。使うのは README(ファイルの読み取り)、`conductor_core::{load_definition, parse_env_file}`(core は cli の通常の依存)、`tempfile`(cli の `[dev-dependencies]` にある `tempfile = "3.27.0"`。単体テストのビルドにも入る)。**依存を足さずに単体テストへ移せる**(写しで確かめた)。
- 名前の衝突: テスト名 18 個はすべて違う。補助の衝突は、定数 `HEADING` だけ(`docs_retry.rs` と `docs_env.rs` で値が違う)。`readme` は統合と単体の 2 か所にあるが、1 つにする。単体の `graph_example`・`render`・`jobs` は、ほかと重ならない。
- 1 つのファイルにすると、補助はすべて使われる(`DOCS_CHECK_WORDS` は `ci_の_docs_check_…` と `assert_docs_check_words`、`block_after_heading` は環境変数とリトライ)。`#![allow(dead_code)]` は要らなくなる(写しで clippy が通る)。
- cli の `nfr_source.rs` の 2 つの検査(`src_に_unsafe_と_low_level_がない`、`src_にネットワークのコードがない`)は `src/tests/` も見る。移す中身に、止める語(`unsafe`、`low_level`、`std::net`、`curl`、`tokio` など)は無い(写しで 6 件が通る)。
- CI(`.github/workflows`)と README は、`docs_*` のファイル名を指さない(`docs-check` は README の語句の検査で、テストのファイルと無関係)。

### 1.3 まとめた後(`crates/cli/src/tests/docs.rs`、約 450 行)

- 並び: 補助(`DOCS_CHECK_WORDS`、`readme`、`assert_readme_has`、`assert_docs_check_words`、`block_after_heading`)→ README の並びの 5 つの節(リトライ、環境変数、`graph`、`--dry-run`、`init`)。各節は、統合テストだった語句の検査、単体テストだった例の比べ方の順。
- 定数 `HEADING` は `RETRY_HEADING` と `ENV_HEADING` にする(D1)。
- 無くなるファイル: §1.1 の 10 ファイルと `tests/docs_support/` のディレクトリ。`src/tests/mod.rs` の `mod docs_support;`、`mod docs_dry_run;`、`mod docs_graph;`、`mod docs_init;` を `mod docs;` にする。
- 統合テストの実行ファイルが 5 つ減る。テスト 10 件が統合から単体へ階層が変わる(パス `tests::docs::<名前>`)。単体の 8 件はパスが `tests::docs_<話題>::<名前>` から `tests::docs::<名前>` になる。

## 2. converter(項目 2)

| # | 元(HEAD) | 移す先 | 移すもの | 確かめたこと |
|---|---|---|---|---|
| C1 | `crates/converter/tests/convert_roundtrip.rs`(31 行、1 件) | `crates/converter/tests/convert_ac.rs` | 定数 `PIPELINE_TOML`・`PIPELINE_YAML`(`include_str!("fixtures/…")`。同じディレクトリなのでパスは同じ)と `出力は毎回同じ` | `convert_ac.rs` は `convert_text` と `TargetFormat` を import 済み。名前の衝突なし。置き場所は `convert_text_はファイルに触れず_ラベルを使う`(:115)の後(同じ `convert_text` の話題) |
| C2 | `crates/converter/src/tests/convert_ac.rs`(57 行、2 件) | `crates/converter/src/tests/convert_roundtrip.rs` | 補助 `fixture`・`request` と `ac_7_1_1_toml_から_yaml`・`ac_7_1_2_アンカーつきの_yaml_から_toml` | import に `std::path::{Path, PathBuf}`、`ConversionRequest`、`convert` を足す。`fixture`・`request` の名前は移す先に無い。ファイルの最後に節「受け入れ基準(ファイルを読む入口 `convert` の結果の木)」として置く |
| C3 | `crates/converter/tests/common/mod.rs`(`pub mod kinds;` だけ)と `common/kinds.rs`(一覧 `CONVERSION_ERROR_KINDS`、網羅の `match`、コンパイル時の確かめ) | `crates/converter/tests/nfr_security.rs` | 一覧の 3 つの部分 | `common` を使うのは `nfr_security.rs` だけ(`git grep "mod common"` で 1 件)。別名は、ファイルが使う `ConversionErrorKind as K` にそろえる(`kinds.rs` の `as C` を `K` に書き換える。テストの本体ではない)。`pub` は外す(テストの実行ファイルの根なので要らない)。置き場所は、一覧を使うテスト `表示は_どの種類でも入力の値を含まない` がある節(表示の節)の先頭。一覧の前の説明は、`kinds.rs` のモジュールの説明を `//` のコメントにして写す。`mod common;` と `use common::kinds::…` を消す |
| C4 | `crates/converter/src/tests/mod.rs:17-22`(説明と `mod convert_ac;`) | 同じ | 説明「下位のモジュールのファイル名は、元の統合テスト(`tests/` の下)と同じにしている」は、C1・C2 の後に成り立たない(`convert_roundtrip` の統合テストが無くなり、`read_resolve` は前から統合テストに無い)。下位のモジュールの中身を並べる説明にする。`mod convert_ac;` を消す | — |

- 製品のコードの検査 `tests/nfr_security.rs` の `src_files()` は `src/` の直下の `*.rs` だけを読む(`src/tests/` を見ない)。一覧を受け入れても、検査の数(10 件 + 無視 1)と中身は変わらない。検査の語は `["std::", "process"].concat()` のように分けて組み立てているので、一覧の文字は当たらない(このファイル自身も対象の外)。
- 名前の入れ替わりの注意: 移した後、統合テストの `convert_ac` に「出力が毎回同じ」が、単体テストの `convert_roundtrip` に `ac_7_1_1_…`・`ac_7_1_2_…` の木の比較が入る。言語に依らない仕様書の根拠の欄を直す(§7.2)。

## 3. cli の小さな単体テスト(項目 3。Q5: A)

### 3.1 `args_grammar.rs` の今の節(HEAD の行)

| 行 | 節の見出し | 中身 |
|---|---|---|
| 388-486 | `// ---- init(…) ----` | `init` の文法 7 件 |
| 488-614 | `// ---- --run-id と --all(…) ----` | 5 件 |
| 616-682 | `// ---- resolve_run_id(…) ----` | 補助 `resolved` と 4 件。**うち :643 `fr2_5_remove_on_complete_は_run_と_pipeline_だけが受け付ける` と :658 `fr2_5_remove_on_complete_は値を取らず_2_回の指定は誤り` は `--remove-on-complete` の文法で、`resolve_run_id` と無関係(記述の「紛れ込んだ 2 件」は実物でこの 2 件)** |
| 684- | `// ---- graph(…) ----` 以降 | — |

- `--remove-on-complete` の節は、今の `args_grammar.rs` に無い(`--dry-run` の節の :781・:792 と、報告の順の節の :885 以降に、ほかのオプションと一緒に出るだけ)。

### 3.2 移す・並べ直すもの

| # | 元(HEAD) | 移す先(`args_grammar.rs`) | 確かめたこと |
|---|---|---|---|
| A1 | `src/tests/init_types.rs:24-34` `ハイフンだけが標準出力でそれ以外はファイル`(34 行のファイルの残る 1 件) | `init` の節の最後(:486 の後) | import の `OsString`・`InitTarget` は `args_grammar.rs` に既にある。名前の衝突なし。テストの前に、元のファイルの説明の 1 行目を `///` の説明として付ける(本体ではない) |
| A2 | `src/tests/run_id_arg.rs:20-41` `run_id_arg_は空を固定_規則どおりを値_規則違反を値を含めない誤りにする`(1 件) | `--run-id と --all` の節の最後(:614 の後。`run_id_の無効な値は_68_…` の次) | import の `RunIdArg`・`UsageErrorKind`・`RunId` は既にある。補助の関数 `run_id` があるが、移すテストは呼ばない(変数 `empty`・`arg`・`e` だけ)。名前の衝突なし |
| A3 | `args_grammar.rs:643-668` の `fr2_5_…` 2 件 | 新しい節 `// ---- --remove-on-complete ----`(括弧の中の番号は、テストの名前の先頭 `fr2_5_` と同じ番号を、ほかの節の見出しと同じ書き方で添える)を `resolve_run_id` の節の後(`graph` の節の前)に作り、そこへ(D4) | 同じファイルの中の並べ替え。本体は変えない。`resolve_run_id` の節は 2 件が続けて並ぶ |
| A4 | `src/tests/graph_long_id.rs`(67 行、2 件と補助 `definition(long)`・`long_id()`) | `src/tests/graph_report.rs` の最後に、節「65 文字以上のジョブ ID」として | **補助 `definition` が `graph_report.rs:26` の `definition(ids, skips, extra)` と名前が衝突する**。移す側を `long_id_definition` にする(呼ぶ 2 行が変わる。D2)。import(`render_text`、`Language`、`GraphLayout`、`build_graph`、`layout_graph`、`PipelineDefinition`、`parse_definition`)は `graph_report.rs` にすべてある。元のファイルの説明(4 行)は節の見出しの下のコメントにする |

- 無くなるファイル: `init_types.rs`、`run_id_arg.rs`、`graph_long_id.rs`。`src/tests/mod.rs` の 3 つの `mod` を消す。
- `args_grammar.rs` の `#[cfg(unix)]` のテスト(:316、:347、:1120)と、`run_id_の無効な値は_68_…` の中の `#[cfg(unix)]` の塊は、置き場所を変えない(A1〜A3 はその外)。
- モジュールの説明: `args_grammar.rs` に「引数から作る型(`InitTarget::from_path`、`RunIdArg::parse`)と `resolve_run_id` も確かめる」の 2 行、`graph_report.rs` に最後の節の 1 行を足す。

## 4. runtime(項目 4)

| # | 元(HEAD) | 移す先 | 確かめたこと |
|---|---|---|---|
| R1 | `crates/runtime/tests/runner_env.rs`(89 行。補助 `child_env` と 5 件) | `crates/runtime/tests/runner_basic.rs` の、`#[cfg(windows)]` の 3 件(:280 以降)の**前**に、節「環境変数の引数」として | 使う import(`ExecutionResult`、`InterruptSignal`、`run_job`、`common::*` の `helper_command`・`command_line`・`lines`)は `runner_basic.rs` にすべてある。`child_env` の名前は無い。5 件とも OS の分岐を持たない(Windows でも UNIX でもコンパイルされ、動く。`PATH` を使う 2 件も両方の OS で使える)。`#[cfg(windows)]` の 3 件はファイルの最後のままにし、前に節の見出し「Windows の cmd(Windows だけ)」を足す。モジュールの説明に、元のファイルの説明(2 行)を足す |
| R2 | `crates/runtime/tests/state_read_raw.rs`(93 行。定数 `COMPLETED` と 4 件) | `crates/runtime/tests/state_read.rs` の最後に、子のモジュール `mod raw { … }` として(D3) | **`state_read.rs:20` は `use common::without_run_id::read_state;`(引数 1 つの包み)で、移すテストは `conductor_runtime::read_state`(引数 2 つ)を呼ぶ。同じ名前 `read_state` が衝突する**。子のモジュールの中で `use super::common::{def, put};`(`common` は親のモジュールにあるので `super::` を付ける)、`use conductor_core::RunId;`、`use conductor_runtime::{StateErrorKind, read_state, read_state_raw};` を置く。本体は字下げだけが変わる(`cargo fmt` が 1 行を折り返す)。`mod raw` は `#[cfg]` を付けない(4 件とも OS に依らない)。`state_read.rs` の `#[cfg(unix)]` の 3 件と補助 `within_5s`(:106、:219、:241、:253)は、置き場所を変えない |

- 無くなるファイル: `runner_env.rs`、`state_read_raw.rs`。
- パス: R1 の 5 件は統合テストの関数の名前のまま(実行ファイルが `runner_basic` に変わる)。R2 の 4 件は `raw::<名前>` になる(名前の最後の部分は同じ)。
- runtime の `nfr_source.rs` は `src/` だけを見る(`tests/` は対象の外)。5 件の数と中身は変わらない。

## 5. cli のテストの補助の写し(項目 5。Q3: A)

### 5.1 今の形

| 場所(HEAD) | 中身 |
|---|---|
| `crates/cli/tests/common/mod.rs:36-141` | テスト用の写しの `enum Language { En, Ja }`(:38-43)、`FakeEnv`(:45-102。`locale_read`、`color_read`、`with`、`for_language`、`impl Env`)、`FakeSink`(:104-141。`out_text`、`err_text`、`impl Sink`) |
| 同 `:288-296` | `state_error`(説明のコメントだけが単体の写しと違う) |
| `crates/cli/src/tests/common/mod.rs:22-140` | `pub mod kinds;`、同じ `FakeEnv`・`FakeSink`・`state_error`(本体は文字単位で同じ。`diff` で確かめた)。trait は `crate::deps::{Env, Sink}`、言語は `crate::types::Language`(`En`・`Ja` の 2 つ) |

- 違いは、trait の出どころ(統合テストは `conductor_cli::{Env, Sink}`、単体テストは `crate::deps::{Env, Sink}`)と `Language` の型だけ。
- 統合テストの `color_read` は使われないが、`tests/common/mod.rs:16` の `#![allow(dead_code)]` があるので警告にならない。単体テストは、写しのすべての項目を使っている(今、警告なしでコンパイルされる)。

### 5.2 まとめた後(推奨の形。写しで確かめた)

- 新しいファイル `crates/cli/tests/common/fakes.rs`(約 140 行): `FakeEnv`、`FakeSink`、`state_error`。import は `std::cell::{Cell, RefCell}`、`HashMap`、`io`、`Path`、`conductor_runtime::{StateError, StateErrorKind}` と、**`use super::{Env, Language, Sink};`**(trait と言語は、読む側の親のモジュールが用意する)。先頭のモジュールの説明に、注意書き「`use super::Language` は、統合側の写しの enum と単体側の `crate::types::Language` の両方を読む。両者の変種(今は `En`・`Ja`)を食い違わせないこと」を置く(`functional-spec.md` §2.5)。
- 統合テスト `tests/common/mod.rs`: 写しの部分(:45-141、:288-296)を消し、`mod fakes;` と `pub use fakes::{FakeEnv, FakeSink, state_error};` にする。`Language` の enum と `use conductor_cli::{…, Env, …, Sink, …}` は残す(`fakes.rs` が `super::` で使う。ほかの補助も使う)。`use std::io;` は使われなくなるので消す(残すと `unused_imports` で clippy が止まる。写しで確かめた)。`Cell`・`RefCell`・`HashMap`・`Path`・`StateError`・`StateErrorKind` は、ほかの補助が使うので残る。`#![allow(dead_code)]` は子のモジュールにも効く(`color_read` の警告は出ない)。
- 単体テスト `src/tests/common/mod.rs`(29 行になる): `pub mod kinds;`、`#[path = "../../../tests/common/fakes.rs"] mod fakes;`、`pub use fakes::{FakeEnv, FakeSink, state_error};`、`use crate::deps::{Env, Sink};`、`use crate::types::Language;`。`#[path]` は `mod.rs` のあるディレクトリ(`src/tests/common/`)からの相対で、`crates/cli/tests/common/fakes.rs` を指す。`Env`・`Sink`・`Language` の import は、子の `fakes` が `super::` で使うので「使われている」と見なされる(写しで警告なし)。
- 使う側(`args_grammar.rs` の `common::FakeEnv::with` など、統合テストの `common::{FakeEnv, FakeSink, …}`)は変えない(再公開の名前が同じ)。
- 製品のコードの検査: cli の `nfr_source.rs` の `テスト専用の経路を実行ファイルに持たない` は、`src/tests/` の外の `#[path`・`include!`・`include_str!`・`include_bytes!` を止める。`#[path` は `src/tests/common/mod.rs`(`is_unit_test` が真)にあるので、検査の外。`tests/common/fakes.rs` は `src/` の外なので、`src/` を見る検査のどれにも入らない。写しで 6 件が通る。
- `fakes.rs` は `tests/` の直下ではない(`tests/common/` の下)ので、cargo は統合テストの実行ファイルにしない。

## 6. init のひな形の原本(項目 6。Q4: A)

### 6.1 今の形

| 場所(HEAD) | 中身 |
|---|---|
| `crates/cli/tests/flow_init.rs:27-91` | `ORIGINAL_JA`・`ORIGINAL_EN`(生の文字列、各 約 30 行)。使うのは `expected_template`(:95-101) |
| `crates/cli/src/tests/init_template.rs:23-85` | 同じ 2 つ(文字単位で同じ)。使うのは `日本語と英語のひな形は_原本と_1_文字も違わず一致する`(:89-90) |
| `README.md` の「init の使い方」の節 | 日本語のひな形の例。単体テスト `readme_のひな形の例は_実際の日本語のひな形と一致する` が、本番の `render_template(Ja, "my-pipeline")` と比べる |

### 6.2 まとめた後

- 新しいデータのファイル `crates/cli/tests/fixtures/init_ja.toml`・`init_en.toml`(名前は D8)。中身は HEAD の生の文字列と**バイト単位で同じ**(末尾の改行 1 つ。写しで `cmp` が一致)。
- `flow_init.rs`: `const ORIGINAL_JA: &str = include_str!("fixtures/init_ja.toml");`(英語も同じ)。`src/tests/init_template.rs`: `include_str!("../../tests/fixtures/init_ja.toml")`。定数の名前と型(`&str`)は変えないので、使う側の行は変わらない。説明に「統合テスト/単体テストも同じファイルを読む」を書く。
- `include_str!` は `src/tests/init_template.rs`(単体テスト)と `tests/flow_init.rs`(統合テスト)にだけあり、cli の `nfr_source.rs` の `include` の検査の外(写しで通る)。converter の単体テストが `include_str!("../../tests/fixtures/…")` を使う前例がある。
- README との関係: README のテストは本番の出力と比べ、ひな形のテストは本番の出力とデータのファイルを比べる。「README ↔ 本番 ↔ データのファイル」の鎖は変わらない。README のテストは変えない。
- **(記述との違い・足した点)改行**: `include_str!` は改行をそのまま取り込む(Rust のソースの CRLF は字句解析で LF になるが、取り込んだファイルはならない)。HEAD の原本は `.rs` の中なので、Windows の CI(`actions/checkout` は、Git for Windows の既定 `core.autocrlf=true` で取り出すと見込まれる。README を読むテストが `replace("\r\n", "\n")` をしているのも、この備え)でも LF だった。データのファイルにすると、Windows で CRLF になり、全文の一致と `flow_init` の 9 件のうちひな形を比べるものが落ちる見込み(写しの使い捨ての git で、CR が 29 個入ることを確かめた)。リポジトリに `.gitattributes` は無い。対策は D5(推奨: リポジトリの根に `.gitattributes` を置き、`crates/cli/tests/fixtures/*.toml text eol=lf`)。converter の `tests/fixtures/*` と core の `tests/fixtures/*.md` は、木や行(`lines()`)で比べるので、CRLF でも落ちない(今のまま)。

## 7. 仕様書

### 7.1 `docs/spec/rust`

| # | 場所(HEAD) | 今 | 直した後 | コミット |
|---|---|---|---|---|
| S1 | `02:273`(偽の実装の表の前の文) | 「(`crates/cli/tests/common/mod.rs`。単体テストの `crates/cli/src/tests/common/mod.rs` は、`FakeEnv`・`FakeSink` の写しを持つ)」 | 「(`crates/cli/tests/common/mod.rs`。`FakeEnv`・`FakeSink` は `crates/cli/tests/common/fakes.rs` にあり、単体テストの `crates/cli/src/tests/common/mod.rs` も、写しを持たずに同じファイルを `#[path]` で読む)」 | 11 |
| S2 | `02:357`(converter のファイルの表の `tests/` の行) | 「`tests/`(`mod.rs` と、元の統合テストと同じ名前の 4 つ)」 | 「`tests/`(`mod.rs`、`convert_roundtrip.rs`、`nfr_security.rs`、`read_resolve.rs`)」 | 2 |
| S3 | `02:389`(cli のファイルの表の `tests/` の行) | 「`tests/`(`mod.rs`、`common/`、`docs_support.rs` と、元の統合テストと同じ名前のファイル)」「クレートの内部(…)を直接確かめる単体テスト」 | 「`tests/`(`mod.rs`、`common/`、README の検査の `docs.rs` と、引数・表示などのファイル)」「クレートの内部(…)と README を直接確かめる単体テスト」 | 9 |
| S4 | `07:428`(テストの表の `env` 引数の行) | `crates/runtime/tests/runner_env.rs` | `crates/runtime/tests/runner_basic.rs` | 4 |
| S5 | `11:227-228`(runtime の表) | `runner_basic.rs` の行と `runner_env.rs` の行 | `runner_env.rs` の行を消し、`runner_basic.rs` の目的の「起動前の合図。」の後に「v0.9.0 以降は `run_job` の `env` 引数(足した値が子に見える、引き継いだ環境が残る、同じ名前の上書き、並びの後ろが勝つ)。」を足す | 4 |
| S6 | `11:233` | 「`state_read.rs`、`state_read_raw.rs`、`state_write.rs`」 | `state_read_raw.rs` を消し、目的の後に「`state_read.rs` の子のモジュール `raw` は、完了のステートも返す `read_state_raw` と `read_state` の差」を足す | 5 |
| S7 | `11:244-245`(converter の統合テストの表) | `convert_ac.rs` の行(「…2 件は単体テストの `src/tests/convert_ac.rs`」)と `convert_roundtrip.rs` の行 | `convert_roundtrip.rs` の行を消す。`convert_ac.rs` の行を「受け入れ基準と、`convert`(ファイルを読む入口)、出力が毎回同じ。変換の結果を読んだ木を比べる 2 件と往復の木の比較は単体テストの `src/tests/convert_roundtrip.rs`」に | 1、2 |
| S8 | `11:246`、`11:248` | `nfr_security.rs` の目的「3 章」、`common/kinds.rs` の行 | `common/kinds.rs` の行を消し、`nfr_security.rs` の目的を「3 章。変換の失敗の種類の一覧の写し(`CONVERSION_ERROR_KINDS`。網羅の `match` と、並び・重複のコンパイル時の確かめつき)を持つ」に | 3 |
| S9 | `11:256-257`(converter の単体テストの表) | `convert_roundtrip.rs`「往復の木の比較」、`convert_ac.rs`「変換の結果の木を比べる受け入れ基準」 | `convert_ac.rs` の行を消し、`convert_roundtrip.rs` を「往復の木の比較と、変換の結果の木を比べる受け入れ基準(2 件)」に | 2 |
| S10 | `11:270`、`11:275`、`11:280`(cli の統合テストの表) | `docs_env.rs`… の行、`docs_retry.rs` の行、`docs_support/mod.rs` の行 | 3 行を消す | 9(270 の後半)、10 |
| S11 | `11:278`(cli の統合テストの `common/mod.rs` の行) | 「…、`state_error`、`job_id_of`、…」 | 末尾に「`FakeEnv`・`FakeSink`・`state_error` は `common/fakes.rs` から再公開する」を足し、次に 2 行を足す: `common/fakes.rs`(偽の `Env`・`Sink` と `state_error`。単体テストも `#[path]` で同じファイルを読む。`Env`・`Sink`・`Language` は読む側の親のモジュールが用意する)、`fixtures/init_{ja,en}.toml`(`init` のひな形の原本。`flow_init.rs` と単体テストの `init_template.rs` が `include_str!` で読む) | 11、12 |
| S12 | `11:290`(cli の単体テストの `common/mod.rs` の行) | 「統合テストの `tests/common/mod.rs` の補助の写し(…)」 | 「統合テストの `tests/common/fakes.rs`(`FakeEnv`・`FakeSink`・`state_error`)を `#[path]` で読む(写しを持たない)」 | 11 |
| S13 | `11:292`、`11:295`、`11:299`、`11:302`、`11:303` | `docs_support.rs`、`run_id_arg.rs`、`graph_long_id.rs`、`init_types.rs`、`docs_graph.rs`… の行 | 5 行を消す。代わりに `docs.rs` の行「README の検査(必須の語句、版の表記、`docs-check` の 7 語、リトライと `.env` の例が実際の読み取りに通る。v0.9.0 以降は、図・計画・ひな形の例が実際の出力と一致)。補助(`readme`、`DOCS_CHECK_WORDS`、`block_after_heading`、`assert_readme_has`、`assert_docs_check_words`)も持つ」を `args_grammar.rs` の行の次に足す | 6、7、8、9、10 |
| S14 | `11:293`(`args_grammar.rs`) | 「…`--force` を含む)」 | 後ろに「。引数から作る型(`InitTarget::from_path`(`-` だけが標準出力)、`--run-id` の指定 `RunIdArg::parse`(空は固定、規則どおりは値、規則違反は値を含めない `invalidValue`))と `resolve_run_id`」を足す | 6、7 |
| S15 | `11:298`(`graph_report.rs`) | 「…Mermaid のエスケープ(08 の 11 章)」 | 後ろに「。text で、65 文字以上のジョブ ID を切らずに出す(レーンの図と辺の一覧。`escape_full` と遷移の表示の `escape_name` の使い分け)」を足す | 8 |
| S16 | `11:301`(`init_template.rs`) | 「…`validate` に通る)」 | 後ろに「。原本は統合テストの `tests/fixtures/init_{ja,en}.toml` を `include_str!` で読む」を足す | 12 |

- 変えない: `11:221` と `11:286`(core と cli の単体テストの `common/kinds.rs`。残るファイル)、`02:296`、`07:5`(`runner_*` の glob。`runner_basic`・`runner_signal` が残る)、`09:3`(converter の `tests/*` と `src/tests/*` の glob)、`11:166`・`11:178`(ソースの検査の記述。変わらない)。
- 仕様書の章の本文にテストの件数(`1,149` など)の記述は無い(rust 11 は「ファイルごとのテストの件数は書かない」)。

### 7.2 `docs/spec/language-agnostic/11-quality-and-build.md`(受け入れテストの表の根拠の欄。D6)

| # | 行(HEAD) | 今の根拠 | 直した後 | コミット |
|---|---|---|---|---|
| L1 | 588(TOML から YAML) | `convert_ac::ac_7_1_1_toml_から_yaml` | `convert_roundtrip::ac_7_1_1_toml_から_yaml` | 2 |
| L2 | 589(YAML から TOML) | `convert_ac::ac_7_1_2_アンカーつきの_yaml_から_toml` | `convert_roundtrip::ac_7_1_2_アンカーつきの_yaml_から_toml` | 2 |
| L3 | 605(キーの順序は辞書順) | `convert_roundtrip::出力は毎回同じ` | `convert_ac::出力は毎回同じ` | 1 |
| L4 | 718(書式の版がない) | `runtime::state_read_raw::fr3_4 …` | `runtime::state_read::raw::fr3_4 …` | 5 |
| L5 | 884(壊れたステート) | `runtime::state_read_raw` | `runtime::state_read` | 5 |
| L6 | 658、676(リトライ) | `docs_retry` | `docs` | 10 |
| L7 | 929(環境変数の README) | `docs_env` | `docs` | 10 |
| L8a | 946(分岐の図 B1〜B4) | `docs_graph` | `docs` | 9(この行の `docs_graph` は、README の分岐の例を実際の出力と比べる**単体テスト**(`src/tests/docs_graph.rs` の `readme_の分岐の例は_実際の出力と一致する`)を指す。統合テストの `docs_graph` は語句の検査だけで、図の出力を比べない。単体の `docs_graph.rs` が `docs.rs` になるコミット 9 で直す) |
| L8b | 958(README の `graph` の説明) | `docs_graph` | `docs` | 10(この行は「必須の語句」(統合テスト)と「図の例が実際の出力と一致」(単体テスト)の**両方**を指す。両方が `docs.rs` にそろうコミット 10 で直す) |
| L9 | 980(`--dry-run` の README) | `docs_dry_run` | `docs` | 10 |
| L10 | 998(`init` の README) | `docs_init` | `docs` | 10 |

- 根拠の欄は「モジュールの名前(::テスト名)」の書き方で、統合と単体を区別しない(今の書き方のまま)。根拠の欄がどちらのテストを指すかは、行の確かめる中身で決めた。L8a(946 行目)は単体テストだけを指すのでコミット 9 で直す。L8b・L9・L10(958、980、998 行目)は「必須の語句」(統合テスト)と「例が実際の出力と一致」(単体テスト)の両方を指すので、コミット 9 の後も統合の側が `docs_graph` などのまま残る。両方がそろうコミット 10 で直す。L6・L7 は統合テストだけを指し、コミット 10 で動く。
- **見つけた、同じ種類の古い記述(初期の記述に無い。D7)**: 772 行目(使い方の誤りの全種類)の根拠「テストの側の種類の一覧の写し(`tests/common/kinds.rs`。…)」。使い方の誤りの一覧 `USAGE_ERROR_KINDS` は cli の単体テストの `src/tests/common/kinds.rs` にあり、`tests/common/kinds.rs` は core(と、統合まで converter)のファイル。この intent で無くなるのは converter の `tests/common/kinds.rs` だけで、core の同じ名前のファイルは残る。推奨は「今回は直さない」(この intent の移動と無関係な前からの食い違い)。
- 変えない: `flow_init::…`(988、989、991、994、996、997 行)、`init_template::…`(992、993 行)、`args_grammar`(996 行)、`graph_report::…`(946 行)。ファイルの名前もテストの名前も変わらない。

## 8. 範囲の外で、触らないもの(確かめた)

- 本番のコード(`crates/*/src` の `src/tests/` の外)。`Cargo.toml`、`Cargo.lock`(dev-dependencies も足さない)、CI の設定(`.github/workflows`。`nfr_perf` を名前で指す `startup-bench` も変えない)、README。
- `nfr_perf.rs`、`msrv_consistency.rs`、`platform`、converter の src の直下、core のテストと補助。cli の `bin_support`・`signal_support`、`signals_order.rs`・`signals_real.rs`。製品のコードの検査のテスト(cli と runtime の `nfr_source.rs`、converter の `nfr_security.rs` の検査の部分)。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。

## 9. 網羅に使った検索のコマンド

```sh
# 対象のファイルと、ほかから指す所
git grep -n -E "docs_(dry_run|env|graph|init|retry|support)|convert_roundtrip|convert_ac|state_read_raw|runner_env|init_types|run_id_arg|graph_long_id|common/mod\.rs|common/kinds|init_template\.rs|flow_init\.rs" -- ':!aidlc'
git grep -n "mod common" -- crates/converter crates/cli/tests crates/runtime/tests
git grep -n "include_str!" -- crates                       # 前例(converter、core)と、ひな形
grep -rn "src/tests\|tests/\|fixtures" docs/spec/rust/*.md  # 仕様書の、テストのファイルの記述
grep -n "\-\-test\|nfr_\|docs-check" .github/workflows/*.yml   # CI が名前で指すテスト
# テストの数と名前、補助の名前の衝突
grep -c "#\[test\]" crates/cli/tests/docs_*.rs crates/cli/src/tests/docs_*.rs crates/cli/src/tests/{init_types,run_id_arg,graph_long_id}.rs crates/converter/tests/convert_roundtrip.rs crates/converter/src/tests/convert_ac.rs crates/runtime/tests/{runner_env,state_read_raw}.rs
grep -n "^fn \|^const \|^use " <移す元> <移す先>         # 関数・定数・import の衝突(HEADING、definition、read_state)
grep -n "^// ---- \|^#\[cfg" crates/cli/src/tests/args_grammar.rs crates/runtime/tests/{runner_basic,state_read}.rs
diff <(sed -n 45,141p crates/cli/tests/common/mod.rs) <(sed -n 34,125p crates/cli/src/tests/common/mod.rs)   # 写しの差
# 件数と名前
cargo test --workspace --locked 2>&1 | grep -E "^test result|Running"
cargo test --workspace --locked -- --list 2>&1 | grep -E "Running|: test$"
```
