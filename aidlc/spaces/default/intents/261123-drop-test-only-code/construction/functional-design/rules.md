# ビジネスルール(Rules): テストだけが使う本番のコードの削除

詳細の場所は `inventory.md`、承認が要るテストの変更(T 表)、消すテストの対の表、件数の見込み、決めてほしい点は `functional-spec.md` の §4、§4.6、§6、§10。

```yaml
rules:
  # ---- 削除(方針: テストのコードの都合で本番のコードを曲げない。本番にテストのための要素を足さない) ----
  - id: BR1.1
    statement: core の Command の From<&str> と From<String> を消す。テストは本番の読み取りと同じく Command::Line(..) を書く。runtime のテストの補助のうち、文字列だけで呼ばれる common::job と common::command_line は引数を impl Into<String> にし、文字列と配列の両方で呼ばれる runner_basic.rs の run は Command を受け取る(D5。決定: 推奨のとおり。ユーザー、2026-11-24)。From だけを試す単体テスト 1 件は消す。
    category: constraint
    applies_to: crates/core/src/types.rs(単体テストを含む)、core/src/test_support.rs、crates/core/tests/{common,graph_common}/mod.rs、definition_valid.rs、nfr_security.rs、crates/runtime/src/runner.rs と split.rs の単体テスト、crates/runtime/tests/common/mod.rs、runner_basic.rs、nfr_robust.rs、nfr_content.rs
    trigger: 削除の段
    logic: inventory.md 1a、functional-spec.md §2.4
    violation: 本番の使用の無い変換の実装が残る、または Into<Command> を求めるテストの補助が残る
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: cli の RunIdArg::as_str を消す。テストは RunIdArg の値そのもの(Empty、または Id(RunId))で比べる。
    category: constraint
    applies_to: crates/cli/src/types.rs、crates/cli/tests/run_id_arg.rs、args_grammar.rs、docs/spec/rust/02
    trigger: 削除の段
    logic: inventory.md 1b
    violation: テストだけが呼ぶ文字列の取り出しが残る
    source: intent-statement.md(SM1)
  - id: BR1.3
    statement: runtime の InterruptSignal から is_raised、PartialEq、Eq を消す。テストは signal_number() で確かめる(None は立っていない)。Debug の raised の項目は signal_number().is_some() で書き、出力を変えない(D8。決定: 推奨のとおり。ユーザー、2026-11-24)。Default は、消すと clippy の new_without_default で止まるため残す(D1。決定: 推奨のとおり。ユーザー、2026-11-24)。
    category: constraint
    applies_to: crates/runtime/src/types.rs(単体テストを含む)、crates/cli/tests/signals_real.rs、signals_order.rs、docs/spec/rust/02・07
    trigger: 削除の段
    logic: inventory.md 1c、functional-spec.md §10 の D1・D8
    violation: 本番が使わない問い合わせや等価性が残る、または Debug の出力が変わる
    source: intent-statement.md(SM1)
  - id: BR1.4
    statement: 本番で使われない Default(core の JobStatus、runtime の Durability、core の graph::StateOverlay、cli の Language)を、derive と #[default] ごと消す。空の StateOverlay は項目を書いて作る。Default を試すテスト(Durability の 1 件、StateOverlay の 1 件(D3。消す方針は決定: 推奨のとおり。ユーザー、2026-11-24。個々の削除は functional-spec.md §4.6 として設計の承認で認めてもらう(承認待ち)))は消し、JobStatus::default() の 1 行は消す。
    category: constraint
    applies_to: crates/core/src/types.rs、core/src/graph/mod.rs、crates/runtime/src/types.rs、crates/cli/src/types.rs、crates/core/tests/graph_overlay.rs、graph_types.rs、docs/spec/rust/02
    trigger: 削除の段
    logic: inventory.md 1d
    violation: 本番で使われない既定の値が残る
    source: intent-statement.md(SM1)
  - id: BR1.5
    statement: core の sanitize(escape_display の上限つきの包み)と再公開を消し、呼ぶ所は escape_display(x, Some(n)) にする。cli の再公開と core の包みが同じことの確かめは消す。converter の sanitize(core に依存しないための写し)は変えない。
    category: constraint
    applies_to: crates/core/src/error.rs(単体テストを含む)、core/src/lib.rs、crates/cli/src/report/escape.rs の説明、crates/cli/tests/escape_table.rs、docs/spec/rust/02・08・11・12
    trigger: 削除の段
    logic: inventory.md 1e
    violation: 別の関数を呼ぶだけの公開の関数が残る
    source: intent-statement.md(SM1)
  - id: BR1.6
    statement: core の EnvFileError::reason() を消す。表示は kind.reason() を使い、文は変えない。テストは err.kind.reason() で確かめる。
    category: constraint
    applies_to: crates/core/src/envfile/mod.rs、crates/core/tests/env_file_parse.rs、env_types.rs、docs/spec/rust/02・10
    trigger: 削除の段
    logic: inventory.md 1f
    violation: 項目の写しを返すだけのメソッドが残る
    source: intent-statement.md(SM1)

  # ---- 公開範囲を狭める ----
  - id: BR2.1
    statement: core の decide_before_run を decide.rs の中だけの非公開の関数にし(D6。決定: 推奨のとおり。ユーザー、2026-11-24)、flow と根の再公開を消す。テストは公開の decide_from_state で確かめる。core の模型 Sim は持っている状態から PipelineState を組んで渡す。runtime の flow_with_state.rs の decide の写しは消し、decide_from_state を直接呼ぶ(D7。決定: 推奨のとおり。ユーザー、2026-11-24)。この関数を指すドキュメントのリンクを直す。
    category: constraint
    applies_to: crates/core/src/flow/decide.rs、flow/mod.rs、flow/next_step.rs、lib.rs、crates/core/tests/flow_sequences.rs、crates/runtime/tests/flow_with_state.rs、crates/cli/src と crates/runtime/src の説明、docs/spec/rust/02・12
    trigger: 狭める段
    logic: inventory.md 2a
    violation: テストだけが外から呼ぶ関数が公開のまま残る、または同じ判断の写しがテストに残る
    source: intent-statement.md(SM1)
  - id: BR2.2
    statement: core の flow::classify を pub(super) にし、再公開を消す(使うのは next_step.rs)。
    category: constraint
    applies_to: crates/core/src/flow/classify.rs、flow/mod.rs、docs/spec/rust/02
    trigger: 狭める段
    logic: inventory.md 2b
    violation: crate の中でしか使わない関数が公開のまま残る
    source: intent-statement.md(SM1)
  - id: BR2.3
    statement: core の MAX_RUN_ID_LEN を run_id.rs の中だけの定数に、is_valid_env_name、is_reserved_env_name、JobDefinition::effective_success_codes を pub(crate) にし、根の再公開を消す(D6。決定: 推奨のとおり。ユーザー、2026-11-24)。effective_success_codes を使う統合テストは、読み取りの結果の success_exit_codes で確かめる。
    category: constraint
    applies_to: crates/core/src/run_id.rs、envfile/mod.rs、types.rs、lib.rs、crates/core/tests/definition_valid.rs、docs/spec/rust/02
    trigger: 狭める段
    logic: inventory.md 2c
    violation: crate の中でしか使わない定数や関数が外から見える
    source: intent-statement.md(SM1)
  - id: BR2.4
    statement: cli の report::escape の escape_display の再公開をやめ、非公開の use にする。テストは本体の conductor_core::escape_display を直接使う。escape_name と escape_full は公開のまま。
    category: constraint
    applies_to: crates/cli/src/report/escape.rs、crates/core/src/error.rs の説明、crates/cli/tests/escape_table.rs、docs/spec/rust/02・08・11
    trigger: 狭める段
    logic: inventory.md 2d
    violation: 同じ関数への公開の入口が 2 つ残る
    source: intent-statement.md(SM1)
  - id: BR2.5
    statement: cli の lib.rs の args、deps、panic_hook、types を非公開のモジュールにする(中の公開の項目は根で再公開されている)。モジュールの表のリンクを、非公開の項目へのリンクにならない書き方にする。
    category: constraint
    applies_to: crates/cli/src/lib.rs、docs/spec/rust/02
    trigger: 狭める段
    logic: inventory.md 2e
    violation: 根の再公開と同じものへの、モジュールのパスの入口が残る
    source: intent-statement.md(SM1)
  - id: BR2.6
    statement: cli の根の GraphOptions と DryRunOptions の再公開を消す。CliCommand の項目の形は変えない(範囲の外)。
    category: constraint
    applies_to: crates/cli/src/lib.rs、docs/spec/rust/02
    trigger: 狭める段
    logic: inventory.md 2f、functional-spec.md §3
    violation: 名前で使われない型の再公開が残る
    source: intent-statement.md(SM1)

  # ---- 判断が要ったもの(Q2〜Q5) ----
  - id: BR3.1
    statement: core の RunIdError を、種類を持たない誤りの型 1 つ(単位構造体。D2。決定: 推奨のとおり。ユーザー、2026-11-24)にする。RunId::parse は、空、64 バイト超、A-Za-z0-9._- 以外の文字、先頭が . のどれかで誤りを返す。単体テストは、4 つの場合の入力(空、65 文字、先頭の .、使えない文字)が通らないことを確かめる。language-agnostic の 4 つの名前は変えない(D4。決定: 推奨のとおり。ユーザー、2026-11-24)。
    category: constraint
    applies_to: crates/core/src/run_id.rs(単体テストを含む)、docs/spec/rust/02・05・10・12
    trigger: 判断の段
    logic: inventory.md 3a、functional-spec.md §2.5
    violation: 誰も見ない誤りの種類が残る、または 4 つの場合のどれかが通る
    source: intent-statement.md(SM2)
  - id: BR4.1
    statement: runtime は core の関数と型(check_consistency、apply_interrupt、apply_outcome、apply_skip、initial_state、ConsistencyResult、MismatchKind)を再公開しない。使う所(cli の本番、runtime と cli のテスト)は conductor_core から use する。conductor_runtime を通したこれらの名前の参照は 0 件。runtime の説明のリンクを直す。
    category: constraint
    applies_to: crates/runtime/src/lib.rs、types.rs、state/mod.rs、crates/core/src/consistency.rs の説明、crates/cli/src/ctx.rs、pipeline.rs、step.rs、report/mod.rs、report/messages.rs、crates/runtime/tests/common/mod.rs、flow_with_state.rs、nfr_content.rs、nfr_performance.rs、crates/cli/tests/report_messages.rs、docs/spec/rust/02・05・06・10
    trigger: 判断の段
    logic: inventory.md 3b、functional-spec.md §2.6
    violation: 同じものへの入口が 2 つ残る、または conductor_runtime を通した参照が残る
    source: intent-statement.md(SM3)
  - id: BR5.1
    statement: 誤りの種類の Display と Error(core の DefinitionErrorKind と EnvFileErrorKind の Display と Error、converter の ConversionErrorKind の Error)を消す。converter の ConversionErrorKind の Display は本番が使うので残す。テストは label() で確かめる。
    category: constraint
    applies_to: crates/core/src/error.rs、envfile/mod.rs、crates/converter/src/error.rs、crates/core/tests/env_types.rs、docs/spec/rust/03・08・10
    trigger: 判断の段
    logic: inventory.md 3c
    violation: 本番が使わない種類の表示や誤りの実装が残る
    source: intent-statement.md(SM4)
  - id: BR5.2
    statement: 本番で使われない Hash(converter の TargetFormat と ConversionErrorKind、core の EnvFileErrorKind と DefinitionErrorKind)を derive から消す。種類を集めるテストは Vec に入れて contains で確かめる。
    category: constraint
    applies_to: crates/core/src/error.rs、envfile/mod.rs、crates/converter/src/error.rs、types.rs、crates/core/tests/nfr_security.rs、crates/converter/tests/nfr_security.rs
    trigger: 判断の段
    logic: inventory.md 3d
    violation: 本番が使わない Hash が残る
    source: intent-statement.md(SM4)
  - id: BR5.3
    statement: ライブラリの誤りの型(core の DefinitionError と EnvFileError、runtime の CommandError と StateError と CreateError、converter の ConversionError)の Display と std::error::Error は残す。
    category: constraint
    applies_to: crates/core/src、crates/runtime/src、crates/converter/src の誤りの型
    trigger: 常に
    logic: functional-spec.md §1
    violation: ライブラリの誤りの型の Display または Error が消える
    source: intent-statement.md(SM5)

  # ---- 足すテスト ----
  - id: BR8.1
    statement: 新しいテストは足さない(見込み 0 件)。消す・狭める項目の代わりは、既存のテストの書き換えと、公開の入口の既存のテストで持つ。実装で足す必要が出たら、その数と理由を記録する。
    category: validation
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §5
    violation: 一覧に無いテストを足す、または増減が説明されない
    source: intent-statement.md(SM8)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run と graph の出力)を変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテストが、T 表の書き換えのほかは、期待を変えずに通る
    violation: 既存のテストの期待する出力の文字列を変えないと通らない
    source: intent-statement.md(SM7)
  - id: BR9.2
    statement: 既存のテストの変更は functional-spec.md §4 の T1〜T32 と §4.6 の 4 つの削除だけにする。変える・消すテストは、消す・狭める名前のすべての参照を grep -rn で網羅し(tests の下の補助のディレクトリと、src の #[cfg(test)] の単体テストを含む)、写しの木で本番だけを変えたコンパイルのエラーの場所と照らした一覧による。テストの名前は変えない。消すテストは §4.6 の 4 件だけで、個々の削除は設計の承認で認めてもらう(承認待ち)。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§4.6、inventory.md §0.2
    violation: 一覧に無いテストのファイルの変更、テスト名の変化、または理由と対の無い削除
    source: intent-statement.md(SM6、SM8)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,157 − 消したテスト 4 + 足したテスト 0 = 1,153」に一致すること(macOS)。テスト名の一覧の差は、4 つの削除だけ。各項目の完了は、inventory.md の完了の判定の検索で確かめる。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 実装の完了
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、テスト名の変化、または参照の残り
    source: intent-statement.md(SM6、SM8)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と、Windows の対象のコンパイル)、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。runtime の製品のコードにパニックしうる書き方を、cli の src に panic!( を書かない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(SM9)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。仕様書は rust と language-agnostic だけを直し、版の注記を足さずに今の形を書く。凍結した仕様書は触らない。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、docs/spec/rust、docs/spec/language-agnostic
    trigger: 常に
    logic: functional-spec.md §1
    violation: 版が変わる、版の注記が足される、または凍結した仕様書が変わる
    source: intent-statement.md(SM7)
  - id: BR9.6
    statement: 消す・狭める名前を指すドキュメントコメントと仕様書の記述を、同じコミットで直す。cargo doc --workspace --no-deps の警告は、基準の 5 件から増やさない。
    category: validation
    applies_to: crates/*/src のドキュメントコメント、docs/spec/rust、docs/spec/language-agnostic
    trigger: 各コミット
    logic: inventory.md 各項目の「ドキュメントコメント」「仕様書」、§0.2
    violation: 消した名前を指す説明やリンクが残る
    source: intent-statement.md(SM6)
```
