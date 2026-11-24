# 変える場所の一覧(Inventory): converter の testing モジュールの削除

HEAD 2d98f2a(converter-testing-module の意図の把握の記録のコミット)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。初期の記述と実物が違ったところは「(記述との違い)」と書く。設計の要点、テストの移動と変更の一覧(T…)、消すテスト、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent の Q3 と、この intent の Q1〜Q3): **テストのコードの都合で本番のコードを曲げない**。本番に、テストのための要素(`pub`、`#[doc(hidden)]`、テストのための引数や関数)を足さない。`#[cfg(test)]` の中は可。途中の表現(`Value` の木)を比べるテストは src の単体テストへ移し(Q2: A)、期待の木と比べる今の確かめ方と期待の値は変えない。`Value` の `PartialEq` はテストのときだけコンパイルする(Q3: A)。利用者から見える動き(変換の出力、誤りの表示、終了コード)は変えない。版は上げない。

決めてほしい点(`functional-spec.md` §10 の D1〜D4)は、ユーザーが決めた(すべて推奨のとおり。2026-11-24)。D1: 補助を使うテストはすべて移す。D2: `src/tests/` のディレクトリ。D3: `opposite` の単体テストを消す。D4: 言語に依らない仕様書は直さない。この文書の「(D1)」〜「(D4)」は、その決定のとおりの形を書いている。T1〜T7 の表と、D3 による個々のテストの削除は、設計の承認で認めてもらう(承認待ち)。

範囲の外(初期の記述): cli のライブラリの公開範囲の大きい部分。converter の公開の入口(`convert`、`convert_text`、`ConversionRequest`、`ConvertedText`、`TargetFormat`、`ConversionError`、`ConversionErrorKind`)の形。

## 0. 基準と、設計の検証

### 0.1 テストの件数(`cargo test --workspace --locked`、macOS、rustc 1.100.0、debug、HEAD 2d98f2a)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 112 | 270 | 382 |
| runtime | 69 | 128 | 197 |
| converter | 52 | 64 通過 + 1 無視 | 116 通過 + 1 無視 |
| cli | 2 | 456 | 458 |
| 全体 | | | **1,153 通過、1 無視、0 失敗** |

converter の統合テストの内訳(基準): `convert_ac.rs` 10、`convert_roundtrip.rs` 14、`nfr_security.rs` 12 + 1 無視、`read_resolve.rs` 28。テスト名の一覧は `cargo test --workspace --locked -- --list` の `: test` の行(1,154 行。無視の 1 件を含む)。core、runtime、cli のテストは変えない。

### 0.2 設計の検証(写しの木での試し)

T 表を検索だけに頼らないために、HEAD 2d98f2a の写し(`git archive HEAD` を `target/conv-testing/work` に展開)に、この一覧の全項目(本番)と `functional-spec.md` §4 の T 表を当て、次を確かめた。写しは確かめたあと消した(`target/conv-testing` ごと)。リポジトリのコードは変えていない。仕様書(`docs/spec`)の書き換えはコンパイルとテストに関わらないので、写しには当てていない。

- **本番のコードだけを変えた段階**(1、2、3 を当てた形)で、`cargo build --workspace --locked`(テストを含まない本番の形)は通る。`cargo check --workspace --locked --all-targets --keep-going --message-format=short` のエラーは **12 件**で、出た場所はすべて T 表にある:
  - src の単体テスト: `types.rs:131`、`:132`、`:134`、`:135`(`opposite` の 4 か所。T5 で消すテスト)
  - `tests/common/mod.rs:24`(`testing` の import。T6)
  - `tests/convert_roundtrip.rs:20`(`testing::Value` の import)、`:37`(`opposite`)(T2)
  - `tests/read_resolve.rs:139`、`:359`、`:360`、`:381`、`:383`(`testing::Value` の直接の参照。T1)
  - import の誤りの陰で出ない参照(`common::yaml`・`toml` を呼ぶ所)も、T1〜T4 の移すテストに含まれる(§4.0 の検索で網羅)。
- T 表を当てたあと、`cargo fmt --all -- --check` が通る(当てる途中で、`tests/nfr_security.rs` の末尾に空行が残ると止まった。削除の範囲は末尾の空行を含める)。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc` の両方で通る。`cargo clippy -p conductor-converter --lib --locked -- -D warnings`(テストを含まない形。`Value` の `PartialEq` が無い)も通る。
- `cargo test --workspace --locked` が **1,152 通過、1 無視、0 失敗**(core 112 + 270、runtime 69 + 128、converter **96** + **19 + 1 無視**、cli 2 + 456。`functional-spec.md` §6 の見込みと一致)。
- テスト名の一覧の差(名前の最後の部分で比べた。どちらも並べ替えて比べる): 消えた名前は `反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る` の 1 件だけ。増えた名前は 0 件。名前の変わったテストは 0 件(★ なし)。パスの変わったテストは 45 件(`read_resolve::…` などの統合テストのバイナリの名前から、単体テストの `tests::read_resolve::…` などへ。行の差は 91 行 = 45 × 2 + 1)。単体テストのパスの内訳は `tests::read_resolve` 28、`tests::convert_roundtrip` 13、`tests::convert_ac` 2、`tests::nfr_security` 2。名前の最後の部分の重複(別のクレートの同じ名前)は、前後とも 7 件で変わらない。
- `cargo doc --workspace --no-deps --locked` の警告は、基準と同じ 5 件(core 1、runtime 4。この intent と無関係)。消した `testing` へのドキュメントのリンクは無い(文の中の名前だけ)。
- 行カバレッジ(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`): 全体の行 **98.09%**。converter の src の各ファイルは 89.89%〜100%。`src/tests/` の下のファイルは計測の表に出ない(テストのコード)。
- ソースの検査(統合テスト `tests/nfr_security.rs` の 3 件。`src/*.rs` を読み、最初の `#[cfg(test)]` より前を製品のコードとみなす)が通る。写しで、`types.rs` の `Value` の説明に `` `#[cfg(test)]` `` の文字を書くと、検査の「製品のコード」の範囲が `enum Value` の前で切れることに気づき、説明の書き方を変えた(BR3.2。§1 と §2 の「説明」)。
- 写しで変わったファイル: 本番 2(`lib.rs`、`types.rs`)、テストの新しいファイル 5(`src/tests/mod.rs`、`read_resolve.rs`、`convert_roundtrip.rs`、`convert_ac.rs`、`nfr_security.rs`)、統合テスト 4(`tests/common/mod.rs`、`convert_ac.rs`、`convert_roundtrip.rs`、`nfr_security.rs`)、消したファイル 1(`tests/read_resolve.rs`)。converter のほかの src、core、runtime、cli は変わらない。

---

## 1. converter の `testing` モジュール

- 今: `crates/converter/src/lib.rs:160-176`(`/// テストのための公開。…`、`#[doc(hidden)]`、`pub mod testing { pub use crate::types::Value; pub fn read_tree(…) }`)。`read_tree` は `read_value(text, format.opposite(), file)` を呼ぶだけ。
- 本番の読み取り: 0 件(cli の src と cli のテストは `testing` を使わない。cli が使うのは `convert`、`TargetFormat`、`ConversionError`、`ConversionErrorKind`、`ConversionRequest`)。
- 目標: モジュールと説明を消す。`read_value`(非公開の関数。`lib.rs:148-158`)は今のまま残す(`convert_text` が使う)。単体テストは、クレートの中から `read_value` と `crate::types::Value` を直接使う(どちらも本番の変更なしで使える)。
- 説明: `lib.rs:35-36`(「ほかは、クレートの内部(`pub(crate)`)か、テストのための `#[doc(hidden)]` である。」を「ほかは、クレートの内部である。」に)、`:41-42`(`#[doc(hidden)]` の箇条を消す)。`types.rs:17-18`(「(テストのために `#[doc(hidden)]` で公開している)」を消す)。
- 単体テストの置き場所(D2。決定: 推奨のとおり。ユーザー、2026-11-24): `lib.rs` の**最後**に `#[cfg(test)] mod tests;` を置き、`src/tests/` のディレクトリに置く(`functional-spec.md` §2.2)。宣言の前のコメントに `#[cfg(test)]` の文字を書かない(BR3.2)。
- テストの使用(T1〜T4、T6): `tests/common/mod.rs:24`、`:27-33`(`yaml`・`toml`)、`tests/read_resolve.rs`(全 28 件)、`tests/convert_roundtrip.rs`(13 件)、`tests/convert_ac.rs:39-60`(2 件)、`tests/nfr_security.rs:423-564`(2 件と補助)。
- 仕様書: rust `02-architecture.md:132`(`testing::{Value, read_tree}` の行)、`:331`(`lib.rs` の責務の `testing`)。language-agnostic: 該当なし。
- 完了の判定: `grep -rn "testing::\|read_tree\|doc(hidden)" crates/converter docs/spec/rust` が 0 件。

## 2. `TargetFormat::opposite`

- 今: `crates/converter/src/types.rs:30-38`(`impl TargetFormat { pub fn opposite(self) -> Self }`)。単体テスト `types.rs:137-145`(`反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る`)。
- 本番の読み取り: 0 件(`lib.rs:177` の `testing::read_tree` だけ。`read_value` と `convert_text` は `match target` で入力の形式を決め、`opposite` を使わない)。
- 目標: `impl TargetFormat` ごと消す。`TargetFormat` の説明(「入力の形式は、これと反対の形式とみなす」)は残す(性質の説明で、関数の説明ではない)。
- 単体テスト: `反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る` を消す(T5。D3。決定: 消す。推奨のとおり。ユーザー、2026-11-24。消す関数だけを確かめるテスト。個々の削除は設計の承認で認めてもらう(承認待ち))。
- テストの使用: `tests/convert_roundtrip.rs:37`(`from.opposite()`)。移す先では、補助の関数の中の `match`(`Yaml → Toml`、`Toml → Yaml`)にする(T2。テストの中だけの書き方)。
- 仕様書: rust `02-architecture.md:129`(「(`opposite()`)」)。language-agnostic: 該当なし。
- 完了の判定: `grep -rn "opposite" crates docs/spec/rust docs/spec/language-agnostic` が 0 件。

## 3. `Value` の手書きの `PartialEq`

- 今: `crates/converter/src/types.rs:70-95`(`impl PartialEq for Value`。NaN どうしを等しく、`0.0` と `-0.0` を区別し、マップはキーの順を問わない)。そのための import `types.rs:20`(`use std::collections::HashMap;`)。
- 本番の読み取り: 0 件(写しで、`PartialEq` を `#[cfg(test)]` にしても、本番の形(`cargo build`、`cargo clippy --lib`)が通ることを確かめた)。
- 目標(Q3: A): `impl` の前に `#[cfg(test)]` を付ける。`use std::collections::HashMap;` は、ファイルの先頭から `eq` の本体の先頭へ移す(先頭に残すと、本番の形で「使われない import」になり、`-D warnings` で止まる)。`impl` は、`types.rs` の製品の項目の最後(`enum Value` の後、`#[cfg(test)] mod tests` の前)に置いたまま(BR3.2)。`#[derive(Debug, Clone)]` は今のまま。
- 説明: `types.rs:58-59`(`Value` の説明)の後ろに「等しさはテストのときだけ持つ(`cfg(test)`。本番の処理は木を比べない)。」を足す。**`#[cfg(test)]` の形の文字を説明に書かない**(ソースの検査の範囲が切れるため。§0.2)。
- テストの使用: `types.rs` の単体テストの 3 件(`nan_は_nan_と等しく_符号の違う0は異なる`、`種類が違えば等しくない`、`マップの等しさはキーの順を見ず_配列は順を見る`)は、変えずに通る。移すテストの `assert_eq!` は、単体テストなので `PartialEq` を使える。
- 仕様書: rust `02-architecture.md:134`(内部の途中の表現 `Value` の行)に、等価の比べ方はテストのときだけ持つことを足す。rust `09-converter.md:162`(内部表現の等価の定義)は、往復の保証の基準の定義なので変えない。language-agnostic: 該当なし。
- 完了の判定: `grep -n -B1 "impl PartialEq for Value" crates/converter/src/types.rs` の前の行が `#[cfg(test)]`。`grep -n "^use std::collections::HashMap" crates/converter/src/types.rs` が 0 件。

## 4. テスト(移す・変える・消す。詳細は `functional-spec.md` §4)

| ファイル(HEAD) | 件数 | 扱い |
|---|---|---|
| `tests/read_resolve.rs` | 28 | 28 件すべてを `src/tests/read_resolve.rs` へ移す。統合テストのファイルは消える(T1) |
| `tests/convert_roundtrip.rs` | 14 | 13 件を `src/tests/convert_roundtrip.rs` へ移す。`出力は毎回同じ` の 1 件は統合テストに残る(T2) |
| `tests/convert_ac.rs` | 10 | `ac_7_1_1_toml_から_yaml`、`ac_7_1_2_アンカーつきの_yaml_から_toml` の 2 件を `src/tests/convert_ac.rs` へ移す。8 件は残る(T3) |
| `tests/nfr_security.rs` | 12 + 1 無視 | `ランダムな入力と切り詰めた入力で_パニックしない`、`変換できたランダムな構造は往復する` の 2 件と補助(`Lcg`、`yaml_key`)を `src/tests/nfr_security.rs` へ移す。10 件 + 1 無視は残る(T4) |
| `tests/common/mod.rs` | — | `yaml`・`toml`・`s`・`i`・`m`・`a`・`with_timeout` を `src/tests/mod.rs` へ移し、`pub mod kinds;` だけを残す(T6) |
| `tests/common/kinds.rs` | — | 変えない(`nfr_security.rs` の残るテストが使う) |
| `src/types.rs` の単体テスト | 4 | 1 件を消す(T5)。3 件は変えない |

## 5. 仕様書(`docs/spec/rust`)

直す節と語句の一覧(各コミットで直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない。版を上げないので、「v0.9.8 までは…」などの版の注記は足さず、今の形を書く)。

| # | ファイル:行(HEAD) | 節 | 今の語句 | 直した後 | コミット |
|---|---|---|---|---|---|
| S1 | rust `02:129` | 2.3 `conductor-converter` | 「入力の形式は、その反対とみなす(`opposite()`)」 | 「入力の形式は、その反対とみなす」 | 5 |
| S2 | rust `02:132` | 2.3 | `testing::{Value, read_tree}`(`#[doc(hidden)]`)の行 | 行を消す | 5 |
| S3 | rust `02:331` | 5.3 `crates/converter/src` | `lib.rs` の責務「`convert`、`convert_text`、読み取りの切り替え、`testing`」 | 「`convert`、`convert_text`、読み取りの切り替え」 | 5 |
| S4 | rust `02:327-339` の表 | 5.3 | (行なし) | 行を足す。ファイルの列は `tests/`(`mod.rs` と、元の統合テストと同じ名前の 4 つ)、責務の列は「途中の表現(`Value` の木)を比べる単体テスト(`lib.rs` の最後の `#[cfg(test)] mod tests;`)」 | 1 |
| S5 | rust `02:134` | 2.3 | 「内部の途中の表現 `Value`: … (`null`・日時などは持たない)。」 | 文の後ろに「等価の比べ方(09 の 6 節)は、テストのときだけの `PartialEq`(`#[cfg(test)]`)が持つ。本番の処理は木を比べない。」を足す | 6 |
| S6 | rust `09:3` | 冒頭 | 「テスト(`crates/converter/tests/*`、fixtures を含む)」 | 「テスト(`crates/converter/tests/*`、fixtures を含む。途中の表現を比べる単体テスト `crates/converter/src/tests/*`)」 | 1 |
| S7 | rust `11:188` | 4.1 方針 | 「単体テスト(各 `src/*.rs` の `#[cfg(test)]`)」 | 「単体テスト(各 `src/*.rs` の `#[cfg(test)]`。converter は、途中の表現を比べる単体テストを `src/tests/` にも置く)」 | 1 |
| S8 | rust `11:246` | 4.2 converter の表 | `read_resolve.rs` の行 | 行を消し、表の後ろに単体テストの小さな表(S12)を足す | 1 |
| S9 | rust `11:245` | 4.2 | `convert_roundtrip.rs` の目的「往復(変換して読み戻した木が入力の木と等しい)」 | 「出力が毎回同じ(往復の木の比較は単体テストの `src/tests/convert_roundtrip.rs`)」 | 2 |
| S10 | rust `11:244` | 4.2 | `convert_ac.rs` の目的「受け入れ基準と、`convert`(ファイルを読む入口)」 | 「受け入れ基準と、`convert`(ファイルを読む入口)。変換の結果を読んだ木を比べる 2 件は単体テストの `src/tests/convert_ac.rs`」 | 3 |
| S11 | rust `11:149` | 3 章の表 | 「変換の入力」の行の、テストの列 `converter/tests/nfr_security.rs` | 「`converter/tests/nfr_security.rs`、`converter/src/tests/nfr_security.rs`(ランダムな入力とランダムな構造の往復)」 | 4 |
| S12 | rust `11:248` の後 | 4.2 | (なし) | 「converter の単体テスト(`crates/converter/src/tests/`。`#[cfg(test)]`)」の表を足す: `mod.rs`(補助 `yaml`・`toml`・`s`・`i`・`m`・`a`・`with_timeout`)、`read_resolve.rs`(文書の確認と展開、失敗の順序、暴走する入力)、`convert_roundtrip.rs`(往復の木の比較)、`convert_ac.rs`(変換の結果の木を比べる受け入れ基準)、`nfr_security.rs`(ランダムな入力とランダムな構造の往復)。注記: このディレクトリは、`tests/nfr_security.rs` のソースの検査(`src/*.rs`)の対象の外。コミット 1 で `mod.rs` と `read_resolve.rs` の行を作り、2〜4 で行を足す | 1〜4 |
| S13 | rust `11:150` | 3 章の表 | 「時間の上限」の行(「プロセスを起動するテストは、すべて 30 秒の上限つき(超えたら子プロセスを強制終了して失敗)」、場所 `各 common/mod.rs`、`bin_support/mod.rs`) | **変えない**。この行はプロセスを起動するテスト(runtime・cli の `common/mod.rs` と `bin_support/mod.rs`)の上限を指す。converter の `with_timeout` はプロセスを起動せず、スレッドで暴走の入力を 30 秒で打ち切る別の補助で、`src/tests/mod.rs` へ移っても、この行の内容は変わらない(converter の `with_timeout` の新しい場所は S12 の表に書く) | なし |

- 件数: rust 11 の 4.2 は「ファイルごとのテストの件数は書かない」と定めており、仕様書にテストの件数の記述は無い(`grep -rn "1,153\|1,15[0-9]" docs/spec/rust docs/spec/language-agnostic` が 0 件)。直す件数は無い。
- language-agnostic: `testing`・`read_tree`・`opposite`・`PartialEq` の記述は無い。11 章の受け入れテストの「根拠」の欄は、参照実装のテストを `ファイル名::テスト名` で書く(`read_resolve::…`、`convert_roundtrip::…`、`convert_ac::ac_7_1_1_…`、`converter::nfr_security`)。移す先のファイル名を元と同じにする(D2。決定: 推奨のとおり)ので、根拠の欄の名前はそのまま正しく、**直さない**(D4。決定: 推奨のとおり。ユーザー、2026-11-24)。
- 残す記述: rust `09:162`(内部表現の等価の定義)、`09:288`(往復の保証)、rust `11:147`、`:159`、`:167`、`:180`(残る統合テスト `converter/tests/nfr_security.rs` を指す)。
- 完了の判定(直した後): `functional-spec.md` §7 の検索。

## 6. 範囲の外で、触らないもの(確かめた)

- converter の公開の入口(`convert`、`convert_text`、`ConversionRequest`、`ConvertedText`、`TargetFormat`、`ConversionError`、`ConversionErrorKind`)の形と振る舞い。`read_value` の形。
- converter のほかの src(`error.rs`、`intake.rs`、`raw.rs`、`resolve.rs`、`scalar.rs`、`toml_read.rs`、`toml_write.rs`、`yaml_read.rs`、`yaml_write.rs`)と、その単体テスト(すでに `Value` を `assert_eq!` で比べるものがあるが、単体テストなので変えずに通る)。
- `tests/common/kinds.rs`、`tests/fixtures/*`。
- cli(`conductor_converter::testing` を使わない。写しの試しで変更 0 件)、core、runtime。
- cli のライブラリの公開範囲(別の intent)。

## 7. 網羅に使った検索のコマンド

```sh
git grep -n "read_tree\|testing::\|opposite()\|mod testing\|doc(hidden)" -- ':!aidlc' ':!docs/spec/go' ':!docs/spec/python' ':!docs/spec/typescript' ':!docs/spec/csharp' ':!docs/spec/powershell'
grep -rn "opposite" crates docs/spec/rust docs/spec/language-agnostic
grep -rln "conductor_converter" crates                       # converter を使うファイル(cli は testing を使わない)
grep -n "mod common\|use common\|common::" crates/converter/tests/*.rs   # 補助 yaml()・toml()・with_timeout を使う所
grep -n "fn \|#\[test\]" crates/converter/tests/*.rs           # テストの一覧(T 表の網羅)
grep -n "cfg(test)" crates/converter/src/*.rs                  # ソースの検査の範囲(最初の #[cfg(test)] の位置)
grep -rn "testing\|read_tree\|opposite\|doc(hidden)\|read_resolve\|convert_roundtrip\|convert_ac\|nfr_security\|途中の表現\|PartialEq\|converter/tests\|src/\*.rs" docs/spec/rust docs/spec/language-agnostic
# 写しの木(本番だけを変えた段階)で、エラーの場所を出す
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
```
