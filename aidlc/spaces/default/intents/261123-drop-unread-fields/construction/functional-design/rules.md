# ビジネスルール(Rules): 読まれない項目の削除

詳細の場所は `inventory.md`(変える場所、仕様書の S1〜S21)、承認が要るテストの変更(T 表)、★ の名前の変更、確かめの行を消すもの、件数の見込み、決めてほしい点は `functional-spec.md` の §4、§4.5、§4.6、§6、§10。

```yaml
rules:
  # ---- 項目 1(方針: テストのコードの都合で本番のコードを曲げない。本番にテストのための要素を足さない) ----
  - id: BR1.1
    statement: core の FlowEdge から explicit を消す。target_position は行き先の位置だけを返し、TransitionResolution::Default の区別は位置(記述順の次)を決めるのに残す。辺の並べ方、まとめ方、飛び越すジョブは変えない。テストは辺を行き先、種類、飛び越すジョブで比べ、explicit だけの確かめの 4 行は消す(書いた遷移か記述順の次かの区別は、本番が読まないので、テストでも確かめなくなる)。
    category: constraint
    applies_to: crates/core/src/graph/mod.rs、graph/edges.rs、crates/core/tests/graph_edges.rs、graph_lookup_perf.rs、graph_drawable.rs、graph_types.rs、graph_common/mod.rs
    trigger: 項目 1 のコミット
    logic: inventory.md 1、functional-spec.md §2.1、§4.1
    violation: 書かれるだけで読まれない辺の項目が残る、または辺の行き先・並び・まとめ方が変わる
    source: intent-statement.md(SM1)

  # ---- 項目 2(EnvOrigin と origin) ----
  - id: BR2.1
    statement: 計画の環境変数の名前から出どころを消す。EnvNameEntry は名前だけの構造体(EnvNameEntry { name })として残し、PlanEntry.env_names: Vec<EnvNameEntry> と再公開は今のまま(D1。決定: 代案。ユーザー、2026-11-24)。merge_env_names は名前だけを返す。cli の src は変えず、--dry-run の表示は名前だけのまま変わらない。
    category: constraint
    applies_to: crates/core/src/plan/mod.rs、plan/simulate.rs、envfile/merge.rs、crates/core/tests/plan_types.rs
    trigger: 項目 2a のコミット
    logic: inventory.md 2a、functional-spec.md §2.2、§4.2、§10 の D1
    violation: 計画が名前のほかの読まれない項目を持つ、cli の src が変わる、または --dry-run の出力が変わる
    source: intent-statement.md(SM1)
  - id: BR2.2
    statement: core の EnvOrigin と EffectiveEntry.origin を消し、統合の項目は名前と値の組にする。並べ方、後ろが勝つ規則、自動の変数の扱いは変えない。EffectiveEntry の手書きの Debug は残し、値を伏せ字にする。テストは層を値で確かめ、出どころだけの確かめの行は消す。
    category: constraint
    applies_to: crates/core/src/envfile/mod.rs、envfile/merge.rs、lib.rs、crates/core/tests/env_merge.rs、env_merge_perf.rs、env_types.rs
    trigger: 項目 2b のコミット
    logic: inventory.md 2b、functional-spec.md §2.2、§4.3
    violation: 読まれない出どころが残る、ジョブに足す環境(名前、値、並び)が変わる、または Debug に値が出る
    source: intent-statement.md(SM1)

  # ---- 項目 3(.env の内容の path と項目の line) ----
  - id: BR3.1
    statement: core の EnvFileContent.path を消す。parse_env_file の file は、誤りと警告のファイル名に使うので残す。テストはファイル名を警告の file で確かめる。同じコミットで、rust の仕様書 05 の 14.1 の LoadedEnvFiles の行から、実物に無い add(記述, content) を消す(D3。決定: 推奨のとおり。ユーザー、2026-11-24。S21)。
    category: constraint
    applies_to: crates/core/src/envfile/mod.rs、envfile/parse.rs、crates/core/tests/env_file_parse.rs、env_merge_perf.rs、plan_perf.rs、plan_types.rs、crates/cli/tests/plan_report.rs、docs/spec/rust/05-execution-flow.md
    trigger: 項目 3a のコミット
    logic: inventory.md 3a、functional-spec.md §4.4
    violation: 読まれないパスが残る、または誤りと警告の表示のファイル名が変わる
    source: intent-statement.md(SM1)
  - id: BR3.2
    statement: core の EnvEntry.line を消す。重複キーの警告の行(EnvWarning.line)は今の line_no のまま書く。EnvEntry の手書きの Debug は残し、値を伏せ字にする。テストは項目を名前、値、並びで確かめる。
    category: constraint
    applies_to: crates/core/src/envfile/mod.rs、envfile/parse.rs、crates/core/tests/env_parse_perf.rs、env_merge_perf.rs、plan_perf.rs、plan_types.rs、env_types.rs、crates/cli/tests/plan_report.rs
    trigger: 項目 3b のコミット
    logic: inventory.md 3b、functional-spec.md §4.4
    violation: 読まれない行番号が残る、重複キーの警告の行が変わる、または Debug に値が出る
    source: intent-statement.md(SM1)

  # ---- 項目 4(仕様書) ----
  - id: BR4.1
    statement: docs/spec/rust の該当箇所(EnvOrigin、EnvNameEntry の origin、名前と出どころ、FlowEdge の explicit とどちらかが true、EnvFileContent の path、EnvEntry の line、LoadedEnvFiles の add)を、参照実装の形に合わせて直す(inventory.md §4 の S1〜S3、S6〜S9、S12〜S15、S17〜S19、S21。S5 の再公開の EnvNameEntry は D1 の決定で変えない)。計画の表示が名前だけであること、重複キーの警告の行、図の描き方の記述は残す。版の注記は足さない。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md、03-definition-format.md、05-execution-flow.md、11-quality-and-build.md、12-rebuild-guide.md
    trigger: 各項目のコミット
    logic: inventory.md §4、functional-spec.md §7 の仕様書の検索
    violation: 消した項目の記述が残る、または残すべき記述が消える
    source: intent-statement.md(SM2)
  - id: BR4.2
    statement: docs/spec/language-agnostic の該当箇所(名前の出どころ、(name, origin)、FlowEdge の explicit とどちらかが true、.env の項目の名前、値、行、EnvFileContent の解決済みのパス)を、参照実装に合わせて直す(Q2: A。inventory.md §4 の S4、S9〜S11、S16、S18、S20。計画の名前は rust の EnvNameEntry { name } と同じ形の list<{ name }>)。定義ファイルのキーとしての format_version と description と、その検査の記述は変えない。凍結した仕様書は触らない。
    category: constraint
    applies_to: docs/spec/language-agnostic/02-architecture.md、03-definition-format.md、05-execution-flow.md、12-rebuild-guide.md
    trigger: 各項目のコミット
    logic: inventory.md §4、functional-spec.md §7 の仕様書の検索
    violation: 消した項目の記述が残る、残すべき記述が消える、または凍結した仕様書が変わる
    source: intent-statement.md(SM2)

  # ---- 範囲の外(Q3) ----
  - id: BR5.1
    statement: 定義の format_version と description は PipelineDefinition の項目として残す。キーの検査、誤りの表示、仕様書の定義の形の記述も変えない。
    category: constraint
    applies_to: crates/core/src/types.rs、crates/core/src/definition、docs/spec
    trigger: 常に
    logic: functional-spec.md §1、§3
    violation: format_version または description の項目や検査が変わる
    source: intent-statement.md(SM3)

  # ---- 足すテスト ----
  - id: BR8.1
    statement: 新しいテストは足さない(見込み 0 件)。テストも消さない。消す項目の代わりは、既存のテストの書き換えと、残る項目の既存の確かめで持つ。実装で足す必要が出たら、その数と理由を記録する。
    category: validation
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §5
    violation: 一覧に無いテストを足す・消す、または増減が説明されない
    source: intent-statement.md(SM6)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run と graph の出力、.env の誤りと警告の表示)を変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテストが、T 表の書き換えのほかは、期待を変えずに通る
    violation: 既存のテストの期待する出力の文字列を変えないと通らない
    source: intent-statement.md(SM5)
  - id: BR9.2
    statement: 既存のテストの変更は functional-spec.md §4 の T1〜T20 だけにする。一覧は、消す名前のすべての参照を grep -rn で網羅し(tests の下の補助のディレクトリと、src の #[cfg(test)] の単体テストを含む)、写しの木で本番だけを変えたコンパイルのエラーの場所と照らして作る。名前を変えるテストは §4.5 の ★ 2 件だけ(名前を変える方針は決定: D2、推奨のとおり、ユーザー、2026-11-24。個々の名前は承認待ち)。確かめの行を消すものは §4.6 の 13 行(コメントの行を含む)だけで、個々の削除は設計の承認で認めてもらう(承認待ち)。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§4.5、§4.6、inventory.md §0.2
    violation: 一覧に無いテストのファイルの変更、一覧に無いテスト名の変化、または理由と残る確かめの無い行の削除
    source: intent-statement.md(SM4、SM6)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,153 − 消したテスト 0 + 足したテスト 0 = 1,153」に一致すること(macOS)。テスト名の一覧の差は ★ の 2 件の名前の変更だけ。各項目の完了は、inventory.md の完了の判定と functional-spec.md §7 の検索で確かめる。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 実装の完了
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、一覧に無いテスト名の変化、または参照の残り
    source: intent-statement.md(SM4、SM6)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と、Windows の対象のコンパイル)、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。runtime の製品のコードにパニックしうる書き方を、cli の src に panic!( を書かない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(SM7)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。仕様書は rust と language-agnostic だけを直し、版の注記を足さずに今の形を書く。凍結した仕様書は触らない。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、docs/spec/rust、docs/spec/language-agnostic
    trigger: 常に
    logic: functional-spec.md §1
    violation: 版が変わる、版の注記が足される、または凍結した仕様書が変わる
    source: intent-statement.md(SM5)
  - id: BR9.6
    statement: 消す項目を指すドキュメントコメントと仕様書の記述を、同じコミットで直す。cargo doc --workspace --no-deps の警告は、基準の 5 件から増やさない。
    category: validation
    applies_to: crates/core/src と crates/cli/src のドキュメントコメント、docs/spec/rust、docs/spec/language-agnostic
    trigger: 各コミット
    logic: inventory.md 各項目の「説明」「仕様書」、§0.2
    violation: 消した項目を指す説明やリンクが残る
    source: intent-statement.md(SM4)
```
