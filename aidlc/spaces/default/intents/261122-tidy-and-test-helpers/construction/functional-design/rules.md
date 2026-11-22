# ビジネスルール(Rules): 動きを変えない小さな整理とテストの補助

詳細の場所は `inventory.md`、承認が要るテストの変更(T 表)と足すテスト(N 表)、件数の見込み、決めてほしい点は `functional-spec.md` の §4、§5、§6、§10。

```yaml
rules:
  # ---- 段 1: 説明のずれと、使われないもの ----
  - id: BR1.1
    statement: core の types.rs の型の生成元の説明を、今の姿(PipelineState・JobState は core の flow::record が作り、runtime は読み書きだけ。JobExecution は runtime の起動、cli のリトライの待機の中断、core の --dry-run の模擬が作る)に合わせる。コードは変えない。
    category: policy
    applies_to: crates/core/src/types.rs
    trigger: 段 1
    logic: inventory.md 1a
    violation: 説明と実装(flow/record.rs、plan/simulate.rs:280)が食い違う
    source: intent-statement.md(SM1。説明のずれ)
  - id: BR1.2
    statement: sanitize と escape_display の説明を「本文はエスケープ後の文字数で上限以内、切ったときは … を足す(全体は最大で上限 + 1 文字)」にする。テスト名は変えず、テストの中に注記を足す(D1。ユーザーが決定、2026-11-23)。
    category: policy
    applies_to: crates/core/src/error.rs
    trigger: 段 1
    logic: inventory.md 1b、functional-spec.md T40
    violation: 説明が「上限までに収める」のまま、またはテスト名を承認なく変える
    source: intent-statement.md(SM1、Q2)
  - id: BR1.3
    statement: cli の args.rs の文法の表を、OPTIONS の 11 行(--dry-run、--assume、--format、--state、--force を含む)と一致させる。
    category: policy
    applies_to: crates/cli/src/args.rs
    trigger: 段 1
    logic: inventory.md 1c
    violation: 表に無いオプションがある、または表と OPTIONS が食い違う
    source: intent-statement.md(SM1)
  - id: BR1.4
    statement: 仕様書 docs/spec/rust/08 の拒否の理由の表から、今は無い SkipTarget・AfterEnd・AlreadyExecuted の 3 行を消す。05 章の「OutOfOrder → 何も変えない」も apply_skip の今の型に合わせる。
    category: policy
    applies_to: docs/spec/rust/08-output-and-i18n.md、docs/spec/rust/05-execution-flow.md
    trigger: 段 1
    logic: inventory.md 1d
    violation: 仕様書にコードに無い拒否の理由が残る
    source: intent-statement.md(SM1)
  - id: BR1.5
    statement: 使われない Command::program_name と、その単体テスト 1 件を消し、仕様書 02 章の 1 行を直す。件数の減り(−1)は理由と一緒に記録する。
    category: constraint
    applies_to: crates/core/src/types.rs、docs/spec/rust/02-architecture.md
    trigger: 段 1
    logic: inventory.md 1e、functional-spec.md T38
    violation: program_name が残る、または件数の減りが説明されない
    source: intent-statement.md(SM1、Q4)

  # ---- 段 2: 文言の定数 ----
  - id: BR2.1
    statement: cli の messages.rs の同じ文言(スキップの語、スキップの理由と watch の補足、実行済み、再開位置の印、詳細の見出し、成功・警告の語、完了の語)を、それぞれ 1 つの定数・関数にまとめる。表示は 1 文字も変えない。
    category: constraint
    applies_to: crates/cli/src/report/messages.rs、mod.rs、watch.rs、graph_text.rs、plan_text.rs、docs/spec/rust/08-output-and-i18n.md
    trigger: 段 2
    logic: inventory.md 2a
    violation: 同じ英語・日本語の組の定数が 2 つ以上残る、または出力が変わる
    source: intent-statement.md(SM1)
  - id: BR2.2
    statement: INTERNAL_ERROR と SIGNAL_SETUP_FAILED は、ERROR_PREFIX と同じ文字列のリテラル(マクロ error_prefix!)から concat! で作る。
    category: constraint
    applies_to: crates/cli/src/report/messages.rs
    trigger: 段 2
    logic: inventory.md 2a
    violation: '「conductor: error: 」の直書きが 2 か所以上残る'
    source: intent-statement.md(SM1)
  - id: BR2.3
    statement: core の環境変数の名前の規則の句と、予約名の接頭辞 CONDUCTOR_ を、それぞれ 1 か所(マクロ env_name_rule!、reserved_env_prefix!)から作る。RESERVED_ENV_PREFIX も同じマクロから作る。文は 1 文字も変えない。
    category: constraint
    applies_to: crates/core/src/run_id.rs、envfile/mod.rs、definition/walk/env.rs、definition/walk/mod.rs
    trigger: 段 2
    logic: inventory.md 2b、2c
    violation: 句や接頭辞の直書きが 2 か所以上残る、または文が変わる
    source: intent-statement.md(SM1)
  - id: BR2.4
    statement: walk の ROOT_KEYS は、版のキーに version::VERSION_KEY を使う。
    category: constraint
    applies_to: crates/core/src/definition/walk/mod.rs
    trigger: 段 2
    logic: inventory.md 2d
    violation: '"format_version" の直書きが ROOT_KEYS に残る'
    source: intent-statement.md(SM1)
  - id: BR2.5
    statement: 間隔の理由の文の「1 秒以上 24 時間以下」と MIN_SECS・MAX_SECS を、const _ による assert でコンパイル時に結ぶ。文は変えない。
    category: constraint
    applies_to: crates/core/src/definition/duration.rs、definition/walk/retry.rs
    trigger: 段 2
    logic: inventory.md 2e
    violation: 定数を変えても文の数が古いまま通る
    source: intent-statement.md(SM1)
  - id: BR2.6
    statement: ジョブに渡す CONDUCTOR_RUN_ID と、読む既定の DEFAULT_RUN_ID_ENV は意味が別なので、まとめない。理由を注記する(D2。ユーザーが決定、2026-11-23)。
    category: policy
    applies_to: crates/core/src/envfile/merge.rs
    trigger: 段 2
    logic: inventory.md 2f
    violation: 片方を変えるともう片方が黙って変わる形にする
    source: intent-statement.md(SM1)

  # ---- 段 3: 処理の重複と境界 ----
  - id: BR3.1
    statement: runtime の一時ファイルの作り方(名前の形、新規作成のくり返し、連番、親が空なら .)を中立のモジュール temp_file の 1 か所に置き、連番は 1 つにする(D5。ユーザーが決定、2026-11-23)。file_create と platform は state の内部を使わない。一時ファイルの名前の形と、呼び出し側が渡すディレクトリは変えない。
    category: constraint
    applies_to: crates/runtime/src/temp_file.rs、state/writer.rs、state/mod.rs、state/reset_all.rs、file_create.rs、platform/unix.rs、lib.rs、docs/spec/rust/02、06
    trigger: 段 3
    logic: inventory.md 3a
    violation: 一時ファイルの作り方や連番の static が 2 か所以上に残る、または file_create・platform が crate::state を使う
    source: intent-statement.md(SM1)
  - id: BR3.2
    statement: cli の表示の行の追加(push_line)を report の 1 か所に置き、graph・plan・watch が使う。
    category: constraint
    applies_to: crates/cli/src/report/mod.rs、graph_text.rs、plan_text.rs、watch.rs
    trigger: 段 3
    logic: inventory.md 3b
    violation: 同じ本体の写しが残る
    source: intent-statement.md(SM1)
  - id: BR3.3
    statement: "表示の場所 job '<ID>' を job_place の 1 か所で組み立てる。"
    category: constraint
    applies_to: crates/cli/src/report/mod.rs
    trigger: 段 3
    logic: inventory.md 3c
    violation: format! の写しが残る
    source: intent-statement.md(SM1)
  - id: BR3.4
    statement: reset の案内(run ID の有無で選ぶ)を messages::reset_hint の 1 か所で選び、bool から StateContext を選ぶ関数は StateContext::for_read に置く。StateContext の値は変えない。
    category: constraint
    applies_to: crates/cli/src/report/messages.rs、report/mod.rs、ctx.rs、aux_commands.rs
    trigger: 段 3
    logic: inventory.md 3d
    violation: 案内の選び方が 2 か所以上に残る
    source: intent-statement.md(SM1)
  - id: BR3.5
    statement: --dry-run の計画の最後の語は、plan_text で PlanEnd を 1 回だけ見て決める(エスケープ済みの ID を別の引数で重ねて渡さない)。
    category: constraint
    applies_to: crates/cli/src/report/plan_text.rs、messages.rs
    trigger: 段 3
    logic: inventory.md 3e
    violation: PlanEnd とそこから取り出した値を同じ関数に渡す形が残る
    source: intent-statement.md(SM1)
  - id: BR3.6
    statement: graph_mermaid の辺の両端の取り出しを 1 段の let-else にする。
    category: policy
    applies_to: crates/cli/src/report/graph_mermaid.rs
    trigger: 段 3
    logic: inventory.md 3f
    violation: 二段の let-else が残る
    source: intent-statement.md(SM1)

  # ---- 段 4: 到達しない分岐と効かない処理 ----
  - id: BR4.1
    statement: decide_before_run の規則 5 のコードを消す(結果は規則 6 と同じ)。説明の番号 BR3.5 の 1〜7 は残し、5 は 6 に含まれると書く。仕様書 05 章の表にも同じ注記。
    category: constraint
    applies_to: crates/core/src/flow/decide.rs、docs/spec/rust/05-execution-flow.md
    trigger: 段 4
    logic: inventory.md 4a
    violation: 規則 5 のコードが残る、または判断の結果が変わる
    source: intent-statement.md(SM1)
  - id: BR4.2
    statement: lexopt の Parser::next の誤りは、UnexpectedValue の枝と、1 つの守りの枝(版を上げたときのため。注記つき)だけにする。到達しない MissingValue の枝を消す(D3。ユーザーが決定、2026-11-23)。
    category: constraint
    applies_to: crates/cli/src/args.rs
    trigger: 段 4
    logic: inventory.md 4b
    violation: 到達しない枝が 2 つ以上残る、または注記が無い
    source: intent-statement.md(SM1)
  - id: BR4.3
    statement: 表示の組み立ての match を 1 つにし、到達しない => String::new() を消す。SignalSetupFailed・Version は言語を判定しない(ロケールを読まない)。
    category: constraint
    applies_to: crates/cli/src/report/mod.rs
    trigger: 段 4
    logic: inventory.md 4c
    violation: 到達しない枝が残る、または固定の文でロケールを読む
    source: intent-statement.md(SM1)
  - id: BR4.4
    statement: cli のリトライの知らせは、リトライの設定と終了コードを 1 つの let-else で取り出し、使われない既定値 0 を持たない。
    category: constraint
    applies_to: crates/cli/src/retry.rs
    trigger: 段 4
    logic: inventory.md 4d
    violation: map_or(0, …) や _ => 0 が残る
    source: intent-statement.md(SM1)
  - id: BR4.5
    statement: 終了コードからの分類・返す終了コード・conductor_error(成功コードに無い 0)を classify_exit_code(戻り値 ExitCodeClass)の 1 か所で決め、classify の手順 5 と --dry-run の模擬の両方が使う。出力と終了コードは変えない。--dry-run の模擬は、リトライの対象の判定(is_retry_target)も本体と共有し、到達しない unwrap_or(Classification::Error) と写しを持たない(D4。classify_exit_code を切り出すとユーザーが決定、2026-11-23)。
    category: constraint
    applies_to: crates/core/src/plan/simulate.rs、flow/classify.rs、flow/retry.rs
    trigger: 段 4
    logic: inventory.md 4e
    violation: unwrap_or または retryable の写しが残る、または計画の結果が変わる
    source: intent-statement.md(SM1)
  - id: BR4.6
    statement: retry の max と multiplier は、範囲の検査と同じ所で u32 にする。後からの u32::try_from(..).ok() を持たない。
    category: constraint
    applies_to: crates/core/src/definition/walk/retry.rs
    trigger: 段 4
    logic: inventory.md 4f
    violation: try_from(m).ok() が残る、または理由の文の数が変わる
    source: intent-statement.md(SM1)
  - id: BR4.7
    statement: 間隔の秒の範囲の検査だけで 0 を拒否し、余分な n >= 1 を持たない。
    category: constraint
    applies_to: crates/core/src/definition/duration.rs
    trigger: 段 4
    logic: inventory.md 4g
    violation: n >= 1 が残る
    source: intent-statement.md(SM1)

  # ---- 段 5: 文字列での比較 ----
  - id: BR5.1
    statement: コマンド名の逆引きは CommandKind::ALL と from_name(name() の逆)で行う。args.rs に名前の文字列の match を持たない。
    category: constraint
    applies_to: crates/cli/src/types.rs、args.rs、docs/spec/rust/02-architecture.md
    trigger: 段 5
    logic: inventory.md 5a
    violation: 名前の表が 2 か所に残る
    source: intent-statement.md(SM1)
  - id: BR5.2
    statement: コマンドごとの位置引数の要否は、網羅の match(Operand)で決める。否定の matches! で決めない。
    category: constraint
    applies_to: crates/cli/src/args.rs
    trigger: 段 5
    logic: inventory.md 5b
    violation: 新しいコマンドを足しても網羅の誤りにならない形が残る
    source: intent-statement.md(SM1)
  - id: BR5.3
    statement: 使い方の誤りの対象(オプションの綴り)は、args.rs では Opt::subject で作る。types.rs の RunIdArg::parse は定数 RUN_ID_OPTION を使い、args.rs の const assert で Opt::RunId の表の綴りと結ぶ(types から args を参照しない)。ほかに "--to" などの直書きを持たない。誤りの対象の値は変えない。
    category: constraint
    applies_to: crates/cli/src/args.rs、types.rs
    trigger: 段 5
    logic: inventory.md 5c
    violation: オプションの綴りの直書きが残る
    source: intent-statement.md(SM1)
  - id: BR5.4
    statement: reset --all の候補の見分けは、解釈の失敗の原因の列挙(FailureCause::RunIdMismatch)で行う。理由の文の一致で判定しない。公開の StateError は変えない。
    category: constraint
    applies_to: crates/runtime/src/state/reader.rs
    trigger: 段 5
    logic: inventory.md 5d
    violation: e.reason == reason::… の比較が残る
    source: intent-statement.md(SM1)

  # ---- 段 6: walk の整理 ----
  - id: BR6.1
    statement: success_exit_codes の読み取りの結果を、書いていない・誤り・正しい値の 3 つの場合を持つ型(SuccessCodes)の 1 つのフィールドで持つ。真偽値 success_declared を持たない。
    category: constraint
    applies_to: crates/core/src/definition/walk/mod.rs、walk/retry.rs
    trigger: 段 6
    logic: inventory.md 6a
    violation: success と success_declared の 2 つのフィールドが残る、または重なりの検査の結果が変わる
    source: intent-statement.md(SM1)
  - id: BR6.2
    statement: build_jobs は RawJob の所有権を受け取り、値を移す(clone しない)。
    category: policy
    applies_to: crates/core/src/definition/walk/mod.rs
    trigger: 段 6
    logic: inventory.md 6b
    violation: 不要な clone が残る
    source: intent-statement.md(SM1)
  - id: BR6.3
    statement: TOML の整数の値を i64 にする式は、definition::integer_value の 1 か所に置き、walk と version が使う。
    category: constraint
    applies_to: crates/core/src/definition/mod.rs、walk/mod.rs、version.rs
    trigger: 段 6
    logic: inventory.md 6c
    violation: from_str_radix の写しが残る
    source: intent-statement.md(SM1)
  - id: BR6.4
    statement: 入れ子の深さの検査は、テーブルと配列で同じ補助(walk_children)を使う。深さの検査を子の走査より先に行う順序を保つ。
    category: constraint
    applies_to: crates/core/src/definition/depth.rs
    trigger: 段 6
    logic: inventory.md 6d
    violation: 同じ形の枝が残る、または深さの結果が変わる
    source: intent-statement.md(SM1)

  # ---- 段 7: テストの補助 ----
  - id: BR7.1
    statement: cli の README のテストは、README の読み取り、docs-check の 7 語、必須語句の検査(assert_readme_has、assert_docs_check_words)、見出しの後のコードブロックの取り出しを tests/docs_support の 1 か所から使う。必須語句の検査の失敗の文は変えない(docs_retry.rs の docs-check の 7 語は「README に … がない」のまま)。
    category: constraint
    applies_to: crates/cli/tests/docs_*.rs、tests/docs_support/mod.rs
    trigger: 段 7
    logic: inventory.md 7a、functional-spec.md T1〜T6
    violation: readme() や 7 語の配列、必須語句のループの写しが残る、または必須語句の失敗の文が変わる
    source: intent-statement.md(SM2)
  - id: BR7.2
    statement: cli のテストのジョブの状態の取り出し(status_of、status_in)と、シグナルの送信(send、send_self)を、それぞれ共通の場所(common、signal_support)から使う。args_grammar.rs は FakeEnv::with を使う。
    category: constraint
    applies_to: crates/cli/tests/flow_run.rs、flow_pipeline.rs、flow_retry.rs、binary_signal.rs、signals_order.rs、signals_real.rs、args_grammar.rs、tests/common、tests/signal_support
    trigger: 段 7
    logic: inventory.md 7b、7c、7d、functional-spec.md T7〜T14、T20
    violation: 写しが残る
    source: intent-statement.md(SM2)
  - id: BR7.3
    statement: cli の偽物は、使われない状態(MemRuntime.writes)を持たず、MemRuntime も ScriptedRuntime と同じく、起動するときだけ台本を消費し、ScriptedRuntime は受け取った分割済みのコマンドを記録し(run_lines)、Default は derive で作る。
    category: constraint
    applies_to: crates/cli/tests/common/mod.rs
    trigger: 段 7
    logic: inventory.md 7e、functional-spec.md T15〜T19
    violation: 書くだけのフィールドや手書きの Default が残る、または _line を捨てる
    source: intent-statement.md(SM2)
  - id: BR7.4
    statement: runtime のテストの run ID なしの包みと定義の組み立ては、tests/common(without_run_id、definition)から使う。
    category: constraint
    applies_to: crates/runtime/tests/state_read.rs、state_write.rs、nfr_content.rs、nfr_performance.rs、flow_with_state.rs、tests/common
    trigger: 段 7
    logic: inventory.md 7f、functional-spec.md T21、T23〜T27
    violation: 包みや定義の全フィールドの直書きの写しが残る
    source: intent-statement.md(SM2)
  - id: BR7.5
    statement: core の check_consistency の単体テストは core の consistency.rs に置く。runtime の src/state/consistency.rs(テストだけのファイル)とモジュールの宣言、表の行を消す。テストの名前と中身は変えない。
    category: constraint
    applies_to: crates/runtime/src/state/consistency.rs、state/mod.rs、lib.rs、crates/core/src/consistency.rs、docs/spec/rust/02、12
    trigger: 段 7
    logic: inventory.md 7g、functional-spec.md T28、T29
    violation: runtime にテストだけのモジュールが残る、または件数の合計が変わる
    source: intent-statement.md(SM2)
  - id: BR7.6
    statement: JobExecution は、テストでも組み立ての関数(exited、not_started、start_failed、new)で作り、全フィールドの手組みをしない。
    category: constraint
    applies_to: crates/core/tests/flow_sequences.rs、retry_flow.rs、graph_edges.rs、crates/core/src/flow/classify.rs・next_step.rs・record.rs(単体テスト)、crates/runtime/tests/nfr_content.rs
    trigger: 段 7
    logic: inventory.md 7h、functional-spec.md T25、T39、T41〜T45
    violation: grep "JobExecution {" で型の定義のほかに当たる
    source: intent-statement.md(SM2)
  - id: BR7.7
    statement: core のテストの SECRET、run、kinds、median、ステートの組み立て、JobExecution の補助を tests/common の 1 か所から使う。
    category: constraint
    applies_to: crates/core/tests/*.rs、tests/common/mod.rs
    trigger: 段 7
    logic: inventory.md 7i、functional-spec.md T30〜T37
    violation: core の tests に写しが残る
    source: intent-statement.md(SM2)
  - id: BR7.8
    statement: converter の反対の形式は TargetFormat::opposite の 1 か所で決める。
    category: constraint
    applies_to: crates/converter/src/types.rs、lib.rs、tests/convert_roundtrip.rs
    trigger: 段 7
    logic: inventory.md 7j、functional-spec.md T46
    violation: 反対の形式の match の写しが残る
    source: intent-statement.md(SM2)
  - id: BR7.9
    statement: 補助の置き換えで、テストの名前、件数、確かめる中身、期待する出力の文字列を変えない。値が変わりうる置き換え(★ の T9、T19、T33、T37)は、テストの結果が変わらない根拠を設計に書き、設計の承認のときに承認を取る(承認待ち)。
    category: constraint
    applies_to: テスト
    trigger: 段 7
    logic: functional-spec.md §4
    violation: 承認のない変更、または期待の変更
    source: intent-statement.md(Q2)
  - id: BR7.10
    statement: 仕様書 docs/spec/rust の 02 章の偽物の表と 11 章のテストのファイルの表を、新しい補助のモジュールと置き場所に合わせる。
    category: policy
    applies_to: docs/spec/rust/02-architecture.md、11-quality-and-build.md
    trigger: 段 7 の後
    logic: functional-spec.md §8 の 49
    violation: 仕様書の表に無い補助のモジュールがある
    source: intent-statement.md(SM2)

  # ---- 新しいテスト ----
  - id: BR8.1
    statement: 足すテストは、整理で確かめられるようになったもの(N1 コマンド名の往復、N2 cli の層で分割済みのコマンドを分け直さない、N3 リトライで同じコマンドを渡す、N4 一時ファイルの補助、N5 反対の形式)の 5 件に限る。test-after で、正しい場合と境界・誤りの場合を 2 つ以上確かめる。
    category: constraint
    applies_to: テスト
    trigger: 各項目の実装の後
    logic: functional-spec.md §5
    violation: 一覧に無いテストを足す、または増減が説明されない
    source: intent-statement.md(SM3、Q3)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run の出力)を変えない。版は上げない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテストがそのまま通る(T 表の置き換えを除く)
    violation: 既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM4)
  - id: BR9.2
    statement: 既存のテストのファイルの変更は functional-spec.md §4 の T1〜T46 だけにする。件数が減るのは T38 の 1 件だけ。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§7
    violation: 一覧に無いテストのファイルの変更
    source: intent-statement.md(SM5、Q2、Q4)
  - id: BR9.3
    statement: 段の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,142 − 1 + 足したテスト(見込み +5)= 1,146」に一致すること。移動したテストは移動先で数える。テスト名の一覧の差は T38、N1〜N5、移動の道の違いだけ。
    category: validation
    applies_to: テスト
    trigger: 各段の終わり
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、またはテスト名の変化
    source: intent-statement.md(SM5、Q1、Q3、Q4)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(SM6)
```
