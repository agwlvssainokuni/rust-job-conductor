# ビジネスルール(Rules): 動きを変えない整理の第 2 弾

詳細の場所は `inventory.md`、承認が要るテストの変更と性能の基準は `functional-spec.md` の §4、§5。

```yaml
rules:
  - id: BR1.1
    statement: 完了と再開位置は apply_skip が書くことを、core の lib.rs、decide.rs、next_step.rs、resume.rs の説明に正しく書く。resume_point_on_interrupt は pipeline のジョブのあいだの中断にだけ使うと書く。コードは変えない。
    category: policy
    applies_to: crates/core/src/lib.rs、flow/decide.rs、flow/next_step.rs、flow/resume.rs
    trigger: 段 1
    logic: inventory.md 1a
    violation: 説明と実装(record.rs:171-173、pipeline.rs:109)が食い違う
    source: intent-statement.md(段 1)
  - id: BR1.2
    statement: StateContext::Plain(reset だけ)、OutputFailed と EXIT_STATE(watch・graph・--dry-run の出力の失敗と init の失敗も 65)、config_path(convert 以外)、read_state_raw(graph --state も)の説明と、仕様書 08 章(rust と language-agnostic)の Plain の行を、実装に合わせる。
    category: policy
    applies_to: crates/cli/src/report/mod.rs、exit.rs、types.rs、deps.rs、docs/spec/rust/08-output-and-i18n.md、docs/spec/language-agnostic/08-output-and-i18n.md
    trigger: 段 1
    logic: inventory.md 1b
    violation: 説明と実装が食い違う
    source: intent-statement.md(段 1)
  - id: BR1.3
    statement: escape の説明を、同じ規則の関数が core(本体)と converter(写し)にあるという今の姿に合わせる(runtime には無い)。テストのファイルはコメントだけ直す(T8)。
    category: policy
    applies_to: crates/cli/src/report/escape.rs、crates/cli/tests/escape_table.rs(コメント)、docs/spec/rust/11-quality-and-build.md
    trigger: 段 1(最終の文は段 2 の 2i のあと)
    logic: inventory.md 1c
    violation: 関数の数や置き場所の説明が食い違う
    source: intent-statement.md(段 1)
  - id: BR1.4
    statement: core・runtime・cli の lib.rs の関数とモジュールの表、runtime の state/mod.rs の説明を、今のモジュールと公開の関数(init、graph、plan、envfile、run_id、create_file、reset_all など)に合わせる。
    category: policy
    applies_to: crates/core/src/lib.rs、crates/runtime/src/lib.rs、crates/runtime/src/state/mod.rs、crates/cli/src/lib.rs
    trigger: 段 1(段 3 で足す run_id モジュールは段 3 で載せる)
    logic: inventory.md 1d
    violation: 表に無いモジュール・関数がある、または無いものが載る
    source: intent-statement.md(段 1)
  - id: BR1.5
    statement: PipelineState.definition_key の「仮」の注記を、確定した説明(definition_path と同じ文字列、ステートの必須のキー、照合には使わない)に直す。フィールドは消さない。
    category: policy
    applies_to: crates/core/src/types.rs
    trigger: 段 1
    logic: inventory.md 1e
    violation: 注記が「仮」のまま、またはフィールドが消える
    source: intent-statement.md(段 1)
  - id: BR1.6
    statement: docs/spec/rust/11-quality-and-build.md のテストのファイルごとの件数を書くのをやめる(ファイル名と目的は残す)。
    category: policy
    applies_to: docs/spec/rust/11-quality-and-build.md
    trigger: 段 1
    logic: inventory.md 1f
    violation: 件数が残り、実物とずれる
    source: intent-statement.md(段 1)
  - id: BR1.7
    statement: core と converter の lib.rs に #![forbid(unsafe_code)] を置く。
    category: constraint
    applies_to: crates/core/src/lib.rs、crates/converter/src/lib.rs
    trigger: 段 1
    logic: inventory.md 1g
    violation: unsafe を書けてしまう
    source: intent-statement.md(段 1)
  - id: BR1.8
    statement: 理由の文に書いた上限の数と、その定数を、const _ による assert でコンパイル時に結ぶ(core 4、converter 3、runtime 1、cli の英語の文 2)。文は 1 文字も変えない。
    category: constraint
    applies_to: crates/core/src/definition/{intake,version}.rs、crates/core/src/envfile/mod.rs、crates/converter/src/error.rs、crates/runtime/src/error.rs、crates/cli/src/report/messages.rs
    trigger: 段 1
    logic: inventory.md 1h
    violation: 定数を変えても文の数が古いまま通る
    source: intent-statement.md(段 1)
  - id: BR2.1
    statement: 位置で状態を引く status_at を core の flow の 1 か所に置き、decide.rs、record.rs、next_step.rs の pending_ids、resume.rs が使う。@end に着いたときの残りのジョブの列挙を、ジョブの数に比例する時間にする。
    category: policy
    applies_to: crates/core/src/flow/{mod,decide,record,next_step,resume}.rs
    trigger: 段 2
    logic: inventory.md 2a
    violation: 重複が残る、または二乗の時間が残る、または結果が変わる
    source: intent-statement.md(段 2、Q4 の A)
  - id: BR2.2
    statement: merge_env を、名前ごとの最後の位置を表で引いて 1 回で並べる形にし、項目の数に比例する時間にする。BR4.3 の並び(同じ名前は最後に足した位置、大文字小文字だけ違う名前は別)と戻り値の型を変えない。.env の解析の重複の除去は任意。
    category: policy
    applies_to: crates/core/src/envfile/merge.rs(任意で parse.rs)
    trigger: 段 2
    logic: inventory.md 2b
    violation: 二乗の時間が残る、または並びが変わる
    source: intent-statement.md(段 2、Q4 の A)
  - id: BR2.3
    statement: 既定の定義ファイル名と場所の決定を、core の DEFAULT_DEFINITION_FILE と resolve_path に寄せる。cli の公開の DEFAULT_INIT_PATH は名前と値を保つ。
    category: policy
    applies_to: crates/cli/src/ctx.rs、types.rs、crates/core/src/definition/intake.rs
    trigger: 段 2
    logic: inventory.md 2c
    violation: 既定の名前 conductor.toml の文字列が core の外に残る
    source: intent-statement.md(段 2)
  - id: BR2.4
    statement: 終点の語 @end の文字列を core の TransitionTarget::END_LABEL の 1 か所にし、DisplayEvent::Transition.target を &TransitionTarget で渡す。表示は 1 文字も変えない。
    category: policy
    applies_to: crates/core/src/types.rs、definition/walk/mod.rs、crates/cli/src/ctx.rs(段 3 で step.rs)、report/{mod,graph_text,graph_mermaid}.rs
    trigger: 段 2
    logic: inventory.md 2d、functional-spec.md §4 T1
    violation: 終点の語の文字列が複数残る、または行き先を文字列で渡す
    source: intent-statement.md(段 2、Q2 の A)
  - id: BR2.5
    statement: 表示の <ファイル>[:<行>] の組み立てを place_of の 1 か所にし、パスの表示は show_path を使う。
    category: policy
    applies_to: crates/cli/src/report/mod.rs
    trigger: 段 2
    logic: inventory.md 2e
    violation: 同じ式が複数残る
    source: intent-statement.md(段 2)
  - id: BR2.6
    statement: graph の text と Mermaid の、辺の種類のラベルと主線の辺の判定を、kind_label と is_main_edge の 1 か所にする。
    category: policy
    applies_to: crates/cli/src/report/graph_text.rs、graph_mermaid.rs
    trigger: 段 2
    logic: inventory.md 2f
    violation: 同じ判定が 2 か所に残る
    source: intent-statement.md(段 2)
  - id: BR2.7
    statement: 使い方の誤りの表示と 68(Ctx::usage_error)、標準出力の失敗と 65(Ctx::show_output)、分類から終了コード(exit::outcome_exit_code)を、それぞれ 1 か所にする。誤りの順序と文は変えない。
    category: policy
    applies_to: crates/cli/src/{ctx,dry_run,aux_commands,graph,init,run,pipeline,exit}.rs
    trigger: 段 2
    logic: inventory.md 2g
    violation: 同じ手順が複数残る、または終了コードが変わる
    source: intent-statement.md(段 2)
  - id: BR2.8
    statement: runtime のステートファイルの削除(remove_state_file)と、親が空なら "." にする処理(dir_or_current)を、それぞれ 1 か所にする。unsafe と cfg を増やさない。一時ファイルのパスの作り方は変えない。
    category: policy
    applies_to: crates/runtime/src/state/{writer,reset_all,mod}.rs、file_create.rs、platform/unix.rs
    trigger: 段 2
    logic: inventory.md 2h
    violation: 同じ処理が複数残る、または誤りの種類が変わる
    source: intent-statement.md(段 2)
  - id: BR2.9
    statement: cli の escape の本体を core の escape_display に寄せ、cli は再公開と薄い包みだけにする。公開の道 conductor_cli::report::escape は保つ。converter の写しは残す(converter は core に依存しない)。
    category: policy
    applies_to: crates/core/src/error.rs、crates/cli/src/report/escape.rs
    trigger: 段 2
    logic: inventory.md 2i
    violation: cli に同じ規則の写しが残る、または converter が core に依存する
    source: intent-statement.md(段 2)
  - id: BR2.10
    statement: aux_commands.rs の型の書き直しとフルパスの冗長をなくす。
    category: policy
    applies_to: crates/cli/src/aux_commands.rs
    trigger: 段 2
    logic: inventory.md 2j
    violation: 冗長が残る
    source: intent-statement.md(段 2)
  - id: BR2.11
    statement: 上限に近い入力の性能のテストを足す。A2(ジョブ 1,000 個、@end の列挙を 100 回)は 5 回の中央値が 50 ms 以内、A3a(.env 1 ファイル 10,000 項目の merge_env)は 50 ms 以内、A3b(2 ファイル 10,000 項目ずつ)は 100 ms 以内。直す前と後の時間を記録する。
    category: validation
    applies_to: crates/core/tests/flow_perf.rs、crates/core/tests/env_merge_perf.rs(新規)
    trigger: 段 2
    logic: functional-spec.md §5
    violation: 二乗の時間に戻っても検知できない、または CI の遅い機械で揺れて落ちる
    source: intent-statement.md(段 2、Q4 の A)
  - id: BR3.1
    statement: ジョブの実行のあとの表示(report_execution、result_detail)を ctx.rs の外(step.rs)に置き、definition_errors を非公開にする。
    category: policy
    applies_to: crates/cli/src/ctx.rs、step.rs
    trigger: 段 3
    logic: inventory.md 3a
    violation: 表示の組み立てが ctx.rs に残る
    source: intent-statement.md(段 3)
  - id: BR3.2
    statement: Session が Checked をフィールドとして持ち、同じフィールドの写しをなくす。
    category: policy
    applies_to: crates/cli/src/ctx.rs、run.rs、pipeline.rs、step.rs、retry.rs
    trigger: 段 3
    logic: inventory.md 3b
    violation: 同じフィールドが 2 つの型に残る
    source: intent-statement.md(段 3)
  - id: BR3.3
    statement: run と pipeline の検査を check_job と check_pipeline の 1 か所にし、prepare_run、prepare_pipeline、prepare_checked が使う。検査の順序(シグナル、定義 64、command 64、ジョブID 68、--assume 68、run ID 68、.env 64、ステート 65)と表示を変えない。
    category: policy
    applies_to: crates/cli/src/ctx.rs
    trigger: 段 3
    logic: inventory.md 3c
    violation: 手順の重複が残る、または誤りの順序が変わる
    source: intent-statement.md(段 3)
  - id: BR3.4
    statement: run ID の解決(resolve_run_id、aux_run_id)を引数の解釈の外の run_id モジュールに置き、仕様書 02 章と 12 章を合わせる。
    category: policy
    applies_to: crates/cli/src/args.rs、ctx.rs、run_id.rs(新規)、docs/spec/rust/02-architecture.md、12-rebuild-guide.md
    trigger: 段 3
    logic: inventory.md 3d、functional-spec.md §4 T7
    violation: 解決が args.rs に残る
    source: intent-statement.md(段 3)
  - id: BR4.1
    statement: CliCommand.run_id を Option<RunIdArg>(Empty、Id(RunId))で表し、RunId::parse を引数の段階の 1 回だけにする。誤りの種類と対象は変えない。
    category: policy
    applies_to: crates/cli/src/types.rs、args.rs、run_id.rs、ctx.rs
    trigger: 段 4
    logic: inventory.md 4a、functional-spec.md §4 T7
    violation: 空文字で「run ID なし」を表す、または二重に検査する
    source: intent-statement.md(段 4、Q2 の A)
  - id: BR4.2
    statement: 起動前の判断の理由を SkipCause(SkipTarget、AfterEnd、AlreadyExecuted)と RejectReason(OutOfOrder)に分け、DecisionReason を消す。到達しない分岐(record.rs、simulate.rs、messages.rs の 2 か所)をなくす。実際に出る文は変えない。
    category: policy
    applies_to: crates/core/src/types.rs、flow/{decide,record}.rs、plan/{mod,simulate}.rs、crates/cli/src/{step,run,pipeline,dry_run}.rs、report/{mod,messages}.rs
    trigger: 段 4
    logic: inventory.md 4b、functional-spec.md §4 T2〜T6
    violation: 到達しない分岐が残る、または表示が変わる
    source: intent-statement.md(段 4、Q2 の A)
  - id: BR4.3
    statement: PlanMode、EdgeKind、EdgeTarget の定義を消し、RunMode、TransitionOn、TransitionTarget の別名(pub use … as)にする。From<Classification> for JobStatus と TransitionOn::parse を足し、walk の文字列の比較をなくす。
    category: policy
    applies_to: crates/core/src/types.rs、plan/mod.rs、graph/mod.rs、flow/record.rs、definition/walk/mod.rs
    trigger: 段 4
    logic: inventory.md 4c
    violation: 同じ意味の列挙型の定義が 2 つ残る、または文字列で比べる
    source: intent-statement.md(段 4)
  - id: BR4.4
    statement: runtime/tests/flow_with_state.rs の補助、テスト名、期待を、今の cli の流れ(スキップは変わったときだけ書く、削除は remove_on_complete のときだけ、pipeline は記述順に決める、完了のステートは既定で残る)に合わせる。件数は 20 のまま。
    category: policy
    applies_to: crates/runtime/tests/flow_with_state.rs
    trigger: 段 4
    logic: inventory.md 4d、functional-spec.md §4 T9
    violation: 古い流れを再現したまま、または名前が今の動きと矛盾する
    source: intent-statement.md(段 4、Q3 の A)
  - id: BR5.1
    statement: 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run の出力)を変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテスト
    violation: 回帰
    source: intent-statement.md
  - id: BR5.2
    statement: 既存のテストは変えずに通す。例外は functional-spec.md §4 の T1〜T9 だけで、組み立ての値と import(と型の値の名前の置き換え)に限り、期待する出力の文字列は変えない。ユーザーの承認が要る。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§7
    violation: 承認のないテストの変更、または期待する出力の文字列の変更
    source: intent-statement.md(Q2 の A、Q3 の A)
  - id: BR5.3
    statement: 段の前と後でテストの件数を記録する。合格は失敗 0 で、件数が「基準 1,128 + 足したテスト(見込み +7)」に一致すること。増減は理由と一緒に記録する。
    category: validation
    applies_to: テスト
    trigger: 各段の終わり
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減
    source: intent-statement.md(Q1 の A、Q3 の A、Q4 の A)
  - id: BR5.4
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、cfg を足さない。converter は core に依存しない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(合格の条件)
```
