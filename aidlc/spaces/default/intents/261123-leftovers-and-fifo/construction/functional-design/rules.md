# ビジネスルール(Rules): 残りの整理と読み込みの順序のそろえ

詳細の場所は `inventory.md`、承認が要るテストの変更(T 表)、消すテストの対の表、足すテスト(N 表)、性能の基準、件数の見込み、決めてほしい点は `functional-spec.md` の §4、§4.6、§5、§6、§10。

```yaml
rules:
  # ---- 段 1: 動きを変えない整理 ----
  - id: BR1.1
    statement: 古くなった説明(inventory.md 1a の 13 か所)を実装に合わせる。runtime の run_job の説明に JobDefinition と「job は結果の job_id と command のため」を書かない。経緯の文を消す。到達しない守りは残し、理由を「parse_args を通る限り起こらない、内部の不変条件の守り」と書く。
    category: constraint
    applies_to: crates/runtime/src/lib.rs、runner.rs、types.rs、state/reader.rs、state/writer.rs、crates/cli/src/report/messages.rs、report/mod.rs、step.rs、run.rs、dry_run.rs、aux_commands.rs、docs/spec/rust/05
    trigger: 段 1
    logic: inventory.md 1a
    violation: 実装と食い違う説明、または事実でない守りの理由が残る
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: 使われないもの(apply_interrupt_in、EffectiveEnv の job_id と names()、テストの補助の使わない引数(D2。決定(ユーザー、2026-11-23))、cli の interrupted_signal、runtime の command_line)と、模擬の分類の二重の計算、decide_before_run の常に真の条件をなくす。classify_exit_code と ExitCodeClass は classify.rs の中だけの非公開にする。
    category: constraint
    applies_to: crates/core/src/flow/record.rs、envfile/mod.rs、envfile/merge.rs、plan/simulate.rs、flow/classify.rs、flow/decide.rs、crates/cli/src/step.rs、crates/runtime/src/split.rs
    trigger: 段 1
    logic: inventory.md 1b
    violation: 製品のコードから読まれない関数・フィールド・引数、または同じ値の二重の計算が残る
    source: intent-statement.md(SM1)
  - id: BR1.3
    statement: 「一時ファイルに書いて置き換える」処理は runtime の temp_file::replace_via_temp の 1 か所にし、ステートと init --force が使う。上限つきの読み取りは新しい bounded_read.rs(read_limited と形 B の read_regular_file)の 1 か所にし、.env が使う(段 2 でステートも使う)。ステートの読み込みの段(read_state_file_raw、parse_state_for)をなくす。build_retry は値を移し、欠けていればジョブを組み立てない(D3。決定(ユーザー、2026-11-23))。記録の hint は Option<usize> にそろえる(D4。決定(ユーザー、2026-11-23))。runtime の製品のコードにパニックしうる書き方を使わない。
    category: constraint
    applies_to: crates/runtime/src/temp_file.rs、state/writer.rs、file_create.rs、bounded_read.rs、state/reader.rs、envfile.rs、crates/core/src/definition/walk/retry.rs、walk/mod.rs、definition/intake.rs、flow/record.rs、crates/converter/src/yaml_write.rs
    trigger: 段 1
    logic: inventory.md 1c
    violation: fs::rename または read_to_end が 2 か所以上に残る、段だけの関数が残る、または unwrap・expect などを runtime の製品のコードに書く
    source: intent-statement.md(SM1)
  - id: BR1.4
    statement: cli のトレイト Runtime の run_job は job を受け取らず、runtime の run_job(line, env, signal) と同じ形にする。理由は、job の引数は本番では不要で、テストの偽物の都合(job.id の参照)だけで残っており、テストのコードの都合で本番のコードを曲げるべきではないため(ユーザーの判断、2026-11-23)。消えるのはトレイトの run_job に渡す引数だけで、retry.rs の中で job を使う所(merge_env の環境変数の合成、retry_decision のリトライの判断、知らせのジョブ ID)と run_job_with_retry の引数は今のまま残る。テストの偽物は、テストの側の事情として、env の CONDUCTOR_JOB_ID からジョブ ID を取り、今と同じ記録(run:<id>、skip-run:<id>、run_envs、run_lines)、台本の引き当て、during_job・after_job を保つ。CONDUCTOR_JOB_ID が無ければ偽物はパニックで止める(D1。決定(ユーザー、2026-11-23))。
    category: constraint
    applies_to: crates/cli/src/deps.rs、retry.rs、crates/cli/tests/common/mod.rs、docs/spec/rust/02
    trigger: 段 1
    logic: inventory.md 1d、functional-spec.md §2.3
    violation: トレイトや SystemRuntime に使わない job の引数が残る、retry.rs の job の使い方が変わる、または偽物の記録が変わって既存のテストの期待が変わる
    source: intent-statement.md(SM1、Q3)
  - id: BR1.5
    statement: core の flow・plan・envfile・graph の下位モジュールを非公開にし、関数と型は今の再公開の名前で出す。定数 EXIT_START_FAILURE・EXIT_SIGNAL_BASE・EXIT_UNLISTED_ZERO・ASSUME_EXIT_CODE_MIN・ASSUME_EXIT_CODE_MAX・RESERVED_ENV_PREFIX・MAX_REPORTED_ERRORS を pub(crate) にする。外から使われる MAX_ENV_FILE_BYTES・MAX_ENV_ENTRIES は狭めない。runtime の pub(crate) のうちモジュールの中でしか使わないもの(inventory.md 1f の表)を非公開にする。狭めたあと、ビルド・テスト・clippy と検索で外からの参照が無いことを確かめる。
    category: constraint
    applies_to: crates/core/src/flow/mod.rs、flow/classify.rs、plan/mod.rs、envfile/mod.rs、graph/mod.rs、lib.rs、crates/runtime/src/state/writer.rs、temp_file.rs、split.rs、runner.rs、bounded_read.rs、docs/spec/rust/02・03・05・10
    trigger: 段 1
    logic: inventory.md 1e、1f、§8
    violation: 外から参照の無い pub または pub(crate) が残る、または外から使われるものを狭めてテストの期待を変える
    source: intent-statement.md(SM1)
  - id: BR1.6
    statement: 名前と型を使い方に合わせる。cli の escape_path を escape_full にする(D6。決定(ユーザー、2026-11-23))。run ID の有無は bool でなく StateContext(Read / ReadRunId)で渡す(for_read(Option<&RunId>)、ConsistencyMismatch.context、reset_hint(StateContext)。D5。決定(ユーザー、2026-11-23))。converter の RawNode の行番号は Option<usize>(0 の番兵を使わない)。ConversionError.reason は &'static str。表示の文は 1 文字も変えない。
    category: constraint
    applies_to: crates/cli/src/report/escape.rs、report/mod.rs、report/messages.rs、report/plan_text.rs、report/graph_text.rs、ctx.rs、aux_commands.rs、crates/converter/src/raw.rs、toml_read.rs、yaml_read.rs、error.rs、intake.rs、resolve.rs、docs/spec/rust/02・08・10・11
    trigger: 段 1
    logic: inventory.md 1g
    violation: escape_path・with_run_id・line: 0・reason: String が残る、または表示が変わる
    source: intent-statement.md(SM1)
  - id: BR1.7
    statement: 入力の大きさの二乗の時間がかかる確認をなくす。定義の env の表の大文字小文字の検査は小文字の名前の HashSet、遷移の重複の確認は種類ごとの真偽値(公開の型 TransitionOn に derive を足さない)、env_file_refs の重複の確認は HashSet で行う。cli の -v の図の辺は出どころのジョブで 1 回まとめ、record_skip は土台を複製しない。誤りの種類・位置・文、並び、表示は今と同じ。
    category: constraint
    applies_to: crates/core/src/definition/walk/env.rs、walk/mod.rs、envfile/mod.rs、crates/cli/src/report/graph_text.rs、step.rs
    trigger: 段 1
    logic: inventory.md 1h
    violation: seen.iter().any、out.iter().any、out.contains、ジョブごとの全辺の filter、base.cloned() が残る
    source: intent-statement.md(SM2)
  - id: BR1.8
    statement: テストの補助の重複をまとめる。core の src の単体テストの定義の手組みは #[cfg(test)] の test_support に、cli の tests の FakeEnv の言語分け・Deps を組んで run_cli_with を呼ぶ形・state_error は common に(common::state_error の理由は ASCII の "reason"。D12。決定(ユーザー、2026-11-23))、runtime の tests の def・put・paths と、run_job の名残のコマンドの取り出しは common に。補助の名前と引数を保ち、テストの本体と期待の文字列は変えない。
    category: constraint
    applies_to: crates/core/src/test_support.rs と各モジュールの単体テスト、crates/cli/tests/common/mod.rs、report_messages.rs、flow_init.rs、crates/runtime/tests/common/mod.rs、runner_*.rs、nfr_robust.rs、state_*.rs、run_id_state.rs、crates/runtime/src/runner.rs
    trigger: 段 1
    logic: inventory.md 1i-1〜1i-3、1i-5
    violation: 同じ補助の写しが残る、またはテストの期待が変わる
    source: intent-statement.md(SM1、SM5、Q2)
  - id: BR1.9
    statement: 深さの境界の重複したテストは、functional-spec.md §4.6 の 4 つ(core の単体 2、converter の単体 1 と統合 1)だけを消し、消すテストと同じことを確かめて残るテストを対にして示す(D10。消す方針の決定(ユーザー、2026-11-23)。個々のテストの削除は設計の承認で認めてもらう(承認待ち))。TOML の深い入力の単体テストと、見出しの深さの単体テストは残す。
    category: validation
    applies_to: crates/core/src/definition/depth.rs、crates/converter/src/yaml_read.rs、crates/converter/tests/read_resolve.rs
    trigger: 段 1
    logic: inventory.md 1i-4、functional-spec.md §4.6
    violation: 表に無いテストを消す、または残るテストと対にせずに消す
    source: intent-statement.md(SM5、Q2)

  # ---- 計算量の確かめ ----
  - id: BR2.1
    statement: 性能のテスト P1(定義の env の表 10,000 項目の parse_definition、上限 350 ms)を足す。結果の中身(10,000 項目が書いた順、大文字小文字の衝突は 1 件の誤り)も確かめる。
    category: validation
    applies_to: crates/core/tests/definition_perf.rs
    trigger: 1h-1 の実装の後
    logic: functional-spec.md §5.2
    violation: 上限を超える、または前後の時間が記録されない
    source: intent-statement.md(SM2、Q6)
  - id: BR2.2
    statement: 性能のテスト P2(1 つのジョブに success 20,000 個、warning 20,000 個の遷移の parse_definition、上限 1,600 ms)を足す。結果の中身(duplicateTransition が 39,998 件、位置の順)も確かめる。
    category: validation
    applies_to: crates/core/tests/definition_perf.rs
    trigger: 1h-2 の実装の後
    logic: functional-spec.md §5.2
    violation: 上限を超える、または前後の時間が記録されない
    source: intent-statement.md(SM2、Q6)
  - id: BR2.3
    statement: 性能のテスト P3(env_file_refs、記述 10,000 個 + 同じ 10,000 個、上限 40 ms)を足す。結果の中身(10,000 個が書いた順)も確かめる。
    category: validation
    applies_to: crates/core/tests/env_refs_perf.rs
    trigger: 1h-3 の実装の後
    logic: functional-spec.md §5.2
    violation: 上限を超える、または前後の時間が記録されない
    source: intent-statement.md(SM2、Q6)
  - id: BR2.4
    statement: 性能の上限は、直す前の測定(inventory.md §0.2、debug、5 回の中央値)から見積もった直した後の時間の約 5 倍の余裕のある上限にし、大きな退行だけを止める(D9。決定(ユーザー、2026-11-23)。前回の D6 と同じ考え方)。直す前の形に戻ったことはテストで確実には検知できないので、直す前と後の時間を code-summary と test-results に記録する。CI の coverage(計測つき、debug、並行)の余裕は手元で測れないので、実装の後に CI で P1〜P3 が出した時間を記録し、上限との余裕が 3 倍を切っていたら上限を上げる(値と理由を記録する)。-v の図と record_skip は性能のテストを足さず、一時の測定で前後の時間を記録する(D8。決定(ユーザー、2026-11-23))。
    category: validation
    applies_to: 性能のテストと記録
    trigger: 1h の各項目の実装の後
    logic: functional-spec.md §5.2
    violation: 前後の時間の記録が無い、または上限の根拠が書かれない
    source: intent-statement.md(SM2、Q6)

  # ---- 段 2: 利用者から見える変更と版 ----
  - id: BR3.1
    statement: converter の読み込みを、定義ファイルと同じ形 B(パスで種別 → 通常のファイルだけ開く → 開いたハンドルで種別と大きさ → 上限 + 1 バイトまで → UTF-8)にする。書き手のいない名前付きパイプは開かずに unreadable(通常のファイルではありません)。無い・ディレクトリ・権限なし・上限超・UTF-8 でないの種類と理由は変えない。仕様書に「v0.9.7 までは、先に開いたため、書き手のいない名前付きパイプで止まった」と注記する。名前付きパイプのテストは tests/ に置く(src のテストに cfg(unix) を増やさない)。
    category: constraint
    applies_to: crates/converter/src/intake.rs、crates/converter/tests/convert_ac.rs、docs/spec/rust/09、docs/spec/language-agnostic/09・11
    trigger: 段 2
    logic: inventory.md 2a
    violation: 開いてから種別を見る、名前付きパイプで止まる、またはほかの場合の種類・理由が変わる
    source: intent-statement.md(SM3、Q5)
  - id: BR3.2
    statement: ステートの読み込み(read_state、read_state_raw、reset --all の inspect_run_state)を、.env と共有する bounded_read::read_regular_file(形 B)にする。名前付きパイプとディレクトリのステートは unreadable で、理由は今の「開く、または読み取ることができません」(D7。決定(ユーザー、2026-11-23))。無い・長さ 0 は空、ちょうど 1 MiB は受理、通常のファイルが超えれば tooLarge、読む権限のないものは 1 MiB 超でも unreadable は変えない。/dev/zero などの文字デバイスへのリンクのステートも開かずに unreadable になり、既存のテストの期待が変わる(★ T23。承認待ち)。read_limited は、ステートの read_bounded が無くなるこのコミットで非公開にする。仕様書 06 の §5.1 の表を改訂し「v0.9.7 までは…」と注記する。runtime の製品のコードにパニックしうる書き方を使わない。
    category: constraint
    applies_to: crates/runtime/src/state/reader.rs、bounded_read.rs、docs/spec/rust/06・02・03、docs/spec/language-agnostic/06
    trigger: 段 2
    logic: inventory.md 2b
    violation: 開いてから種別を見る、名前付きパイプで止まる、ディレクトリの表示が変わる、または unwrap などを使う
    source: intent-statement.md(SM3、Q5)
  - id: BR3.3
    statement: 文字列の command の先頭の語が空白だけ(trim して空。例 '  ')なら、配列の先頭の要素と同じく Empty(定義の誤り、終了コード 64)にする。空白を含むが空白だけでない先頭の語は今のまま。誤りの表示に入力の値を含めない。仕様書 07 に「v0.9.7 までは…起動の失敗(127)だった」と注記する。
    category: constraint
    applies_to: crates/runtime/src/split.rs、docs/spec/rust/07・12、docs/spec/language-agnostic/07・11
    trigger: 段 2
    logic: inventory.md 2c
    violation: '  ' が分けられて起動の失敗(127)になる、または配列と文字列で扱いが食い違う
    source: intent-statement.md(SM3、Q5)
  - id: BR3.4
    statement: 最後のコミットで版を 0.9.8 にする(7239aa2 と同じ範囲: Cargo.toml × 4、Cargo.lock、binary_basic.rs の版の期待、仕様書の版の表記)。「v0.9.7 までは」「v0.9.7 以降」などの注記は上げない。タグとリリースはしない。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、crates/cli/tests/binary_basic.rs、docs/spec/rust、docs/spec/language-agnostic
    trigger: 段 2 の最後
    logic: inventory.md 2d
    violation: 版の表記が食い違う、または注記の版まで上げる
    source: intent-statement.md(SM4、memory:M6)

  # ---- 足すテスト ----
  - id: BR8.1
    statement: 足すテストは functional-spec.md §5.1 の N1〜N8(見込み +8 件)。test-after(実装のコミットの後にテストのコミット)。N4・N5・N6・N7 は直す前の木で落ち、直した後に通ることを記録する。N8(bounded_read の read_opened の単体テスト)は、終わりのない読み手でも上限 + 1 バイトで止まり TooLarge になることを、ファイルを使わずに確かめる(★ T23 で /dev/zero のテストが「開かずに unreadable」に変わったあとも、この性質を保つため)。
    category: validation
    applies_to: テスト
    trigger: 各項目の実装の後
    logic: functional-spec.md §5
    violation: 一覧に無いテストを足す、または増減が説明されない
    source: intent-statement.md(SM2、SM3、SM5、Q6)

  # ---- 全体 ----
  - id: BR9.1
    statement: 段 1 は、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run と graph の出力)を変えない。利用者から見える変更は段 2 の 3 つと版だけ。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテストがそのまま通る(T 表の置き換えを除く)
    violation: 段 1 で既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM1、SM2、SM3)
  - id: BR9.2
    statement: 既存のテストの変更は functional-spec.md §4 の T1〜T24 と §4.6 の消すテストだけにする。確かめ方を置き換えるのは ★ T1b、期待を変えるのは ★ T23(段 2 の 2b。/dev/zero へのリンクのステートの期待を TooLarge から Unreadable に、テスト名を「文字デバイスへのリンクのステートは開かずに_unreadable」に。承認待ち)と ★ T22(版)だけ。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§7
    violation: 一覧に無いテストのファイルの変更、またはテスト名の変化
    source: intent-statement.md(SM5、Q2)
  - id: BR9.3
    statement: 段の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,157 − 消したテスト 4 + 足したテスト 8 = 1,161」に一致すること(macOS)。テスト名の一覧の差は §4.6 の 4 つの削除、N1〜N8 の追加、★ T23 の 1 つの名前の変更だけ。
    category: validation
    applies_to: テスト
    trigger: 各段の終わり
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、またはテスト名の変化
    source: intent-statement.md(SM5、Q1、Q2)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。ステートに環境変数を書かない。runtime の製品のコード(src の #[cfg(test)] より前)に .unwrap()・.expect(・panic!(・unreachable!(・todo!(・unimplemented!( を書かない(runtime の nfr_source.rs が検査する)。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(SM6)
```
