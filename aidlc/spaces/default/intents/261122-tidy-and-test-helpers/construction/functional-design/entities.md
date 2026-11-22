# エンティティ(Entities): 動きを変えない小さな整理とテストの補助

永続のデータ(ステートファイル)の形と、利用者に見える出力は変えない。公開の型は、足すだけ(`CommandKind::ALL`・`from_name`、`TargetFormat::opposite`)か、テストが参照しない `report::messages` の名前の整理だけ。ほかは非公開の型と補助。詳細は `inventory.md`。

```yaml
entities:
  # ---- 本体(新しい・変わる型と補助) ----
  - name: CommandKind::ALL / CommandKind::from_name
    description: コマンドの種類の全部(8 つ。記述順)と、名前からの逆引き(cli の types.rs。新規。公開)。from_name は ALL から name() が一致するものを返し、知らない名前は None。args.rs の名前の match を置き換える。
    attributes:
      - name: ALL
        logical_type: '[CommandKind; 8]'
      - name: from_name
        logical_type: fn(&str) -> Option<CommandKind>
  - name: Operand
    description: コマンドの位置引数の要否(cli の args.rs。新規。非公開)。fn operand(kind) が網羅の match で決める。
    attributes:
      - name: Required
        logical_type: run、pipeline、convert
      - name: Optional
        logical_type: validate、graph、init
      - name: Forbidden
        logical_type: watch、reset
  - name: Opt / Opt::subject
    description: オプションの識別子と、誤りの対象の綴り(cli の args.rs。既存。非公開のまま)。args.rs の誤りの対象の直書きを置き換える。
  - name: StateContext::for_read
    description: ステートを読んだときの文脈を run ID の有無から選ぶ(cli の report/mod.rs。ctx.rs の read_context を移す。pub(crate))。StateContext の 5 つの値は変えない。
  - name: messages の定数と関数(cli の report/messages.rs)
    description: 同じ文言を 1 つに。SKIPPED_WORD(SKIPPED_LIST を消す)、SKIPPED_BY_TRANSITION・SKIPPED_AFTER_END(NOTE_BY_* を消し、skip_reason と watch が使う)、ALREADY_EXECUTED(skip_reason と plan_disposition)、RESUME_HERE(GRAPH_・PLAN_ を 1 つに)、DETAIL_COMMAND・DETAIL_ENV・DETAIL_RETRY、class_word(plan_class_word を改名。GRAPH_KIND_SUCCESS・WARNING を消す)、COMPLETED(WATCH_COMPLETED を改名)、PLAN_END_REACHED と plan_stopped_at_error(plan_end を消す)、reset_hint(with_run_id)、マクロ error_prefix!。
  - name: report::push_line / job_place
    description: 表示の行を末尾の空白を除いて足す補助と、job '<ID>' の場所の組み立て(cli の report/mod.rs。pub(crate) と非公開)。
  - name: env_name_rule! / reserved_env_prefix!
    description: 環境変数の名前の規則の句と、予約名の接頭辞 CONDUCTOR_ に展開するマクロ(core の run_id.rs、envfile/mod.rs。crate の中だけ)。concat! と format! の第 1 引数に使う。RESERVED_ENV_PREFIX は reserved_env_prefix!() から作る。
  - name: SuccessCodes
    description: success_exit_codes の読み取りの結果(core の definition/walk/mod.rs。新規。非公開)。RawJob の success と success_declared を置き換える。
    attributes:
      - name: Absent
        logical_type: 書いていない(省略時は [0]。重なりを調べる)
      - name: Invalid
        logical_type: 書いて誤り(重なりを調べない)
      - name: Valid
        logical_type: (Vec<i32>, usize)。正しい値と位置
  - name: integer_value
    description: TOML の整数の値(DeInteger)を i64 にする(core の definition/mod.rs。新規。pub(crate))。walk の integer_of と version の確認が使う。
  - name: walk_children
    description: 入れ子の深さの検査の、子の走査(core の definition/depth.rs。新規。非公開)。テーブルと配列の枝が使う。
  - name: classify_exit_code / ExitCodeClass
    description: 終了コードを成功・警告・エラーに分け、返す終了コードと conductor_error も決める規則(core の flow/classify.rs。classify の手順 5 を切り出す。どちらも pub(crate))。classify と --dry-run の模擬が使う(D4。ユーザーが決定、2026-11-23)。
    attributes:
      - name: classification
        logical_type: Classification
      - name: reported_exit_code
        logical_type: i32(成功・警告は 0、成功コードに無い 0 は 1、そのほかのエラーはその終了コード)
      - name: conductor_error
        logical_type: bool(成功コードに無い 0 のときだけ真)
  - name: RUN_ID_OPTION
    description: '--run-id の綴り(cli の types.rs。新規。pub(crate) const &str = "--run-id")。RunIdArg::parse の誤りの対象に使う。args.rs の const assert で Opt::RunId の表の綴りと結ぶ。'
  - name: is_retry_target
    description: リトライの対象か(core の flow/retry.rs。既存。pub(crate) にして --dry-run の模擬も使う)。
  - name: temp_file
    description: 一時ファイルの作り方(runtime の src/temp_file.rs。新規。非公開のモジュール)。state の内部から移す。
    attributes:
      - name: dir_or_current
        logical_type: fn(&Path) -> &Path(state/mod.rs から移す)
      - name: TEMP_ATTEMPTS
        logical_type: u32 = 16(writer.rs から移す)
      - name: temp_prefix
        logical_type: fn(&str) -> String(.<名前>.tmp.。新規。後始末も使う)
      - name: temp_name
        logical_type: fn(&str, u32, u128, u32) -> String(writer.rs から移す)
      - name: create_temp
        logical_type: fn(&Path, FnMut(u32) -> String) -> io::Result<(PathBuf, File)>(writer.rs から移す)
      - name: SEQUENCE
        logical_type: static AtomicU32(writer.rs と file_create.rs の 2 つを 1 つに。D5。ユーザーが決定、2026-11-23)
      - name: create_temp_beside
        logical_type: fn(&Path, &str) -> io::Result<(PathBuf, File)>(新規。プロセス番号・時刻・連番で名前を作る)
  - name: FailureCause
    description: ステートの解釈の失敗の原因(runtime の state/reader.rs。新規。非公開)。ParseFailure に cause として持たせる。
    attributes:
      - name: RunIdMismatch
        logical_type: 中身の run_id がファイル名の run ID と違う(reset --all で NotOurs)
      - name: Other
        logical_type: そのほか
  - name: TargetFormat::opposite
    description: 反対の形式(converter の types.rs。新規。公開)。Yaml ↔ Toml。
  - name: Command::program_name
    description: 消す(core の types.rs。使われない。Q4)。

  # ---- テストの補助 ----
  - name: crates/cli/tests/docs_support
    description: 新規のモジュール。readme()、DOCS_CHECK_WORDS([&str; 7]。CI の docs-check と同じ語と順)、block_after_heading(text, heading, fence)、assert_readme_has(text, words)(失敗の文「README に … がない」)、assert_docs_check_words(text)(失敗の文「docs-check の語句 … がない」)。
  - name: crates/cli/tests/signal_support
    description: 新規のモジュール。send(signal, pid)、send_self(signal)。
  - name: crates/cli/tests/common(変更)
    description: status_in(state, id)、status_of(h, id) を足す。WriteFailure と ScriptedRuntime を derive(Default) に。ScriptedRuntime.run_lines(RefCell<Vec<RunLine>>、RunLine = (String, CommandLine))を足す。MemRuntime.writes を消す。MemRuntime::run_job は起動するときだけ exit_codes を消費する。
  - name: crates/runtime/tests/common(変更)
    description: without_run_id(read_state、reset_state、initial_state、write_state を run ID なしで呼ぶ)と、definition(name, jobs) を足す。
  - name: crates/core/tests/common
    description: 新規のモジュール。SECRET、run(text)、kinds(text)、median(samples)、state(jobs, resume, completed)、job(id)、exited(job_id, code)。
```
