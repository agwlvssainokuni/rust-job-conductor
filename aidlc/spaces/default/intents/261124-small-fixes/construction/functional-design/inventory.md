# 変える場所の一覧(Inventory): 小さな直し

HEAD 6ce7d45(small-fixes の意図の把握のコミット。版 0.9.8)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。初期の記述と実物が違ったところは「(記述との違い)」と書く。設計の要点、テストの変更の一覧(T…)、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent の判断と、この intent の Q1〜Q3): **テストのコードの都合で本番のコードを曲げない**。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変えない。版は上げない。本番のコードで変えるのは、説明(ドキュメントコメント)の 3〜4 行だけ。消すテストは、同じことを確かめて残るテストを添えて設計の承認で認めてもらう(Q1: A)。`signals_order.rs` は残す(Q2: B)。ソースの検査は `include!` などもすべて止める(Q3: A)。

決めてほしい点(`functional-spec.md` §10 の D1〜D8)は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。D1: core の `lib.rs:38` を直す。D2: rust 02 の 105 行目の段落を「モジュールはすべて非公開」にする。D3: 言語に依らない仕様書の根拠の欄を `plan_simulate_perf` に直す。D4: マクロの名前と `!` を、空白を除いて探す。D5: rust 05 の 341・850 行目を `decide_from_state` に直す。D6: rust 06 の 5 行目を直す。D7: ほかの版の注記は今回は直さない。D8: 移すテストの名前は変えない。この文書の「(D1)」〜「(D8)」は、その決定のとおりの形を書いている。T 表(T1〜T4)と消すテスト 3 件は、設計の承認で認めてもらう(承認待ち)。

範囲の外(初期の記述): テストのファイルの統合、補助の写しの解消、init のひな形の原本の重複、本番の小さなモジュールの統合。

## 0. 基準と、設計の検証

### 0.1 テストの件数(`cargo test --workspace --locked`、macOS、rustc 1.100.0、debug、HEAD 6ce7d45)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 112 | 270 | 382 |
| runtime | 69 | 128 | 197 |
| converter | 96 | 19 通過 + 1 無視 | 115 通過 + 1 無視 |
| cli | 164 | 294 | 458 |
| 全体 | | | **1,152 通過、1 無視、0 失敗** |

- テスト名の一覧は `cargo test --workspace --locked -- --list` の `: test` の行(1,153 行。無視の 1 件を含む)。
- この intent に関わる実行ファイルの件数: core の `plan_perf` 2、`plan_simulate_perf` 2、cli の単体テスト(`conductor_cli` の lib)164(うち `tests::init_types` 3)、cli の `nfr_source` 6。
- `cargo doc --workspace --no-deps --locked` の警告は 5 件(core 1、runtime 4。この intent と無関係)。

### 0.2 設計の検証(写しの木での試し)

T 表を検索だけに頼らないために、HEAD 6ce7d45 の写し(`git archive HEAD` を `target/small-fixes/work` に展開)に、この一覧の全項目と `functional-spec.md` §4 の T 表を当て(D1〜D8 は推奨の形。のちに、ユーザーがすべて推奨のとおりに決めた)、次を確かめた。写しは確かめたあと消した(`target/small-fixes` ごと)。リポジトリのコードは変えていない。

- `cargo fmt --all -- --check` が通る(`cargo fmt` が整える所は無い)。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc` の両方で通る。`cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(本番の形)も通る。
- `cargo test --workspace --locked` が **1,149 通過、1 無視、0 失敗**(core 112 + **269**、runtime 69 + 128、converter 96 + 19 + 1 無視、cli **162** + 294)。実行ファイルごとに、`plan_perf` が無くなり、`plan_simulate_perf` が 3 件、cli の lib の単体テストが 162 件、cli の `nfr_source` が 6 件。
- テスト名の一覧の差(名前の最後の部分で、並べ替えて比べた): **消えた名前 3 件だけ**(`終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる`、`init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml`、`コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ`)。増えた名前・名前の変わったテストは無い(★ なし)。パスで比べた差も同じ 3 行(統合テストの `--list` は実行ファイルの名前を出さないので、移した `千個のジョブがすべて実行する計画は_1秒以内に作れる` は同じ行のまま。移ったことは、実行ファイルごとの件数 `plan_simulate_perf` 2 → 3 で確かめる)。
- 移したテストの本体は、元と一字も違わない(`diff` で関数の始まりから終わりまでを比べた)。
- ソースの検査: 今の製品のコードで `nfr_source.rs` の 6 件がすべて通る。写しの本番のファイル(`src/exit.rs`)に `include_str! ["../Cargo.toml"]`(空白と角かっこの形)を一時的に足すと、`テスト専用の経路を実行ファイルに持たない` だけが失敗し、その行を指す(足した行は消して戻し、6 件が通ることを確かめ直した)。
- `cargo doc --workspace --no-deps --locked` の警告は基準と同じ 5 件(core 1、runtime 4)。
- 行カバレッジ(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`): 全体の行 **98.06%**(80% 以上)。
- 仕様書 rust 02 の書き換え(§2)も写しに当て、§7 の検索(「今の形」の段落に「v0.」が残らない、版の履歴が 3 か所、`Option<&RunIdArg>`、`consistency`)が期待どおりに出ることを確かめた。HEAD では、同じ検索で「今の形」の 3 段落すべてに「v0.」が当たる。
- 使ったツールは rustc・cargo 1.100.0(MSRV と同じ版)。

---

## 1. 古い説明(項目 1)

### 1.1 直す説明

| # | 場所(HEAD) | 今 | 直した後 |
|---|---|---|---|
| C1 | `crates/core/src/plan/mod.rs:18` | 「実行と同じ判断(`decide_before_run`、」 | 「実行と同じ判断(`decide_from_state`、」(次の行の `classify_and_decide` は変えない) |
| C2 | `crates/core/src/types.rs:440`(`NextStep.skipped_job_ids` の説明) | 「スケジューラが呼んだときに `decide_before_run` が `Skip(AfterEnd)` を」 | 「スケジューラが呼んだときに `decide_from_state` が `Skip(AfterEnd)` を」 |

- 名前はリンクにしない(バッククォートだけ。今の書き方のまま。`cargo doc` の警告を増やさない)。
- 計画の生成の本体 `plan/simulate.rs:17` は、すでに `decide_from_state` と書いている。C1 で、型の側の説明がそれに揃う。

### 1.2 検索の結果(`decide_before_run` を公開のものとして書く説明)

`git grep -n "decide_before_run" -- ':!aidlc'` の結果を分けた。

| 場所 | 扱い |
|---|---|
| `crates/core/src/flow/decide.rs`(説明と本体とその単体テスト) | 変えない(非公開の関数があるファイル。範囲の外と初期の記述にある) |
| `crates/core/src/plan/mod.rs:18`、`types.rs:440` | 直す(C1、C2) |
| ほかの crates の src・tests | 0 件 |
| `docs/spec/rust/02-architecture.md:55`、`:194`、`:299` | 変えない(「`flow/decide.rs` の中だけ。公開は `decide_from_state`」と正しく書いている) |
| `docs/spec/rust/05-execution-flow.md:191`、`:199`(4 章の見出し)、`:283`(`decide_from_state` の擬似コードの中)、`:899` | 変えない(判定の本体を、その名前で説明している。公開とは書いていない) |
| `docs/spec/rust/05-execution-flow.md:341` | 「定義の記述順に全ジョブを 1 つずつ見て、`decide_before_run` で決める」。すぐ下の擬似コードは `decide_from_state` を呼ぶ。**呼ぶ側の名前と食い違う**。`decide_from_state` に直す(D5。決定: 直す。推奨のとおり。ユーザー、2026-11-25) |
| `docs/spec/rust/05-execution-flow.md:850` | 「実行と**同じ判断**(`decide_before_run`、`classify_and_decide`)を」。C1 と同じ文で、計画の生成が呼ぶのは `decide_from_state`。**同じ食い違い**。`decide_from_state` に直す(D5。決定: 直す。推奨のとおり。ユーザー、2026-11-25) |
| `docs/spec/rust/12-rebuild-guide.md:40`、`:126` | 変えない(本体は `flow/decide.rs` の中だけ、と正しく書く / 本体の探索の直し方の説明) |
| `docs/spec/language-agnostic/00-README.md:113`、`05-execution-flow.md` | 変えない(言語に依らない擬似コードの仮の名前。`00-README.md` がそう断っている) |
| 凍結した `docs/spec/python/*` | 触らない |

### 1.3 検索で見つけた、同じ種類の古い説明(初期の記述に無い。D1。決定: 直す。推奨のとおり。ユーザー、2026-11-25)

| # | 場所(HEAD) | 今 | 実物 | 直す案 |
|---|---|---|---|---|
| C3 | `crates/core/src/lib.rs:38` | 「`conductor-runtime` は、これらと `check_consistency` を、同じ名前で再公開する。」 | runtime の `lib.rs:123-132` の再公開に、`initial_state`・`apply_*`・`check_consistency` は無い(runtime の `lib.rs:37` も「conductor-core の関数を使う」と書く。仕様書 rust 02 の 2.2 も「使う側は `conductor_core` から使う」) | 「`conductor-runtime` は、これらと `check_consistency` を再公開しない(使う側は、この crate から使う)。」 |

## 2. 仕様書 `docs/spec/rust/02-architecture.md`(項目 2)

### 2.1 引数の型(a)

| # | 行(HEAD) | 今 | 直した後 | 根拠 |
|---|---|---|---|---|
| S1 | 146(2.4 の関数の表の `resolve_run_id`) | `(option: Option<&str>, env_name: &str, env: &dyn Env) -> …` | `(option: Option<&RunIdArg>, env_name: &str, env: &dyn Env) -> Result<Option<RunId>, UsageError>` | `crates/cli/src/run_id.rs:37-41` の `pub fn resolve_run_id(option: Option<&RunIdArg>, env_name: &str, env: &dyn Env) -> Result<Option<RunId>, UsageError>` |

### 2.2 公開の一覧の `consistency`(b)

`crates/core/src/lib.rs:85` に `pub mod consistency;`、`:97` に `pub use consistency::{ConsistencyResult, MismatchKind, check_consistency};` がある。仕様書の 42 行目の「再公開」と「公開モジュール」の一覧に無い(関数の表の 68 行目の `check_consistency` の行はある)。

| # | 行(HEAD) | 直した後 |
|---|---|---|
| S2 | 42 の「再公開(`lib.rs`):」の一覧 | 先頭に `consistency::{ConsistencyResult, MismatchKind, check_consistency}` を足す。一覧の並びは `lib.rs:96-117` の `pub use` の並び(モジュールの名前の順: `consistency`、`definition`、`envfile`、`error`、`flow`、`plan`、`run_id`、`types`)にする(S4 で、`envfile` と `plan` が版の注記の文から一覧に入るため) |
| S3 | 42 の「公開モジュール:」の一覧 | 先頭に `consistency` を足す(`lib.rs:85-95` の並び) |

### 2.3 版の注記を「今の形」と「版の履歴」に分ける(c)

直す段落は 2 章の 3 つ(42、105、137 行目。各クレートの「再公開(`lib.rs`):」で始まる段落)。各段落から版の注記(「v0.」で始まる注記)をすべて抜き、段落の直後に「版の履歴(古い順):」の行と、時系列の箇条書きを置く。**注記の中身は変えず、移すだけ**(足さない・消さない)。並べ方は、注記が述べる変わり目の版の古い順(「v0.9.0 以降」→「v0.9.6 までは」(v0.9.7 で変わった)→「v0.9.7 までは」(v0.9.8 で変わった)/「v0.9.7 以降」)。

| # | 段落(HEAD の行) | 今の段落から抜く注記 | 版の履歴(古い順の箇条書き) |
|---|---|---|---|
| S4 | 42(2.1 `conductor-core`) | 「(v0.9.6 までは `error::*`・`types::*` で、…も出ていた。外から使われないので crate の中に狭めた)」、「v0.9.0 以降は、さらに `envfile::{…}` と `plan::{…}` を再公開する」(この文は今の形の再公開の一覧でもあるので、`envfile` と `plan` の一覧は中身をそのまま「今の形」の再公開の一覧に統合して残し(S2 の並び)、版の履歴には注記だけを移す)、末尾の「v0.9.7 までは `flow` の下位モジュール、…と、これらの定数も公開だった。」 | 1. 「v0.9.0 以降は、`envfile::{…}` と `plan::{…}` も再公開する。」 2. 「v0.9.6 までは `error::*`・`types::*` で、`quote_key`・`Locate`・`TransitionResolution`・`resolve_transition` も出ていた(外から使われないので crate の中に狭めた)。」 3. 「v0.9.7 までは `flow` の下位モジュール、`plan::simulate`、`envfile::{merge, parse}`、`graph::{edges, layout, overlay}` と、上の crate の中の定数も公開だった。」 |
| S5 | 105(2.2 `conductor-runtime`) | 「(後ろの 4 つは v0.9.0 以降)」、「v0.9.6 までは `error`・`types` も公開で、`error::*`・`types::*` で再公開した」、「v0.9.7 以降」(`StateErrorKind::label()` の括弧の中) | 1. 「v0.9.0 以降は、`load_env_files`、`CreateError`、`CreateOutcome`、`create_file` も再公開する。」 2. 「v0.9.6 までは `error`・`types` も公開で、`error::*`・`types::*` で再公開した。」 3. 「v0.9.7 以降は、`StateErrorKind::label()` が公開。」 |
| S6 | 137(2.4 `conductor-cli`) | 末尾の「v0.9.6 までは `platform`・`report::messages`・`report::watch` も公開で、…(外から使われないので crate の中に狭めた)。」 | 1. 同じ文(そのまま) |

- 段落の直後の形(3 か所で同じ):

```markdown
再公開(`lib.rs`): …(今の形。「v0.」の注記なし)

版の履歴(古い順):

- v0.9.0 以降は、…
- v0.9.6 までは …
```

- S4 の直した後の「今の形」(写しで当てた形。一覧の中身は HEAD の文と `lib.rs` のとおり): 「再公開(`lib.rs`): `consistency::{ConsistencyResult, MismatchKind, check_consistency}`、`definition::{load_definition, parse_definition}`、`envfile::{EffectiveEntry, …, summarize_errors}`、`error::{…}`、`flow::{…}`、`plan::{Assumption, …, plan_run}`、`run_id::{…}`、`types::{…}`(`graph` は `conductor_core::graph::…` のパスで使う)。公開モジュール: `consistency`、`definition`(…)、`envfile`、`error`、`flow`、`graph`、`plan`、`run_id`、`types`。(下位モジュールと定数の文は今のまま)(`MAX_ENV_FILE_BYTES`・`MAX_ENV_ENTRIES` は公開)。」
- S5 の「今の形」: 再公開の一覧から「(後ろの 4 つは v0.9.0 以降)」を除き、非公開のモジュールの括弧を「(`#![deny(unsafe_code)]`)」に、`StateErrorKind::label()` の括弧を「(cli の表示が使う)」にする。非公開のモジュールの一覧は「モジュールはすべて非公開(`bounded_read`、`envfile`、`error`、`file_create`、`platform`、`runner`、`split`、`state`、`temp_file`、`types`、`wait`。`#![deny(unsafe_code)]`)」にする(S7。D2 の決定)。
- S6 の「今の形」: 末尾の 1 文を除くだけ(ほかは今のまま)。
- 版の履歴の見出しは、太字や見出し(`####`)にしない(2 章の目次の構成を変えない)。

### 2.4 同じ段落の中で見つけた食い違い(初期の記述に無い。D2。決定: 直す。推奨のとおり。ユーザー、2026-11-25)

| # | 行(HEAD) | 今 | 実物 |
|---|---|---|---|
| S7 | 105 | 「`envfile`、`error`、`file_create`、`platform`、`runner`、`split`、`state`、`types` は非公開」 | runtime の `lib.rs:111-121` のモジュールは 11 個で、すべて非公開(`pub mod` は無い)。`bounded_read`、`temp_file`、`wait` が一覧に無い |

### 2.5 ほかの章の同じ種類の食い違い(直すかは D。決定: すべて推奨のとおり。ユーザー、2026-11-25)

| # | 場所(HEAD) | 食い違い | D |
|---|---|---|---|
| X1 | rust `05:341`、`05:850` | `decide_before_run` を、呼ぶ側の関数として書く(§1.2) | D5。決定: 直す(推奨のとおり。ユーザー、2026-11-25) |
| X2 | rust `06:5` | 「実装の所在: `crates/runtime/src/state/{reader,writer,record,consistency,reset_all}.rs`」。`runtime/src/state/` にあるのは `mod.rs`・`reader.rs`・`reset_all.rs`・`writer.rs` だけで、記録は core の `flow/record.rs`、照合は core の `consistency.rs`(02 の 68 行目・280 行目は正しい) | D6。決定: 直す(「`crates/runtime/src/state/{reader,writer,reset_all}.rs`、…。照合と記録は `crates/core/src/{consistency,flow/record}.rs`」に。推奨のとおり。ユーザー、2026-11-25) |
| X3 | rust `05:426-435`(6.2 の擬似コード) | 「value = そのオプションの値 … `RunId::parse(value)`」。実装では `--run-id` の値は引数の解釈で検査済みの `RunIdArg` になる。動き(結果と誤り)は同じ | 直さない(挙動の擬似コードとして正しい。S1 で 02 の型が正しくなる) |
| X4 | rust `02:150`、`02:152`(2.4 の定数と型の段落)、2 章の表の行、ほかの章(`05` 11 か所、`07` 8 か所、`09` 7 か所 など)の「v0.9.x までは」 | 版の注記が「今の形」の文に混ざる | D7。決定: 今回は直さない(推奨のとおり。ユーザー、2026-11-25) |
| X5 | rust `04`、`10`、`11`、`12` の `resolve_run_id`・`consistency` の記述 | 検索で食い違いは 0 件(`12:76` は `run_id.rs` の関数の名前だけ。`10:140` の `MismatchKind` は core の `consistency.rs` と正しい) | — |

## 3. テストの重複(項目 3)

### 3.1 core の `plan_perf.rs` と `plan_simulate_perf.rs` の P2

| 見る所 | `plan_perf.rs` の `終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる`(:81-134) | `plan_simulate_perf.rs` の `p2_終点のあとの_999_個のスキップの計画は_150ms_以内`(:111-137) |
|---|---|---|
| 定義 | `j0`(成功で `@end`)と `j1`〜`j999`。定義の文字列は 1 バイトも違わない(`definition(true)` と同じ組み立て) | 同じ |
| ステート | `initial_state` → `JobExecution::exited(0)` → `classify_and_decide(…, RunMode::Individual)` → `apply_outcome`。`end_reached && !completed` を確かめる | 同じ |
| 入力 | `commands` は全ジョブ `["echo", id]`、`.env` なし、`run_id: None`、仮定なし | 同じ |
| 確かめ | 項目 1,000 個、先頭が `AlreadyExecuted`、残りがすべて `SkippedAfterEnd`、終わりが `EndReached` | 同じ |
| 時間 | 1 回の計測で 1 秒以内 | 5 回の中央値が 150 ms 以内(より厳しい) |

→ **同じ場面**で、P2 は同じ確かめを、より厳しい上限で持つ。消す(T1)。

`plan_perf.rs` に残る `千個のジョブがすべて実行する計画は_1秒以内に作れる`(:30-79)は、すべてのジョブが `env` と `env_file` を持ち、`.env` に 20 項目、`run_id: Some("r1")` の場面で、P1(ステート・`.env` なし)とは違う。移す(T2)。使う補助は `common::add_env_file` と、`conductor_core` の `EnvEntry`・`EnvFileContent`(`plan_simulate_perf.rs` の import に足す)。`N` は両ファイルで同じ 1000。

### 3.2 cli の `src/tests/init_types.rs` の 3 件

| テスト(HEAD の行) | 確かめること | `args_grammar.rs:391` の `init_の既定は_conductor_toml_と_my_pipeline_で_force_なし` | 扱い |
|---|---|---|---|
| `init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml`(:21) | `InitOptions::default()` が名前 `my-pipeline`、`force` なし、`File("conductor.toml")` | `command(&["init"]).init_options()` が `InitOptions::default()` と等しく、かつ `target` が `File("conductor.toml")`、名前が `my-pipeline`、`force` なし。つまり既定の 3 つの値を同じく確かめる | **消す**(T3) |
| `コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ`(:44) | `CliCommand::new(CommandKind::Init).init_options() == InitOptions::default()` | 引数の解釈は `CliCommand::new(kind)` から作る(`args.rs:488`)。`["init"]` は何も指定しないので、`command(&["init"]).init_options() == InitOptions::default()` が同じことを、公開の解釈の経路で確かめる | **消す**(T3) |
| `ハイフンだけが標準出力でそれ以外はファイル`(:32) | `InitTarget::from_path` の境界(`-x`、`--`、`a/-`、空) | `args_grammar.rs:405` は `-` と `x.toml` だけ | **残す**(移さない。ファイルの統合は別の intent) |

- 消したあと、`init_types.rs` の import は `use crate::types::InitTarget;` だけになる(`CliCommand`・`CommandKind`・`InitOptions` は使われなくなる。残すと `unused_imports` の警告で clippy が止まる)。`use std::ffi::OsString;` は残るテストが使う。
- 消したあと、`CliCommand::new` を使うのは `args.rs:488` だけになる。`crates/cli/src/types.rs:221` の説明「指定のない値で、コマンドを作る(引数の解釈と、テストが使う)。」は「(引数の解釈が使う)。」に直す(C4。説明だけ。本番の処理は変えない)。`InitOptions::default()` は `types.rs` の `init_options` と `args_grammar.rs` が使い続ける。

### 3.3 cli の `signals_order.rs`

残す(Q2: B)。変えない。

### 3.4 仕様書のテストの記述

| # | 場所(HEAD) | 今 | 直した後 | コミット |
|---|---|---|---|---|
| S8 | rust `11:115`(3.1 の性能の表) | 「計画の生成(v0.9.0 以降)。ジョブ 1,000 個がすべて `env` と `env_file` を持ち、…」の行のテストの列 `core/tests/plan_perf.rs` | `core/tests/plan_simulate_perf.rs`(行の文は変えない) | 8 |
| S9 | rust `11:215`(4.2 の core の表) | `plan_types.rs`、`plan_pipeline.rs`、`plan_run.rs`、`plan_assume.rs`、`plan_perf.rs` の目的「…、`--assume`(…)、1,000 ジョブの性能(05 の 16 章)」 | ファイルの列から `plan_perf.rs` を消し、目的から「、1,000 ジョブの性能」を消す(「(05 の 16 章)」は残す) | 8 |
| S10 | rust `11:217`(同) | `plan_simulate_perf.rs`、… の目的「計画の模擬(P1、P2)、…」 | 「計画の模擬(P1、P2)と、すべてのジョブが `env` と `env_file` を持つ 1,000 ジョブの計画の生成(1 秒以内)、…」(この行に、コミット 8 で移したテストを足す) | 8 |
| S11 | rust `11:302`(4.2 の cli の単体テストの表) | `init_types.rs` の目的「v0.9.0 以降。`InitOptions`・`InitTarget`」 | 「v0.9.0 以降。`InitTarget::from_path`(`-` だけが標準出力)。`init` のオプションの既定は `args_grammar.rs` が確かめる」 | 9 |
| S12 | language-agnostic `11:979`(1,000 ジョブの計画の性能の受け入れテストの根拠の欄) | `plan_perf`。区分: 計測(強制) | `plan_simulate_perf`。区分: 計測(強制)(D3。決定: 直す。推奨のとおり。ユーザー、2026-11-25) | 8 |

- 件数: rust 11 は「ファイルごとのテストの件数は書かない」(4.2 の冒頭)。仕様書と README にテストの件数(`1,152` など)の記述は無い(`grep -rn "1,15[0-9]\|1,14[0-9]" docs/spec README.md` が 0 件)。直す件数は無い。
- 消す 3 件の名前を挙げる仕様書の記述は無い(`git grep` で 0 件)。`signals_order` を挙げる rust `07:434`、`11:272`、language-agnostic `11:688` は変えない。
- CI(`.github/workflows`)と README に `plan_perf`・`init_types` の参照は無い。

## 4. ソースの検査(項目 4)

### 4.1 今の形(`crates/cli/tests/nfr_source.rs`)

- `テスト専用の経路を実行ファイルに持たない`(:143-163)の最後の確かめ(:160-162)が、製品のファイル(`src/tests/` の外。`src/bin/` を含む)の、コメント(`//` 以降)を除いた行に `#[path` が無いことを確かめる。`include!(`・`include_str!(`・`include_bytes!(` は見ていない。
- モジュールの説明(:19-20)は「製品のファイルに `#[path` が無いことを確かめる」。

### 4.2 直した後(Q3: A。形は `functional-spec.md` §2.4)

- 既知の限界: 空白を除いた部分一致なので、`my_include!` のような別のマクロ、文字列や `/* */` のコメントの中の語にも当たる(誤検出の側)。`#[cfg_attr(…, path = "…")]` の形は見逃す。

- 同じ確かめで、`#[path`、`include!`、`include_str!`、`include_bytes!` のどれかがあれば失敗にする。行の空白を除いてから比べる(OS 分岐の検査と同じ書き方)。止める形は、マクロの名前と `!`(区切りの `(`・`[`・`{` のどれでも止める。D4。決定: 推奨のとおり。ユーザー、2026-11-25)。
- テストの数は変えない(6 件)。**テストの名前は変えない**(★ なし。`include` の 3 つのマクロは、製品が別のファイル(`src/tests/` やテストのデータ)を取り込む道で、「テスト専用の経路を実行ファイルに持たない」の中身に含まれる。言語に依らない仕様書 11 章の根拠の欄 `nfr_source::テスト専用の経路なし`(801 行目)と `cli::nfr_source`(899 行目)もそのまま正しい)。
- 今の製品のコード: `grep -rn "include\|#\[path" crates/cli/src | grep -v "^crates/cli/src/tests/"` が 0 件。`src/tests/` の下にも `include` の 3 つのマクロは無い(検査の外だが、念のため 0 件を確かめた)。
- 説明: モジュールの説明(:19-20)と、確かめの前のコメント(:160)を合わせる。

### 4.3 仕様書

| # | 場所(HEAD) | 今 | 直した後 | コミット |
|---|---|---|---|---|
| S13 | rust `11:166`(3.3 の表「テスト専用の経路を製品に持たない」) | 「…(`src/tests/` は `lib.rs` の最後の `#[cfg(test)] mod tests;` からだけ読む)。配布は `conductor` だけ」 | 「…(`src/tests/` は `lib.rs` の最後の `#[cfg(test)] mod tests;` からだけ読み、製品のコード(`src/tests/` の外)に `#[path`・`include!`・`include_str!`・`include_bytes!` が無い)。配布は `conductor` だけ」 | 10 |
| S14 | rust `11:178`(3.5) | 「テスト専用の経路なし(この 2 つは単体テスト `src/tests/` を除き、`src/tests/` が `#[cfg(test)]` の宣言からだけ読まれることを確かめる)」 | 「…(この 2 つは単体テスト `src/tests/` を除き、`src/tests/` が `#[cfg(test)]` の宣言からだけ読まれることと、製品のコードに `#[path` と `include` の 3 つのマクロが無いことを確かめる)」 | 10 |

- 変えない: rust `07:29`、`07:400-401`(OS 分岐の検査の範囲の記述。`include` と関わらない)、`11:305`、language-agnostic 11 章の 801 行目・899 行目(テスト専用の経路の受け入れテスト。書き方を名指ししない)。

## 5. 範囲の外で、触らないもの(確かめた)

- 本番の処理のコード(関数の本体、型、可視性)。変えるのは説明の行(C1、C2、C3、C4。C3 は D1 の決定)だけ。
- `crates/core/src/flow/decide.rs`。`signals_order.rs`、`signals_real.rs`。`init_types.rs` の残る 1 件の置き場所。
- テストのファイルの統合、補助の写し(`tests/common` と `src/tests/common`)、init のひな形の原本の重複、本番の小さなモジュールの統合(別の intent)。
- `Cargo.toml`、`Cargo.lock`、CI の設定、README。凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。

## 6. 網羅に使った検索のコマンド

```sh
# 項目 1: 古い説明
git grep -n "decide_before_run" -- ':!aidlc'
git grep -n "decide_from_state" -- crates
grep -n "^pub" crates/runtime/src/lib.rs                       # C3(runtime は core の関数を再公開しない)
# 項目 2: 仕様書 rust 02
grep -n "pub fn resolve_run_id" -A5 crates/cli/src/run_id.rs
grep -n "^pub mod\|^mod\|^pub use" crates/core/src/lib.rs crates/runtime/src/lib.rs crates/cli/src/lib.rs
grep -n "v0\.[0-9]" docs/spec/rust/02-architecture.md
grep -n '^再公開(`lib.rs`)' docs/spec/rust/02-architecture.md
grep -rn "resolve_run_id\|公開モジュール\|再公開\|consistency" docs/spec/rust
ls crates/runtime/src/state/                                    # X2
for f in docs/spec/rust/*.md; do echo "$f $(grep -c 'v0\.[0-9.]* までは' $f)"; done   # X4
# 項目 3: テストの重複
git grep -n "plan_perf\|plan_simulate_perf\|init_types\|signals_order" -- ':!aidlc'
git grep -n "CliCommand::new\|InitOptions::default\|InitTarget::from_path" -- crates
grep -n "init_の既定は_conductor_toml_と_my_pipeline_で_force_なし" -A12 crates/cli/src/tests/args_grammar.rs
grep -rn "1,15[0-9]\|1,14[0-9]" docs/spec README.md                # 件数の記述(0 件)
# 項目 4: ソースの検査
grep -rn "include\|#\[path" crates/cli/src | grep -v "^crates/cli/src/tests/"   # 0 件
grep -rn "nfr_source\|#\[path\|include" docs/spec/rust/07-process-and-signals.md docs/spec/rust/11-quality-and-build.md
grep -rn "nfr_source" docs/spec/language-agnostic
# 件数と名前
cargo test --workspace --locked 2>&1 | grep -E "^test result|Running"
cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$"
```
