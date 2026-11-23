# ビジネスルール(Rules): テストの都合で曲げられた本番のコードと、残りの整理

詳細の場所は `inventory.md`、承認が要るテストの変更(T 表)、消すテストの対の表、足すテスト(N 表)、件数の見込み、決めてほしい点は `functional-spec.md` の §4、§4.6、§5、§6、§10。

```yaml
rules:
  # ---- 段 1: テストの都合で曲げられた所(方針: テストのコードの都合で本番のコードを曲げない) ----
  - id: BR1.1
    statement: cli の Reporter::language_resolved と color_resolved を消す。遅延の判定のテストは、偽の Env の記録(既存の locale_read と、足す color_read)で確かめる。期待する出力の文字列と、var_calls・terminal_calls の確かめは変えない。
    category: constraint
    applies_to: crates/cli/src/report/mod.rs、crates/cli/tests/common/mod.rs、crates/cli/tests/report_messages.rs、docs/spec/rust/02
    trigger: 段 1
    logic: inventory.md 1a
    violation: 本番に、テストだけが読む判定の状態の問い合わせが残る
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: runtime の ResetAllReport から removed を消し、kept だけにする(構造体のまま。D9。決定(ユーザー、2026-11-23))。remove_state_file は Result<(), StateError> にする。テストは、ファイルの一覧と有無で、消したことと消さなかったことを確かめる。
    category: constraint
    applies_to: crates/runtime/src/state/reset_all.rs、state/writer.rs、crates/runtime/tests/run_id_state.rs、state_read.rs、crates/cli/tests/common/mod.rs、flow_run_id.rs、docs/spec/rust/02
    trigger: 段 1
    logic: inventory.md 1b
    violation: 本番が読まない removed、または読まれない bool の戻り値が残る
    source: intent-statement.md(SM1)
  - id: BR1.3
    statement: core の graph::is_lane_drawable と再公開を消す。テストは公開の layout_graph の結果(GraphLayout::Lanes か EdgeList)で確かめる。
    category: constraint
    applies_to: crates/core/src/graph/layout.rs、graph/mod.rs、crates/core/tests/graph_drawable.rs、graph_layout_oracle.rs、docs/spec/rust/02・05・12
    trigger: 段 1
    logic: inventory.md 1c
    violation: 本番の呼び出しの無い公開の関数が残る
    source: intent-statement.md(SM1)
  - id: BR1.4
    statement: core の LoadedEnvFiles::add を消す。テストは、core の tests/common の add_env_file(push_file のあと link)と、cli の 1 か所はその場の 2 行で、本番と同じ手順で組み立てる。
    category: constraint
    applies_to: crates/core/src/envfile/mod.rs、crates/core/tests/common/mod.rs、plan_types.rs、env_merge_perf.rs、plan_perf.rs、env_merge.rs、crates/cli/tests/plan_report.rs
    trigger: 段 1
    logic: inventory.md 1d
    violation: 偽の実装やテストの便宜のための公開のメソッドが残る
    source: intent-statement.md(SM1)
  - id: BR1.5
    statement: runtime の remove_stale_temps は、今の時刻を引数で受け取らず、関数の中で取る。
    category: constraint
    applies_to: crates/runtime/src/state/writer.rs
    trigger: 段 1
    logic: inventory.md 1e
    violation: 本番もテストも同じ値を渡す引数が残る
    source: intent-statement.md(SM1)
  - id: BR1.6
    statement: core の DefinitionErrorKind::code と EnvFileErrorKind::code、converter の ConversionErrorKind::code を消す。テストは種類そのもの(HashSet の要素)で数える。仕様書の識別子との対応は、3 つの型の説明に「変種の名前の先頭を小文字にしたもの」と書く。識別子が重ならないことの確かめは、関数と一緒に無くなる。
    category: constraint
    applies_to: crates/core/src/error.rs、envfile/mod.rs、crates/converter/src/error.rs、lib.rs、crates/core/tests/env_types.rs、crates/converter/tests/nfr_security.rs、docs/spec/rust/02・03・08・10・12
    trigger: 段 1
    logic: inventory.md 1f
    violation: 本番の使用の無い識別子の関数が残る、または仕様書の識別子との対応がどこにも書かれない
    source: intent-statement.md(SM1)
  - id: BR1.7
    statement: 誤りの種類の一覧 ALL(DefinitionErrorKind、EnvFileErrorKind、ConversionErrorKind、runtime の CreateError、cli の UsageErrorKind)を本番から消し、各クレートの tests/common/kinds.rs に写しを置く(D2。決定(ユーザー、2026-11-23))。写しには、変種ごとの位置を返す網羅の match(_ を使わない)と、並びと重複のコンパイル時の確かめを付け、種類が増えればコンパイルで止まるようにする(D1。決定(ユーザー、2026-11-23))。CommandKind::ALL は本番が使うので残す。本番の型に derive を足さない。
    category: constraint
    applies_to: crates/core/src/error.rs、envfile/mod.rs、crates/converter/src/error.rs、crates/runtime/src/file_create.rs、crates/cli/src/types.rs、crates/{core,converter,cli}/tests/common/kinds.rs と、それを使うテスト、docs/spec/rust/02
    trigger: 段 1
    logic: inventory.md 1g、functional-spec.md §2.3
    violation: 本番に、テストだけが使う種類の一覧が残る、または写しに網羅の match が無い
    source: intent-statement.md(SM1、Q5)
  - id: BR1.8
    statement: cli のパニックのフックの差し替え口(on_panic、install_panic_hook_with と、その再公開)をなくす。本体は非公開の write_internal_error(&mut dyn Write) -> i32 にし、本体の 2 件のテストを名前と中身を変えずに src の単体テストへ移す。フックを設定するとパニックのとき 70 で終わることのテストは消し、自動のテストで確かめられなくなることを仕様書に記録する。cli の src の単体テストに panic!( を書かない。
    category: constraint
    applies_to: crates/cli/src/panic_hook.rs、lib.rs、crates/cli/tests/panic_hook.rs(消す)、docs/spec/rust/02・10・11、docs/spec/language-agnostic/11
    trigger: 段 1
    logic: inventory.md 1h、functional-spec.md §2.4
    violation: テストのための終了の差し替え口が残る、または確かめられなくなることが記録されない
    source: intent-statement.md(SM1、Q3)
  - id: BR1.9
    statement: cli のシグナルの SignalError::AlreadyRegistered と、プロセスで登録済みかの印(static REGISTERED)をなくす。SignalError は Failed だけの列挙型にする(D5。決定(ユーザー、2026-11-23))。Signals::start はプロセスで 1 回だけ呼ぶこと(本番は prepare_session の 1 か所)を説明に書く。公開の継ぎ目 run_cli_with と run_cli の説明にも、シグナルの監視の登録はプロセスで 1 回だけで、本番の SystemSignals で 2 回目の登録は契約の外(Windows は失敗して 70、UNIX 系は監視が 2 つ)と書く。テストは、1 つのテストで 1 回だけ登録する。場面の無くなるテストは消す。
    category: constraint
    applies_to: crates/cli/src/platform/mod.rs、platform/windows.rs、deps.rs、ctx.rs、lib.rs、crates/cli/tests/signals_real.rs、flow_run.rs、common/mod.rs、docs/spec/rust/02・07・10・11・12
    trigger: 段 1
    logic: inventory.md 1i、functional-spec.md §2.5
    violation: 同じプロセスで何度も登録するテストのための種類や印が残る、またはテストが 1 つで 2 回以上登録する
    source: intent-statement.md(SM1、Q4)
  - id: BR1.10
    statement: core の graph::node_state_of を overlay.rs の中だけの非公開に、graph::flow_edges を pub(crate) にする。テストは公開の build_graph(重ね合わせなし)の辺と、公開の overlay_from_state で確かめる(D8。決定(ユーザー、2026-11-23))。辺の取り出しの性能のテストは build_graph の辺を測り、名前と上限は変えず、前後の時間を記録する。
    category: constraint
    applies_to: crates/core/src/graph/mod.rs、edges.rs、overlay.rs、crates/core/tests/graph_edges.rs、graph_lookup_perf.rs、graph_types.rs、docs/spec/rust/02・05・11
    trigger: 段 1
    logic: inventory.md 1j
    violation: core の中でしか使わない関数が公開のまま残る
    source: intent-statement.md(SM1)

  # ---- 段 2: ふつうの整理 ----
  - id: BR2.1
    statement: 古くなった説明(inventory.md 2a の 12 か所)を実装に合わせる。.env の重複キーの警告は validate と graph が出す(コメントと仕様書の 6 か所)。run_validate の説明に .env の検査を書く。runtime と converter のモジュールの表を今のモジュールと責務に合わせる。「自動の再試行はしない」は、[job.retry] のリトライと矛盾して読めないように主語を限る。env_file_refs の Some の場合と Reporter::show の notice を正す。
    category: constraint
    applies_to: crates/cli/src/report/mod.rs、aux_commands.rs、lib.rs、pipeline.rs、crates/core/src/flow/next_step.rs、envfile/mod.rs、crates/runtime/src/lib.rs、crates/converter/src/lib.rs、docs/spec/rust/03・08・10、docs/spec/language-agnostic/03・08・10
    trigger: 段 2
    logic: inventory.md 2a
    violation: 実装や 04 章と食い違う説明が残る
    source: intent-statement.md(SM2)
  - id: BR2.2
    statement: PipelineState から definition_key と job_order を消す。ステートファイルの形(definition_key と job_order のキーの書き出し、definition_key の必須と型の確認、job_order と [[job]] の並びの一致の検査、resume_job_id が job_order に含まれることの検査、誤りの理由と順序)は 1 バイトも変えない。書き出しの job_order は job_states の ID の並びから、照合の記録の並びも job_states の ID から取る。
    category: constraint
    applies_to: crates/core/src/types.rs、consistency.rs、flow/record.rs、crates/runtime/src/state/reader.rs、state/writer.rs、PipelineState を手で組むテスト(inventory.md 2b)、docs/spec/rust/02・06・12、docs/spec/language-agnostic/06(D12。決定(ユーザー、2026-11-23))
    trigger: 段 2
    logic: inventory.md 2b
    violation: 製品で読まれない、または常に別の項目と同じ値の項目が型に残る、またはステートファイルのバイト列が変わる
    source: intent-statement.md(SM2)
  - id: BR2.3
    statement: runtime の CommandError から reason の項目を消し、理由は CommandErrorKind::reason() で引く。Display は kind の理由を書く(表示の文は変えない)。CommandError::new と、使われない CommandError::reason は置かない(D6。決定(ユーザー、2026-11-23))。
    category: constraint
    applies_to: crates/runtime/src/error.rs(単体テストを含む)、split.rs、crates/cli/tests/report_messages.rs、docs/spec/rust/07・10
    trigger: 段 2
    logic: inventory.md 2c
    violation: kind から決まる値の写しが項目として残る
    source: intent-statement.md(SM2)
  - id: BR2.4
    statement: core の ExitCodeClass を消し、classify_exit_code が JobOutcome を直接返す。分類と返す終了コードは変えない。
    category: constraint
    applies_to: crates/core/src/flow/classify.rs
    trigger: 段 2
    logic: inventory.md 2d
    violation: 詰め直すだけの中間の型が残る
    source: intent-statement.md(SM2)
  - id: BR2.5
    statement: 小さい整理を行う。core の EXIT_START_FAILURE・EXIT_SIGNAL_BASE・EXIT_UNLISTED_ZERO を classify.rs の中だけの非公開に、cli の run_dry_run を非公開に、runtime の inspect_run_state の常に真の条件を消し、runtime の writer.rs のテストだけの use を mod tests の中へ、converter の convert の Path の包み直しをやめる。
    category: constraint
    applies_to: crates/core/src/flow/classify.rs、crates/cli/src/dry_run.rs、crates/runtime/src/state/reader.rs、state/writer.rs、crates/converter/src/lib.rs、docs/spec/rust/02・10
    trigger: 段 2
    logic: inventory.md 2e
    violation: 使う所より広い公開範囲、常に真の条件、製品のコードの側のテストだけの use、不要な包み直しが残る
    source: intent-statement.md(SM2)
  - id: BR2.6
    statement: ステートが無いときの起動前の判断(記録は空、再開位置なし、end_reached は偽)を、core の flow::decide_from_state(job_id, definition, Option<&PipelineState>) の 1 か所にまとめる(D7。決定(ユーザー、2026-11-23))。cli の step::decide を消し、run・pipeline と、計画の plan_run・plan_pipeline がこの関数を使う。
    category: constraint
    applies_to: crates/core/src/flow/decide.rs、flow/mod.rs、lib.rs、plan/simulate.rs、crates/cli/src/step.rs、run.rs、pipeline.rs、docs/spec/rust/02・05
    trigger: 段 2
    logic: inventory.md 2f
    violation: 同じ match が 2 か所以上に残る
    source: intent-statement.md(SM2)

  # ---- 足すテスト ----
  - id: BR8.1
    statement: 足すテストは functional-spec.md §5 の N1(decide_from_state の単体テスト。見込み +1 件。D11。決定(ユーザー、2026-11-23))だけ。test-after(実装のコミットの後にテストのコミット)。正しい場合と、境界(end_reached、再開位置)と誤り(定義にないジョブ ID)を確かめる。
    category: validation
    applies_to: crates/core/src/flow/decide.rs
    trigger: BR2.6 の実装の後
    logic: functional-spec.md §5
    violation: 一覧に無いテストを足す、または増減が説明されない
    source: intent-statement.md(SM5)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run と graph の出力)を変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテストが、T 表の置き換えのほかは、期待を変えずに通る。2b の前後で、同じ定義で作ったステートファイルのバイト列が同じ
    violation: 既存のテストの期待する出力の文字列を変えないと通らない、またはステートファイルのバイト列が変わる
    source: intent-statement.md(SM4)
  - id: BR9.2
    statement: 既存のテストの変更は functional-spec.md §4 の T1〜T31 と §4.6 の 5 つの削除だけにする。変える・消すテストは、消す・変える名前のすべての参照を grep -rn で網羅した一覧による。テストの名前を変えるのは ★ T1、T2、T30 だけ(名前を変える方針は D4 で決定(ユーザー、2026-11-23)。個々の名前の変更は設計の承認で認めてもらう(承認待ち))。消すテストは §4.6 の 5 件だけ(消す方針は D3・D10 で決定(ユーザー、2026-11-23)。個々の削除は設計の承認で認めてもらう(承認待ち))。確かめられなくなるのは、パニックのフックの設定そのもの(§4.6)だけ。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§4.6、§7
    violation: 一覧に無いテストのファイルの変更、テスト名の変化、または理由と対の無い削除
    source: intent-statement.md(SM3、SM5)
  - id: BR9.3
    statement: 段の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,161 − 消したテスト 5 + 足したテスト 1 = 1,157」に一致すること(macOS)。テスト名の一覧の差は、5 つの削除、N1 の追加、★ の 3 つの名前の変更、単体テストへ移した 2 件の場所の変更だけ。各項目の完了は、本番のコードからの参照が 0 件であることを検索で確かめる。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 各段の終わり
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、テスト名の変化、または参照の残り
    source: intent-statement.md(SM3、SM5)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。runtime の製品のコードにパニックしうる書き方を、cli の src に panic!( を書かない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(SM6)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。仕様書は rust と language-agnostic だけを直し、版の注記を足さずに今の形を書く。凍結した仕様書は触らない。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、docs/spec/rust、docs/spec/language-agnostic
    trigger: 常に
    logic: functional-spec.md §1
    violation: 版が変わる、版の注記が足される、または凍結した仕様書が変わる
    source: intent-statement.md(SM4)
```
