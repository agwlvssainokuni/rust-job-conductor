# 変える場所の一覧(Inventory): テストの都合で曲げられた本番のコードと、残りの整理

HEAD c5f1a3a(unbend-for-tests の意図の把握の記録のコミット)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。直前の調査の候補と実物が違ったところは「(調査との違い)」と書く。各段の設計の要点、テストの変更の一覧(T…)、消すテストの対の表、足すテスト(N…)、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent 261123-leftovers-and-fifo の Q3): **テストのコードの都合で本番のコードを曲げない**。本番から、テストのためだけの関数・項目・引数・公開範囲・差し替え口をなくし、テストは (a) 公開の入口から確かめる、(b) 偽物の側で記録する、(c) テストの補助に同じ処理を書く、(d) 単体テストへ移す、のどれかで代わりを持つ。依存の注入の継ぎ目(`Runtime`・`Sink`・`Env`・`Signals` のトレイト、`run_cli_with`、`Deps`)は設計上の継ぎ目なので残す。

## 0. 基準

### 0.1 テストの件数(`cargo test --locked --workspace`、macOS(Apple M2)、rustc 1.100.0、debug、HEAD c5f1a3a)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 113 | 271 | 384 |
| runtime | 70 | 128 | 198 |
| converter | 53 | 64 通過 + 1 無視 | 117 通過 + 1 無視 |
| cli | 0 | 462 | 462 |
| 全体 | | | **1,161 通過、1 無視、0 失敗** |

前の intent(leftovers-and-fifo)の見込み 1,161 と一致する。この intent に関わるファイルの件数(基準): core `env_types.rs` 6、`nfr_security.rs` 5、`graph_drawable.rs` 21、`graph_layout_oracle.rs` 11、`graph_edges.rs` 9、`graph_types.rs` 5、`graph_lookup_perf.rs` 2、`env_merge.rs` 8、`env_merge_perf.rs` 2、`plan_types.rs` 3、`plan_perf.rs` 2。runtime `run_id_state.rs` 15、`state_read.rs` 15、`state_write.rs` 10。converter `nfr_security.rs` 12 + 1 無視。cli `report_messages.rs` 36、`args_grammar.rs` 61、`panic_hook.rs` 3、`signals_real.rs` 1、`flow_run.rs` 53、`flow_run_id.rs` 24、`plan_report.rs` 7。

数え方: `cargo test --locked --workspace` の出力の `Running unittests src/lib.rs`(単体)と `Running tests/*.rs`(統合)の各 `test result:` の `passed` と `ignored` をクレートごとに足した(作業の出力は `target/` の下に置き、終わったあと消した)。

---

## 段 1(テストの都合で曲げられた所)

### 1a. cli の `Reporter::language_resolved` と `color_resolved`

- 今: `crates/cli/src/report/mod.rs:289-299`(`#[doc(hidden)]`、説明は「遅延のテスト用。契約の外」)。本番の使用は 0 件。使うのは `crates/cli/tests/report_messages.rs:664`、`:669`、`:673`、`:704`(テスト `言語と色の判定は_最初に必要になるまで行われない` と `バージョンとシグナルの準備の失敗は_ロケールを判定しない`)だけ。
- 偽物の側の記録はすでにある(調査との違い: 記録の項目が調査より多い): `crates/cli/tests/common/mod.rs:41-57` の `FakeEnv` は `var_calls`・`terminal_calls` に加えて、読まれた環境変数の名前 `names` と、ロケールの変数が読まれたかを返す `locale_read()` を持つ(`flow_run.rs:61`、`flow_pipeline.rs:34` が使用中)。
- 目標: 2 つのメソッドを消す。テストは偽の `Env` の記録で確かめる(方針 (b))。`FakeEnv` に `color_read()`(`NO_COLOR` が読まれた、または `stdout_is_terminal` が呼ばれた)を足す。`language_resolved()` は `env.locale_read()`、`color_resolved()` は `env.color_read()` に置き換える(色の判定は `NO_COLOR` と端末の問い合わせだけで決まり、言語の判定は `LC_ALL`・`LC_MESSAGES`・`LANG` だけを読むので、「判定したか」と「問い合わせたか」は一致する)。
- 仕様書: rust `02-architecture.md:142` の「`Reporter::language_resolved`・`color_resolved` はテスト用(契約の外。`#[doc(hidden)]`)」を消す。
- 完了の判定: `grep -rn "language_resolved\|color_resolved" crates docs/spec/rust docs/spec/language-agnostic` が 0 件。

### 1b. runtime の `ResetAllReport.removed`

- 今: `crates/runtime/src/state/reset_all.rs:41-42`(`removed: usize`)、`:117-122`(`remove` が `remove_state_file` の真偽値で数を足す)。本番(cli `aux_commands.rs:57-61`)は `kept` だけを読む。`removed` を読むのはテストだけ: runtime `tests/run_id_state.rs:230`、`:259`、`:276`、`:297`、`:313`、`:340`、`tests/state_read.rs:290`。組み立てるのは cli `tests/flow_run_id.rs:310`(上書きの値)と `tests/common/mod.rs:807-815`(`MemRuntime::reset_all_states` が数を数える)。
- 付随(調査との違い): `crates/runtime/src/state/writer.rs:104` の `remove_state_file` が返す `bool`(消したか)を読むのは `reset_all.rs:119` の `removed` の数えだけ(`writer.rs:99` の `reset_state` は捨てる)。`removed` を消すと `bool` も読まれなくなる。
- 目標: `ResetAllReport` は `kept` だけを持つ(構造体のまま。D9。決定(ユーザー、2026-11-23))。`remove_state_file` は `Result<(), StateError>`(無いファイルは成功)。テストは、ファイルの一覧(`names(p)`)と有無(`exists`)で確かめる(方針 (a))。7 か所のうち 6 か所は、同じテストの中にすでにファイルの確かめがある。`run_id_state.rs:313`(大きすぎる・版違い)だけは、残したファイル 2 つの有無の確かめを足す。
- 仕様書: rust `02-architecture.md:117` の `ResetAllReport { removed: usize, kept: Vec<String> }` を `{ kept: Vec<String> }` に。版を上げないので、「v0.9.8 までは」などの版の注記は書かず、今の形だけを書く(この intent の仕様書の変更すべてで同じ。rules.md の BR9.5)。
- 完了の判定: `grep -rn "\.removed\b\|removed:" crates docs/spec/rust` が 0 件(`docs/spec/rust/02` の注記を除く)。

### 1c. core の `graph::is_lane_drawable`

- 今: `crates/core/src/graph/layout.rs:111-116`(`skip_targets(graph).is_some()`)、再公開 `graph/mod.rs:35`。本番の呼び出しは 0 件(cli の図は `layout_graph` の結果 `GraphLayout::Lanes`/`EdgeList` で分ける)。使うのは core `tests/graph_drawable.rs:22`(import)、`:27`(補助 `drawable`。呼ぶ所 27 か所)、`:210`、`:225`、`:238`、`:267`、`tests/graph_layout_oracle.rs:30`(import)、`:182`。
- 目標: 関数と再公開を消す。テストは公開の入口 `layout_graph` で確かめる(方針 (a)): 補助 `drawable` の本体を `matches!(layout_graph(&build_graph(..)), GraphLayout::Lanes(_))` に、`:210` などの `assert!(!is_lane_drawable(&graph))` は消す(直後の行に、同じことを確かめる `assert_eq!(layout_graph(&graph), GraphLayout::EdgeList)` がすでにある)。`graph_layout_oracle.rs:182` は `assert!(matches!(layout_graph(&graph), GraphLayout::Lanes(_)), …)`。
- 仕様書: rust `02-architecture.md:64` の関数の列から消す、`05-execution-flow.md:773` の見出し「(`is_lane_drawable`、`skip_targets`)」を「(`skip_targets`。`layout_graph` が使う)」に、`12-rebuild-guide.md:116` の列から消す。language-agnostic には名前の記載が無い。
- 完了の判定: `grep -rn "is_lane_drawable" crates docs/spec/rust docs/spec/language-agnostic` が 0 件。

### 1d. core の `LoadedEnvFiles::add`

- 今: `crates/core/src/envfile/mod.rs:289-293`(説明は「偽の実装やテストの便宜」)。本番(runtime `envfile.rs`)は `push_file` と `link` を使う。使うのはテストだけ(13 か所): core `tests/plan_types.rs:62`、`tests/env_merge_perf.rs:115`、`:138`、`:139`、`tests/plan_perf.rs:44`、`tests/env_merge.rs:78`、`:79`、`:80`、`:108`、`:109`、`:110`、`:172`、cli `tests/plan_report.rs:417`。
- 目標: メソッドを消す。テストの補助に同じ処理を書く(方針 (c)): core の `tests/common/mod.rs` に `pub fn add_env_file(files: &mut LoadedEnvFiles, reference: &str, content: EnvFileContent)`(`push_file` してから `link`)を足し、core の 12 か所はそれを呼ぶ(`plan_perf.rs` は `mod common;` を足す。`common` は `#![allow(dead_code)]` 済み)。cli の 1 か所は、その場で `let i = files.push_file(..); files.link(reference, i);` と書く(cli の `common` に 1 か所のための補助を置かない)。
- 仕様書: 名前の記載は無い(`grep -rn "add(" docs/spec` で確かめた)。
- 完了の判定: `grep -rn "\.add(" crates` が 0 件、`grep -n "pub fn add\b" crates/core/src/envfile/mod.rs` が 0 件。

### 1e. runtime の `remove_stale_temps` の引数 `now`

- 今: `crates/runtime/src/state/writer.rs:132`(`fn remove_stale_temps(dir, state_name, now: SystemTime)`)。呼ぶ所は本番 `:88` も単体テスト `:324`、`:333` も `SystemTime::now()`。
- 目標: `now` の引数を外し、関数の中で `SystemTime::now()` を取る。単体テスト `古い一時ファイルだけを消す` は、ファイルの更新時刻を「今 − 48 時間」にしてから呼ぶので、そのまま確かめられる(方針 (d) のまま。テストは同じモジュールの中)。
- 完了の判定: `grep -n "remove_stale_temps" crates/runtime/src/state/writer.rs` の 3 か所に `SystemTime::now()` の引数が無い。

### 1f. core と converter の誤りの種類の `code()`

- 今: core `crates/core/src/error.rs:134-170`(`DefinitionErrorKind::code`。説明は「機械が読む用。entities.md の `kind` の名前」)、`crates/core/src/envfile/mod.rs:117-133`(`EnvFileErrorKind::code`)、converter `crates/converter/src/error.rs:70-82`(`ConversionErrorKind::code`。`#[doc(hidden)]`、「契約 C4 の外(テスト用)」)。本番の使用は 0 件(`grep -rn "\.code()" crates/*/src` は `runner.rs:129` の `status.code()` と単体テストだけ)。
- テストの使用: core `tests/env_types.rs:57`、`:62-64`(識別子が重ならない、3 つの識別子がある)、`:70-71`、core `src/error.rs:432`(単体テスト `全種類に識別子と表示があり重複しない`)、converter `tests/nfr_security.rs:313-319`(作った誤りの種類を識別子の集合で数える)、converter `src/error.rs:269`(単体テスト `表示は_どの種類でも入力の値を含まない` の `!kind.code().is_empty()`)。
- 目標: 3 つの `code()` を消す。テストは種類そのもの(`HashSet<Kind>`。3 つの種類とも `Hash` を derive 済み)で数える(方針 (a))。識別子が重ならないことの確かめは、関数と一緒に消える(テストのための関数だけを試していた)。
- entities.md の `kind` の名前との対応(調査との違い: 対応の規則がある): 33 種・13 種・6 種のすべてで、仕様の識別子は変種の名前の先頭を小文字にしたもの(`FileMissing` → `fileMissing`、`NotUtf8` → `notUtf8`、`RootNotMap` → `rootNotMap`)。`code()` の表の代わりに、3 つの型の説明に「仕様書の識別子(`kind`)は、変種の名前の先頭を小文字にしたもの」と 1 文書く。
- 仕様書: rust `02-architecture.md:96`、`:99`(`code()` の記載)、`03-definition-format.md:393`(「識別子は `EnvFileErrorKind::code()`」)、`08-output-and-i18n.md:130`(表の見出し「識別子(`code()`)」→「識別子」)、`10-errors-and-exit-codes.md:120`、`12-rebuild-guide.md:24`(「`code()` で一意」→「変種と 1 対 1」)。converter `src/lib.rs:41-42` の `#[doc(hidden)]` の列から `code` と `ALL` を消す(`testing` は範囲の外で残す)。
- 完了の判定: `grep -rn "fn code(" crates/*/src` が `crates/core/src/flow/classify.rs` の単体テストの補助 `code(c: i32)` だけ、`grep -rn "\.code()" crates | grep -v "status.code()\|output.status\|\.status\.code()"` が 0 件、`grep -rn "code()" docs/spec/rust docs/spec/language-agnostic | grep -v "status.code()"` が 0 件。

### 1g. 誤りの種類の一覧 `ALL`(5 つ。`CommandKind::ALL` を除く。Q5: A)

| 一覧 | 今の場所 | 本番の使用 | テストの使用 |
|------|----------|------------|--------------|
| `DefinitionErrorKind::ALL`(33) | `crates/core/src/error.rs:173-208` | 0 件 | core `tests/env_types.rs:56`、`:57`、`:59`、`:287`、`tests/nfr_security.rs:229`、`src/error.rs:432`、`:436`(単体)、cli `tests/report_messages.rs:79`、`:1043` |
| `EnvFileErrorKind::ALL`(13) | `crates/core/src/envfile/mod.rs:136-150` | 0 件 | core `tests/env_types.rs:69`、`:70`、`:72`、`:74`、`:274`、`:281`、cli `tests/report_messages.rs:92`、`:1053` |
| `ConversionErrorKind::ALL`(6) | `crates/converter/src/error.rs:84-93`(`#[doc(hidden)]`) | 0 件 | converter `tests/nfr_security.rs:314`、`src/error.rs:264`(単体)、cli `tests/report_messages.rs:193`、`:1067` |
| `CreateError::ALL`(5) | `crates/runtime/src/file_create.rs:52-61` | 0 件 | cli `tests/report_messages.rs:272` |
| `UsageErrorKind::ALL`(8) | `crates/cli/src/types.rs:271-281` | 0 件 | cli `tests/args_grammar.rs:484`、`tests/report_messages.rs:134` |

- 範囲の外: `CommandKind::ALL`(`crates/cli/src/types.rs:76`。本番の `from_name` が使う)。
- 目標: 5 つの `ALL` を本番から消し、テストの側に写しを置く。写しには網羅の `match` を付け、種類が増えたらコンパイルで止まるようにする(D1・D2。決定(ユーザー、2026-11-23))。
  - 置き場所: core `crates/core/tests/common/kinds.rs`(`DEFINITION_ERROR_KINDS`、`ENV_FILE_ERROR_KINDS`)、converter `crates/converter/tests/common/kinds.rs`(`CONVERSION_ERROR_KINDS`)、cli `crates/cli/tests/common/kinds.rs`(5 つ全部)。各 `common/mod.rs` に `pub mod kinds;`。`tests/common/` の下のファイルは、それ自体はテストのクレートにならない。
  - 形(D1。決定(ユーザー、2026-11-23): 推奨のとおり): `pub const DEFINITION_ERROR_KINDS: [DefinitionErrorKind; 33] = [...]` と、変種ごとに一覧の位置を返す `const fn position(kind) -> usize`(網羅の `match`。`_` を使わない)と、`const _: () = { 位置 i の要素の position が i であることを、i = 0..33 で assert! }`。種類を足すと `match` がコンパイルで止まり、一覧の並びの誤りや重複もコンパイルで止まる(`const` の `assert!` と `while` は Rust 1.57 以降。MSRV 1.100 で使える)。
  - core のテストの `nfr_security.rs` は `mod common;` を足す。
- 単体テストの扱い: core `src/error.rs` の `全種類に識別子と表示があり重複しない` は消す(§4.6。統合テスト `env_types.rs` と同じことを確かめていた)。converter `src/error.rs` の `表示は_どの種類でも入力の値を含まない` は消す(D3。消す方針は決定(ユーザー、2026-11-23)。個々の削除は §4.6 として設計の承認で認めてもらう(承認待ち))。
- `args_grammar.rs` の `使い方の誤りの全種類が網羅されている`(`UsageErrorKind::ALL.len() == 8` だけ)は消す(§4.6。一覧の長さは型 `[_; 8]` とコンパイル時の確かめが保つ。D10。消す方針は決定(ユーザー、2026-11-23)。個々の削除は設計の承認で認めてもらう(承認待ち))。
- 仕様書: rust `02-architecture.md:96`、`:99` の `ALL: [...]` の記載を消す。
- 完了の判定: `grep -rn "::ALL\b" crates | grep -v "CommandKind::ALL\|Self::ALL"` が 0 件、`grep -rn "pub const ALL" crates/*/src` が `crates/cli/src/types.rs` の `CommandKind` の 1 件だけ。

### 1h. cli のパニックのフックの差し替え口(Q3: A)

- 今: `crates/cli/src/panic_hook.rs:30-49`。`pub fn on_panic(err, exit: &mut dyn FnMut(i32))`、`pub fn install_panic_hook_with(exit: fn(i32))`(差し替えはテストのため。仕様書 rust 10:112 にも「テストのため」とある)、`pub fn install_panic_hook()`(本番。`main.rs:24`)。`lib.rs:111` で 3 つを再公開。テストは `crates/cli/tests/panic_hook.rs` の 3 件。
- 目標: 差し替え口をなくす。
  - 本体は非公開の `fn write_internal_error(err: &mut dyn Write) -> i32`(固定の英語の文を書き、`flush` し、書き込みの失敗は無視して `EXIT_INTERNAL`(70)を返す)。
  - `pub fn install_panic_hook()` は `std::panic::set_hook(Box::new(|_| std::process::exit(write_internal_error(&mut std::io::stderr()))))`。
  - `on_panic` と `install_panic_hook_with` を消し、`lib.rs` の再公開は `install_panic_hook` だけ。モジュール `panic_hook` は公開のまま。
  - テスト(方針 (d)): `panic_hook.rs` の `#[cfg(test)] mod tests` に、`フックの本体は固定の英語の文を書き_70_で終わる` と `標準エラーに書けなくても_70_で終わる` を、名前と確かめる中身を変えずに移す(`on_panic(&mut err, &mut |c| code = Some(c))` を `let code = write_internal_error(&mut err)` に)。`crates/cli/tests/panic_hook.rs` は消す。
  - 「設定したフックは_パニックのとき終了の関数を_70_で呼ぶ」は消し、**確かめられなくなる**(§4.6): `install_panic_hook` が `set_hook` でフックを設定し、パニックのとき 70 で `process::exit` を呼ぶこと。リリースのバイナリにパニックを起こす経路を持たせない方針(cli `nfr_source.rs` の `テスト専用の経路を実行ファイルに持たない`)のため、実バイナリでも確かめない。残る確かめ: 本体の 2 件(単体)と、`main.rs` が最初に `install_panic_hook()` を呼ぶこと(読んで確かめる。コードは 3 行)。
  - cli の src に置く単体テストは、cli `tests/nfr_source.rs` の検査(`panic!(` を src に書かない、`unsafe`・`low_level`・`cfg(unix)` などを書かない)に当たらない書き方にする(2 件とも `panic!` を使わない)。cli の src の単体テストは、これが初めて(今は 0 件)。
- 仕様書: rust `02-architecture.md:142`(再公開の列から `install_panic_hook_with`・`on_panic` を消す)、`10-errors-and-exit-codes.md:112`(差し替えの文を「本体は非公開の関数で、単体テストで確かめる。フックの設定そのもの(パニックで 70 で終わること)は自動のテストでは確かめない」に)、`11-quality-and-build.md:148`(テストの場所を `cli/src/panic_hook.rs` の単体テストに)、language-agnostic `11-quality-and-build.md:800`(AC-368 のテストの名前から 3 つ目を消し、場所を単体テストに)。`lib.rs:56` のモジュールの表の「終了の呼び出しは差し替えられる」を消す、`panic_hook.rs:17` のモジュールの説明の同じ文を消す。
- 完了の判定: `grep -rn "install_panic_hook_with\|on_panic" crates docs/spec/rust docs/spec/language-agnostic` が 0 件、`ls crates/cli/tests/panic_hook.rs` が無い。

### 1i. cli のシグナルの `SignalError::AlreadyRegistered` と登録済みの印(Q4: A)

- 今: `crates/cli/src/platform/mod.rs:33-46`(`static REGISTERED: AtomicBool`、`start_signal_watcher` が 2 回目に `AlreadyRegistered` を返す)、`crates/cli/src/deps.rs:58-63`(`SignalError { AlreadyRegistered, Failed }`)、`crates/cli/src/ctx.rs:199`(`Ok(()) | Err(SignalError::AlreadyRegistered) => Ok(signal)`)、`crates/cli/src/platform/windows.rs:29`(`ctrlc::Error::MultipleHandlers` を `AlreadyRegistered` に)。テスト: `crates/cli/tests/signals_real.rs:44-66`(2 回目の `start` が `AlreadyRegistered` で、配線されない)、`tests/flow_run.rs:696-701`(`すでに登録済みでも_流れは成功と同様に続ける`)、`tests/common/mod.rs:136`(`SignalMode::Already`)、`:162`。
- 本番で 2 回登録しない根拠: `Signals::start` を呼ぶのは `Ctx::start_signals`(`ctx.rs:196`)だけで、それを呼ぶのは `Ctx::prepare_session`(`ctx.rs:136-140`)だけ。`prepare_session` を呼ぶのは `run.rs:41`(`run`)と `pipeline.rs:43`(`pipeline`)の各 1 か所で、1 回の `run_cli` は 1 つのコマンドしか実行しない。`main.rs:26` は `run_cli` を 1 回だけ呼ぶ。テストのうち本番の `SystemSignals` を使うのは `signals_real.rs` と `signals_order.rs` だけ(`run_cli` を同じプロセスで呼ぶテストは無い。`grep -rn "run_cli(\|SystemSignals" crates/cli/tests` で確かめた)。
- 目標:
  - `REGISTERED` と 2 回目の判定を消し、`start_signal_watcher` は OS ごとの `spawn_watcher` を呼ぶだけにする。
  - `SignalError` は `Failed` だけの列挙型にする(D5。決定(ユーザー、2026-11-23))。Windows の `ctrlc::set_handler` の失敗は、種類によらず `Failed`。
  - `ctx.rs` は `Ok(()) => Ok(signal)`、`Err(SignalError::Failed) => …70`。
  - `Signals::start` の説明に「プロセスで 1 回だけ呼ぶ(本番は `prepare_session` の 1 か所)。2 回呼んだときの動きは契約の外(UNIX 系は監視が 2 つになり、Windows は `Failed`)」と書く。
  - 公開の継ぎ目 `run_cli_with`(`crates/cli/src/lib.rs:139`)と `run_cli`(`:124`)の説明に、「シグナルの監視の登録はプロセスで 1 回だけ。本番の `SystemSignals` を渡して `run`・`pipeline` をプロセスで 2 回実行すると、2 回目の登録は契約の外(Windows は登録に失敗して 70 で終わる。UNIX 系は監視が 2 つになる)」と書く(確認担当の指摘)。仕様書 rust `02-architecture.md` の公開の関数の表と `07-process-and-signals.md:294` にも同じ文。
  - テスト(Q4: A。1 つのテストで 1 回だけ登録する): `signals_real.rs` の 2 回目の登録の部分(`:57-65`)を消し、名前を `最初のシグナルの番号だけが残る` に変える(★。§4.4)。登録は 1 回で、もともと 1 つのテストのファイルに 1 つのテスト(別のプロセス)なので、ファイルを分ける必要は無い。`signals_order.rs` は今も 1 回だけ登録する(変えない)。`flow_run.rs` の `すでに登録済みでも_流れは成功と同様に続ける` を消す(§4.6。場面そのものが無くなる)。`common/mod.rs` の `SignalMode::Already` を消す。
- 仕様書: rust `02-architecture.md:258`(`AlreadyRegistered` / `Failed` → `Failed`)、`07-process-and-signals.md:294`(「2 回目の呼び出しは…」→「呼ぶのはプロセスで 1 回(`prepare_session` の 1 か所)」)、`:336`(Windows の写し)、`:434`(テストの表)、`10-errors-and-exit-codes.md:99`、`:220`(`SignalError`(1 種))、`11-quality-and-build.md:268`、`12-rebuild-guide.md:199`。`cli/src/lib.rs:40` の表、`deps.rs:59` の説明。language-agnostic には記載が無い。
- 完了の判定: `grep -rn "AlreadyRegistered\|REGISTERED\|SignalMode::Already" crates docs/spec/rust docs/spec/language-agnostic` が 0 件。

### 1j. core の `graph::node_state_of` と `flow_edges` の公開

- 今: `crates/core/src/graph/mod.rs:34`(`pub use edges::flow_edges`)、`:36`(`pub use overlay::{.., node_state_of, ..}`)。本番の使用は core の中だけ(`overlay.rs` の `overlay_from_state` が `node_state_of`、`build_graph` が `flow_edges`)。cli は使わない。テストの使用: core `tests/graph_types.rs:92-97`(`node_state_of` の 5 つの値)、`tests/graph_edges.rs:19`(import)、`:50`、`:63`、`:78`、`:93`、`:107`、`:122`、`:134`、`:145`、`:160`(9 か所)、`tests/graph_lookup_perf.rs:27`(import)、`:140`、`:165`。
- 目標: `flow_edges` は `pub(crate)`(`graph/mod.rs` は `pub(crate) use edges::flow_edges;`)。`node_state_of` は `overlay.rs` の中だけの非公開の関数(再公開を消す。`pub(crate) use` にすると使われない import の警告になるため)。テストは公開の入口から確かめる(方針 (a)):
  - `graph_edges.rs`: ファイルの中の補助 `fn edges(def: &PipelineDefinition) -> Vec<FlowEdge> { build_graph(def, None).edges }` を置き、9 か所の `flow_edges(` を `edges(` に。
  - `graph_lookup_perf.rs` の P4: `flow_edges(&def)` を `build_graph(&def, None).edges` に(2 か所)。測る処理に、節(ジョブ 1,000 個と終点)を作る時間が加わる。今の P4 の中央値は 117.5 ms(上限 600 ms。この intent の基準の木で測った)で、節を作る部分は重ね合わせなしで文字列の複製 1,001 個なので、上限との余裕は保たれる見込み。実装で前後の時間を記録する。テストの名前と上限は変えない。
  - `graph_types.rs` の `ジョブの状態は成功と警告を_ok_にまとめる`: 公開の `overlay_from_state` で確かめる(D8。決定(ユーザー、2026-11-23))。core の `common::state(&[("a", Success), ("b", Warning), ("c", Skipped), ("d", Error), ("e", Pending)], None, false)` の重ね合わせの `states` が `[Ok, Ok, Skipped, Error, Pending]`(`mod common;` を足す)。名前と確かめる 5 つの対応は変えない。
- 仕様書: rust `02-architecture.md:64`(`graph::flow_edges`・`node_state_of` は crate の中、と書く)、`05-execution-flow.md:748`・`:768` は関数の説明として残し「crate の中」と添える、`11-quality-and-build.md:120`(P4 の測る処理を「`build_graph`(重ね合わせなし)の辺」に)。language-agnostic `05`・`12` は論理の手順の名前なので変えない。
- 完了の判定: `grep -rn "flow_edges\|node_state_of" crates | grep -v "^crates/core/src/graph/"` が 0 件、`grep -n "pub use edges::flow_edges\|node_state_of" crates/core/src/graph/mod.rs` が 0 件。

---

## 段 2(ふつうの整理)

### 2a. 古くなった説明

| # | 今の場所 | 今の文(要旨) | 目標 |
|---|----------|----------------|------|
| 2a-1 | `crates/cli/src/report/mod.rs:101` | `EnvWarnings` は「`validate` だけが出す」 | 「`validate` と `graph` が出す(`check_definition_and_env`)」(コメントの末尾の既存の規則の番号はそのまま) |
| 2a-2 | 仕様書 rust `08-output-and-i18n.md:351`、`03-definition-format.md:413`、`10-errors-and-exit-codes.md:123`、language-agnostic `08-output-and-i18n.md:369`、`03-definition-format.md:579`、`10-errors-and-exit-codes.md:147`(調査との違い: 調査は rust 08 の 1 か所。同じ文が 6 か所) | 「`validate` だけが出す(表示する)」 | 「`validate` と `graph` が出す」(04 章の 7.5 と `graph` の節(rust 04:195・:207、language-agnostic 04:233・:245)と一致させる) |
| 2a-3 | `crates/cli/src/aux_commands.rs:79-81` | `run_validate` は「定義の検証と、全ジョブの `command` の検査だけを行う」 | 「定義の検証、全ジョブの `command` の検査、`pipeline` と同じ範囲の `.env` の読み取りと検査を行う(重複キーは警告で、誤りではない)」 |
| 2a-4 | `crates/runtime/src/lib.rs:56-70`(モジュールの表) | `bounded_read` の行が無い。`state::reader` は「上限 + 1 バイトの読み取り…」、`state::writer` は「一時ファイルの新規作成、同期、置き換え、後始末」、`temp_file` は「名前と新規作成、連番、親のディレクトリ」 | `bounded_read` の行(「上限つきの読み取り(BoundedRead) \| 通常のファイルだけを開き、上限 + 1 バイトまで読む(形 B)。`.env` とステートが共有する」)を足す。`state::reader` は「読み取りは `bounded_read`。長さ 0 は空、版の確認(先)、走査、矛盾の検査」、`state::writer` は「内容の組み立てと同期。置き換えは `temp_file::replace_via_temp`。古い一時ファイルの後始末」、`temp_file` は「一時ファイルの名前と新規作成、連番、親のディレクトリの扱い、一時ファイルに書いて置き換える処理(`replace_via_temp`)」 |
| 2a-5 | `crates/cli/src/lib.rs:75` | 「排他制御はしない。自動の再試行もしない」(落ちたときの文脈) | 「排他制御はしない。落ちたあとの自動の再実行もしない(`[job.retry]` のリトライは、1 回の実行の中で、ジョブの終了コードによって行う)」 |
| 2a-6 | `crates/cli/src/pipeline.rs:20` | 「エラーで停止し、再試行しない」 | 「エラー(`[job.retry]` のリトライを使い切った後の結果)で停止する」 |
| 2a-7 | `crates/cli/src/pipeline.rs:102` | 「エラーで停止する(再試行しない)」 | 「エラーで停止する(リトライは `run_job_with_retry` で済んでいる)」 |
| 2a-8 | `crates/core/src/flow/next_step.rs:29` | 「自動の再試行はしない」 | 「この関数はリトライを決めない(リトライは、この関数より前に `retry_decision` が決める)」 |
| 2a-9 | `crates/runtime/src/lib.rs:110` | 「時間の上限、自動の再試行、…(いずれも、この版では提供しない)」 | 「自動の再試行」を「自動の再試行(リトライは cli が `[job.retry]` に従い、[`wait_for`] で待って [`run_job`] を呼び直す)」に |
| 2a-10 | `crates/converter/src/lib.rs:73-82`(モジュールの表) | `raw` と `scalar` の行が無い。`intake` は「ファイルを開き、上限 + 1 バイトまで読む」 | `raw`(「素の木(Raw) \| YAML と TOML の読み取りの結果の、展開前の木(アリーナ。再帰しない)」)、`scalar`(「スカラーの解決(Scalar) \| YAML 1.2 のコアスキーマで、スカラーの型を決める」)を足す。`intake` は「パスで種別を見て、通常のファイルだけを開き、上限 + 1 バイトまで読む(形 B)。UTF-8 の確認」 |
| 2a-11 | `crates/core/src/envfile/mod.rs:366` | `env_file_refs` は「`Some` ならそのジョブの分だけ(`run`)」 | 「`Some` なら `[pipeline].env_file` とそのジョブの分(`run`)」(コードは `std::iter::once(&def.env_file).chain(jobs)`) |
| 2a-12 | `crates/cli/src/report/mod.rs:301-302`(`Reporter::show`) | 「`error` は常に、`verbose` は `-v` のときだけ標準エラーへ」(`Notice` が無い) | 「`error` と `notice` は常に、`verbose` は `-v` のときだけ標準エラーへ」 |

- 確かめた上で変えないもの: `crates/runtime/src/runner.rs:96` の「再試行しない」(`run_job` 1 回の範囲の文で正しい)。
- 完了の判定: `grep -rn "validate\` だけ\|validate だけ" crates/*/src docs/spec/rust docs/spec/language-agnostic` が 0 件、`grep -rn "自動の再試行はしない\|自動の再試行もしない\|再試行しない。" crates/cli/src crates/core/src` が 0 件、`grep -n "bounded_read" crates/runtime/src/lib.rs` で表の行がある、`grep -n "\`raw\`\|\`scalar\`" crates/converter/src/lib.rs` で表の行がある。

### 2b. ステートの `PipelineState.definition_key` と `job_order`

- 今: `crates/core/src/types.rs:365`(`definition_key`。製品のコードで読まれない。書き出し `writer.rs:184` は `definition_path` から書く)、`:369`(`job_order`)。`job_order` を読む製品のコードは `crates/core/src/consistency.rs:51`(照合)と `writer.rs:196-200`(書き出し)。読み込み `reader.rs:243-283` は、`job_order` と `[[job]]` の並びが数も順序も一致しなければ `corrupt` にするので、読んだステートでは常に `job_states` の ID の並びと同じ。作る所 `initial_state`(`flow/record.rs:43-47`)は、両方とも定義の記述順。`apply_*` は複製で変えない。
- 手で組む所(`grep -rn "PipelineState {" crates`。15 か所): core `src/flow/record.rs:43`(本番の `initial_state`)、`src/consistency.rs:79`(単体)、`tests/common/mod.rs:70`、runtime `src/state/reader.rs:286`(本番)、`src/state/writer.rs:236`(単体)、`tests/state_write.rs:30`、`tests/run_id_state.rs:33`、cli `tests/report_messages.rs:49`、`tests/graph_report.rs:297`、`tests/nfr_robust.rs:166`、`tests/flow_pipeline.rs:441`、`tests/plan_report.rs:86`。テストのすべてで `job_order` は `job_states` の ID の並びと同じ(確かめた)ので、照合の結果は変わらない。
- 項目を読むテスト: core `src/flow/record.rs:294`(`s.job_order == ["a", "b"]`)、`:300-301`(`definition_key`)、runtime `src/state/reader.rs:376`(`state.job_order == ["a", "b"]`)。
- 目標:
  - 型から `definition_key` と `job_order` を消す。
  - 書き出し(`writer.rs::serialize`): `definition_key` は今のまま `definition_path` から、`job_order` は `job_states` の ID の並びから書く(ファイルの形は 1 バイトも変わらない)。
  - 読み込み(`reader.rs::walk`): `definition_key` は必須のキーとして存在と型(文字列)を確かめるだけで持たない。`job_order` は読んで、`[[job]]` の並びとの一致と `resume_job_id` が含まれることの検査に使い、型には持たない(検査の順序と誤りの理由は今のまま)。
  - 照合(`consistency.rs`): 記録の並びを `state.job_states` の ID から取る。
  - 単体テスト: `record.rs:294` は `job_states` の ID の並びを確かめる、`:300-301` は消す(`definition_key` と `definition_path` が同じ文字列で書かれることは、runtime `state_write.rs` の `ステートファイルの内容は版つきで_e3_のキーを持つ` と単体テスト `直列化に定義のパスが_2_つのキーで入る` が確かめる)。`reader.rs:376` は `job_states` の ID の並び。
  - 手で組む所 13 か所(本番の 2 か所を除く)から、2 つの項目の行を消す。
- 仕様書: rust `02-architecture.md:83`(型の項目)、`06-state-file.md:39`(「読むときは別々に保持するが」→「読むときは存在と型を確かめるだけで、型には持たない」)、`:299`(擬似コード `recorded = state.job_order` → `recorded = state.job_states の ID の並び`(読み込みで一致を確かめ済み))、`12-rebuild-guide.md:187`。language-agnostic `06-state-file.md:41` の「読むときは別々に保持するが」を「読んだ値を保持するかは実装による」に(D12。決定(ユーザー、2026-11-23))。ファイルの形(キー、検査、誤りの理由)の記述は変えない。
- 完了の判定: `grep -rn "\.definition_key\|definition_key:\|\.job_order\|job_order:" crates` が 0 件(TOML の文字列の中のキーを除く)、`grep -rn "\"definition_key\"\|\"job_order\"" crates/runtime/src` が書き出しと読み込みの 4 か所のまま。

### 2c. runtime の `CommandError.reason`

- 今: `crates/runtime/src/error.rs:33-48`。`reason: &'static str` は `kind` から決まる写し(`CommandError::new` が `kind` から引く)。読むのは `#[error("{}", .reason)]` だけ(cli は種類の文だけを出す。`report/mod.rs:572-576`)。組み立てる所: runtime `src/split.rs:115`、`:128`、`:150`(`CommandError::new`)、runtime `src/error.rs:150`、`:152`(単体テスト `コマンドエラーの表示は固定の文` の `CommandError::new`。functional-spec の T31)、cli `tests/report_messages.rs:116-119`、`:123-126`、`:469-472`(テストの組み立て 3 か所。`reason: "reason"` などの、本物と違う値)。
- 目標: 項目を消し、`CommandErrorKind::reason(self) -> &'static str`(`EnvFileErrorKind::reason` と同じ形)にする。`Display` は `#[error("{}", .kind.reason())]`(`StateError` の `.kind.label()` と同じ書き方)。`CommandError::new` は `CommandError { kind }` と同じになるので消し、`split.rs` は `CommandError { kind }` で作る。`CommandError` 自身の `reason()` は作らない(読む所が無い。D6。決定(ユーザー、2026-11-23))。cli のテストの組み立て 3 か所から `reason: …` の行を消す。
- 仕様書: rust `07-process-and-signals.md:168`(`CommandError { kind, reason }` → `CommandError { kind }`、理由は `CommandErrorKind::reason()`)、`10-errors-and-exit-codes.md:137`(`Display` は `kind.reason()`)。
- 完了の判定: `grep -rn "CommandError {" crates` の全部が `kind` だけ、`grep -n "pub reason" crates/runtime/src/error.rs` が `StateError` の 1 件だけ。

### 2d. core の `ExitCodeClass`

- 今: `crates/core/src/flow/classify.rs:91-121`。非公開の `ExitCodeClass { classification, reported_exit_code, conductor_error }` を `classify_exit_code` が返し、`classify`(`:82-87`)が `JobOutcome` に詰め直すだけ(前の intent で非公開にした。呼ぶ所は `classify` の 1 か所)。
- 目標: `classify_exit_code(job, code) -> JobOutcome`(`classification` は `Some(..)`)。`classify` は `return classify_exit_code(job, code)` 相当。`ExitCodeClass` を消す。
- 完了の判定: `grep -rn "ExitCodeClass" crates docs/spec` が 0 件。

### 2e. 小さい整理

| # | 今の場所 | 今 | 目標 |
|---|----------|----|------|
| 2e-1 | `crates/core/src/flow/classify.rs:20`、`:23`、`:26` | `pub(crate) const EXIT_START_FAILURE`・`EXIT_SIGNAL_BASE`・`EXIT_UNLISTED_ZERO`(使うのは `classify.rs` の中だけ。`grep -rn` で確かめた) | 非公開の `const`。仕様書 rust `02-architecture.md:42`・`10-errors-and-exit-codes.md:19` の「crate の中」を「`flow/classify.rs` の中」に |
| 2e-2 | `crates/cli/src/dry_run.rs:53` | `pub(crate) fn run_dry_run`(呼ぶのは同じファイルの `enter_flow:47` だけ) | 非公開の `fn` |
| 2e-3 | `crates/runtime/src/state/reader.rs:110` | `Ok(None) if bytes.is_empty() => Inspected::NotOurs`(`parse_state_raw_for` は空のときだけ `Ok(None)` を返す(`:161-163`)ので、条件は常に真) | `Ok(None) => Inspected::NotOurs`、`Ok(Some(_)) => Inspected::Ours`(説明「空のファイルは…」は残す) |
| 2e-4 | `crates/runtime/src/state/writer.rs:25-29` | `#[cfg(test)] use conductor_core::JobStatus;`、`#[cfg(test)] use std::fs::OpenOptions;`(説明「単体テストが使う」) | `mod tests` の中の `use` に移す |
| 2e-5 | `crates/converter/src/lib.rs:126` | `intake::read_input(Path::new(&request.input_path))`(`input_path` は `PathBuf`) | `intake::read_input(&request.input_path)`。使わなくなる `use std::path::Path;`(`:108`。`Path` を使うのは `:126` だけ)を消す |

- 完了の判定: `grep -n "pub(crate) const EXIT_" crates/core/src/flow/classify.rs` が 0 件、`grep -n "pub(crate) fn run_dry_run" crates/cli/src/dry_run.rs` が 0 件、`grep -n "if bytes.is_empty() =>" crates/runtime/src/state/reader.rs` が 0 件、`grep -n "^#\[cfg(test)\]" crates/runtime/src/state/writer.rs` が `mod tests` の 1 件、`grep -n "Path::new(&request" crates/converter/src/lib.rs` が 0 件。

### 2f. ステートが無いときの判断の重複

- 今: cli `crates/cli/src/step.rs:56-72`(`decide`: `Option<&PipelineState>` から `(job_states, resume_job_id, end_reached)` を取り出し、無ければ `(&[], None, false)` で `decide_before_run`。呼ぶ所 `run.rs:47`、`pipeline.rs:58`)と、core `crates/core/src/plan/simulate.rs:166-175`(`plan_run` の同じ `match`)。core の `plan_pipeline`(`simulate.rs:79-85`)は、ステートを必ず持つ(無ければ `initial_state`)ので、`Some` と同じ取り出し。
- 目標: core の `flow/decide.rs` に薄い関数 `pub fn decide_from_state(job_id: &str, definition: &PipelineDefinition, state: Option<&PipelineState>) -> Decision` を置き(名前と場所は D7。決定(ユーザー、2026-11-23))、`flow` と `lib.rs` から再公開する。cli の `step::decide` を消し、`run.rs`・`pipeline.rs` は core の関数を呼ぶ。`simulate.rs` の `plan_run` は `decide_from_state(job_id, def, active_state(input))`、`plan_pipeline` は `decide_from_state(&job.id, def, Some(&state))`。
- テスト: 単体テスト N1(`decide.rs`)を足す(D11。決定(ユーザー、2026-11-23))。既存の cli の流れのテストと計画のテストがそのまま確かめる。
- 仕様書: rust `02-architecture.md:42`(再公開の列)、`:55`(関数の表に 1 行)、`:199`(`step::decide` → `core::decide_from_state`)、`05-execution-flow.md` の 4 章に 1 文(「ステートが無いときは、記録が空、再開位置なし、`end_reached` は偽として判断する」)。cli `step.rs:21` のモジュールの表から `decide` の行を消す。
- 完了の判定: `grep -rn "None => (&\[\]\[..\], None, false)" crates` が `decide.rs` の 1 件だけ、`grep -rn "fn decide(" crates/cli/src` が 0 件。

---

## 8. 使った検索のコマンド(テストの変更の一覧の網羅のため)

すべて HEAD c5f1a3a で、リポジトリのルートで実行した。

```sh
# 基準の件数
cargo test --locked --workspace > target/unbend-fd/baseline.txt 2>&1   # 集計後に target/unbend-fd を消した
# 1a
grep -rn "language_resolved\|color_resolved" crates docs/spec/rust docs/spec/language-agnostic README.md
grep -rn "var_calls\|terminal_calls\|locale_read\|\.names\.borrow\|env\.names\b" crates/cli/tests
# 1b
grep -rn "\.removed\|removed:" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "ResetAllReport" crates docs/spec/rust
grep -rn "remove_state_file" crates/runtime/src
# 1c
grep -rn "is_lane_drawable" crates docs/spec/rust docs/spec/language-agnostic
grep -c "drawable(" crates/core/tests/graph_drawable.rs
# 1d
grep -rn "\.add(" crates
grep -rn "LoadedEnvFiles::add\|fn add\b" crates docs/spec/rust docs/spec/language-agnostic
# 1e
grep -rn "remove_stale_temps" crates docs/spec
# 1f・1g
grep -rn "fn code\b\|fn code(\|pub const ALL\|const ALL" crates/*/src
grep -rn "\.code()\|::ALL\b" crates
grep -rn "doc(hidden)" crates/*/src
grep -rn "code()\|\`code\`\|::ALL\|\`ALL\`" docs/spec/rust docs/spec/language-agnostic
# 1h
grep -rn "install_panic_hook\|on_panic\|panic_hook" crates docs/spec/rust docs/spec/language-agnostic README.md
grep -rn "#\[cfg(test)\]" crates/cli/src
# 1i
grep -rn "AlreadyRegistered\|SignalError\|REGISTERED\|registered" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "start_signals\|prepare_session\|fn run_cli\b\|run_cli(" crates/cli/src
grep -rn "run_cli(\|SystemSignals\|SignalMode" crates/cli/tests
# 1j
grep -rn "node_state_of\|flow_edges" crates docs/spec/rust docs/spec/language-agnostic
cargo test -q -p conductor-core --locked --test graph_lookup_perf -- --nocapture --test-threads=1
# 補助のディレクトリの確認(tests/*/ の下に、消す名前の参照が無いこと)
grep -rn "flow_edges\|is_lane_drawable\|node_state_of\|\.add(\|code()\|::ALL\|language_resolved\|\.removed\|AlreadyRegistered\|on_panic\|install_panic_hook_with\|job_order\|definition_key" crates/*/tests/*/
# 2a
grep -rn "validate\` だけ\|validate だけ" docs/spec/rust docs/spec/language-agnostic crates/*/src README.md
grep -n "重複キー" docs/spec/language-agnostic/04-cli.md docs/spec/rust/04-cli.md README.md
grep -rn "再試行" crates/*/src
grep -rn "temp_file::\|bounded_read::" crates/runtime/src
# 2b
grep -rn "definition_key\|job_order" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "PipelineState {" crates
# 2c
grep -rn "CommandError" crates docs/spec/rust docs/spec/language-agnostic
grep -rn "\.reason\b" crates/cli/src crates/runtime/src
# 2d・2e
grep -rn "EXIT_START_FAILURE\|EXIT_SIGNAL_BASE\|EXIT_UNLISTED_ZERO\|ExitCodeClass\|classify_exit_code" crates docs/spec
grep -rn "run_dry_run" crates docs/spec/rust
grep -n "fn parse_state_raw_for" -A25 crates/runtime/src/state/reader.rs
grep -n "input_path" crates/converter/src/types.rs
# 2f
grep -rn "decide(\|decide_before_run" crates/cli/src crates/core/src
```
