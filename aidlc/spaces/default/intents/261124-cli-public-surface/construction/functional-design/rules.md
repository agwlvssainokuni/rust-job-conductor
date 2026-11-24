# ビジネスルール(Rules): cli のライブラリの公開範囲を狭める

詳細の場所は `inventory.md`(公開の項目ごとの判定、変える場所、補助の扱い、仕様書の S1〜S24、写しでの検証)、承認が要るテストの変更(T 表)、件数の見込み、コミットの並び、決めてほしい点は `functional-spec.md` の §4、§6、§8、§10。D1〜D7 は、ユーザーが決めた(すべて推奨のとおり。2026-11-24)。T 表(T1〜T10)と移すテストの一覧は、設計の承認で認めてもらう(承認待ち)。

```yaml
rules:
  # ---- 公開範囲(方針: テストのコードの都合で本番のコードを曲げない。本番にテストのための要素を足さない) ----
  - id: BR1.1
    statement: cli のライブラリの公開に残すのは 13 項目だけ。main が使う run_cli、StdSink、SystemEnv、install_panic_hook。依存の注入の継ぎ目 run_cli_with、Deps、Runtime、Sink、Env、Signals、SystemRuntime、SystemSignals。継ぎ目を使うのに要る型 SignalError(Q3。A)。継ぎ目の形(trait のメソッドと引数、Deps の項目、SignalError の変種)と、run_cli・run_cli_with のシグネチャは変えない。
    category: constraint
    applies_to: crates/cli/src/lib.rs
    trigger: コミット 12
    logic: inventory.md 1.1(P1〜P13)、1.2
    violation: 13 項目のどれかが公開でなくなる、継ぎ目の形が変わる、またはほかの項目が公開に残る
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: ほかの 15 項目を crate の中に狭める。pub mod exit・report・run_id を mod にし、pub use args::parse_args と pub use types::{CliCommand, CommandKind, GraphFormat, InitOptions, InitTarget, Language, Level, ParsedArgs, RunIdArg, UsageError, UsageErrorKind} を消す。run_cli_with が使う parse_args・CommandKind・ParsedArgs は非公開の use で取り込む。下位モジュールの中の pub は pub のまま残す(pub(crate) に書き換えない)。本番で使われなくなる項目は無く(写しで dead_code の警告 0 件)、本番のコードを消さない。本番に pub、doc(hidden)、テストのための引数や関数を足さない。本番に足すのは lib.rs の最後の #[cfg(test)] mod tests; だけ。
    category: constraint
    applies_to: crates/cli/src/lib.rs
    trigger: コミット 12(宣言はコミット 1)
    logic: inventory.md 1.1(N1〜N15)、1.2、functional-spec.md §2
    violation: 狭める項目が crate の外から見える、本番にテストのための要素が増える、または dead_code の警告が出る
    source: intent-statement.md(SM1)
  - id: BR1.3
    statement: 公開でなくなる項目へのドキュメントのリンクを直す。lib.rs の入口の関数の表から parse_args の行を消し、公開するのは入口の関数と差し替え口だけである旨の 1 文を足し、モジュールの構成の表の run_id・exit・report をリンクでない名前にする。src/report/escape.rs の説明の tests/escape_table.rs を src/tests/escape_table.rs にする。cargo doc --workspace --no-deps の警告は基準の 5 件(core 1、runtime 4)から増やさない(cli は 0 件)。
    category: validation
    applies_to: crates/cli/src/lib.rs、crates/cli/src/report/escape.rs
    trigger: コミット 2(escape.rs)、コミット 12(lib.rs)
    logic: inventory.md 1.3、§0.2
    violation: 非公開の項目へのリンクの警告が出る、または移した場所を指さない説明が残る
    source: intent-statement.md(SM1、SM6)

  # ---- 継ぎ目に要る型の基準 ----
  - id: BR2.1
    statement: 「継ぎ目を使うのに要る型」は、公開に残す trait と関数(run_cli_with、Deps、Runtime・Sink・Env・Signals)の引数・戻り値・項目に出てくる cli の型で、偽物を書くのに要るもの。core・runtime・converter の型は、それぞれのクレートが公開しているので対象の外。この基準に当たる cli の型は SignalError(Signals::start の戻り値)だけで、解析の結果の型(CliCommand、ParsedArgs、UsageError など)、Language、Level は当たらない。
    category: constraint
    applies_to: crates/cli/src/lib.rs、crates/cli/src/deps.rs(読むだけ)
    trigger: 設計
    logic: inventory.md 1.1 の表と、その後の注
    violation: 基準に当たらない型が公開に残る、または偽物を書くのに要る型が公開から外れる
    source: intent-statement.md(SM1、SM2)

  # ---- テストの移動 ----
  - id: BR3.1
    statement: 本体か、そのファイルの補助で、狭めた関数を呼ぶか狭めた型の値を作って確かめるテスト 162 件を、src の単体テストへ移す(Q1。A。D3。決定: 狭めた項目で確かめるテストだけを移す。推奨のとおり。ユーザー、2026-11-24。T1〜T7 の表は承認待ち)。ファイルごと移すのは args_grammar(60)、report_messages(36)、graph_report(28)、graph_long_id(2)、plan_report(7)、init_template(7)、exit_codes(7)、escape_table(3)、run_id_arg(1)、init_types(3)。一部を移すのは docs_graph の 6 件、docs_dry_run の 1 件、docs_init の 1 件。移すテストの名前は変えない(★ なし)。本体の確かめの行(呼ぶ関数、入力、期待の値、誤りの種類と対象)は変えず、変えるのはファイルの先頭の mod と import、conductor_cli:: のパスを crate:: にすることだけ(cargo fmt が整える所を除く)。#[cfg(unix)]・cfg!(unix)・panic!( もそのまま移す。
    category: constraint
    applies_to: crates/cli/src/tests/*.rs、crates/cli/tests/ の上の 13 ファイル
    trigger: コミット 1〜10
    logic: functional-spec.md §4.1、§4.2、§4.7
    violation: 確かめの行や期待の値が変わる、テスト名が変わる、または一覧に無いテストが移る
    source: intent-statement.md(SM2)
  - id: BR3.2
    statement: 移す先は src/tests/ のディレクトリ(mod.rs、common/、docs_support.rs と、元の統合テストと同じ名前のファイル)とし、lib.rs のファイルの最後に #[cfg(test)] mod tests; を置く(D1。決定: 推奨のとおり。ユーザー、2026-11-24)。cli のソースの検査(tests/nfr_source.rs。src を再帰で読み、ファイル全体を検査する)は、src/tests/ の下を OS 分岐と panic!( の 2 つの検査の外にし、その代わりに、lib.rs の末尾が #[cfg(test)] と mod tests; であることと、製品のファイル(src/tests/ の外)に #[path が無いこと(src/tests/ を本番から読む道が無いこと)を、同じテスト(テスト専用の経路を実行ファイルに持たない)で確かめる。テストの数は変えない(D2。決定: 検査の範囲を製品のコードに限る。推奨のとおり。ユーザー、2026-11-24)。ユーザーは、プロジェクトの規則「NEVER cfg(unix)/cfg(windows) を platform の外に置かない」(project.md の Forbidden)を「製品のコードについて」の規則と読むことを認めた。テストのときだけコンパイルする src/tests/ の #[cfg(unix)]・cfg!(unix) は、この規則に反しない。unsafe・low_level・ネットワークの語の検査は src/tests/ も対象のまま。テストの数は変えない。本番の src に unsafe と cfg(unix)/cfg(windows) を足さない。cli は #![forbid(unsafe_code)] のまま。
    category: constraint
    applies_to: crates/cli/src/lib.rs、crates/cli/src/tests/、crates/cli/tests/nfr_source.rs
    trigger: コミット 1(以後すべて)
    logic: functional-spec.md §2.2、§2.3、inventory.md §3、§0.2
    violation: 本番の可視性が広がる、製品のコードの検査の範囲が狭まる(src/tests/ 以外が外れる)、src/tests/ が cfg(test) の外から読まれる、または unsafe・ネットワークの検査から src/tests/ が外れる
    source: intent-statement.md(SM2)
  - id: BR3.3
    statement: 公開の入口(run_cli_with と差し替え口)だけで確かめるテストは統合テストに残し、本体を変えない。統合テストに残して書き方を変えるのは次だけ(Q2 の「書き方を変える」。D4、D5。決定: どちらも推奨のとおり。ユーザー、2026-11-24。T8・T9 の表は承認待ち)。flow_init.rs は、期待のひな形を render_template ではなく原本の写し(ORIGINAL_JA・ORIGINAL_EN と名前の置き換え。期待のバイト列は同じ)で書き、Language は tests/common のテストの言語にする。flow_validate.rs は common::Language::En にする。docs_graph・docs_dry_run・docs_init は移すテストを外し、モジュールの説明を足す。tests/common/mod.rs は、テストの言語 Language を足し、pub mod kinds; を消し、コメントを直す。tests/common/kinds.rs は src/tests/common/kinds.rs へ移る。signals_order.rs・signals_real.rs は変えない。
    category: constraint
    applies_to: crates/cli/tests/
    trigger: コミット 9〜11
    logic: functional-spec.md §4.2〜§4.4
    violation: 残るテストの確かめや期待の値が変わる、公開の入口だけで確かめるテストが単体テストへ移る、または一覧に無い統合テストが変わる
    source: intent-statement.md(SM2、SM5)
  - id: BR3.4
    statement: 単体テストが使う補助は src/tests/ に写す(D7。決定: 推奨のとおり。ユーザー、2026-11-24)。src/tests/common/mod.rs に FakeEnv(項目、locale_read、color_read、with、for_language、impl Env)、FakeSink(項目、out_text、err_text、impl Sink)、state_error を、src/tests/docs_support.rs に readme を、統合テストの補助から一字も変えずに写し、説明に元の場所を書く。統合テストの補助(tests/common/mod.rs、tests/docs_support/mod.rs)は残す。src/tests/common/mod.rs に #![allow(dead_code)] を付けず、各コミットで使う項目だけを置く。本番の lib.rs に、補助を共有するための名前の別名(extern crate self など)を足さない。
    category: constraint
    applies_to: crates/cli/src/tests/common/mod.rs、crates/cli/src/tests/docs_support.rs
    trigger: コミット 5、9、10
    logic: inventory.md §4、functional-spec.md §2.2、§8
    violation: 写しの本体が元と違う、使われない項目の警告が出る、または本番に共有のための要素が足される
    source: intent-statement.md(SM2)

  # ---- 仕様書 ----
  - id: BR4.1
    statement: docs/spec/rust の該当箇所を、実装の形に合わせて直す(inventory.md §5 の S1〜S24)。02 の 2.4 の再公開と公開モジュールの記述を 13 項目にし、関数の表と終了コードの定数の、公開でなくなるものに「crate の中」を付ける。02 の 4 章の偽の実装と 5.4 の lib.rs・tests/ の行、07 の 1 章・8.2 と 11 の 3.3〜3.5 の OS 分岐と panic!( の検査の範囲、08 の 9 章と 11 の 3.3 のエスケープの表の場所、11 の 4.1 の方針、4.2 の cli の統合テストの表(移した行を消す)と単体テストの表(足す)、docs の行、common の行を直す。テストの件数は仕様書に書かれていないので、直す件数は無い。版の注記は足さない。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md、07-process-and-signals.md、08-output-and-i18n.md、11-quality-and-build.md
    trigger: 各コミット(S の「コミット」の列)
    logic: inventory.md §5、functional-spec.md §7 の仕様書の検索
    violation: 狭めた名前を公開として書く記述が残る、移したテストのファイルを tests/ として書く記述が残る、または変えない記述(inventory.md §5 の「変えない記述」)が変わる
    source: intent-statement.md(SM3)
  - id: BR4.2
    statement: docs/spec/language-agnostic は直さない(公開範囲の記述が無く、11 章の根拠の欄は ファイル名::テスト名 で場所を書かず、移す先のファイル名とテスト名が元と同じなので正しいまま。D6。決定: 直さない。推奨のとおり。ユーザー、2026-11-24)。凍結した仕様書(go、python、typescript、csharp、powershell)は触らない。
    category: constraint
    applies_to: docs/spec/language-agnostic、docs/spec/go、python、typescript、csharp、powershell
    trigger: 常に
    logic: inventory.md §5、functional-spec.md §10 の D6
    violation: 言語に依らない仕様書や凍結した仕様書が変わる
    source: intent-statement.md(SM3)

  # ---- 足すテスト・消すテスト ----
  - id: BR8.1
    statement: 新しいテストは足さない(見込み 0 件)。消すテストも無い(0 件)。nfr_source.rs に足す 2 つの確かめ(lib.rs の末尾の宣言、製品のファイルに #[path が無いこと)は、検査の範囲を直す代わりの確かめで、テストの数を変えない。実装で増減が出たら、その数と理由を記録する。
    category: validation
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4.6、§5
    violation: 一覧に無いテストを足す・消す、または増減が説明されない
    source: intent-statement.md(SM5)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(出力、終了コード、ステートファイル、--dry-run の出力)を変えない。本番の処理のコード(run_cli・run_cli_with の本体、流れ、表示、引数の解析、終了コードの決定)は変えない。変わるのは lib.rs の可視性と説明、escape.rs の説明だけ。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 統合テスト 294 件と移した単体テスト 162 件が、期待を変えずに通る
    violation: 既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM4)
  - id: BR9.2
    statement: テストの変更は functional-spec.md §4 の T1〜T10 だけにする。一覧は conductor_cli:: のすべての参照(複数行の use を含む)と補助(common、docs_support、kinds、for_language)の参照を grep で網羅し、写しの木で本番だけを変えたコンパイルのエラー(17 ファイル、62 件)と照らして作る。名前を変えるテストは無い。消すテストは無い。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、inventory.md §0.2、§7
    violation: 一覧に無いテストのファイルの変更、テスト名の変化、または理由の無い削除
    source: intent-statement.md(SM5)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,152 − 消したテスト 0 + 足したテスト 0 = 1,152」(と無視 1。cli は単体 164、統合 294)に一致すること(macOS)。テスト名の一覧の差は、名前の最後の部分で比べて 0 件。パスで比べた差は 324 行。完了は functional-spec.md §7 の検索で確かめ、狭めた名前を tests/ から参照する所が 0 件であることを含める。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 各コミット(cli の件数)と実装の完了
    logic: functional-spec.md §6、§7、§8 の件数の列
    violation: 件数の説明できない増減、テスト名の変化、または狭めた名前の参照の残り
    source: intent-statement.md(SM5)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と x86_64-pc-windows-msvc の対象のコンパイル、テストを含まない本番の形 -p conductor-cli --lib --bins)、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、本番の src の cfg を足さない。誤りの表示に入力の値を含めない。
    category: validation
    applies_to: 全体
    trigger: 各コミットと実装の完了
    logic: CI と同じ検査。functional-spec.md §7、§8 の各コミットの確かめ
    violation: マージの関門で止まる
    source: intent-statement.md(SM6)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。仕様書は rust だけを直し、版の注記を足さずに今の形を書く。タグとリリースはしない。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、docs/spec/rust
    trigger: 常に
    logic: functional-spec.md §1
    violation: 版が変わる、または版の注記が足される
    source: intent-statement.md(SM4)
  - id: BR9.6
    statement: 移したテストと残るテストのモジュールの説明(どこに何があるか)、写しの補助の説明(元の場所)、tests/common/mod.rs のコメント、nfr_source.rs の説明を、同じコミットで直す。src/tests/ のファイルに #[cfg(test)] を書かない(宣言の側で付く)。
    category: validation
    applies_to: crates/cli/src/tests/ と crates/cli/tests/ のドキュメントコメント
    trigger: 各コミット
    logic: functional-spec.md §4.2、§4.4、§4.5
    violation: 移した場所を指さない説明が残る
    source: intent-statement.md(SM5)
```
