# ビジネスルール(Rules): テストのファイルの統合

詳細の場所は `inventory.md`(項目ごとの元と移す先、名前の衝突、`cfg` の扱い、仕様書の S1〜S16・L1〜L10、写しでの検証)、承認が要るテストの移動(T 表)、件数の見込み、コミットの並び、決めてほしい点は `functional-spec.md` の §4、§6、§8、§10。D1〜D8 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。下の規則の D の箇所は、その決定のとおりの形を書いている。T 表(T1〜T13)は、設計の承認で認めてもらう(承認待ち)。

```yaml
rules:
  # ---- 項目 1: cli の README のテスト(Q2: A) ----
  - id: BR1.1
    statement: cli の README のテスト 18 件(統合テストの tests/docs_{retry,env,graph,dry_run,init}.rs の 10 件と、単体テストの src/tests/docs_{graph,dry_run,init}.rs の 8 件)を、単体テスト crates/cli/src/tests/docs.rs 1 つにまとめる。並びは、補助のあとに README の並びの 5 つの節(リトライ、環境変数、graph、--dry-run、init)。各節は語句の検査、例の比べ方の順。元の 10 ファイルと tests/docs_support/ を消し、src/tests/mod.rs の mod を mod docs にする。定数 HEADING の衝突は RETRY_HEADING・ENV_HEADING に名前を変えて解く(D1。決定: 推奨のとおり。ユーザー、2026-11-25)。テストの名前・確かめの行・期待の値は変えない。
    category: constraint
    applies_to: crates/cli/src/tests/docs.rs、crates/cli/src/tests/mod.rs、crates/cli/tests/docs_*.rs、crates/cli/tests/docs_support/、crates/cli/src/tests/docs_*.rs
    trigger: コミット 9、10
    logic: functional-spec.md §2.1、§4.1 の T10・T11、inventory.md §1
    violation: 元のファイルが残る、テストの数が 18 でない、または本体で HEADING の 3 行のほかが変わる
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: README の補助(DOCS_CHECK_WORDS、readme、assert_readme_has、assert_docs_check_words、block_after_heading)は docs.rs の先頭の 1 か所にだけ置く。readme の写しは残さない。1 つのファイルではすべての補助が使われるので、#![allow(dead_code)] は外す。
    category: constraint
    applies_to: crates/cli/src/tests/docs.rs
    trigger: コミット 9、10
    logic: functional-spec.md §2.1、§7 の fn readme() の数(1)
    violation: readme が 2 か所以上にある、または使われない補助のために警告を抑える
    source: intent-statement.md(SM1)
  - id: BR1.3
    statement: 統合テストから単体テストへ移す 10 件は、cli の公開の入口を使わず、README・conductor_core・tempfile だけを使う。tempfile は cli の dev-dependencies に既にあり、単体テストからも使えるので、依存(dev-dependencies を含む)を足さない。移す中身は、cli の nfr_source.rs が src/tests/ も見る 2 つの検査(unsafe・low_level、ネットワークの語)に当たらない。
    category: validation
    applies_to: crates/cli/Cargo.toml、crates/cli/src/tests/docs.rs
    trigger: コミット 10
    logic: inventory.md §1.2、§0.2
    violation: Cargo.toml か Cargo.lock が変わる、または nfr_source の 6 件のどれかが落ちる
    source: intent-statement.md(SM1)

  # ---- 項目 2: converter ----
  - id: BR2.1
    statement: converter の統合テスト tests/convert_roundtrip.rs の 1 件(出力は毎回同じ)と定数 PIPELINE_TOML・PIPELINE_YAML を tests/convert_ac.rs へ移し、元を消す(T1)。モジュールの説明を合わせる。
    category: constraint
    applies_to: crates/converter/tests/convert_ac.rs、crates/converter/tests/convert_roundtrip.rs
    trigger: コミット 1
    logic: functional-spec.md §2.2、inventory.md §2 の C1
    violation: 元のファイルが残る、または convert_ac の件数が 9 でない
    source: intent-statement.md(SM2)
  - id: BR2.2
    statement: converter の単体テスト src/tests/convert_ac.rs の 2 件と補助 fixture・request を src/tests/convert_roundtrip.rs の最後の節へ移し、元を消す(T2)。src/tests/mod.rs の mod convert_ac を消し、「ファイル名は元の統合テストと同じ」の説明を、下位のモジュールの中身を並べる説明にする。
    category: constraint
    applies_to: crates/converter/src/tests/convert_ac.rs、crates/converter/src/tests/convert_roundtrip.rs、crates/converter/src/tests/mod.rs
    trigger: コミット 2
    logic: functional-spec.md §2.2、inventory.md §2 の C2、C4
    violation: 元のファイルが残る、または成り立たない説明が残る
    source: intent-statement.md(SM2)
  - id: BR2.3
    statement: converter の tests/common(kinds だけ)の、誤りの種類の一覧・網羅の match・コンパイル時の確かめを tests/nfr_security.rs の表示の節の先頭へ移し、tests/common/ を消す(T3)。別名はファイルの K にそろえ、pub を外す。製品のコードの検査(src の直下の 10 ファイル以上を読む部分)は変えず、数(10 件 + 無視 1)も変わらない。
    category: constraint
    applies_to: crates/converter/tests/nfr_security.rs、crates/converter/tests/common/
    trigger: コミット 3
    logic: functional-spec.md §2.2、inventory.md §2 の C3
    violation: tests/common/ が残る、一覧の網羅の確かめが無くなる、または検査の数や中身が変わる
    source: intent-statement.md(SM2)

  # ---- 項目 3: cli の小さな単体テスト(Q5: A) ----
  - id: BR3.1
    statement: cli の src/tests/init_types.rs の残る 1 件(ハイフンだけが標準出力でそれ以外はファイル)を src/tests/args_grammar.rs の init の節の最後へ移し、元を消す(T6)。
    category: constraint
    applies_to: crates/cli/src/tests/init_types.rs、crates/cli/src/tests/args_grammar.rs
    trigger: コミット 6
    logic: functional-spec.md §2.3、inventory.md §3.2 の A1
    violation: 元のファイルが残る、または init の節の外に置く
    source: intent-statement.md(SM2)
  - id: BR3.2
    statement: cli の src/tests/run_id_arg.rs の 1 件を args_grammar.rs の --run-id と --all の節の最後へ移し、元を消す(T7)。同じコミットで、resolve_run_id の節に紛れ込んだ fr2_5_remove_on_complete_ の 2 件を、resolve_run_id の節の後に作る --remove-on-complete の節へ並べ直す(T8。D4。決定: 推奨のとおり。ユーザー、2026-11-25)。本体は変えない。モジュールの説明に、引数から作る型と resolve_run_id も確かめることを足す。
    category: constraint
    applies_to: crates/cli/src/tests/run_id_arg.rs、crates/cli/src/tests/args_grammar.rs
    trigger: コミット 7
    logic: functional-spec.md §2.3、inventory.md §3.1、§3.2 の A2、A3
    violation: 元のファイルが残る、resolve_run_id の節に resolve_run_id と無関係のテストが残る、または節の並びが init → --run-id と --all → resolve_run_id → --remove-on-complete → graph でない
    source: intent-statement.md(SM2)
  - id: BR3.3
    statement: cli の src/tests/graph_long_id.rs の 2 件と補助を src/tests/graph_report.rs の最後の節へ移し、元を消す(T9)。補助 definition の衝突は、移す側を long_id_definition に名前を変えて解く(呼ぶ 2 行が変わる。D2。決定: 推奨のとおり。ユーザー、2026-11-25)。
    category: constraint
    applies_to: crates/cli/src/tests/graph_long_id.rs、crates/cli/src/tests/graph_report.rs
    trigger: コミット 8
    logic: functional-spec.md §2.3、inventory.md §3.2 の A4
    violation: 元のファイルが残る、または本体で補助の名前のほかが変わる
    source: intent-statement.md(SM2)

  # ---- 項目 4: runtime ----
  - id: BR4.1
    statement: runtime の tests/runner_env.rs の補助 child_env と 5 件を、tests/runner_basic.rs の #[cfg(windows)] の 3 件の前の節へ移し、元を消す(T4)。5 件は OS に依らないので #[cfg] を付けない。#[cfg(windows)] の 3 件はファイルの最後のままにする。
    category: constraint
    applies_to: crates/runtime/tests/runner_env.rs、crates/runtime/tests/runner_basic.rs
    trigger: コミット 4
    logic: functional-spec.md §2.4、inventory.md §4 の R1
    violation: 元のファイルが残る、移したテストに cfg が付く、または Windows の対象で clippy が止まる
    source: intent-statement.md(SM2)
  - id: BR4.2
    statement: runtime の tests/state_read_raw.rs の定数 COMPLETED と 4 件を、tests/state_read.rs の最後の子のモジュール mod raw へ移し、元を消す(T5。D3。決定: 推奨のとおり。ユーザー、2026-11-25)。子のモジュールの中で use super::common::{def, put} と、conductor_runtime の read_state・read_state_raw・StateErrorKind、conductor_core の RunId を import する(親の run ID なしの read_state と衝突させない)。mod raw に cfg は付けず、state_read.rs の #[cfg(unix)] のテストは動かさない。
    category: constraint
    applies_to: crates/runtime/tests/state_read_raw.rs、crates/runtime/tests/state_read.rs
    trigger: コミット 5
    logic: functional-spec.md §2.4、inventory.md §4 の R2
    violation: 元のファイルが残る、本体で字下げのほかが変わる、または state_read の件数が 19 でない
    source: intent-statement.md(SM2)

  # ---- 項目 5: cli のテストの補助の写し(Q3: A) ----
  - id: BR5.1
    statement: cli の FakeEnv・FakeSink・state_error を crates/cli/tests/common/fakes.rs 1 つにだけ置く。統合テストの tests/common/mod.rs は mod fakes; と pub use fakes::{FakeEnv, FakeSink, state_error}; で読み、単体テストの src/tests/common/mod.rs は #[path = "../../../tests/common/fakes.rs"] mod fakes; と同じ再公開で読む。2 つの common/mod.rs の写しを消す(T12)。使う側の行は変えない。
    category: constraint
    applies_to: crates/cli/tests/common/fakes.rs、crates/cli/tests/common/mod.rs、crates/cli/src/tests/common/mod.rs
    trigger: コミット 11
    logic: functional-spec.md §2.5、inventory.md §5.2
    violation: 偽の実装が 2 か所以上にある、または使う側のテストの行が変わる
    source: intent-statement.md(SM3)
  - id: BR5.2
    statement: fakes.rs は use super::{Env, Language, Sink}; で、trait と言語を読む側の親のモジュールから取る。統合テストは conductor_cli の公開の trait とテスト用の写しの enum Language を、単体テストは crate::deps::{Env, Sink} と crate::types::Language を親に置く。統合テストで使われない項目(color_read)は親の #![allow(dead_code)] で警告にならない。使われなくなる import(統合テストの use std::io)は消す。
    category: constraint
    applies_to: crates/cli/tests/common/fakes.rs、crates/cli/tests/common/mod.rs、crates/cli/src/tests/common/mod.rs
    trigger: コミット 11
    logic: functional-spec.md §2.5、inventory.md §0.2(unused_imports で止まったこと)、§5.2
    violation: Language の型ごとに写しが要る、unused_imports か dead_code の警告で clippy が止まる、または macOS か Windows の対象でコンパイルが通らない
    source: intent-statement.md(SM3)
  - id: BR5.3
    statement: #[path] は src/tests/common/mod.rs(単体テスト)にだけ置き、製品のコード(src/tests/ の外)に置かない。cli の nfr_source.rs の、製品のコードの #[path と include の 3 つのマクロを止める検査は、数(6 件)と中身を変えずに通る。fakes.rs は src/ の外なので、src/ を見る検査の対象にならない。
    category: validation
    applies_to: crates/cli/src、crates/cli/tests/nfr_source.rs
    trigger: コミット 11 と実装の完了
    logic: inventory.md §5.2、functional-spec.md §7 の検索(src/tests の外の include・#[path が 0 件)
    violation: 製品のコードに #[path か include が入る、または nfr_source の 6 件のどれかが落ちる
    source: intent-statement.md(SM3)

  # ---- 項目 6: init のひな形の原本(Q4: A) ----
  - id: BR6.1
    statement: init のひな形の原本(日本語と英語)を crates/cli/tests/fixtures/init_ja.toml・init_en.toml(D8。決定: 推奨のとおり。ユーザー、2026-11-25)にだけ置き、中身は今の生の文字列とバイト単位で同じにする。tests/flow_init.rs は include_str!("fixtures/…")、src/tests/init_template.rs は include_str!("../../tests/fixtures/…") で読む。定数の名前と型(&str)は変えず、使う側の行は変えない(T13)。include_str! はテストの側(src/tests/ と tests/)にだけあり、製品のコードの検査の外。README のテストは本番の出力と比べるままで、変えない。
    category: constraint
    applies_to: crates/cli/tests/fixtures/、crates/cli/tests/flow_init.rs、crates/cli/src/tests/init_template.rs
    trigger: コミット 12
    logic: functional-spec.md §2.6、inventory.md §6
    violation: 原本の文字列がテストのソースに残る、データのファイルが元の文字列と 1 バイトでも違う、または製品のコードに include_str! が入る
    source: intent-statement.md(SM4)
  - id: BR6.2
    statement: データのファイルは、どの OS の取り出しでも改行を LF に保つ。リポジトリの根に .gitattributes を作り、crates/cli/tests/fixtures/*.toml text eol=lf を置く(D5。決定: 推奨のとおり。ユーザー、2026-11-25。データのファイルと同じコミット)。Windows での実行は CI の test(windows)で確かめる。
    category: constraint
    applies_to: .gitattributes、crates/cli/tests/fixtures/
    trigger: コミット 12 と CI
    logic: inventory.md §0.2、§6.2、functional-spec.md §10 の D5
    violation: git check-attr で eol が lf でない、または Windows の CI でひな形の全文の一致が落ちる
    source: intent-statement.md(SM4)

  # ---- 仕様書 ----
  - id: BR7.1
    statement: docs/spec/rust の、動かしたファイルを指す記述を、そのファイルを動かすコミットで直す(02 章のファイルの表と偽の実装の文、07 章のテストの表、11 章のテストのファイルの表。S1〜S16)。消えたファイルの行は消すか、移す先の行に中身を足す。仕様書にテストの件数の記述は無いので、直す件数は無い。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md、docs/spec/rust/07-process-and-signals.md、docs/spec/rust/11-quality-and-build.md
    trigger: コミット 1〜12
    logic: inventory.md §7.1、functional-spec.md §8 の各コミット
    violation: 無くなったファイルを指す記述が残る、または移す先の行が中身を書かない
    source: intent-statement.md(SM7)
  - id: BR7.2
    statement: 言語に依らない仕様書 11 章の受け入れテストの根拠の欄のうち、動かしたテストを指す 10 か所(L1〜L10)を、新しいモジュールの名前に直す(D6。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。772 行目の前からの食い違い(tests/common/kinds.rs)は今回は直さない(D7。決定: 直さない。推奨のとおり。ユーザー、2026-11-25)。ほかの章と、凍結した仕様書は変えない。
    category: constraint
    applies_to: docs/spec/language-agnostic/11-quality-and-build.md
    trigger: コミット 1、2、5、9(L8a)、10
    logic: inventory.md §7.2、functional-spec.md §10 の D6、D7
    violation: 根拠の欄が無くなったファイルか、別のファイルへ移ったテストを指す、または決めた所の外が変わる
    source: intent-statement.md(SM7)
  - id: BR7.3
    statement: 実装の完了のとき、functional-spec.md §7 の検索で、無くなったファイルの名前(docs_*、state_read_raw、runner_env.rs、init_types、run_id_arg.rs、graph_long_id、src/tests/convert_ac)が docs/spec/rust と言語に依らない仕様書に 0 件であることと、残ってよい行(core と cli の単体テストの common/kinds.rs、772 行目)だけが当たることを確かめる。
    category: validation
    applies_to: docs/spec
    trigger: 実装の完了
    logic: functional-spec.md §7
    violation: 検索の結果が見込みと違う
    source: intent-statement.md(SM7)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(出力、終了コード、ステートファイル、--dry-run の出力)を変えない。本番のコード(crates/*/src の src/tests/ の外)は 1 行も変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: git diff 16ac9d6 -- <本番のコードのファイル> が空。全テストが期待を変えずに通る
    violation: 本番のコードが変わる、または既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM8)
  - id: BR9.2
    statement: テストの変更は functional-spec.md §4 の T1〜T13 だけにする。一覧は、対象のファイルとテスト名と補助の名前と参照を grep で網羅し、写しの木で全項目を当てて作る。移すテストの名前・確かめの行・期待の値は変えない。本体で変わってよいのは、補助の名前の衝突を避ける 5 行(D1 の 3 行、D2 の 2 行)と、子のモジュールへ移す 4 件の字下げ(D3)だけ。消すテスト・足すテスト・名前を変えるテストは無い(★ なし)。実装で増減が出たら、その数と理由を記録し、名前を変える・消すなら承認を取る。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§7(本体の比べ)、inventory.md §0.2、§9
    violation: 一覧に無いテストのファイルの変更、確かめの行や期待の値の変化、テスト名の変化、または理由の無い増減
    source: intent-statement.md(SM5)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごと・実行ファイルごとに記録する。合格は失敗 0 で、全体が「基準 1,149 − 消したテスト 0 + 足したテスト 0 = 1,149」(と無視 1。cli の単体 172、統合 284)に一致すること(macOS)。テスト名の一覧の差は、名前の最後の部分で比べて 0。パスつきの差は、§4.1 の T 表の 28 件の前と後だけ。統合テストの間で移る 6 件は、実行ファイルごとの件数(convert_ac 9、runner_basic 17)で確かめる。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 各コミット(件数)と実装の完了
    logic: functional-spec.md §6、§7、§8 の件数の列
    violation: 件数の説明できない増減、または名前の最後の部分の差が 0 でない
    source: intent-statement.md(SM5)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と x86_64-pc-windows-msvc の対象のコンパイル、本番の形 -p conductor-cli --lib --bins)、全テスト、行カバレッジ 80% 以上、MSRV 1.100、cargo doc の警告が基準の 5 件のまま、が通る。新しい依存(dev-dependencies を含む)、unsafe、製品のコードの cfg(unix)/cfg(windows) を足さない。テスト名とドキュメントコメントは日本語。
    category: validation
    applies_to: 全体
    trigger: 各コミットと実装の完了
    logic: CI と同じ検査。functional-spec.md §7、§8 の各コミットの確かめ
    violation: マージの関門で止まる
    source: intent-statement.md(SM9)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。タグとリリースはしない。凍結した仕様書(go、python、typescript、csharp、powershell)は触らない。CI の設定と、CI が名前で指すテストのファイル(nfr_perf.rs)は変えない。1 項目 1 コミット程度の小さな並び(12 コミット)にし、どのコミットでもビルドとテストが通る。移す先が新しいファイルなら git mv(docs_graph.rs → docs.rs)、既にあるファイルへの統合なら、同じコミットで中身を足して元を git rm する。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、.github、docs/spec、コミットの並び
    trigger: 常に
    logic: functional-spec.md §1、§8、§8.1
    violation: 版が変わる、凍結した仕様書か CI が変わる、途中のコミットでビルドかテストが止まる、または同じテストが 2 つある・無いコミットがある
    source: intent-statement.md(SM8)
  - id: BR9.6
    statement: 製品のコードの検査(cli の nfr_source.rs 6 件、runtime の nfr_source.rs 5 件、converter の nfr_security.rs 10 件 + 無視 1)は、テストの数と中身を変えずに、各コミットで通る。converter の nfr_security.rs は、誤りの種類の一覧を受け入れるだけで、検査の部分は変えない。
    category: validation
    applies_to: crates/cli/tests/nfr_source.rs、crates/runtime/tests/nfr_source.rs、crates/converter/tests/nfr_security.rs
    trigger: 各コミットと実装の完了
    logic: functional-spec.md §7、inventory.md §0.2
    violation: 検査のテストの数か中身が変わる、またはどれかが落ちる
    source: intent-statement.md(SM6)
```
