# 機能の仕様: v0.10.4 の大きなファイルの分割

出典:
- 要件: `requirements.md`(承認済み)。承認のときに直すと決めたレビューの指摘 R-01〜R-03 を含む。
- 設計の質問: `functional-design-questions.md` の Q1〜Q4(まとめは確かめ済み)。
- 規則と実体: `rules.md`、`entities.md`。
- 引き渡し: `inception/reverse-engineering/developer-scan.md` の「この intent のための詳細」の 1〜6。

行の番号は 9376cb7 の時点の目安です。作業では、名前で場所を確かめます。

## 0. 数の言葉(R-01)

| 数 | 数えたもの |
|---|---|
| 40 | `report/mod.rs` の中で `DisplayEvent::` を含む行の数。依頼の文はこの数を使った。`render` の外の文書コメント 1 行を含む |
| 37 | `DisplayEvent` の変種の数 |
| 35 | `render` の `match` の枝の数。`Graph \| Plan \| InitTemplate` を 1 つの枝で扱う |
| 9 | 表示の形の種類(引き渡しの 1.3 の A〜I)。分けた後の `render.rs` の節の数 |

確かめの基準: 分けた後の `render` の網羅の `match` が、37 の変種をすべて扱うこと。網羅でなければコンパイルが止まるので、ビルドが通ることで確かめられる。

## 1. 全体

| 順 | 対象 | 子のファイル | 規則 |
|---|---|---|---|
| 1 | `crates/core/src/definition/walk/mod.rs` | `values.rs`、`transition.rs`、`cross_check.rs` | BR2.1〜BR2.4 |
| 2 | `crates/cli/src/args.rs` → `args/mod.rs` | `options.rs`、`tokens.rs`、`values.rs` | BR3.1〜BR3.4 |
| 3 | `crates/cli/src/report/mod.rs` | `event.rs`、`lines.rs`、`escape.rs`、`render.rs` | BR1.1〜BR1.5 |
| 4 | 文書と小さな直し | — | BR4.1〜BR4.4 |
| 5 | 版 | — | BR5.1 |

順は Q2 A のとおり。どのファイルも、移すだけのコミットを先に作り、関数に分けるコミットを後に作る。

## 2. `walk/mod.rs`(BR2)

### 2.1 置き場所

| 子 | 移す項目 | 見える範囲 |
|---|---|---|
| `values.rs` | `Walker` のメソッド `string`、`boolean`、`exit_codes`、`command`。自由関数 `exit_code_of`、`integer_of` | メソッドは `pub(super)`。自由関数も `pub(super)` |
| `transition.rs` | `RawTransition`(とそのフィールド)、メソッド `transitions`、`transition_on` | `RawTransition` とフィールド、`transitions` は `pub(super)`。`transition_on` は private |
| `cross_check.rs` | メソッド `cross_check`、`check_exit_codes` | `cross_check` は `pub(super)`。`check_exit_codes` は private |

`mod.rs` に残す項目:
- 子モジュールの宣言(`env`、`retry`、`values`、`transition`、`cross_check`)
- キーの表 4 つ(BR2.2)
- `NO_JOBS_REASON`
- 型 `PipelineHead`、`SuccessCodes`、`RawJob`、`Walker`
- 基本のメソッド `push`、`type_error`、`check_keys`
- `[pipeline]` の読み取り `pipeline_section`、`pipeline_head`、`log_dir`
- `job`、入口の `walk`、`build_jobs`、`names_of`

### 2.2 名前の引き方

- 子の `impl Walker<'_>` のメソッドは、兄弟の子からも親からも `self.…` で呼べる。`pub(super)` は `walk` の中のすべてで見える。
- `retry.rs:28` の `use super::{…, exit_code_of, integer_of}` は変えない。`mod.rs` に `use values::{exit_code_of, integer_of};` を置き、`super::` から今の名前で引けるようにする。private の `use` も、子からは `super::` で見える。
- `RawJob` のフィールド `transitions: Vec<RawTransition>` のため、`mod.rs` に `use transition::RawTransition;` を置く。

### 2.3 保つこと

- 誤りを積む呼び出しの順(引き渡しの 2.3)は変えない。移すだけで、呼ぶ側の行は変えない。
- `walk/mod.rs` の中で、`const PIPELINE_KEYS` と `const JOB_KEYS` を含む行を、今の定義より前に置かない。文書コメントの中の語も同じ(`spec_numbers.rs` の `array_strings` は最初に見つかった行を使う)。
- この段では、関数に分けるコミットは無い(移すだけ)。

## 3. `args.rs` → `args/mod.rs`(BR3)

### 3.1 置き場所

| 子 | 移す項目 |
|---|---|
| `options.rs` | `Opt`、`Allowed`、`OptSpec`、`ALL_BUT_CONVERT`、`RUN_AND_PIPELINE`、`OPTIONS`、コンパイル時の検査 2 つと `is_long_spelling`、`spec`、`spec_by_long`、`OptSpec::allows`、`Opt::subject`、`Check`、`CHECKS`、`Given`、`Operand`、`operand_rule`、`unexpected_argument` |
| `tokens.rs` | `Token`、`Special`、`map_unexpected_value`、`to_text` |
| `values.rs` | `parse_assumptions`、`parse_graph_format`、`parse_interval`、`parse_run_id_option`、`parse_target` |

`mod.rs` に残す項目:
- 文法の表の `//!`
- 子モジュールの宣言と `use`
- 入口の `pub fn parse_args`
- `build_command` と、そこから分けた関数(3.2)

子の項目のうち、親や兄弟が使うものは `pub(super)` にする。クレートの外へは何も出さない。

- `git mv crates/cli/src/args.rs crates/cli/src/args/mod.rs` で移し、履歴を追えるようにする。この移しは、中身を変えないコミットにする。

### 3.2 `build_command` の分け方(FR3.2)

`build_command` は次の順に関数を呼ぶだけにする。今の 1〜9 の順(引き渡しの 3.2)を保つ。

| 順 | 関数 | 中身(今の行) | 返すもの |
|---|---|---|---|
| 1 | `collect_tokens` | 字句を集める。知らないオプションがあれば誤り(454-467) | `Given`、位置引数 |
| 2 | `command_kind` | コマンドの名前(469-476) | `CommandKind` |
| 3 | `check_options` | `CHECKS` の順の「使えない」と「重複」、`--all` と `--run-id`、`graph` の `--run-id`(478-515) | `()`。誤りなら `Err` |
| 4 | `fill_options` | 共通の値(517-538)。順は `--assume` → `--format` → `--interval` → `--run-id` | `CliCommand`、`--to` の値 |
| 5 | `check_operands` | 位置引数の数(540-549) | 1 つ目の位置引数 |
| 6 | `fill_operand` | コマンドの枝(551-579)の `match kind`。5 つの枝のうち、中身のある `Convert` と `Init` は `convert_operand`・`init_operand` に分ける | `()` |

- どの関数も、最初の誤りで `Err` を返す。返す誤りの種類と対象は変えない。
- 確かめるのは `src/tests/args_grammar.rs` の 69 のテスト。
- 目安: `build_command` は 30 行以下。

## 4. `report/mod.rs`(BR1)

### 4.1 置き場所

| 子 | 移す項目 | 見える範囲 |
|---|---|---|
| `event.rs` | `StateContext` と `for_read`、`ResultDetail`、`DisplayEvent` と `level`、`FramePrefix`、`CLEAR_SCREEN` | 型は `pub` のまま。`mod.rs` で `pub use event::{DisplayEvent, FramePrefix, ResultDetail, StateContext};` |
| `lines.rs` | `show_path`、`error_line`、`place_of`、`definition_line`、`env_file_lines`、`env_warning_line`、`job_place`、`command_line`、`usage_text`、`state_line`、`conversion_line`、`job_result_line` | `pub(super)` |
| `escape.rs` | `MAX_DISPLAY_CHARS`、`escape_name`、`escape_full`、`push_line` | `escape_name`・`escape_full`・`push_line` は `pub(crate)` のまま。`mod.rs` で `pub(crate) use escape::{escape_full, escape_name, push_line};` |
| `render.rs` | `render` の枝の中身を、変種ごとの private のメソッドにしたもの(4.2) | メソッドは `pub(super)` |

`mod.rs` に残す項目:
- `//!`
- 子モジュールの宣言(今の 7 つと、新しい 4 つ)
- `use` と `pub use`
- `Reporter`、`new`、`language`、`color_enabled`、`show`
- 振り分けだけの `render`
- `detect_language`

`report/messages.rs` は動かさない。

今の道筋を保つもの:
- `crate::report::{DisplayEvent, Reporter, StateContext, ResultDetail, FramePrefix, escape_name, escape_full}`
- 子モジュールの `super::{escape_full, escape_name, push_line}`(`graph_text.rs`、`graph_dot.rs`、`plan_text.rs`、`watch.rs`)
- `messages.rs:28` の `crate::report::StateContext`

### 4.2 `render` の分け方(FR1.2、Q4 A)

`render` は網羅の `match` のまま残す。各枝は、1 行で `render.rs` のメソッドを呼ぶだけにする。`render.rs` は表示の形の 9 つの節(`// ---- A. 固定の文 ----` のような区切りの注記)に分け、節の中に変種ごとのメソッドを並べる。

| 節 | メソッド(変種) | 言語の判定 |
|---|---|---|
| A. 固定の文 | `signal_setup_failed`、`version` | しない |
| B. 誤りの一覧 | `definition_errors`、`env_file_errors`、`env_warnings`、`command_errors` | する |
| C. 関数に任せる誤りの 1 行 | `usage`、`state`、`conversion` | する |
| D. 場所つきの誤りの 1 行 | `consistency_mismatch`、`rejected`、`start_failure`、`success_code_not_listed`、`completion_undetermined` | する |
| E. `ERROR_PREFIX` と文 | `output_failed`、`init_exists`、`init_failed`、`init_stdout_failed`、`log_not_created` | する |
| F. `WARNING_PREFIX` と文 | `reset_all_kept`、`log_write_failed`、`log_output_cut_short` | する |
| G. 通知 | `retry_scheduled`、`init_created`、`init_replaced` | する |
| H. `-v` の行 | `job_result`、`job_skipped`、`transition`、`interrupted`、`definition_valid` | する |
| I. 標準出力 | `watch_table`(色を判定)、`watch_empty`、`watch_frame`(`self.render` を呼び直す)、`pass_through`(`Graph \| Plan \| InitTemplate`)、`help` | `pass_through` はしない。ほかはする |

- メソッドは、取り出した中身を引数で受け取る(例: `fn rejected(&self, job_id: &str, reason: …) -> String`)。`&DisplayEvent` は受け取らない。
- 言語は、メソッドの中で `self.language()` を呼んで判定する。判定は、今と同じく、その枝が呼ばれたときに初めて行う(`OnceCell` で 1 回だけ)。判定しない節(A と `pass_through`)は `self.language()` を呼ばない。
- 色は `watch_table` だけが `self.color_enabled()` を呼ぶ。
- メソッドの中身は、今の枝の中身をそのまま写す。文、改行、`format!` の形は変えない。
- 確かめるのは `src/tests/report_messages.rs` の全テスト。特に 694 行と 726 行(判定を後回しにすること)を見る。

### 4.3 コミットの単位

1. `event.rs` に移す
2. `escape.rs` に移す
3. `lines.rs` に移す
4. `render.rs` を作り、枝を変種ごとのメソッドにする(関数に分けるコミット)

1〜3 は移すだけのコミットで、それぞれ 1 コミットにする。

## 5. 分け終わりの判定(R-02)

| 項目 | 基準 |
|---|---|
| `walk/mod.rs` | 420 行以下。2.1 の移す項目が `mod.rs` に無い(`grep -n 'fn string\|fn cross_check\|fn transitions' walk/mod.rs` が 0 件) |
| `args/mod.rs` | 300 行以下。3.1 の移す項目が `mod.rs` に無い。`build_command` は 30 行以下で、3.2 の 6 つを呼ぶだけ |
| `report/mod.rs` | 220 行以下。4.1 の移す項目が `mod.rs` に無い。`render` の各枝は 1 行(メソッドの呼び出しだけ) |
| 子のファイル | 10 個(`walk` に 3、`args` に 3、`report` に 4)がある |
| テスト | 既存のテストの数と名前が変わらない。`#[test]` は 1,472 のまま(FR4.4 は既存のテストの一覧を直すだけで、テストは増えない) |
| 公開の範囲 | 3.1・4.1 の道筋で、今の名前が引ける。クレートの外に見える名前は変わらない(`lib.rs` の `pub` の行が変わらない) |

行数は、本番のコードのファイルの全体の行数(`wc -l`)で測る。

## 6. 文書と小さな直し(BR4)

- **6.1 仕様書(FR4.1)**: 次の箇所の、ファイルとモジュールの記述を、分けた後の形に直す。
  - `docs/spec/rust/02-architecture.md` のモジュールの表(`walk`、`args`、`report` の行)
  - `docs/spec/rust/07-process-and-signals.md:44`
  - `docs/spec/rust/08-output-and-i18n.md:3`、エスケープの場所(`report/escape.rs`)
  - `docs/spec/rust/12-rebuild-guide.md` の、今の形を書いた箇所

  `12-rebuild-guide.md` の段の中で、その段の時点のファイルを書いた記述は履歴として残す。必要なら「(v0.10.4 で分けた)」と注記する。
- **6.2 文書コメント(FR4.1)**: 次の文書コメントを直す。
  - `crates/cli/src/lib.rs:47-59` のモジュールの表
  - `crates/core/src/lib.rs:57`
  - `report/mod.rs` と `walk/mod.rs` の `//!` にある子モジュールの説明
- **6.3 team.md(FR4.2)**: Code Style の行「使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える」の `args.rs` を、`args/options.rs`(オプションの表)と `args/mod.rs`(文法の表の `//!`)の扱いに合わせて直す。書き換えた後の文はコード生成の計画に載せ、承認を経てから直す。
- **6.4 古い名前(FR4.3)**: `crates/core/src/error.rs:176` の `report::escape` を、今の道筋 `report::escape_full`(`report/escape.rs`)に直す。
- **6.5 一覧の直し(FR4.4)**: `src/tests/report_messages.rs:99` の `with_all_events` に、`DefinitionValid` と `ResetAllKept` を足す。このテストの確かめ(水準と、空でないこと)が通るようにする。

## 7. コミットの区分(R-03)

| 区分 | 中身 | 件名の例 |
|---|---|---|
| 移し | 項目を子のファイルに移すだけ。中身の行は変えない(`use`・`pub(super)`・`pub use` の行の追加と、`git mv` を除く) | `core: walk の値の読み取りを values.rs に移す` |
| 分け | 1 つの関数を、いくつかの関数に分ける(`build_command`、`render`) | `cli: render の枝を表示の形ごとのメソッドに分ける` |
| 文書 | 仕様書、文書コメント、team.md | `docs: rust 版の仕様書のモジュールの記述を分けた後の形にする` |
| 小さな直し | 6.4、6.5 | `cli: with_all_events に足りない 2 つの変種を足す` |
| 版 | 0.10.4 | `版を 0.10.4 にする` |

- 1 つのコミットには、1 つの区分の中身だけを入れる。
- FR4.4 は、分けのコミットとは別の「小さな直し」のコミットにする。
- 文書コメントの直しは、その対象を移すコミットに含めてよい。モジュールの説明の行は、移しの結果に合わせて直す必要があるため。

並べる順:
1. `walk` の移し(3 コミット)
2. `args` の移し(4 コミット: `git mv` と、3 つの子)と分け(1 コミット)
3. `report` の移し(3 コミット)と分け(1 コミット)
4. 小さな直し(2 コミット)
5. 仕様書(1〜2 コミット)
6. team.md(1 コミット)
7. 版(1 コミット)
