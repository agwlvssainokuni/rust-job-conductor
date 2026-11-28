# 要件: v0.10.4 の大きなファイルの分割

出典:
- 依頼の文(`project-description.json`)
- 要件の質問(`requirements-analysis-questions.md` の Q1〜Q6。まとめは確かめ済み)
- コードの知識(`aidlc/spaces/default/codekb/rust-job-conductor/` の `architecture.md`、`code-structure.md`、`component-inventory.md`、`code-quality-assessment.md`)
- 引き渡し(`inception/reverse-engineering/developer-scan.md` の「この intent のための詳細」の 1〜6)
- チームの実践(`team.md`)と project.md

## 1. 意図の分析

- **目的**: 本番のコードで大きな 3 つのファイルを、責務ごとのファイルと関数に分け、読みやすくする。
  - `crates/cli/src/report/mod.rs`(746 行)
  - `crates/core/src/definition/walk/mod.rs`(695 行)
  - `crates/cli/src/args.rs`(679 行)
- **作業の種類**: 整理の intent。`conductor` の動きは変えない。テストは足さない(既存のテストで、変わらないことを確かめる)。版は 0.10.4 にする。
- **背景**: リファクタリングの候補の D(大きなファイル)のうち、人が 1〜3 を選んだ。開発者は Rust に不慣れなため、小さな単位でレビューできる形が要る(team.md)。
- **成功の基準**: 次の 4 つを満たす。
  - 3 つのファイルが分かれている。
  - 既存のテストがすべて通り、数も名前も変わらない(版の期待と、Q5 の一覧の直しを除く)。
  - CI が通る。
  - 公開の範囲と、誤りの表示が変わらない。

## 2. 機能の要件

### FR1. `report/mod.rs` を分ける

- **FR1.1** `report/mod.rs` から、次の 3 つを子のファイルに出す(引き渡しの 1.1)。
  - 表示の出来事の定義: `DisplayEvent` と `level`、`StateContext`、`ResultDetail`、`FramePrefix`、`CLEAR_SCREEN`
  - 1 行の組み立て: `show_path`、`error_line`、`place_of`、`definition_line`、`env_file_lines`、`env_warning_line`、`job_place`、`command_line`、`usage_text`、`state_line`、`conversion_line`、`job_result_line`
  - エスケープ: `MAX_DISPLAY_CHARS`、`escape_name`、`escape_full`、`push_line`

  `mod.rs` には、`Reporter`、`render`、`detect_language`、子モジュールの宣言、`pub use` を残す。
- **FR1.2** `Reporter::render` の `match` の 35 の枝を、表示の形の 9 種ごとの関数に分ける(Q1 A。9 種は引き渡しの 1.3 の A〜I)。`render` は振り分けだけにする。
- **FR1.3** 言語と色の判定は、今と同じく、必要な枝で初めて行う。
  - 言語を判定しないもの: 固定の文と `Graph | Plan | InitTemplate`
  - 色を判定するもの: `WatchTable` だけ
  - `WatchFrame` は `Reporter` を通して描き直す
- **FR1.4** 今の名前は、`pub use` で今の道筋(`crate::report::…`)のまま引けるようにする(引き渡しの 1.4)。
  - `crate::report::{DisplayEvent, Reporter, StateContext, ResultDetail, FramePrefix, escape_name, escape_full}`
  - 子モジュールが `super::` で引く `escape_full`・`escape_name`・`push_line`
  - `messages.rs` の `crate::report::StateContext`

  `report/messages.rs` は動かさない(`spec_numbers.rs` が `pub const HEADER_` を読む)。

### FR2. `walk/mod.rs` から外に出す

- **FR2.1** `walk/mod.rs` から、次の 3 つを `env.rs`・`retry.rs` と同じ形(子の `impl Walker<'_>` と `pub(super)`。前例は c34d3f7)で、子のファイルに出す(引き渡しの 2.1)。
  - 値の読み取り: `string`、`boolean`、`exit_codes`、`command`
  - 遷移: `transitions`、`transition_on`、`RawTransition`
  - ジョブをまたぐ検査: `cross_check`、`check_exit_codes`

  `log_dir` と自由関数 `exit_code_of`・`integer_of` の置き場所は、設計で決める。
- **FR2.2** キーの表(`ROOT_KEYS`、`PIPELINE_KEYS`、`JOB_KEYS`、`TRANSITION_KEYS`)は `walk/mod.rs` に残す(Q2 A)。`spec_numbers.rs` は変えない。`walk/mod.rs` の中で、`const PIPELINE_KEYS`・`const JOB_KEYS` を含む行を、今の定義より前に置かない。
- **FR2.3** `env.rs`・`retry.rs` が親から使う名前(引き渡しの 2.2)は、今と同じく `super::` で引けるようにする。

### FR3. `args.rs` を分ける

- **FR3.1** `args.rs` を `args/mod.rs` にし(Q6 A)、次の 3 つを子のファイルに出す(引き渡しの 3.1)。
  - オプションの表と検査: `Opt`、`Allowed`、`OptSpec`、`OPTIONS`、コンパイル時の検査、`spec`・`spec_by_long`、`Check`・`CHECKS`、`Given`、`Operand`・`operand_rule`
  - 字句: `Token`、`Special`、`map_unexpected_value`、`to_text`
  - 値の解釈: `parse_assumptions`、`parse_graph_format`、`parse_interval`、`parse_run_id_option`、`parse_target`

  `parse_args` と `build_command` の置き場所は、設計で決める。
- **FR3.2** `build_command` の `match kind` の 5 つの枝(引き渡しの 3.2 の 9)を、コマンドの枝ごとの関数に分ける。
- **FR3.3** 外の入口は `parse_args` だけのまま、`crate::args::parse_args` で引けるようにする(`lib.rs:113`、`src/tests/args_grammar.rs:23`)。

### FR4. 文書と小さな直し

- **FR4.1** 次の文書を、分けた後の形に直す(Q3 A。引き渡しの 6)。
  - `docs/spec/rust/02-architecture.md`、`07-process-and-signals.md`、`08-output-and-i18n.md`、`12-rebuild-guide.md` の、モジュールとファイルの記述
  - `crates/cli/src/lib.rs`、`crates/core/src/lib.rs`、`report/mod.rs`、`walk/mod.rs` の文書コメントにある、モジュールの説明

  `12-rebuild-guide.md` の段の中で、その段の時点のファイルを書いた記述(履歴)は、変えずに残す。
- **FR4.2** team.md の Code Style にある「使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える」の `args.rs` を、移した先のパスに直す(Q4 A)。コード生成の計画に今の文と書き換えた後の文を載せ、承認を経てから直す。この行の外は変えない。
- **FR4.3** `crates/core/src/error.rs:176` の文書コメントにある古い名前 `report::escape` を、今の名前に直す(Q5 A)。
- **FR4.4** `crates/cli/src/tests/report_messages.rs:99` の `with_all_events` に、足りない `DefinitionValid` と `ResetAllKept` を足す(Q5 A)。足した変種でも、そのテストの確かめが通ること。

### FR5. 版

- **FR5.1** すべてを片づけてから、版を 0.10.4 にする。
  - 範囲は ad775bc(版を 0.10.3 にする)と同じ 22 ファイル(引き渡しの 5)。
  - 件名は「版を 0.10.4 にする」で、1 コミットにする。

## 3. 非機能の要件

- **NFR1 動きを変えない**: 次の 3 つが、今と同じであること。既存のテストがすべて、名前と期待の値を変えずに通る(例外は版の期待と FR4.4)。
  - 出力、終了コード、ステート
  - 誤りの順: `walk` の同じ位置の誤りの順と、`build_command` が返す最初の誤り(引き渡しの 2.3・3.2)
  - 言語と色の判定の時(FR1.3)
- **NFR2 公開の範囲を変えない**: 次の 2 つを守る。
  - クレートの外に見える名前を増やさず、減らさない。
  - crate の中の名前は `pub use` で今の道筋のまま残す。テストのために `pub` を足さない(project.md の NEVER)。
- **NFR3 品質**: 次の 3 つを満たす。
  - `cargo fmt`、`cargo clippy -D warnings`、`cargo test --workspace` が通る。
  - 行カバレッジが 80% 以上。
  - CI(3 つの OS、MSRV、coverage、docs-check)が通る。
- **NFR4 依存**: 新しい依存を足さない。
- **NFR5 ソースの検査**: 新しいファイルも `nfr_source.rs` の検査を通す。`#[path]`・`include!` は使わない。`cfg(unix)` などは `platform` の外に置かない。
- **NFR6 小さな単位**: 移すだけのコミットと、関数に分けるコミットを分ける。1 つのコミットは 1 つの移しか、1 つの分けにする(team.md)。

## 4. 制約

- 直接 `main` にコミットする(team.md)。
- `report/messages.rs` と、`walk/mod.rs` のキーの表は動かさない。

## 5. 前提

- 子のファイルの名前は設計で決める。例: `report/event.rs`・`lines.rs`・`escape.rs`、`walk/values.rs`・`transition.rs`・`cross_check.rs`、`args/options.rs`・`tokens.rs`・`values.rs`。
- 引き渡しの行の範囲は `9376cb7` の時点のもの。作業では、行の番号ではなく名前で場所を確かめる。

## 6. 範囲の外

- `report/messages.rs` の分割(候補の D-4。人が選ばなかった)。
- 単体テストの置き場所の決まりを team.md に書くこと(次の intent で検討する)。
- `render` 以外の長い関数(`run_job_with_retry`、`Walker::deliver`、`run_pipeline`)。

## 7. 未決の事項(設計で決める)

- 子のファイルの名前と、それぞれに置く項目の境目。境目は次の 3 つ。
  - `log_dir`、`exit_code_of`、`integer_of`、`parse_args`、`build_command`
  - `push_line`(エスケープの側か、別にするか)
  - `detect_language`
- 9 種の関数の名前と引数。`Reporter` を渡すか、言語の閉包を渡すか。
- コミットの順。
