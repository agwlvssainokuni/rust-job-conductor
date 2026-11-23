# 変える場所の一覧(Inventory): テストだけが使う本番のコードの削除

HEAD 9e91512(drop-test-only-code の意図の把握の記録のコミット)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。調査の候補と実物が違ったところは「(調査との違い)」と書く。設計の要点、テストの変更の一覧(T…)、消すテストの対の表、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前々の intent 261123-leftovers-and-fifo の Q3 と、この intent の Q1〜Q5): **テストのコードの都合で本番のコードを曲げない**。本番に、テストのための要素を新しく足さない。消す・狭める項目を使っていたテストは、(a) 公開の入口から確かめる、(b) 偽物の側で確かめる、(c) テストの補助に同じ処理を書く、(d) 同じことを確かめる残りの確かめに任せる(消す関数だけを試す確かめは、関数と一緒に消える)、のどれかで代わりを持つ。

決めてほしい点(`functional-spec.md` §10 の D1〜D8)は、ユーザーがすべて推奨のとおりに決めた(2026-11-24)。この文書の「(D1)」〜「(D8)」は、その決定のとおりの形を書いている。D3 は消す方針が決定で、個々の削除は `functional-spec.md` §4.6 として設計の承認で認めてもらう(承認待ち)。

## 0. 基準と、設計の検証

### 0.1 テストの件数(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug、HEAD 9e91512)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 113 | 271 | 384 |
| runtime | 70 | 128 | 198 |
| converter | 52 | 64 通過 + 1 無視 | 116 通過 + 1 無視 |
| cli | 2 | 457 | 459 |
| 全体 | | | **1,157 通過、1 無視、0 失敗** |

前の intent(unbend-for-tests)の見込み 1,157 と一致する。この intent に関わるファイルの件数(基準): core `env_types.rs` 6、`env_file_parse.rs` 10、`definition_valid.rs` 9、`flow_sequences.rs` 9、`graph_overlay.rs` 5、`graph_types.rs` 5、`nfr_security.rs` 5。runtime `runner_basic.rs` 12、`nfr_robust.rs` 6、`nfr_content.rs` 3、`nfr_performance.rs` 1、`flow_with_state.rs` 20、`runner_env.rs` 5、`runner_signal.rs` 6。converter `nfr_security.rs` 12 + 1 無視。cli `args_grammar.rs` 60、`run_id_arg.rs` 1、`escape_table.rs` 4、`report_messages.rs` 36、`signals_real.rs` 1、`signals_order.rs` 1。

数え方: 出力の `Running unittests src/lib.rs`(単体)と `Running tests/*.rs`(統合)の各 `test result:` の `passed` と `ignored` を、括弧の中の `conductor-<クレート>` ごとに足した(作業の出力は `target/drop-fd/` の下に置き、終わったあと消した)。

### 0.2 設計の検証(写しの木での試し)

T 表の網羅を検索だけに頼らないために(前の intent で、型を定義するファイルの中の単体テストが検索から漏れた)、HEAD の写し(`git archive HEAD` を `target/drop-fd/work` に展開。README とデモも含む)に、この一覧の全項目と §4 の T 表の書き換えを当て、次を確かめた。写しは確かめたあと消した。リポジトリのコードは変えていない。

- 本番のコードだけを変えた段階で `cargo check --workspace --all-targets --keep-going` が出したエラーの場所が、T 表の場所と一致する(検索で見つけた場所の漏れの確認)。エラーの出た場所はすべて T 表にある。
- T 表を当てたあと、`cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc`(Windows でだけコンパイルされるテスト `runner_basic.rs` の `cmd_` の 3 件を含む)の両方で通る。
- `cargo test --workspace --locked` が **1,153 通過、1 無視、0 失敗**(§6 の見込みと一致。core 112 + 270、runtime 69 + 128、converter 52 + 64、cli 2 + 456)。
- `cargo doc --workspace --no-deps` の警告は、基準で 5 件(core 1、runtime 4。この intent と無関係)。本番の変更だけでは 20 件増える(消す・狭める名前へのドキュメントのリンク。各項目の「ドキュメントコメント」に挙げた)。ドキュメントコメントを直して、基準の 5 件に戻す(§7)。
- `InterruptSignal` の `Default` を消すと、clippy の `new_without_default`(引数のない `pub fn new()` を持つ公開の型に `Default` を求める)で `-D warnings` が止まる(調査との違い)。決定: `Default` は残す(D1。推奨のとおり。ユーザー、2026-11-24)。写しの試しでは `#[allow]` を一時的に付けて確かめたが、テストの変更と件数は残しても同じ。

---

## 1. 削除

### 1a. core の `Command` の `From<&str>`・`From<String>`

- 今: `crates/core/src/types.rs:81-91`。本番の使用は 0 件(定義の読み取りは `Command::Line(..)`・`Command::Argv(..)` を直接作る)。
- テストの使用(写しのコンパイルで確かめた。`.into()` と `impl Into<Command>` の形が多く、名前の検索では拾い切れない):
  - core: 単体 `src/types.rs:591-597`(テスト `文字列から_line_を作れる`。`From` だけを試す)、単体の補助 `src/test_support.rs:24`、`tests/common/mod.rs:62`、`tests/graph_common/mod.rs:37`、`tests/definition_valid.rs:49`、`:79`、`tests/nfr_security.rs:34`。
  - runtime: 単体 `src/runner.rs:229`(補助 `line`)、`src/split.rs:271`、`:274`(テスト `check_command_は分けた結果を返す`)、統合の補助 `tests/common/mod.rs:36-38`(`command_line(command: impl Into<Command>)`)、`:68-71`(`job(id, command: impl Into<Command>)`)、`tests/runner_basic.rs:23-30`(補助 `run(command: impl Into<Command>)`)とその呼び出しのうち文字列を渡す 10 か所(`:41`、`:125`、`:146`、`:153`、`:162`、`:190`、`:200`、`:230`、`:292`、`:332`。後ろの 2 つは `#[cfg(windows)]` のテスト)、`:171`、`:173`(`Command::from("'")`)、`tests/nfr_robust.rs:150`、`:160`、`:163`、`:171`、`tests/nfr_content.rs:88`、`:93`。`common::job`・`common::command_line` を文字列で呼ぶ所(`flow_with_state.rs:70`、`:735`、`nfr_content.rs:100`、`nfr_performance.rs:36`、`nfr_robust.rs:180`、`:187`、`runner_env.rs:29`、`runner_signal.rs:38`、`:118`、`:154`、`:196`、`runner_basic.rs:218`、`:261`)は、補助の引数を `impl Into<String>` にすれば変えずに済む(D5)。
  - converter・cli: 0 件。
- 目標: 2 つの `impl` を消す。テストは `Command::Line(..)` を書く(本番の読み取りと同じ作り方。方針 (c))。runtime の補助は D5 のとおり。単体テスト `文字列から_line_を作れる` は消す(§4.6)。
- 仕様書: rust・language-agnostic に記載は無い(`grep -rn "From<&str>\|From<String>\|Command::from" docs/spec/rust docs/spec/language-agnostic` で確かめた)。
- 完了の判定: `grep -rn "From<&str> for Command\|From<String> for Command\|Command::from(\|Into<Command>" crates` が 0 件。

### 1b. cli の `RunIdArg::as_str`

- 今: `crates/cli/src/types.rs:50-56`。本番の使用は 0 件(`dry_run.rs:80`、`retry.rs:59` の `as_str()` は `RunId::as_str`)。
- テストの使用: `crates/cli/tests/run_id_arg.rs:25`、`:32`、`tests/args_grammar.rs:495`、`:507`、`:514`、`:702`、`:795`(`map(RunIdArg::as_str)` の形)。
- 目標: メソッドを消す。テストは `RunIdArg` の値そのもので比べる(`RunIdArg` は `PartialEq` を持つ。方針 (a))。`args_grammar.rs` はファイルの中の補助 `fn run_id(text: &str) -> Option<RunIdArg>`(`Some(RunIdArg::Id(RunId::parse(text).expect(..)))`)を置き、`Some("r1")` との比較を `run_id("r1")` との比較に、`Some("")` との比較を `Some(RunIdArg::Empty)` との比較にする。`run_id_arg.rs:25` は消す(直前の `:24` が `RunIdArg::Empty` であることを確かめる)、`:32` は `assert!(matches!(&arg, RunIdArg::Id(id) if id.as_str() == ok))`(書かれた値をそのまま持つこと)。
- 仕様書: rust `02-architecture.md:158` の `RunIdArg { Empty, Id(RunId) }` の説明から `as_str` を消す。
- 完了の判定: `grep -rn "RunIdArg::as_str\|empty.as_str()\|arg.as_str()" crates` が 0 件、`grep -n "fn as_str" crates/cli/src/types.rs` が 0 件。

### 1c. runtime の `InterruptSignal` の `is_raised`・`PartialEq`・`Eq`(`Default` は残す。D1)

- 今: `crates/runtime/src/types.rs:66-69`(`is_raised`)、`:80-84`(`Default`)、`:95-101`(`PartialEq`・`Eq`)。本番の使用は 0 件(cli と runtime は `signal_number()` だけを読む)。`Debug`(`:86-93`)が `is_raised()` を呼んで `raised` の項目を書く。
- テストの使用: runtime 単体 `src/types.rs:158`、`:160`(`== InterruptSignal::default()`)、`:168`、`:170`(`assert_ne!(s, InterruptSignal::new())`)、`:177`、cli `tests/signals_real.rs:46`、`tests/signals_order.rs:38`。
- 目標: `is_raised`・`PartialEq`・`Eq` を消す。`Debug` の `raised` の項目は `signal_number().is_some()` で書き、出力の形を変えない(D8。単体テスト `合図はスレッド間で共有できる` の `raised: true` の確かめが変わらない)。テストは `signal_number()` で確かめる(方針 (a)): `!is_raised()` は `signal_number() == None`、`is_raised()` は `signal_number().is_some()`。単体テストで、すぐ後ろの行が同じことを `signal_number()` で確かめている `:158`、`:168`、`:177` と、等価性だけを確かめる `:160`、`:170` は消す。
- `Default`(調査との違い): 消すと clippy の `new_without_default` が `pub const fn new()`(`:47`)に当たり、`-D warnings` で止まる。決定: 残す(D1。推奨のとおり。ユーザー、2026-11-24)。`Default` の `impl`(`:80-84`)と仕様書の記載は変えない。単体テスト `:160` の `assert_eq!(s, InterruptSignal::default())` は `==`(`PartialEq`)を使うので、`Default` を残しても消す(T17)。
- 仕様書: rust `02-architecture.md:125`(`is_raised` を消す)、`07-process-and-signals.md:277`(`is_raised()` / `signal_number()` の行を `signal_number()` の行に。「立っているか」は `Some` かで分かる)、`:280`(「等価性」の行を消す)。language-agnostic `07:305` は論理の説明(「立っているか」と番号)なので変えない。
- 完了の判定: `grep -rn "is_raised\|impl PartialEq for InterruptSignal\|impl Eq for InterruptSignal" crates docs/spec/rust` が 0 件。

### 1d. 本番で使われない `Default`(4 つ)

| # | 今の場所 | テストの使用 | 目標 |
|---|----------|--------------|------|
| 1d-1 | core `crates/core/src/types.rs:265-267`(`JobStatus` の derive と `#[default]`) | 単体 `src/types.rs:603`(テスト `新しいジョブの状態は_pending` の `JobStatus::default() == Pending`) | derive と `#[default]` を消す。`:603` の 1 行を消す(同じテストの `:602` が、新しいジョブの状態が `Pending` であることを確かめる) |
| 1d-2 | runtime `crates/runtime/src/types.rs:142`、`:145`(`Durability`) | 単体 `src/types.rs:227-230`(テスト `耐久性の既定は_durable`。`Default` だけを試す) | derive と `#[default]` を消す。テストは消す(§4.6) |
| 1d-3 | core `crates/core/src/graph/mod.rs:73`(`StateOverlay`) | `tests/graph_overlay.rs:110`(空の重ね合わせ)、`tests/graph_types.rs:43-53`(テスト `状態の重ね合わせの既定は空で_再開位置もない`) | derive の `Default` を消す。`graph_overlay.rs:110` は `StateOverlay { states: vec![], resume_job_id: None }` と書く。`graph_types.rs` のテストは消す(§4.6。D3: 消す方針は決定(推奨のとおり。ユーザー、2026-11-24)、個々の削除は承認待ち)、使わなくなる import の `StateOverlay`(`:22`)を消す |
| 1d-4 | cli `crates/cli/src/types.rs:297`、`:299`(`Language`) | 0 件 | derive と `#[default]` を消す |

- 本番の使用はどれも 0 件(写しで、本番のコードだけを変えてコンパイルが通ることを確かめた。`..Default::default()` で間接に使う所も無い)。
- 仕様書: rust `02-architecture.md:125` の `Durability { Durable(既定), Fast }` と `:158` の `Language { En(既定), Ja }` から「(既定)」を消す(言語の判定は 08 章のまま)。`JobStatus`・`StateOverlay` の `Default` の記載は無い。
- 完了の判定: `grep -rn "JobStatus::default\|Durability::default\|StateOverlay::default\|Language::default" crates` が 0 件、`grep -rn "#\[default\]" crates/*/src` が cli `types.rs` の `GraphFormat` の 1 件だけ。

### 1e. core の `sanitize`

- 今: `crates/core/src/error.rs:205-210`(`escape_display(input, Some(max_chars))` を呼ぶだけ)、再公開 `crates/core/src/lib.rs:105`。本番の呼び出しは同じファイルの `:173`(`render_error`)と `:238`(`quote_key`)。
- テストの使用: core 単体 `src/error.rs:281`、`:285`、`:317`、`:324`、`:329`、`:335`(テスト `書式文字と双方向制御文字もエスケープする`、`キー名の制御文字はエスケープされる`、`キー名は_64_文字で切られる`、`エスケープ後も_64_文字を超えない`)、cli `tests/escape_table.rs:56-65`(テスト `u1_core_の_sanitize_と同じ結果`)。
- 目標: 関数と再公開を消す。呼ぶ所は `escape_display(x, Some(n))`。単体テストは同じ置き換え(名前と期待は変えない)。`escape_table.rs` の `u1_core_の_sanitize_と同じ結果` は消す(§4.6。cli の再公開と core の包みが同じ関数を呼ぶことの確かめで、包みが無くなる)。converter の `sanitize`(`crates/converter/src/error.rs:161`。core に依存しないための写し)は範囲の外で、変えない。
- ドキュメントコメント: cli `src/report/escape.rs:17`(「`sanitize` も同じ関数を呼ぶ」を消す)、cli `tests/escape_table.rs:15-19`(モジュールの説明の「core の `sanitize` も同じ関数を使う」を消す)。
- 仕様書: rust `02-architecture.md:42`(再公開の列)、`:101`(補助の行の `sanitize(..)` を消す)、`:287`、`08-output-and-i18n.md:605`、`11-quality-and-build.md:162`(「core の `sanitize`・」を消す)、`12-rebuild-guide.md:22`、`:23`(`sanitize` を `escape_display`(上限つき)に)。
- 完了の判定: `grep -rn "sanitize" crates/core crates/cli docs/spec/rust` が 0 件(converter の写しは対象外)。

### 1f. core の `EnvFileError::reason()`

- 今: `crates/core/src/envfile/mod.rs:162-166`(`self.kind.reason()` を返すだけ)。本番の使用は同じファイルの `:169`(`render`)だけ。cli は `kind.reason()` を直接使う。
- テストの使用: core `tests/env_file_parse.rs:179`、`tests/env_types.rs:270`(テスト `誤りの理由は種類の固定の文で_種類の文は変更前と同じ` の `e.reason() == kind.reason()`)。
- 目標: メソッドを消し、`render` は `self.kind.reason()`。`env_file_parse.rs:179` は `err.kind.reason()`。`env_types.rs:270` は消す(メソッドの写しを確かめる行。直後の `:271` が表示に種類の理由を含むことを確かめる)。
- 仕様書: rust `02-architecture.md:100`(「理由のメソッド `reason()`…(`kind.reason()`。…)」を「理由は `kind.reason()`」に)、`10-errors-and-exit-codes.md:122`(「と `reason()`」を消す)。
- 完了の判定: `grep -n "pub fn reason(&self)" crates/core/src/envfile/mod.rs` が 0 件、`grep -rn "err\.reason()\|e\.reason()" crates/core` が 0 件。

---

## 2. 公開範囲を狭める

### 2a. core の `decide_before_run`

- 今: `crates/core/src/flow/decide.rs:46`(`pub fn`)、再公開 `flow/mod.rs:31`、`lib.rs:109`。本番の呼び出しは `decide_from_state`(同じファイルの `:107`)だけ。
- テストの使用: core `tests/flow_sequences.rs:26`(import)、`:68-76`(模型 `Sim::decide`)、runtime `tests/flow_with_state.rs:33`(import)、`:140-156`(補助 `decide`。core の `decide_from_state` と同じ `match` の写し)、その呼び出し `:256`、`:350`。単体 `decide.rs` の `#[cfg(test)]` のテストは同じモジュールなので変えない。
- 目標: `decide.rs` の中だけの非公開の `fn` にし(D6)、2 つの再公開を消す。テストは公開の `decide_from_state` で確かめる(方針 (a)): `Sim::decide` は持っている状態から `PipelineState` を組んで `decide_from_state(id, &self.def, Some(&state))`(模型の項目と判断は変えない)。`flow_with_state.rs` の補助 `decide` は消し、2 か所は `decide_from_state` を直接呼ぶ(D7)。
- ドキュメントコメント: core `lib.rs:28`(関数の表の行を `decide_from_state` に)、`:67`、`flow/next_step.rs:39`(`[decide_before_run](super::decide_before_run)` を `decide_from_state` に)、`flow/decide.rs:92`(公開の説明から非公開の関数へのリンクを、ふつうの文字に)、cli `lib.rs:42`、`deps.rs:21`、`run.rs:17`、`pipeline.rs:17`、runtime `lib.rs:77`、runtime `tests/flow_with_state.rs:15` の `decide_before_run` を `decide_from_state` に。
- 仕様書: rust `02-architecture.md:42`(再公開の列から消す)、`:55`(関数の表の行に「`flow/decide.rs` の中だけ。公開は `decide_from_state`」と添える)、`:200`、`:241`、`:248`、`:305`、`12-rebuild-guide.md:40`。`05-execution-flow.md` の 4 章などの判断の手順の名前は、手順の説明として残す。language-agnostic の名前は「仮の名前」(`00-README.md:113`)なので変えない。
- 完了の判定: `grep -rn "decide_before_run" crates/*/tests crates/cli crates/runtime` が 0 件、`grep -rn "pub fn decide_before_run\|decide_before_run," crates/core/src` が 0 件。

### 2b. core の `flow::classify`

- 今: `crates/core/src/flow/classify.rs:56`(`pub fn`)、再公開 `flow/mod.rs:30`。使うのは `flow/next_step.rs:17`、`:56` と、同じファイルの単体テストだけ。外の使用(cli・runtime・テスト)は 0 件。
- 目標: `pub(super)` にし(D6)、再公開を消す。テストの変更は無い。
- ドキュメントコメント: `flow/mod.rs:22`(「`flow::classify` など」を「`flow::classify_and_decide` など」に)。
- 仕様書: rust `02-architecture.md:42`(下位モジュールの例の `flow::classify` を `flow::classify_and_decide` に)、`:53`(関数の表の行に「`flow` の中だけ」と添える)。05 の 2.3 の手順の名前は残す。
- 完了の判定: `grep -n "pub use classify::classify\|pub fn classify(" crates/core/src/flow/*.rs` が 0 件。

### 2c. core の `MAX_RUN_ID_LEN`・`is_valid_env_name`・`is_reserved_env_name`・`JobDefinition::effective_success_codes`

| # | 今の場所 | 使う所 | 目標 | テスト |
|---|----------|--------|------|--------|
| 2c-1 | `crates/core/src/run_id.rs:24`(`pub const MAX_RUN_ID_LEN`)、再公開 `lib.rs:116` | `run_id.rs:52` だけ | `run_id.rs` の中だけの非公開の `const`(D6) | 変更なし |
| 2c-2 | `crates/core/src/run_id.rs:83`(`pub fn is_valid_env_name`)、再公開 `lib.rs:116` | core の `envfile/parse.rs`、`definition/walk/env.rs`、`definition/walk/mod.rs`、`run_id.rs` | `pub(crate)` | 変更なし(単体 `run_id.rs:160-169` は同じモジュール) |
| 2c-3 | `crates/core/src/envfile/mod.rs:63`(`pub fn is_reserved_env_name`)、再公開 `lib.rs:102` | core の `envfile/parse.rs`、`definition/walk/env.rs` | `pub(crate)` | 変更なし |
| 2c-4 | `crates/core/src/types.rs:144`(`pub fn effective_success_codes`) | core の `flow/classify.rs:91` | `pub(crate)` | `tests/definition_valid.rs:52`、`:57`(下) |

- `definition_valid.rs`: `:52`(`extract.effective_success_codes() == [0]`)は消す(直前の `:51` が `success_exit_codes == None` を確かめ、`None` を `[0]` として扱うことは core の単体テスト `types.rs:524` と `classify.rs` の単体テストが確かめる)。`:57` は `transform.success_exit_codes == Some(vec![0, 2])`(読み取りの結果をそのまま比べる。写しで通ることを確かめた)。
- cli・runtime・converter の使用は 0 件。
- 仕様書: rust `02-architecture.md:42`(再公開の列から `MAX_RUN_ID_LEN`・`is_valid_env_name`・`is_reserved_env_name` を消す)、`:51`、`:63`(関数の表の 2 行に「crate の中」と添える)、`:76`(`effective_success_codes()` に「crate の中」と添える)、`:288`、`:293`(ファイルの表に「crate の中」と添える)。03 章の `is_valid_env_name`・`effective_success_codes` は規則の説明として残す。
- 完了の判定: `grep -rn "MAX_RUN_ID_LEN" crates | grep -v "^crates/core/src/run_id.rs"` が 0 件、`grep -rn "pub fn is_valid_env_name\|pub fn is_reserved_env_name\|pub fn effective_success_codes\|pub const MAX_RUN_ID_LEN" crates` が 0 件、`grep -rn "effective_success_codes" crates/*/tests` が 0 件。

### 2d. cli の `report::escape` の `escape_display` の再公開

- 今: `crates/cli/src/report/escape.rs:21`(`pub use conductor_core::escape_display;`)。cli の本番は同じファイルの `escape_name`・`escape_full` の中だけで使う。
- テストの使用: cli `tests/escape_table.rs:21`(import)と、そのテストの中の呼び出し。
- 目標: `use conductor_core::escape_display;`(非公開の import)。`escape_table.rs` は `conductor_core::escape_display` を直接使う(本体は core。cli のテストはすでに `conductor_core` に依存する)。`escape_name`・`escape_full` は公開のまま。
- ドキュメントコメント: `src/report/escape.rs:15-19`(「ここは再公開と、」を「ここは」に)、core `src/error.rs:181`(「conductor-cli の `report::escape` は、これを再公開する」を「conductor-cli の `report::escape` が包む」に)、cli `tests/escape_table.rs:17`。
- 仕様書: rust `02-architecture.md:154`(cli の関数の表の `escape_display` の行を消し、`escape_name`・`escape_full` の説明に「本体は core の `escape_display`」)、`:376`、`08-output-and-i18n.md:566`、`:605`、`11-quality-and-build.md:162`、`:258`。
- 完了の判定: `grep -rn "pub use conductor_core::escape_display\|escape::{escape_display\|report::escape::escape_display" crates` が 0 件。

### 2e. cli の `lib.rs` の `pub mod` 4 つ

- 今: `crates/cli/src/lib.rs:88`(`args`)、`:91`(`deps`)、`:96`(`panic_hook`)、`:104`(`types`)。4 つのモジュールの公開の項目は、`GraphOptions`・`DryRunOptions`(2f)を除き、すべて根で再公開されている(`lib.rs:108-117`)。`conductor_cli::args::` などのパスの使用は 0 件(テスト、`main.rs`、補助のバイナリ)。
- 目標: 4 つを `mod` にする。公開のモジュールは `exit`、`report`、`run_id` の 3 つ。テストの変更は無い。
- ドキュメントコメント: `lib.rs:50`、`:56`、`:57` のモジュールの表の [`args`]・[`panic_hook`]・[`deps`] を、リンクでない `args` などに(非公開の項目へのリンクの警告になる)。
- 仕様書: rust `02-architecture.md:143`(「公開モジュール: `args`、`deps`、`exit`、`panic_hook`、`report`(…)、`types`」を「`exit`、`report`(…)、`run_id`」に。`run_id` が今の記載から漏れている(調査との違い)ので合わせる)。
- 完了の判定: `grep -n "^pub mod" crates/cli/src/lib.rs` が `exit`、`report`、`run_id` の 3 件。

### 2f. cli の根の `GraphOptions`・`DryRunOptions` の再公開

- 今: `crates/cli/src/lib.rs:115`。外の使用(テスト、`main.rs`)は 0 件。テストは `cmd.graph.format`・`cmd.dry_run.assumptions` のように項目で読む。
- 目標: 再公開から 2 つを消す。2 つの型は `CliCommand` の公開の項目の型なので、項目としては読めるが、名前では書けなくなる(範囲の外の「cli のライブラリの公開範囲の大きい部分」に関わるので、`CliCommand` の形は変えない。functional-spec §3)。
- 仕様書: rust `02-architecture.md:143`(再公開の列から 2 つを消す)。
- 完了の判定: `grep -n "GraphOptions\|DryRunOptions" crates/cli/src/lib.rs` が 0 件。

---

## 3. 判断が要ったもの(Q2〜Q4)

### 3a. core の `RunIdError` を種類のない誤りの型 1 つに(Q2: A)

- 今: `crates/core/src/run_id.rs:29-40`(`Empty`・`TooLong`・`InvalidChar`・`LeadingDot`)、`:48-65`(`RunId::parse` が順に判定)。本番の呼ぶ所(cli `src/types.rs:45`(`--run-id`)と `src/run_id.rs:47`(環境変数の値)の `map_err(|_| ..)`、runtime `src/state/reset_all.rs:107` の `.ok()`)は、すべて種類を見ずに捨てる。
- テストの使用: 単体 `src/run_id.rs:134`、`:135`、`:140-142`、`:152`(テスト `長さの境界は_64_と_65`、`先頭のドットは拒否する`、`使えない文字は拒否する`)。cli・runtime の使用は 0 件。
- 目標: `pub struct RunIdError;`(`Debug, Clone, Copy, PartialEq, Eq`。D2)。`RunId::parse` は 4 つの条件(空、64 バイト超、`A-Za-z0-9._-` 以外の文字、先頭が `.`)のどれかで `Err(RunIdError)`(結果が同じになるので 1 つの条件式にまとめる。D2)。単体テストは `Err(RunIdError::Xxx)` を `Err(RunIdError)` に。4 つの場合の入力(空、65 文字、`.a`・`..`・`.`、`a/b` など 14 個)は残す。
- 仕様書: rust `02-architecture.md:99`(`RunIdError` の行を「種類を持たない(`struct RunIdError;`)。4 つの条件のどれかで返す」に)、`05-execution-flow.md:412-422`(6.1 の表の「誤り」の列を消し、「次の条件のどれかに当てはまれば誤り」に。条件の 4 行は残す)、`10-errors-and-exit-codes.md:125`、`12-rebuild-guide.md:22`(「4 つの誤りを…順に判定」を「4 つの条件のどれかで誤り」に)。language-agnostic `05:410-424`・`10:145` は変えない(D4。決定: 推奨のとおり。ユーザー、2026-11-24)。
- 完了の判定: `grep -rn "RunIdError::" crates docs/spec/rust` が 0 件、`::` の付かない変種名も `grep -rnwE "TooLong|InvalidChar|LeadingDot" crates docs/spec/rust` が 0 件、`Empty` は他の型(`CommandErrorKind`、`RunIdArg`)にもあるので文脈で絞り `grep -rn "RunIdError.*Empty\|\`Empty\`[・,、] *\`TooLong\`" crates docs/spec/rust` が 0 件(rust `02:99`、`05` の 6.1 の表、`10:125` の記載が消えたことを確かめる)。

### 3b. runtime の core の再公開をやめる(Q3: A)

- 今: `crates/runtime/src/lib.rs:129`(`check_consistency`)、`:130`(`apply_interrupt`・`apply_outcome`・`apply_skip`・`initial_state`)、`:139-141`(`types` の `ConsistencyResult`・`MismatchKind`)、`crates/runtime/src/types.rs:136-139`(`pub use conductor_core::{ConsistencyResult, MismatchKind};` と説明)。runtime の本番は、これらを使わない。
- 使う所(`conductor_runtime` を通すもの。`use` の塊の中を `awk` で調べた。§8):
  - cli の本番: `src/ctx.rs:27`(`ConsistencyResult`)、`:28-30`(`check_consistency`)、`src/pipeline.rs:30`(`apply_interrupt`)、`src/step.rs:37`(`apply_outcome`・`apply_skip`・`initial_state`)、`src/report/mod.rs:44`(`MismatchKind`)、`src/report/messages.rs:26`(`MismatchKind`)。
  - runtime のテスト: `tests/common/mod.rs:113`(`conductor_runtime::initial_state`)、`tests/flow_with_state.rs:36-39`(`ConsistencyResult`・`MismatchKind`・`apply_outcome`・`apply_skip`・`check_consistency`)、`tests/nfr_content.rs:21-23`(`apply_outcome`)、`tests/nfr_performance.rs:26`(`ConsistencyResult`・`check_consistency`)。
  - cli のテスト: `tests/report_messages.rs:34-36`(`MismatchKind`)。
  - core・converter: 0 件。
- 目標: 再公開を消し、使う所は `conductor_core` から `use` する(中身は同じ)。runtime の `types.rs:136-139` の再公開と説明を消す。
- ドキュメントコメント: runtime `lib.rs:33`、`:37-40`(関数の表の 5 行を消し、表の下に「ステートの照合(`check_consistency`)と更新(`initial_state`、`apply_*`)は conductor-core の関数を使う」と 1 文)、`:42-44`、`:58`(`types` の行から `ConsistencyResult`)、`:76`、`:79`、`:82`、`:85-86`(リンクを `conductor_core::…` のふつうの文字に)、`src/state/mod.rs:16-17`(「どちらも `lib.rs` で再公開する」を消す)、core `src/consistency.rs:17`(「conductor-runtime は、同じ名前で再エクスポートする」を消す)。
- 仕様書: rust `02-architecture.md:67`、`:68`(「`conductor-runtime` が同じ名前で再公開する」を消し、「2.2 の表」をシグネチャに置き換える)、`:105`(runtime の再公開の列から 7 つを消す)、`:118-122`(runtime の関数の表の 5 行を消す。シグネチャは `:67`・`:68` へ)、`:125`(「この 2 つは `conductor_core` の再公開」を消す)、`:286`、`:307`、`:316`、`05-execution-flow.md:5`、`06-state-file.md:294`、`10-errors-and-exit-codes.md:140`。
- 完了の判定: `grep -rn "pub use conductor_core" crates/runtime/src` が 0 件(再公開が無いので、`conductor_runtime` を通した参照が残ればコンパイルが止まる)。§8 の `awk` で、`conductor_runtime` の `use` の塊にこの 7 つの名前が 0 件。

### 3c. 誤りの種類の `Display`・`Error`(Q4: A)

| # | 今の場所 | 本番の使用 | テストの使用 | 目標 |
|---|----------|------------|--------------|------|
| 3c-1 | core `crates/core/src/error.rs:83-89`(`DefinitionErrorKind` の `Display`・`Error`) | 0 件(`DefinitionError` の表示は `kind.label()`) | `tests/env_types.rs:59`(`k.to_string()`) | 消す。テストは `k.label()` |
| 3c-2 | core `crates/core/src/envfile/mod.rs:90-96`(`EnvFileErrorKind` の `Display`・`Error`) | 0 件(`EnvFileError` の表示は `kind.label()`) | `tests/env_types.rs:69`(`k.to_string()`) | 消す。テストは `k.label()` |
| 3c-3 | converter `crates/converter/src/error.rs:56`(`ConversionErrorKind` の `Error`) | 0 件 | 0 件 | 消す(`Display`(`:50-54`)は本番が使うので残す) |

- テスト名 `定義の誤りの種類は_33_種で表示が空でない` は、確かめる文(`label()`)が表示に出る種類の文なので、名前を変えない。
- ドキュメントコメント: core `error.rs:31`、`envfile/mod.rs:71` の「`Display` はそれを書く」を「誤りの表示(`DefinitionError`・`EnvFileError`)はそれを書く」に。converter `error.rs:38` は `Display` を残すので変えない。
- 仕様書: rust `03-definition-format.md:393`(「日本語の種類の文は `Display`」を「`label()`」に)、`08-output-and-i18n.md:128`(「`Display` もこれを書く」を「誤りの `Display` もこれを書く」に)、`10-errors-and-exit-codes.md:120`(「日本語の文(`Display`)」を「日本語の文(`label()`)」に)。
- 完了の判定: `grep -rn "Display for DefinitionErrorKind\|Display for EnvFileErrorKind\|Error for DefinitionErrorKind\|Error for EnvFileErrorKind\|Error for ConversionErrorKind" crates` が 0 件。

### 3d. 本番で使われない `Hash`(Q4: A)

| # | 今の場所 | テストの使用 | 目標 |
|---|----------|--------------|------|
| 3d-1 | core `crates/core/src/error.rs:33`(`DefinitionErrorKind`) | `tests/nfr_security.rs:20`(import)、`:192`(`HashSet<DefinitionErrorKind>`)、`:202`、`:213`、`:218`、`:223`、`:230`(`insert`)、`:233`(`contains`) | derive から消す。テストは `Vec<DefinitionErrorKind>` に `push` し、`contains` で確かめる(並べて比べる。重複は数えないので結果は同じ) |
| 3d-2 | core `crates/core/src/envfile/mod.rs:73`(`EnvFileErrorKind`) | 0 件 | derive から消す |
| 3d-3 | converter `crates/converter/src/error.rs:40`(`ConversionErrorKind`) | `tests/nfr_security.rs:19`(import)、`:314`(`HashSet<K>`)、`:316`(`contains`) | derive から消す。テストは `Vec<K>` |
| 3d-4 | converter `crates/converter/src/types.rs:24`(`TargetFormat`) | 0 件 | derive から消す |

- 本番の使用はどれも 0 件(cli の文言の表は `match`。写しで確かめた)。仕様書に `Hash` の記載は無い。
- 完了の判定: `grep -rn -A1 "Hash)\]" crates/core/src crates/converter/src | grep "enum \(DefinitionErrorKind\|EnvFileErrorKind\|ConversionErrorKind\|TargetFormat\)"` が 0 件、`grep -rn "HashSet<\(DefinitionErrorKind\|K\)>" crates` が 0 件。

### 3e. 残すもの(Q5: A)

- core の `DefinitionError`・`EnvFileError`、runtime の `CommandError`・`StateError`・`CreateError`、converter の `ConversionError` の `Display` と `std::error::Error` は残す(範囲の外)。converter の `ConversionErrorKind` の `Display` も、本番の表示が使うので残す。

---

## 8. 使った検索のコマンド(テストの変更の一覧の網羅のため)

すべて HEAD 9e91512 で、リポジトリのルートで実行した。テストの補助のディレクトリ(`tests/common/`、`tests/graph_common/`、`tests/bin_support/` など)と、src の `#[cfg(test)]` の単体テスト(当たった行が `#[cfg(test)]` より後ろかを行番号で確かめた)を含む。

```sh
# 基準の件数
cargo test --locked --workspace > target/drop-fd/baseline.txt 2>&1   # 集計後に target/drop-fd を消した
# 1a
grep -rn "From<&str> for Command\|From<String> for Command\|Command::from\|Into<Command>\|command: .*\.into()" crates
# 1b
grep -rn "as_str" crates/cli
# 1c
grep -rn "is_raised\|InterruptSignal::default\|InterruptSignal" crates docs/spec/rust docs/spec/language-agnostic
# 1d
grep -rn -A3 "derive(.*Default" crates/*/src
grep -rn "Default::default()\|::default()\|unwrap_or_default" crates
# 1e
grep -rn "sanitize" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "escape_display" crates docs/spec/rust
# 1f
grep -rn "\.reason()" crates | grep -v "kind.reason()\|kind().reason()"
# 2a・2b
grep -rn "decide_before_run\|decide_from_state" crates
grep -rn "flow::classify\|[^_]classify(" crates | grep -v "classify_and_decide\|classify_exit_code"
# 2c
grep -rn "MAX_RUN_ID_LEN\|is_valid_env_name\|is_reserved_env_name\|effective_success_codes" crates docs/spec/rust docs/spec/language-agnostic
# 2d〜2f
grep -rn "conductor_cli::\(args\|deps\|panic_hook\|types\)\|GraphOptions\|DryRunOptions" crates docs/spec/rust
grep -n "^pub \(fn\|struct\|enum\|trait\|const\|type\|use\)" crates/cli/src/args.rs crates/cli/src/deps.rs crates/cli/src/panic_hook.rs crates/cli/src/types.rs
# 3a
grep -rn "RunIdError\|LeadingDot\|InvalidChar\|TooLong\b" crates docs/spec/rust docs/spec/language-agnostic
# 3b(複数行の use の塊の中も見る)
for f in $(grep -rl "conductor_runtime" crates); do awk -v F=$f '/use conductor_runtime/{u=1} u{print F":"NR": "$0} /;/{u=0}' $f; done \
  | grep -E "check_consistency|apply_interrupt|apply_outcome|apply_skip|initial_state|ConsistencyResult|MismatchKind"
grep -rnE "conductor_runtime::(check_consistency|apply_|initial_state|ConsistencyResult|MismatchKind)" crates docs/spec/rust
# 3c・3d
grep -rn "impl.*for DefinitionErrorKind\|impl.*for EnvFileErrorKind\|impl.*for ConversionErrorKind\|impl.*for TargetFormat" crates/*/src
grep -rn "HashSet\|HashMap\|BTreeSet" crates/*/tests crates/*/src
# Windows でだけコンパイルされる所に、消す名前が無いか
grep -rln "cfg(windows)\|cfg(not(unix))" crates | xargs grep -n "is_raised\|::default()\|\.into()\|sanitize\|as_str()\|MismatchKind\|check_consistency\|apply_\|initial_state\|decide_before_run"
# 写しの木での検証(0.2。target/drop-fd/work に展開し、確かめたあと消した)
git archive HEAD crates Cargo.toml Cargo.lock README.md demo .github | tar -x -C target/drop-fd/work
cargo check --workspace --locked --all-targets --keep-going --message-format=short     # 本番だけ変えた段階のエラーの場所
cargo clippy --workspace --locked --all-targets -- -D warnings                         # T 表を当てたあと
cargo clippy --workspace --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings
cargo test --workspace --locked --no-fail-fast
cargo doc --workspace --no-deps --locked                                               # 警告の数
```
