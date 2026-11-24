# ビジネスルール(Rules): converter の testing モジュールの削除

詳細の場所は `inventory.md`(変える場所、仕様書の S1〜S12、写しでの検証)、承認が要るテストの変更(T 表)、消すテスト、件数の見込み、コミットの並び、決めてほしい点は `functional-spec.md` の §4、§4.5、§6、§8、§10。

```yaml
rules:
  # ---- 項目 1・2(方針: テストのコードの都合で本番のコードを曲げない。本番にテストのための要素を足さない) ----
  - id: BR1.1
    statement: converter の lib.rs から #[doc(hidden)] pub mod testing(Value の再公開と read_tree)と、その説明を消す。非公開の read_value は今のまま残す。公開の入口(convert、convert_text と、その型)は変えない。クレートの説明から、doc(hidden) の公開の記述を消す。
    category: constraint
    applies_to: crates/converter/src/lib.rs、crates/converter/src/types.rs(モジュールの説明)
    trigger: コミット 5(テストを移し終えた後)
    logic: inventory.md 1、functional-spec.md §2
    violation: testing、read_tree、doc(hidden) の参照が残る、または公開の入口の形が変わる
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: TargetFormat::opposite(impl TargetFormat ごと)を消す。それだけを確かめる単体テスト 1 件を消す(D3。決定: 消す。推奨のとおり。ユーザー、2026-11-24。理由は functional-spec.md §4.5。個々の削除は承認待ち)。TargetFormat の「入力の形式は反対とみなす」という性質の説明は残す。テストで反対の形式が要る所は、テストの中の match で求める。
    category: constraint
    applies_to: crates/converter/src/types.rs、crates/converter/src/tests/convert_roundtrip.rs
    trigger: コミット 5
    logic: inventory.md 2、functional-spec.md §4.2、§4.5
    violation: opposite が残る、または本番にテストのための関数が足される
    source: intent-statement.md(SM1)

  # ---- 項目 3(Value の PartialEq) ----
  - id: BR2.1
    statement: Value の手書きの impl PartialEq に #[cfg(test)] を付け、テストのときだけコンパイルする(Q3。A)。HashMap の import は eq の本体の中へ移す。比べ方(NaN どうしは等しい、0.0 と -0.0 は異なる、マップはキーの順を問わない、配列は順を見る)は変えない。impl は types.rs の製品の項目の後ろ(enum Value の後、テストのモジュールの前)に置く。本番の形(cargo build、cargo clippy --lib)が PartialEq なしで通る。
    category: constraint
    applies_to: crates/converter/src/types.rs
    trigger: コミット 6
    logic: inventory.md 3、functional-spec.md §2.1
    violation: PartialEq が本番の成果物に入る、比べ方が変わる、または本番の形で使われない import の警告が出る
    source: intent-statement.md(SM3)

  # ---- テストの移動 ----
  - id: BR3.1
    statement: 補助 yaml()・toml()(testing::read_tree)か Value を使う統合テスト(45 件)を、src の単体テストへ移す(Q2。A。D1。決定: 推奨のとおり。ユーザー、2026-11-24。T1〜T7 の表は承認待ち)。移すテストの名前は変えない(★ なし)。本体の確かめの行(呼ぶ関数、入力、期待の木と値、誤りの種類と理由の文)は変えず、変えるのはファイルの先頭の import、Value の書き方(testing::Value:: を Value:: に)、include_str! のパス、補助の opposite を match にすることだけ。補助 yaml()・toml() は read_value を同じ向き(yaml は target Toml と t.yaml、toml は target Yaml と t.toml)で呼ぶ。
    category: constraint
    applies_to: crates/converter/src/tests/*.rs、crates/converter/tests/read_resolve.rs、convert_roundtrip.rs、convert_ac.rs、nfr_security.rs
    trigger: コミット 1〜4
    logic: functional-spec.md §4.1〜§4.4、§4.7
    violation: 確かめの行や期待の値が変わる、テスト名が変わる、または一覧に無いテストが移る
    source: intent-statement.md(SM2)
  - id: BR3.2
    statement: 移す先は src/tests/ のディレクトリ(mod.rs と、元の統合テストと同じ名前の 4 つのファイル)とし、lib.rs のファイルの最後に #[cfg(test)] mod tests; を置く(D2。決定: 推奨のとおり。ユーザー、2026-11-24)。統合テストのソースの検査(src/*.rs の直下を読み、最初の #[cfg(test)] の文字より前を製品のコードとみなす)を弱めないため、単体テストを src の直下のファイルに置かず、製品のコードより前の説明やコメントに #[cfg(test)] の文字を書かない。本番に pub、doc(hidden)、テストのための引数や関数を足さない。src に unsafe と cfg(unix)/cfg(windows) を足さない。Value の PartialEq に #[cfg(test)] を付けると、types.rs の製品のコードの範囲がその impl の前で終わり、eq の本体はソースの検査の範囲の外になるが、テストのときだけのコードなので害はない(製品のコードの範囲は今と同じく enum Value までを含む)。src/tests/ も、src の直下だけを読むソースの検査の外になる(D2 の結果)が、移すコードには検査の禁止の語(std::process、std::net、std::env、use std::os、Command::new、静的な初期化の語、ファイルの書き込みの API)が無い(functional-spec.md §7 の検索で確かめ、コード生成の記録に残す)。
    category: constraint
    applies_to: crates/converter/src/lib.rs、crates/converter/src/types.rs、crates/converter/src/tests/
    trigger: コミット 1〜6
    logic: functional-spec.md §1、§2.2、inventory.md §0.2
    violation: 本番の可視性が広がる、ソースの検査の製品のコードの範囲が短くなる、または src にテストのための公開が増える
    source: intent-statement.md(SM2)
  - id: BR3.3
    statement: 移さないテスト(convert_ac.rs の 8 件、convert_roundtrip.rs の 1 件、nfr_security.rs の 10 件と無視の 1 件)は統合テストに残し、本体を変えない。変えるのは、使わなくなった mod common と import の削除、モジュールの説明だけ。tests/common/mod.rs は pub mod kinds; だけにする(コミット 4)。tests/read_resolve.rs は消える(コミット 1)。
    category: constraint
    applies_to: crates/converter/tests/
    trigger: コミット 1〜4
    logic: functional-spec.md §4.2〜§4.4、§4.6
    violation: 残るテストの確かめが変わる、または公開の入口だけで確かめるテストが単体テストへ移る
    source: intent-statement.md(SM2)

  # ---- 仕様書 ----
  - id: BR4.1
    statement: docs/spec/rust の該当箇所を、実装の形に合わせて直す(inventory.md §5 の S1〜S12。S13 の 11 章の「時間の上限」の行は、プロセスを起動するテストを指すので変えない)。testing の行と lib.rs の責務の testing、TargetFormat の opposite() を消し、等価の比べ方はテストのときだけ持つことを足し、11 章の converter のテストの表(read_resolve.rs の行を消し、単体テスト src/tests/ の表を足す)、3 章の表の根拠、4.1 の方針、09 の冒頭のテストの場所を直す。テストの件数は仕様書に書かれていないので、直す件数は無い。版の注記は足さない。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md、09-converter.md、11-quality-and-build.md
    trigger: 各コミット(S の「コミット」の列)
    logic: inventory.md §5、functional-spec.md §7 の仕様書の検索
    violation: testing、read_tree、opposite の記述が残る、テストの置き場所の記述が実物と違う、または残すべき記述(内部表現の等価、往復の保証)が消える
    source: intent-statement.md(SM4)
  - id: BR4.2
    statement: docs/spec/language-agnostic は直さない(testing、read_tree、opposite、PartialEq の記述が無く、11 章の根拠の欄の名前は、移す先のファイル名とテスト名が元と同じなので正しいまま。D4。決定: 直さない。推奨のとおり。ユーザー、2026-11-24)。凍結した仕様書(go、python、typescript、csharp、powershell)は触らない。
    category: constraint
    applies_to: docs/spec/language-agnostic、docs/spec/go、python、typescript、csharp、powershell
    trigger: 常に
    logic: inventory.md §5、functional-spec.md §10 の D4
    violation: 言語に依らない仕様書や凍結した仕様書が変わる
    source: intent-statement.md(SM4)

  # ---- 足すテスト・消すテスト ----
  - id: BR8.1
    statement: 新しいテストは足さない(見込み 0 件)。消すテストは opposite の単体テスト 1 件だけ(理由を添えて承認を取る)。実装で増減が出たら、その数と理由を記録する。
    category: validation
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4.5、§5
    violation: 一覧に無いテストを足す・消す、または増減が説明されない
    source: intent-statement.md(SM7)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(変換の出力、誤りの表示、終了コード)を変えない。変換の流れ(convert、convert_text、read_value、読み取り、展開、書き出し、誤りの組み立て)のコードは変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 残る統合テストと cli の convert のテストが、期待を変えずに通る
    violation: 既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM6)
  - id: BR9.2
    statement: テストの変更は functional-spec.md §4 の T1〜T7 だけにする。一覧は testing、read_tree、opposite、補助 common の参照を grep で網羅し(tests の下の補助のディレクトリと、src の #[cfg(test)] の単体テストを含む)、写しの木で本番だけを変えたコンパイルのエラーの場所(12 件)と照らして作る。名前を変えるテストは無い。消すテストは §4.5 の 1 件だけ。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、inventory.md §0.2
    violation: 一覧に無いテストのファイルの変更、テスト名の変化、または理由の無い削除
    source: intent-statement.md(SM5、SM7)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,153 − 消したテスト 1 + 足したテスト 0 = 1,152」(converter は単体 96、統合 19 と無視 1)に一致すること(macOS)。テスト名の一覧の差は、名前の最後の部分で比べて、消した 1 件だけ。完了は functional-spec.md §7 の検索で確かめる。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 実装の完了
    logic: functional-spec.md §6、§7
    violation: 件数の説明できない増減、一覧に無いテスト名の変化、または参照の残り
    source: intent-statement.md(SM5、SM7)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と、Windows の対象のコンパイル)、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、src の cfg を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。
    category: validation
    applies_to: 全体
    trigger: 各コミットと実装の完了
    logic: CI と同じ検査。functional-spec.md §8 の各コミットの確かめ
    violation: マージの関門で止まる
    source: intent-statement.md(SM8)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。仕様書は rust だけを直し、版の注記を足さずに今の形を書く。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、docs/spec/rust
    trigger: 常に
    logic: functional-spec.md §1
    violation: 版が変わる、または版の注記が足される
    source: intent-statement.md(SM6)
  - id: BR9.6
    statement: 消す名前(testing、read_tree、opposite、doc(hidden) の公開)を指すドキュメントコメントと、移したテストのモジュールの説明(どこに何があるか)を、同じコミットで直す。cargo doc --workspace --no-deps の警告は、基準の 5 件から増やさない。
    category: validation
    applies_to: crates/converter/src と crates/converter/tests のドキュメントコメント
    trigger: 各コミット
    logic: inventory.md の各項目の「説明」、§0.2
    violation: 消した名前を指す説明が残る、または警告が増える
    source: intent-statement.md(SM5)
```
