# ビジネスルール(Rules): 小さな直し

詳細の場所は `inventory.md`(説明の C1〜C4、仕様書の S1〜S14、ほかの章の食い違い X1〜X5、テストの重複の比べ方、写しでの検証)、承認が要るテストの変更(T 表)、件数の見込み、コミットの並び、決めてほしい点は `functional-spec.md` の §4、§6、§8、§10。D1〜D8 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。T 表(T1〜T4)と消すテスト 3 件(`functional-spec.md` §4.6)は、設計の承認で認めてもらう(承認待ち)。

```yaml
rules:
  # ---- 古い説明(方針: テストのコードの都合で本番のコードを曲げない。本番で変えるのは説明の行だけ) ----
  - id: BR1.1
    statement: core の plan/mod.rs の 18 行目と types.rs の 440 行目の説明が名指す起動前の判断の関数を、非公開の decide_before_run から、公開の decide_from_state に直す(C1、C2)。名前はリンクにしない(今の書き方のまま)。非公開の関数がある flow/decide.rs の中は変えない。
    category: constraint
    applies_to: crates/core/src/plan/mod.rs、crates/core/src/types.rs
    trigger: コミット 1
    logic: inventory.md §1.1、functional-spec.md §2.1
    violation: 2 つのファイルの説明に decide_before_run が残る、flow/decide.rs が変わる、または説明のほかの行が変わる
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: decide_before_run を公開のもの・呼ぶ側の関数として書く説明を、crates と docs/spec/rust で検索し、結果を一覧にする(inventory.md §1.2)。コードでは BR1.1 の 2 か所だけ。仕様書 rust 05 の 2 か所(341 行目、850 行目)を decide_from_state に直す(D5。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。同じ種類の古い説明 core の lib.rs の 38 行目(runtime が再公開すると書く。C3)を「再公開しない(使う側は、この crate から使う)」に直す(D1。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。ほかは変えない。
    category: validation
    applies_to: crates/core/src、docs/spec/rust/05-execution-flow.md
    trigger: 設計とコミット 1・2
    logic: inventory.md §1.2、§1.3、functional-spec.md §10 の D1、D5
    violation: 一覧に無い説明が変わる、または決めた所が直らない
    source: intent-statement.md(SM1)

  # ---- 仕様書 rust 02 ----
  - id: BR2.1
    statement: 2.4 の関数の表の resolve_run_id の引数の型を、実装(crates/cli/src/run_id.rs)のとおり Option<&RunIdArg> にする(S1)。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md
    trigger: コミット 3
    logic: inventory.md §2.1
    violation: 仕様書のシグネチャが実装と食い違う
    source: intent-statement.md(SM2)
  - id: BR2.2
    statement: 2.1 の再公開の一覧に consistency::{ConsistencyResult, MismatchKind, check_consistency} を、公開モジュールの一覧に consistency を足す(S2、S3)。再公開の一覧は lib.rs の pub use の並び(モジュールの名前の順)にする。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md
    trigger: コミット 4
    logic: inventory.md §2.2
    violation: core の lib.rs の pub mod と pub use にあるものが、仕様書の一覧に無い
    source: intent-statement.md(SM2)
  - id: BR2.3
    statement: 2 章の 3 つの「再公開(lib.rs):」の段落(42、105、137 行目)から版の注記をすべて抜き、各段落の直後に「版の履歴(古い順):」の行と、時系列の箇条書きを置く(S4〜S6)。注記の中身は移すだけで、足さない・消さない。並べ方は、注記が述べる変わり目の版の古い順。ほかの段落・表の行・ほかの章の版の注記は変えない(D7。決定: 今回は直さない。推奨のとおり。ユーザー、2026-11-25)。同じ段落の非公開のモジュールの一覧の欠け(S7)は、「モジュールはすべて非公開(11 個を並べる)」に直す(D2。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md
    trigger: コミット 5
    logic: inventory.md §2.3、§2.4、functional-spec.md §2.2
    violation: 今の形の段落に版の注記が残る、版の履歴の並びが時系列でない、注記の中身が変わる、または 3 段落の外が変わる
    source: intent-statement.md(SM2)
  - id: BR2.4
    statement: 直した後、今の形の 3 段落に「v0.」で始まる版の注記が残らないことを、functional-spec.md §2.2 と §7 の検索で確かめる(今の形の段落 3、そのうち「v0.」を含むもの 0、版の履歴の行 3、箇条書き 7)。ほかの章の同じ種類の食い違い(X1〜X5)は一覧にし、D5〜D7 の決定(推奨のとおり。ユーザー、2026-11-25)のとおり、X1(rust 05)と X2(rust 06 の 5 行目。D6。決定: 直す)を直し、X3〜X5 は変えない。
    category: validation
    applies_to: docs/spec/rust
    trigger: コミット 5 と実装の完了
    logic: functional-spec.md §2.2、§7、inventory.md §2.5
    violation: 検索の結果が見込みと違う
    source: intent-statement.md(SM2)

  # ---- テストの重複 ----
  - id: BR3.1
    statement: core の tests/plan_perf.rs の「終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる」を消す(T1)。tests/plan_simulate_perf.rs の P2 が、同じ定義・同じステート・同じ入力・同じ 4 つの確かめを、より厳しい上限(5 回の中央値 150 ms)で持つ(inventory.md §3.1)。使われなくなる import も消す。
    category: constraint
    applies_to: crates/core/tests/plan_perf.rs
    trigger: コミット 7
    logic: functional-spec.md §4.1、§4.6
    violation: 同じことを確かめるテストが残らない、または一覧に無いテストが消える
    source: intent-statement.md(SM3)
  - id: BR3.2
    statement: plan_perf.rs に残る「千個のジョブがすべて実行する計画は_1秒以内に作れる」を、名前と本体を一字も変えずに plan_simulate_perf.rs の最後へ移し、plan_perf.rs を消す(T2)。変えるのは、移す先の import(add_env_file、EnvEntry、EnvFileContent)と、モジュールの説明の 2 行だけ。テストの名前に番号(p1_ など)を付けない(D8。決定: 変えない。推奨のとおり。ユーザー、2026-11-25)。
    category: constraint
    applies_to: crates/core/tests/plan_perf.rs、crates/core/tests/plan_simulate_perf.rs
    trigger: コミット 8
    logic: functional-spec.md §2.3、§4.2
    violation: 移したテストの本体や名前が変わる、または plan_perf.rs が残る
    source: intent-statement.md(SM3)
  - id: BR3.3
    statement: cli の src/tests/init_types.rs の 3 件のうち、args_grammar.rs の「init_の既定は_conductor_toml_と_my_pipeline_で_force_なし」が同じことを確かめる 2 件(オプションの既定の値、コマンドから取り出した既定)を消す(T3)。残る 1 件(InitTarget::from_path の境界)は変えず、置き場所も変えない(ファイルの統合は別の intent)。import を InitTarget だけにし、モジュールの説明に、既定を確かめる場所を足す。CliCommand::new の説明の「テストが使う」を消す(C4。説明だけ)。消したあとは、引数の解釈を通さない CliCommand::new の直接の確かめが無くなる(functional-spec.md §4.6 に書き、承認で見せる)。
    category: constraint
    applies_to: crates/cli/src/tests/init_types.rs、crates/cli/src/types.rs
    trigger: コミット 9
    logic: functional-spec.md §2.3、§4.3、§4.6、inventory.md §3.2
    violation: 同じことを確かめるテストが残らない、残る 1 件が変わる、または使われない import や古い説明が残る
    source: intent-statement.md(SM3)
  - id: BR3.4
    statement: cli の tests/signals_order.rs は残し、変えない(Q2。B)。signals_real.rs も変えない。
    category: constraint
    applies_to: crates/cli/tests/signals_order.rs、crates/cli/tests/signals_real.rs
    trigger: 常に
    logic: functional-spec.md §4.5
    violation: 2 つのファイルが変わる
    source: intent-statement.md(SM3)
  - id: BR3.5
    statement: 消すテストと移すテストを挙げる仕様書の行を、同じコミットで直す(rust 11 の性能の表、core の統合テストの表、cli の単体テストの表。S8〜S11)。言語に依らない仕様書の、plan_perf を根拠とする 1 か所(S12)を plan_simulate_perf に直す(D3。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。仕様書にテストの件数の記述は無いので、直す件数は無い。消す 3 件は、設計の承認で、同じことを確かめて残るテスト(functional-spec.md §4.6)を添えて認めてもらう(Q1。A)。
    category: validation
    applies_to: docs/spec/rust/11-quality-and-build.md、docs/spec/language-agnostic/11-quality-and-build.md
    trigger: 設計の承認、コミット 8・9
    logic: inventory.md §3.4、functional-spec.md §4.6、§10 の D3
    violation: 無くなったファイルやテストを指す記述が残る、または承認の無いテストが消える
    source: intent-statement.md(SM3)

  # ---- ソースの検査 ----
  - id: BR4.1
    statement: cli の tests/nfr_source.rs の「テスト専用の経路を実行ファイルに持たない」の最後の確かめで、製品のファイル(src/tests/ の外)の、コメントを除いて空白を除いた行に、#[path、include!、include_str!、include_bytes! のどれかがあれば失敗にする(T4。Q3。A。D4。決定: マクロの名前と ! を、行の空白を除いて探す。区切りの (・[・{ のどれでも止める。推奨のとおり。ユーザー、2026-11-25)。テストの数(6 件)と名前は変えない(★ なし)。モジュールの説明と、確かめの前のコメントを合わせる。既知の限界: 空白を除いた部分一致なので、my_include! のような別のマクロや、文字列・/* */ のコメントの中の語にも当たる(誤検出の側)。#[cfg_attr(…, path = "…")] の形は見逃す。
    category: constraint
    applies_to: crates/cli/tests/nfr_source.rs
    trigger: コミット 10
    logic: functional-spec.md §2.4、§4.4
    violation: 製品のコードの include の 3 つのマクロを見逃す、テストの数や名前が変わる、または src/tests/ が検査の範囲に入る
    source: intent-statement.md(SM4)
  - id: BR4.2
    statement: 今の製品のコードで、直した検査が通る(製品のコードに #[path と include の 3 つのマクロは 0 件)。仕様書 rust 11 の 3.3 の表と 3.5 の文(S13、S14)に、止める書き方を足す。製品のコードを、検査を通すために変えない。
    category: validation
    applies_to: crates/cli/src、docs/spec/rust/11-quality-and-build.md
    trigger: コミット 10
    logic: inventory.md §0.2、§4.2、§4.3
    violation: 今の製品のコードで検査が失敗する、または仕様書の記述が検査と食い違う
    source: intent-statement.md(SM4)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(出力、終了コード、ステートファイル、--dry-run の出力)を変えない。本番の src で変えるのは説明の行(C1、C2、C3、C4。C3 は D1 の決定)だけで、関数の本体、型、可視性は変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: git diff 6ce7d45 -- crates/*/src の差がコメントの行だけ。全テストが期待を変えずに通る
    violation: 本番の処理のコードが変わる、または既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM6)
  - id: BR9.2
    statement: テストの変更は functional-spec.md §4 の T1〜T4 だけにする。一覧は、対象のファイルとテスト名と補助の参照を grep で網羅し、写しの木で全項目を当てて作る。名前を変えるテストは無い。足すテストは無い。実装で増減が出たら、その数と理由を記録する。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、inventory.md §0.2、§6
    violation: 一覧に無いテストのファイルの変更、テスト名の変化、または理由の無い増減
    source: intent-statement.md(SM5)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,152 − 消したテスト 3 + 足したテスト 0 = 1,149」(と無視 1。core 統合 269、cli 単体 162)に一致すること(macOS)。移した 1 件は件数に数えない。テスト名の一覧の差は、名前の最後の部分で比べて、消した 3 件だけ。移した 1 件は、実行ファイルごとの件数(plan_simulate_perf が 3)と、本体の一致で確かめる。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 各コミット(件数)と実装の完了
    logic: functional-spec.md §6、§7、§8 の件数の列
    violation: 件数の説明できない増減、または消した 3 件のほかのテスト名の差
    source: intent-statement.md(SM5)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と x86_64-pc-windows-msvc の対象のコンパイル、本番の形 -p conductor-cli --lib --bins)、全テスト、行カバレッジ 80% 以上、MSRV 1.100、cargo doc の警告が基準の 5 件のまま、が通る。新しい依存、unsafe、製品のコードの cfg(unix)/cfg(windows) を足さない。テスト名とドキュメントコメントは日本語。
    category: validation
    applies_to: 全体
    trigger: 各コミットと実装の完了
    logic: CI と同じ検査。functional-spec.md §7、§8 の各コミットの確かめ
    violation: マージの関門で止まる
    source: intent-statement.md(SM7)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。タグとリリースはしない。凍結した仕様書(go、python、typescript、csharp、powershell)は触らない。仕様書に新しい版の注記を足さない(BR2.3 は既存の注記を移すだけ)。1 項目 1 コミットの小さな並びにし、どのコミットでもビルドとテストが通る。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、docs/spec、コミットの並び
    trigger: 常に
    logic: functional-spec.md §1、§8
    violation: 版が変わる、凍結した仕様書が変わる、版の注記が足される、または途中のコミットでビルドかテストが止まる
    source: intent-statement.md(SM6)
```
