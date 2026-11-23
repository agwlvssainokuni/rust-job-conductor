# ビジネスルール(Rules): 型と構造の整理・計算量・利用者から見える変更

詳細の場所は `inventory.md`、承認が要るテストの変更(T 表)と足すテスト(N 表)、性能の基準、件数の見込み、決めてほしい点は `functional-spec.md` の §4、§5、§6、§10。

```yaml
rules:
  # ---- 段 1: 型と構造(動きを変えない) ----
  - id: BR1.1
    statement: 計画の再開位置 ResumeInfo を、あり得る 3 つの場合だけを表す列挙型(FromStart、AfterStop(ジョブID)、Individual(ジョブID))にし、ResumeKind を消す。cli の計画の表示は網羅の match で、_ の枝を持たない。
    category: constraint
    applies_to: crates/core/src/plan/mod.rs、plan/simulate.rs、crates/cli/src/report/plan_text.rs
    trigger: 段 1
    logic: inventory.md 1a
    violation: 種類とジョブ ID の組で、ジョブ ID の無い AfterStop・Individual を作れる
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: 模擬の結果は PlanDisposition::WillRun(SimulatedResult) が持ち、PlanEntry.result を消す。実行しないジョブは結果を持てない。
    category: constraint
    applies_to: crates/core/src/plan、crates/cli/src/report/plan_text.rs、messages.rs
    trigger: 段 1
    logic: inventory.md 1b
    violation: WillRun でないのに結果がある、または WillRun なのに結果がない組を作れる
    source: intent-statement.md(SM1)
  - id: BR1.3
    statement: 図の節は kind(NodeKind = TransitionTarget の別名。Job(ID) または End)で表し、job_id と is_end の二重をなくす(D1。決定(ユーザー、2026-11-23))。
    category: constraint
    applies_to: crates/core/src/graph、crates/cli/src/report/graph_text.rs、graph_mermaid.rs
    trigger: 段 1
    logic: inventory.md 1c
    violation: job_id が無いのに is_end が偽、などの食い違う節を作れる
    source: intent-statement.md(SM1)
  - id: BR1.4
    statement: リトライの倍率と上限は Backoff として 1 つの Option にまとめる。片方だけの組を型で作れない。待ち時間の計算に片方だけの枝を持たない。
    category: constraint
    applies_to: crates/core/src/types.rs、definition/walk/retry.rs、flow/retry.rs、crates/cli/src/report/messages.rs
    trigger: 段 1
    logic: inventory.md 1d
    violation: multiplier と max_interval_secs が別々の Option のまま
    source: intent-statement.md(SM1)
  - id: BR1.5
    statement: run の計画の次の動き NextStepNote は、NextJob(次のジョブとスキップ)、EndReached、Completed、Stopped の列挙型にする。表示は今と同じ 4 つの文で、網羅の match。作れなくなる組は Completed に写す(D7。決定(ユーザー、2026-11-23))。
    category: constraint
    applies_to: crates/core/src/plan、crates/cli/src/report/messages.rs、plan_text.rs
    trigger: 段 1
    logic: inventory.md 1e
    violation: completed・end_reached・stopped の真偽値が残る、または表示の文が変わる
    source: intent-statement.md(SM1)
  - id: BR1.6
    statement: RetryDecision::Retry は待ち時間、最後の終了コード、最大回数を持つ。cli のリトライは Retry の中身で知らせを出し、実行結果とリトライの設定を取り出し直さない。
    category: constraint
    applies_to: crates/core/src/flow/retry.rs、crates/cli/src/retry.rs
    trigger: 段 1
    logic: inventory.md 1f
    violation: cli の retry.rs に到達しない let-else が残る
    source: intent-statement.md(SM1)
  - id: BR1.7
    statement: 読まれないフィールド(JobExecution.job_id・command、JobOutcome.job_id、NextStep.job_id)を消す。JobExecution の組み立ての関数と runtime の run_job は job を受け取らない(D2。決定(ユーザー、2026-11-23))。契約 C1(rust の仕様書 02・05・07)を直し、run_job には「v0.9.6 までは run_job(job, line, env, signal)」と注記する。
    category: constraint
    applies_to: crates/core/src/types.rs、flow/classify.rs、flow/next_step.rs、crates/runtime/src/runner.rs、crates/cli/src/deps.rs、docs/spec/rust
    trigger: 段 1
    logic: inventory.md 1g
    violation: 読む所の無いフィールドが残る、または JobExecution の生成のたびに command を複製する
    source: intent-statement.md(SM1)
  - id: BR1.8
    statement: EnvFileError の理由はメソッド reason()(種類の固定の文)で返し、フィールドに写さない。定義と .env の誤りの表示の書式は core の render_error の 1 か所にする。
    category: constraint
    applies_to: crates/core/src/envfile/mod.rs、crates/core/src/error.rs
    trigger: 段 1
    logic: inventory.md 1h
    violation: reason: String が残る、または同じ書式の format! が 2 か所に残る
    source: intent-statement.md(SM1)
  - id: BR1.9
    statement: cli の Checked は CheckedDefinition を base として持ち、同じ項目を二重に持たない。本番の準備の入口は prepare_session の 1 つ(シグナルの監視が定義の検査より先。cli の既存の決まり)、--dry-run は prepare_checked(D3。決定(ユーザー、2026-11-23))。record_skip は名前つきの SkipRecord を返す。enter_flow は変えない。
    category: constraint
    applies_to: crates/cli/src/ctx.rs、step.rs、run.rs、pipeline.rs、dry_run.rs、retry.rs
    trigger: 段 1
    logic: inventory.md 1i
    violation: prepare_run・prepare_pipeline が残る、Checked と CheckedDefinition に同じ項目がある、(PipelineState, bool) を返す
    source: intent-statement.md(SM1)
  - id: BR1.10
    statement: cli の messages.rs の関数は、固定の文の定数(Text)、種類ごとの fn(kind) -> Text、値を埋める fn(lang, …) -> String の 3 つの形だけにする。日本語の文を他のクレートが持つ種類(定義・.env・変換・ステートの誤り)は Text::new(英語, kind.label()) で、cli に日本語の文を写さない(D10。決定(ユーザー、2026-11-23))。表示は 1 文字も変えない。
    category: constraint
    applies_to: crates/cli/src/report/messages.rs、report/mod.rs、crates/core/src/error.rs、envfile/mod.rs、crates/converter/src/error.rs、crates/runtime/src/error.rs
    trigger: 段 1
    logic: inventory.md 1j
    violation: fn(lang, kind) -> String で日本語を Display に任せる関数、または &'static str を返す言語つきの関数が残る。ステートの誤りの日本語が 2 か所にある
    source: intent-statement.md(SM1)
  - id: BR1.11
    statement: 外から使われない pub を狭める(inventory.md 1k の表)。狭めたあと、ほかのクレートの src とすべてのテストから参照が無いことを、ビルド・テスト・clippy と検索で確かめる。テストが使うもの(LoadedEnvFiles::add、Reporter の language_resolved・color_resolved)は狭めない。根の再公開は名前を並べる形にする。
    category: constraint
    applies_to: crates/core/src/lib.rs、definition/mod.rs、definition/depth.rs、definition/intake.rs、flow/mod.rs、flow/next_step.rs、error.rs、crates/cli/src/lib.rs、report/mod.rs、report/escape.rs、report/watch.rs、report/graph_text.rs、types.rs、crates/runtime/src/lib.rs
    trigger: 段 1
    logic: inventory.md 1k、§8
    violation: 外から参照の無い pub が残る、またはテストが使う pub を狭めてテストを変える
    source: intent-statement.md(SM1)

  # ---- 段 2: 計算量(動きを変えない) ----
  - id: BR2.1
    statement: 計画の模擬は、crate の中のその場で書き換える版(apply_outcome_in、apply_skip_in、apply_interrupt_in)を使い、ジョブごとにステートを複製しない。公開の apply_outcome・apply_skip・apply_interrupt は、複製してから同じ手順を行い、今と同じ値を返す。模擬の探索は定義の位置を手がかりにする(ステートのジョブ ID は一意という前提。D8。決定(ユーザー、2026-11-23))。
    category: constraint
    applies_to: crates/core/src/flow/record.rs、plan/simulate.rs
    trigger: 段 2
    logic: inventory.md 2a
    violation: 模擬がジョブごとに state.clone() する、または公開の関数の結果が変わる
    source: intent-statement.md(SM2)
  - id: BR2.2
    statement: build_graph の重ね合わせは、最初に現れたものを残す表で引く(今の find と同じ結果)。
    category: constraint
    applies_to: crates/core/src/graph/overlay.rs
    trigger: 段 2
    logic: inventory.md 2b
    violation: ジョブごとに重ね合わせを線形に探す
    source: intent-statement.md(SM2)
  - id: BR2.3
    statement: flow_edges は、遷移先の位置の表(最初の位置を残す。job_index と同じ結果)を 1 回作り、resolve_transition に引く関数を渡す。実行の経路(classify_and_decide)は今のまま job_index を使う。
    category: constraint
    applies_to: crates/core/src/flow/next_step.rs、graph/edges.rs
    trigger: 段 2
    logic: inventory.md 2c
    violation: flow_edges が遷移ごとに job_index を呼ぶ
    source: intent-statement.md(SM2)
  - id: BR2.4
    statement: .env の解析の完全に同じ名前の重複は、位置の表と「消した」印で扱い、残る並び(各名前の最後の項目を、最後に現れた順)と警告と上限の扱いを変えない。
    category: constraint
    applies_to: crates/core/src/envfile/parse.rs
    trigger: 段 2
    logic: inventory.md 2d
    violation: 重複のたびに position と remove をする
    source: intent-statement.md(SM2)
  - id: BR2.5
    statement: 計画の環境変数の名前の要約は、値を読まない merge_env_names で作る。並べ方と「最後に残す」規則は merge_env と共有する。
    category: constraint
    applies_to: crates/core/src/envfile/merge.rs、plan/simulate.rs
    trigger: 段 2
    logic: inventory.md 2e
    violation: 計画が値を to_string して複製する、または名前・出どころ・並びが merge_env と食い違う
    source: intent-statement.md(SM2)
  - id: BR2.6
    statement: 上限に近い入力の性能のテスト(P1〜P5)を足し、5 回の中央値を上限と比べる。上限は、直す前の測定(inventory.md §0.2)から見積もった直した後の時間の約 5 倍(丸めた数。P1・P2 は 150 ms、P3 350 ms、P4 600 ms、P5 100 ms)の余裕のある上限で、遅い CI(特に Windows、debug)でも落ちず、大きな退行だけを止める(D6。決定(ユーザー、2026-11-23))。直す前の形に戻ったことはテストでは検知できないので、直す前と直した後の時間を code-summary と test-results に記録して示す。P6(計画の名前の要約)は足さない。結果の中身も確かめる。
    category: validation
    applies_to: crates/core/tests/plan_simulate_perf.rs、graph_lookup_perf.rs、env_parse_perf.rs
    trigger: 段 2 の各項目の実装の後
    logic: functional-spec.md §5.2
    violation: 上限を超える、または前後の時間(2e は一時の測定)が記録されない
    source: intent-statement.md(SM2、Q3)

  # ---- 段 3: 利用者から見える変更と版 ----
  - id: BR3.1
    statement: converter の文法の誤りの理由は、TOML は「TOML として読めません」、YAML は「YAML として読めません」、形式に依らない内部の守りは「入力の構造を解釈できません」(D4、D5。決定(ユーザー、2026-11-23))。種類の文(文法の誤りです、英語 syntax error)は変えない。理由は英語のロケールでも日本語のまま(今の規則)。入力の値を含めない。仕様書に「v0.9.6 までは、理由は種類の文と同じ『文法の誤りです』だった」と注記する。
    category: constraint
    applies_to: crates/converter/src/error.rs、toml_read.rs、yaml_read.rs、resolve.rs、docs/spec/rust/09・10・12・08、docs/spec/language-agnostic/09・10・11・12
    trigger: 段 3
    logic: inventory.md 3a
    violation: 表示が「文法の誤りです: 文法の誤りです」と重なる、または理由に入力の値を含む
    source: intent-statement.md(SM3、Q4)
  - id: BR3.2
    statement: .env の読み込みは、定義ファイルと同じ形 B(パスで種別 → 開く → 開いたハンドルで種別と大きさ → 上限 + 1 バイトまで読む → UTF-8)にする。読む権限のない 1 MiB 超の .env は unreadable。無い、通常でない(名前付きパイプは開かない)、上限超過、UTF-8 でないの種類は変えない。仕様書に「v0.9.6 までは…tooLarge だった」と注記する。runtime の製品のコードなので、.unwrap()・.expect(・panic!(・unreachable!(・todo!(・unimplemented!( を使わず、失敗はすべて EnvFileErrorKind で返す(crates/runtime/tests/nfr_source.rs の「製品のコードに_パニックしうる書き方がない」が検査する)。
    category: constraint
    applies_to: crates/runtime/src/envfile.rs、docs/spec/rust/03・02・11、docs/spec/language-agnostic/03・11
    trigger: 段 3
    logic: inventory.md 3b
    violation: 開く前にパスのメタデータで大きさを見る、名前付きパイプで止まる、または unwrap・expect・unreachable! などのパニックしうる書き方を使う
    source: intent-statement.md(SM3、Q5)
  - id: BR3.3
    statement: 最後のコミットで版を 0.9.7 にする(ee9f409 と同じ範囲: Cargo.toml × 4、Cargo.lock、binary_basic.rs の版の期待、仕様書の 0.9.6 の表記 16 ファイル)。「v0.9.6 までは…」の注記は上げない。タグとリリースはしない。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、crates/cli/tests/binary_basic.rs、docs/spec/rust、docs/spec/language-agnostic
    trigger: 段 3 の最後
    logic: inventory.md 3c
    violation: 版の表記が食い違う、または注記の版まで上げる
    source: intent-statement.md(SM4、memory:M6)

  # ---- 足すテスト ----
  - id: BR8.1
    statement: 足すテストは functional-spec.md §5.1 の N1〜N8・N10〜N12(見込み +11 件。N9 は D6 の決定で欠番)。test-after(実装のコミットの後にテストのコミット)。N11 は直す前の木で落ち、直した後に通ることを記録する。N12 は直す前と後の両方で通ることを記録する。
    category: validation
    applies_to: テスト
    trigger: 各項目の実装の後
    logic: functional-spec.md §5
    violation: 一覧に無いテストを足す、または増減が説明されない
    source: intent-statement.md(SM2、SM3、SM5、Q3)

  # ---- 全体 ----
  - id: BR9.1
    statement: 段 1・2 は、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run と graph の出力)を変えない。利用者から見える変更は段 3 の 2 つと版だけ。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテストがそのまま通る(T 表の置き換えを除く)
    violation: 段 1・2 で既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM1、SM2、SM3)
  - id: BR9.2
    statement: 既存のテストの変更は functional-spec.md §4 の T1〜T26 だけにする。確かめる中身と期待する出力の文字列を変えるのは ★ の T17a(1 行の削除。コミット 8)、T24、T25 だけ(T17b はコミット 9 の組み立ての置き換え)。テストの補助の使われなくなった引数は、名前を _ で始めて残す。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§7
    violation: 一覧に無いテストのファイルの変更、またはテスト名・件数の変化
    source: intent-statement.md(SM5、Q2)
  - id: BR9.3
    statement: 段の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,146 + 足したテスト(見込み +11)= 1,157」に一致すること(macOS)。テスト名の一覧の差は N1〜N8・N10〜N12 の追加だけ。
    category: validation
    applies_to: テスト
    trigger: 各段の終わり
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、またはテスト名の変化
    source: intent-statement.md(SM5、Q1、Q3)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。ステートに環境変数を書かない。runtime の製品のコード(src の #[cfg(test)] より前)に .unwrap()・.expect(・panic!(・unreachable!(・todo!(・unimplemented!( を書かない(runtime の nfr_source.rs が検査する。段 1 の run_job・JobExecution の変更と段 3 の .env の読み込みも同じ)。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(SM6)
```
