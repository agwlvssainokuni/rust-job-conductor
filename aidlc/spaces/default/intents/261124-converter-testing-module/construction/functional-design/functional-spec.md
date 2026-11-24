# 機能の仕様(Functional Spec): converter の testing モジュールの削除

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 A)。**変える場所の正確な一覧(ファイルと行、目標の形、完了の判定、直す仕様書の節と語句 S1〜S12 と変えない S13、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD 2d98f2a で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの変更(§4 の T 表、§4.5 の消すテスト)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない。本番に、テストのための要素(`pub`、`#[doc(hidden)]`、テストのための引数や関数)を足さない**(ユーザーの判断)。`#[cfg(test)]` の中は可。converter から、テストのためだけの公開(`testing` モジュール)と、それだけが使う `TargetFormat::opposite` を消し、`Value` の `PartialEq` をテストのときだけにする [desc] [Q3]。
- 途中の表現(`Value` の木)を読むテストは、**src の単体テストへ移す**(Q2: A)。単体テストは、クレートの中の非公開の `read_value` と `crate::types::Value` を直接使う。**期待の木と比べる今の確かめ方と期待の値は変えない**。移すときに変わるのは、ファイルの先頭(`mod common;` と import)、`Value` の書き方(`conductor_converter::testing::Value::…` を `Value::…` に)、`include_str!` のパス、`from.opposite()` を `match` にすることだけ(§4)。
- **移すテストの名前は変えない**(★ なし)。パスは変わる(統合テストのバイナリ `read_resolve` の `X` から、単体テストの `tests::read_resolve::X` へ)。移す先のファイル名を元の統合テストと同じにし、言語に依らない仕様書の根拠の欄(`read_resolve::X` など)をそのまま正しく保つ(D2、D4)。
- 利用者から見える動き(変換の出力、誤りの表示、終了コード)を変えない。版は上げない(`0.9.8` のまま)[desc]。
- 小さなコミットの並びで、どのコミットでもビルド、`clippy -D warnings`、テストが通る。**テストを先に移し、移し終えてから `testing` と `opposite` を消す**(先に消すと統合テストのコンパイルが止まる)[Q1] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は小さなコミットで進め、結果は最後にまとめて見せる [Q1]。
- 消すテストは 1 件(`opposite` だけを確かめる単体テスト。§4.5。承認待ち)。足すテストは 0 件。
- 新しい依存を足さない [memory:M5]。`unsafe` と src の `cfg(unix)`/`cfg(windows)` を足さない(`#[cfg(unix)]` の `名前付きパイプ` のテストは統合テストに残す。§4.3)。converter は core に依存しない。MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。
- **ソースの検査を弱めない**: 統合テスト `tests/nfr_security.rs` の 3 件は、`src/*.rs`(直下だけ)を読み、最初の `#[cfg(test)]` の文字より前を製品のコードとみなして、`unwrap` などが無いことを確かめる。したがって、(a)単体テストを `src/` の直下のファイルに置かない(製品のコードとして検査され、`.unwrap()` で止まる)、(b)`lib.rs` の `#[cfg(test)] mod tests;` はファイルの最後に置く、(c)製品のコードより前の説明やコメントに `#[cfg(test)]` の文字を書かない(BR3.2)。
- 仕様書は `docs/spec/rust` を直す。`docs/spec/language-agnostic` は直さない(該当する記述が無く、根拠の欄の名前は D2 の形で正しいまま。D4)。凍結した仕様書は触らない。版の注記は足さない。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

| 項目 | 本番から消すもの・変えるもの | 本番の形の変わり方 | テストの代わり |
|------|------------------------------|--------------------|----------------|
| 1 | `lib.rs` の `#[doc(hidden)] pub mod testing`(`Value` の再公開と `read_tree`) | 公開の入口は変わらない。`read_value` は非公開のまま残る。`lib.rs` の最後に `#[cfg(test)] mod tests;`(単体テストの宣言。本番の成果物には入らない) | 単体テスト(`src/tests/`)が `read_value` と `Value` を直接使う |
| 2 | `types.rs` の `TargetFormat::opposite` | 無し(本番は使っていない) | 移す往復のテストの補助は `match` で反対の形式を求める。`opposite` の単体テスト 1 件は消す(§4.5) |
| 3 | `types.rs` の `impl PartialEq for Value` | `#[cfg(test)]` を付け、テストのときだけコンパイルする。`HashMap` の import は `eq` の本体の中へ | `types.rs` の等しさの単体テスト 3 件は変えずに通る |

### 2.1 項目 3 の形(`types.rs`)

```rust
#[derive(Debug, Clone)]
pub enum Value {
    // (今のまま)
}

#[cfg(test)]
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use std::collections::HashMap;

        match (self, other) {
            // (今のまま)
        }
    }
}

#[cfg(test)]
mod tests {
    // (opposite の 1 件を消すほかは今のまま)
}
```

`Value` の説明(今の 2 行)の後ろに、1 行「等しさはテストのときだけ持つ(`cfg(test)`。本番の処理は木を比べない)。」を足す(`#` と角括弧を書かない。§1 のソースの検査)。

### 2.2 単体テストの置き場所(D2。決定: `src/tests/` のディレクトリ。推奨のとおり。ユーザー、2026-11-24)

```text
crates/converter/src/lib.rs        … 最後に `#[cfg(test)] mod tests;`(前のコメントに `#[cfg(test)]` の文字を書かない)
crates/converter/src/tests/mod.rs  … 補助と下位のモジュールの宣言
crates/converter/src/tests/read_resolve.rs       … 元の tests/read_resolve.rs(28 件)
crates/converter/src/tests/convert_roundtrip.rs  … 元の tests/convert_roundtrip.rs のうち 13 件
crates/converter/src/tests/convert_ac.rs         … 元の tests/convert_ac.rs のうち 2 件
crates/converter/src/tests/nfr_security.rs       … 元の tests/nfr_security.rs のうち 2 件
```

`src/tests/mod.rs` の形(元の `tests/common/mod.rs` の補助を移す。`yaml`・`toml` だけが変わる):

```rust
//! 途中の表現(`Value` の木)を比べる単体テスト(`lib.rs` の最後の宣言)。…

mod convert_ac;
mod convert_roundtrip;
mod nfr_security;
mod read_resolve;

use std::sync::mpsc;
use std::time::Duration;

use crate::types::Value;
use crate::{ConversionError, TargetFormat, read_value};

/// `text` を YAML として読み、展開した木にする(`read_value` の `target` は変換後の形式)。
fn yaml(text: &str) -> Result<Value, ConversionError> {
    read_value(text, TargetFormat::Toml, "t.yaml")
}

/// `text` を TOML として読み、展開した木にする。
fn toml(text: &str) -> Result<Value, ConversionError> {
    read_value(text, TargetFormat::Yaml, "t.toml")
}

// s、i、m、a、with_timeout は元の tests/common/mod.rs のまま
```

- 元の `read_tree(text, TargetFormat::Yaml, "t.yaml")` は `read_value(text, TargetFormat::Yaml.opposite(), "t.yaml")` = `read_value(text, TargetFormat::Toml, "t.yaml")` と同じ。誤りのファイル名(`t.yaml`、`t.toml`)も同じなので、確かめる中身は変わらない。
- 下位のモジュールは `use super::{a, i, m, s, toml, with_timeout, yaml};`、`use crate::…;` で使う(親の非公開の項目は子から見える)。本番の項目の可視性は変えない。
- `include_str!` のパスは `"../../tests/fixtures/…"`。`fixture()` の `env!("CARGO_MANIFEST_DIR")` は今のまま(`std::env` ではない)。
- `src/tests/` は、統合テストのソースの検査(`src/*.rs` の直下だけを読む)の対象の外。中身はテストのコードだけなので、検査の目的(製品のコードの性質)は変わらない。

## 3. 実物で確かめて初期の記述と違った点、範囲から外したもの

- **記述と違った点**(実物に従った):
  - 初期の記述は「木を比べるテスト」を約 30 件と見ていたが、補助 `yaml()`・`toml()` で入力を読むテストは 45 件ある(木や値を比べる 25 件、受理と誤りを確かめる 4 件、誤りだけを確かめる 16 件。§4 の「種類」の列)。誤りだけのものの扱いは D1。
  - `types.rs` の単体テストは、`opposite` の 1 件だけが消す対象で、ほかの 3 件(`Value` の等しさ)は変えずに通る。
  - `tests/nfr_security.rs` のソースの検査が「最初の `#[cfg(test)]` の文字より前」を製品のコードとみなすため、置き場所と説明の書き方に制約がある(§1、BR3.2)。写しで、`Value` の説明に `` `#[cfg(test)]` `` と書くと、検査の範囲が `enum Value` の前で切れることを確かめた。
- **範囲から外したもの**: cli のライブラリの公開範囲の大きい部分(初期の記述)。converter の公開の入口の形。converter のほかの src と単体テスト。言語に依らない仕様書(D4)。

## 4. 移す・変える・消すテストの一覧(T1〜T7 は、ユーザーが設計の承認で認めた、2026-11-24)

Q1: A に従い、**確かめる中身(呼ぶ関数、入力、期待の木と値、誤りの種類と理由の文)は変えない**。移すテストの本体の行は、`conductor_converter::testing::Value::…` を `Value::…` にする 5 行(T1)と、`include_str!` のパス(T2〜T4)のほかは、一字も変えない。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §7。T 表は、`testing`・`read_tree`・`opposite`・補助 `common::…` のすべての参照を `grep` で `crates` の全体(`tests/` の下の補助のディレクトリと、src の `#[cfg(test)]` の単体テストを含む)を検索して作り、さらに **HEAD の写しの木で本番のコードだけを変えてコンパイルし、エラーの出た場所(12 件)がすべて T 表にあること**を確かめた(`inventory.md` §0.2)。

```sh
git grep -n "read_tree\|testing::\|opposite()\|mod testing\|doc(hidden)" -- crates
grep -n "mod common\|use common\|common::" crates/converter/tests/*.rs
grep -n "fn \|#\[test\]" crates/converter/tests/*.rs
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
```

cli・core・runtime のテストの参照は 0 件(cli のテストは `TargetFormat`、`ConversionError`、`ConversionErrorKind` だけを使う)。

**移す・残すの規則(D1。決定: 推奨のとおり。ユーザー、2026-11-24)**: 補助 `yaml()`・`toml()`(`testing::read_tree`)か `Value` を使うテストは、**すべて移す**(誤りだけを確かめるものも、同じファイルのまま移す)。使わないテストは統合テストに残し、変えない。

「種類」の列: **木** = 読んだ木や値を期待と比べる / **受理** = 受理(`is_ok`)と誤りの両方を確かめる / **誤り** = 誤りだけを確かめる(`unwrap_err`)。

### 4.1 T1: `tests/read_resolve.rs` → `src/tests/read_resolve.rs`(28 件すべて。ファイルを消す)

| 行 | テスト名(変えない) | 種類 |
|----|--------------------|------|
| 25 | `複数の文書は表せない` | 誤り |
| 32 | `最上位はマップでなければならない` | 木(空の TOML・`{}` が空のマップ)と誤り |
| 53 | `アンカーとエイリアスを展開する` | 木 |
| 68 | `キーに使ったエイリアス` | 木 |
| 74 | `マージキーの単独と並びと優先順位` | 木 |
| 91 | `インラインのマップのマージキーと_マージより前の自身のキー` | 木 |
| 97 | `引用符つきのマージキーは普通のキー` | 木 |
| 109 | `値がマップでないマージキーは表せない` | 誤り |
| 124 | `エイリアスを通した重複するキーは文法の誤り` | 誤り |
| 137 | `要素数ちょうど100000は受理し_1つ多いと大きすぎる` | 木(`Value::Map` に一致)と誤り |
| 146 | `toml_の入力も要素数の上限は同じ` | 受理 |
| 153 | `エイリアスの爆発は_メモリを使い果たさずに大きすぎる` | 誤り |
| 167 | `大きなスカラーのエイリアスの爆発は_大きさで大きすぎる` | 誤り |
| 186 | `マージキーと値を通したエイリアスの爆発は_大きさで大きすぎる` | 誤り |
| 197 | `既定の16_mibでの展開後の大きさの境界` | 受理 |
| 214 | `大きなマップを何度もマージすると_捨てた要素も数える` | 誤り |
| 226 | `読んだ深さが小さくても_展開後の深さが32を超えれば文法の誤り` | 受理 |
| 244 | `極端に深い入力は_異常終了せずに文法の誤りで失敗する` | 誤り |
| 284 | `入力の順で最初の表せない値を_キーの場所つきで報告する` | 誤り |
| 293 | `null_の書き方と_ほかの表せない種類` | 誤り |
| 338 | `toml_の日時は表せない` | 誤り |
| 352 | `str_のタグと_一致するコアのタグは受理する` | 木 |
| 369 | `コアスキーマのスカラー` | 木 |
| 394 | `失敗の順_文法の誤りはその後のすべてより先` | 誤り |
| 407 | `失敗の順_複数の文書_最上位がマップでない_値の順` | 誤り |
| 423 | `失敗の順_展開の上限は表せない値より先` | 誤り |
| 440 | `失敗の順_同じ段の中では入力の順で最初のもの` | 誤り |
| 449 | `誤りの文は入力の値を含まない` | 誤り |

内訳: 木 9(うち誤りも確かめるもの 2)、受理 3、誤り 16。変える行: 先頭の `mod common;`・`use common::{…};`・`use conductor_converter::ConversionErrorKind as K;` を `use super::{a, i, m, s, toml, with_timeout, yaml};`・`use crate::ConversionErrorKind as K;`・`use crate::types::Value;` に。`:139`、`:359`、`:360`、`:381`、`:383` の `conductor_converter::testing::Value::` を `Value::` に。補助 `flat_array_yaml`(`:131`)は今のまま一緒に移る。

### 4.2 T2: `tests/convert_roundtrip.rs` → `src/tests/convert_roundtrip.rs`(13 件。1 件は残る)

| 行 | テスト名(変えない) | 種類 | 扱い |
|----|--------------------|------|------|
| 58 | `定義の形の_toml_は_yaml_を通って往復する` | 木 | 移す |
| 71 | `定義の形の_yaml_は_toml_を通って往復する` | 木(`assert_roundtrip`) | 移す |
| 124 | `配列形式の_command_は往復する_toml_yaml_toml` | 木 | 移す |
| 135 | `配列形式の_command_は往復する_yaml_toml_yaml` | 木 | 移す |
| 146 | `マージした値は優先順位どおりに展開する` | 木 | 移す |
| 171 | `スカラーの境界は往復する_yaml_から` | 木 | 移す |
| 209 | `スカラーの境界は往復する_toml_から` | 木 | 移す |
| 226 | `浮動小数は両方向に往復する` | 木 | 移す |
| 249 | `i64_の端は往復する` | 木 | 移す |
| 261 | `空と入れ子の集まりは往復する` | 木 | 移す |
| 327 | `retry_の表は往復する_yaml_を通って_toml_に戻る` | 木 | 移す |
| 352 | `retry_の表は往復する_yaml_から` | 木 | 移す |
| 360 | `許される最も深い構造は往復する` | 木 | 移す |
| 367 | `出力は毎回同じ` | (`convert_text` だけ) | **統合テストに残す**(変えない) |

変える行: 先頭の import を `use super::{a, i, m, s, toml, yaml};`、`use crate::types::Value;`、`use crate::{TargetFormat, convert_text};` に。`PIPELINE_TOML`・`PIPELINE_YAML` の `include_str!` を `"../../tests/fixtures/…"` に。補助 `assert_roundtrip`(`:36`)の `let to = from.opposite();` を `let to = match from { TargetFormat::Yaml => TargetFormat::Toml, TargetFormat::Toml => TargetFormat::Yaml };` に(テストの中だけの書き方。本番に足さない)。補助 `read`・`command_of`、定数 `ARRAY_COMMAND_TOML`・`ARRAY_COMMAND_YAML`・`RETRY_TOML` は今のまま一緒に移る。

残る `tests/convert_roundtrip.rs`: `mod common;` と `Value` の import を消し、`use conductor_converter::{TargetFormat, convert_text};`、`PIPELINE_TOML`・`PIPELINE_YAML`、`出力は毎回同じ` だけにする。モジュールの説明に、木を比べるテストは単体テストにあることを書く。

### 4.3 T3: `tests/convert_ac.rs` → `src/tests/convert_ac.rs`(2 件。8 件は残る)

| 行 | テスト名(変えない) | 種類 | 扱い |
|----|--------------------|------|------|
| 40 | `ac_7_1_1_toml_から_yaml` | 木(変換の結果を読んだ木と入力の木) | 移す |
| 51 | `ac_7_1_2_アンカーつきの_yaml_から_toml` | 木 | 移す |
| 63、78、91、127、140、148、161、188 | `ac_7_1_3_壊れた入力は種類と行を返し_出力しない`、`入力の形式は変換後の反対で_拡張子は見ない`、`変換はファイルの失敗を決まった順で報告する`、`ちょうど1_mibの正しい入力は変換できる`、`convert_text_はファイルに触れず_ラベルを使う`、`コメントつきの入力は変換でき_コメントは捨てる`、`文法の誤りの理由は形式ごとの文で_種類の文と重ならない`、`名前付きパイプの入力は開かずに_unreadable`(`#[cfg(unix)]`) | (公開の入口だけ) | **統合テストに残す**(変えない) |

- 移す先は、補助 `fixture()`・`request()`(各 4〜8 行)を写して持つ(残る統合テストも使うので、両方に置く)。`include_str!` のパスを `"../../tests/fixtures/…"` に。import は `use super::{toml, yaml};`、`use crate::{ConversionRequest, TargetFormat, convert};`。
- 残る `tests/convert_ac.rs`: `mod common;` と `use common::{toml, yaml};` を消す。ほかの import(`ConversionErrorKind as K`、`ConversionRequest`、`TargetFormat`、`convert`、`convert_text`)は残るテストが使う。モジュールの説明に、2 件は単体テストにあることを書く。
- `名前付きパイプ` のテストの `#[cfg(unix)]` は統合テストに残る(src に `cfg(unix)` を足さない)。

### 4.4 T4: `tests/nfr_security.rs` → `src/tests/nfr_security.rs`(2 件と補助。10 件 + 1 無視は残る)

| 行 | テスト名・項目(変えない) | 種類 | 扱い |
|----|----------------------------|------|------|
| 424 | 補助 `struct Lcg`(疑似乱数) | — | 移す(残るテストは使わない) |
| 436 | `ランダムな入力と切り詰めた入力で_パニックしない` | 受理(変換できたら読み戻せる) | 移す |
| 520 | `変換できたランダムな構造は往復する` | 木 | 移す |
| 561 | 補助 `yaml_key` | — | 移す |
| 47〜421 | 残りの 10 件と無視の 1 件(ソースの検査 3 件、ファイルを作らない、環境変数(子プロセスと親)、タグ、表示に値を含まない、ライブラリの文、キー名のエスケープ、範囲外と空と壊れた入力) | (公開の入口とソースの検索) | **統合テストに残す**(変えない) |

- 移す先: `common::with_timeout`・`common::toml`・`common::yaml` を `with_timeout`・`toml`・`yaml`(`use super::{toml, with_timeout, yaml};`)に。`include_str!` のパスを `"../../tests/fixtures/…"` に。import は `use std::collections::BTreeSet;`、`use crate::{TargetFormat, convert_text};`。
- 残る `tests/nfr_security.rs`: `:423` から末尾(`:564`)までを消す(**末尾の空行も含める**。残すと `cargo fmt --check` で止まる)。`BTreeSet` の import は残るテストの補助 `listing` が使うので残す。`common::kinds` は残る。モジュールの説明に、2 件は単体テストにあることを書く。

### 4.5 T5: 消すテスト(1 件。Q1: A。理由を添えて承認を取る)

| テスト | 場所 | 確かめていること | 消す理由 | 代わりに残る確かめ |
|--------|------|------------------|----------|--------------------|
| `反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る` | `crates/converter/src/types.rs:137-145`(単体) | `TargetFormat::opposite` が `Yaml ↔ Toml` を入れ替え、2 回で元に戻る | 確かめる対象の関数 `opposite` を消す(本番の読み取り 0 件)。関数が無くなるので、確かめるものが無い | 入力の形式が `--to` の反対であることは、本番の `read_value` の `match` が決め、統合テスト `入力の形式は変換後の反対で_拡張子は見ない`(残る)と、移す往復のテスト 13 件が確かめる |

D3 は「消す」に決まった(推奨のとおり。ユーザー、2026-11-24)。この表の個々の削除は、設計の承認で認めてもらう(承認待ち)。D3 の代案(`opposite` を `#[cfg(test)]` の中に残してテストも残す)は、テストのためだけの関数を本番のファイルに残すことになり、Q3 の方針(テストのためだけのもの)とは合うが、確かめる価値が無い(本番が使わない関数のテスト)。

### 4.6 T6: 補助の変更

| ID | ファイル | 変える内容 |
|----|----------|------------|
| T6 | `tests/common/mod.rs` | `yaml`・`toml`・`s`・`i`・`m`・`a`・`with_timeout` と、その import(`mpsc`、`Duration`、`testing`、`ConversionError`、`TargetFormat`)を消し、`pub mod kinds;` だけにする。`#![allow(dead_code)]` は、残る内容が使われるので消す(写しで、消しても警告が出ないことを確かめた)。モジュールの説明を「統合テストの共通の補助(誤りの種類の一覧)」にする。途中のコミット(1〜3)では、残る統合テストが使う補助を残す(§8) |
| T7 | `src/tests/mod.rs`(新しい) | §2.2 の形。`yaml`・`toml` は `read_value` を呼ぶ。`s`・`i`・`m`・`a`・`with_timeout` は元のまま |

### 4.7 名前を変えるテスト(★)

**なし**。移す 45 件の名前は、すべて変えない。パスは、`read_resolve` などの統合テストのバイナリの中の名前から、`conductor_converter` の単体テストの `tests::read_resolve::…` などに変わる(名前の最後の部分は同じ)。

## 5. 足すテスト

なし。移すテストが、今の確かめをそのまま持つ。本番の動きは変わらないので、新しい確かめは要らない。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 112 | 270 | 382 | 0 |
| runtime | 69 | 128 | 197 | 0 |
| converter | **96**(52 − 1 + 45) | **19 + 1 無視**(convert_ac 8、convert_roundtrip 1、nfr_security 10 + 1 無視) | **115 + 1 無視** | **−1** |
| cli | 2 | 456 | 458 | 0 |
| 全体 | | | **1,152 通過、1 無視、0 失敗** | **−1** |

基準と見込みは macOS の件数。写しの木での試しで、この件数になることを確かめた(`inventory.md` §0.2)。Windows と Linux でも、移すテストに OS の分岐は無いので、差は同じ(−1)。

## 7. 確かめ方

- 実装の前と後で、`cargo test --workspace --locked` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,153 − 消したテスト 1 + 足したテスト 0 = 1,152**」に一致すること [Q1]。
- テスト名の一覧の差: 前後で `cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sed 's/: test$//; s/.*:://' | sort` を比べ、差が §4.5 の 1 件(消えた名前)だけであること(移したテストはパスが変わるので、名前の最後の部分で比べる)。あわせて、パスで比べた差が 91 行(移した 45 件の前後 90 行 + 消した 1 行)であること。
- 移したテストの本体が変わっていないこと: `git show -M --color-moved` で、移動として表示されることを見る。`tests/read_resolve.rs` → `src/tests/read_resolve.rs` は名前の変更として検出される(差は import の 3 行と `Value::` の 5 行)。
- テストのファイルの変更は §4 の T 表だけ。コミットごとに `git diff --stat -- crates/converter` を見る。
- **完了の判定**。最後のコミットのあとで、次をすべて行う [Q1]:

```sh
# コード
grep -rn "testing::\|read_tree\|mod testing\|doc(hidden)" crates                          # 0 件
grep -rn "opposite" crates                                                                # 0 件
grep -n -B1 "impl PartialEq for Value" crates/converter/src/types.rs                      # 前の行が #[cfg(test)]
grep -n "^use std::collections::HashMap" crates/converter/src/types.rs                     # 0 件
grep -n "#\[cfg(test)\]" crates/converter/src/lib.rs crates/converter/src/types.rs         # lib.rs は最後の 1 件(宣言)、types.rs は impl と mod tests の 2 件
tail -3 crates/converter/src/lib.rs                                                       # 最後が `mod tests;`
test ! -e crates/converter/tests/read_resolve.rs                                          # 消えている
ls crates/converter/src/tests                                                             # mod.rs と 4 つのファイル
git diff --stat 2d98f2a -- crates/core crates/runtime crates/cli                          # 空
cargo clippy -p conductor-converter --lib --locked -- -D warnings                         # 本番の形(PartialEq なし)で通る
# src/tests/ はソースの検査(src の直下だけを読む)の外なので、禁止の語が無いことを別に確かめる
grep -rnE "std::process|std::net|std::env|use std::os|Command::new|lazy_static|once_cell|LazyLock|LazyCell|OnceLock|OnceCell|thread_local!|static mut|#\[ctor|^ *(pub(\(crate\))? )?static |File::create|fs::write|OpenOptions|create_dir|remove_file|remove_dir|fs::rename|fs::copy" crates/converter/src/tests  # 0 件
# 仕様書(S1〜S12。S13 は変えない)
grep -rn "testing\|read_tree\|opposite" docs/spec/rust docs/spec/language-agnostic        # 0 件(language-agnostic はもともと 0 件)
grep -n "read_resolve.rs" docs/spec/rust/11-quality-and-build.md                          # 単体テストの表の 1 件だけ
grep -n "src/tests" docs/spec/rust/02-architecture.md docs/spec/rust/09-converter.md docs/spec/rust/11-quality-and-build.md  # S4、S6、S7〜S12 の行
git diff --stat 2d98f2a -- docs/spec/language-agnostic docs/spec/go docs/spec/python docs/spec/typescript docs/spec/csharp docs/spec/powershell  # 空
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4)のまま増えないこと。
- ソースの検査の範囲の記録: `src/tests/`(D2 の結果)と、`#[cfg(test)]` を付けた `Value` の `PartialEq` の `eq` の本体は、統合テスト `tests/nfr_security.rs` のソースの検査の外になる(どちらもテストのときだけのコード)。移すコードに検査の禁止の語が無いこと(上の `grep` が 0 件。`env!("CARGO_MANIFEST_DIR")` はマクロで `std::env` ではない)と、`types.rs` の最初の `#[cfg(test)]` が `impl PartialEq` の直前(製品のコードの範囲が `enum Value` までを含む)であることを確かめ、**コード生成の記録に残す**(BR3.2)。
- Windows の対象のコンパイル: `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` をローカルの macOS で走らせる(写しで通ることを確かめた)。実行は CI の `test`(windows)で確かめる。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(写しで 98.09%)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分が無い)[memory:M5]。`unsafe` と src の `cfg` を足さない。converter の `Cargo.toml` に core を足さない。版の表記(`0.9.8`)は変えない(`git diff -- '*Cargo.toml' Cargo.lock` が空)。
- 利用者から見える動き: 残る統合テスト(converter 19 件、cli の `convert` のテスト `binary_basic.rs`・`flow_aux.rs`・`report_messages.rs` など)が、期待を変えずに通ることで確かめる。変換の出力、誤りの表示、終了コードを作るコード(`convert`、`convert_text`、`read_value`、読み取り、展開、書き出し、`error.rs`)は変わらない。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

テストを先に移し(1〜4)、移し終えてから本番を変える(5、6)。各コミットで、その変更に当たる仕様書の行(S)も直す。移すコミットでは、移す前に `testing` がまだあるので、残る統合テストはそのまま通る。

**各コミットの確かめ**: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test -p conductor-converter --locked`(件数がコミットごとの見込みと合うこと)。写しの木では、全項目を当てた最後の形でだけ確かめたので、途中の形はこのコミットごとの確かめで見る。最後の確かめ(7)で全テストと §7 を行う。

| # | コミット | 変えるもの | converter の件数の見込み(単体 / 統合) |
|---|----------|------------|----------------------------------------|
| 1 | converter: read_resolve のテストを単体テストへ移す | `lib.rs` の最後に `#[cfg(test)] mod tests;`。`src/tests/mod.rs`(補助 `yaml`・`toml`・`s`・`i`・`m`・`a`・`with_timeout`。下位は `mod read_resolve;` だけ)、`src/tests/read_resolve.rs`(T1)。`tests/read_resolve.rs` を消す(`git mv` で移してから直すと、履歴で移動が追える)。`tests/common/mod.rs` は変えない(残る統合テストが使う)。仕様書 S4、S6、S7、S8、S12(`mod.rs` と `read_resolve.rs` の行) | 80 / 36 + 1 |
| 2 | converter: 往復の木を比べるテストを単体テストへ移す | `src/tests/convert_roundtrip.rs`(T2)、`src/tests/mod.rs` に `mod convert_roundtrip;`。`tests/convert_roundtrip.rs` は 1 件だけに。仕様書 S9、S12(行を足す) | 93 / 23 + 1 |
| 3 | converter: 受け入れ基準の木を比べる 2 件を単体テストへ移す | `src/tests/convert_ac.rs`(T3)、`mod convert_ac;`。`tests/convert_ac.rs` から 2 件と `common` を外す。仕様書 S10、S12 | 95 / 21 + 1 |
| 4 | converter: ランダムな入力の 2 件を単体テストへ移し、統合テストの補助を減らす | `src/tests/nfr_security.rs`(T4)、`mod nfr_security;`。`tests/nfr_security.rs` から 2 件と補助を外す。`tests/common/mod.rs` を `pub mod kinds;` だけに(T6)。仕様書 S11、S12 | 97 / 19 + 1 |
| 5 | converter: testing モジュールと TargetFormat::opposite を消す | `lib.rs` の `testing` と説明(§1 の項目 1)、`types.rs` の `opposite` と説明、`opposite` の単体テスト(T5)。仕様書 S1、S2、S3 | 96 / 19 + 1 |
| 6 | converter: Value の PartialEq をテストのときだけにする | `types.rs` の `#[cfg(test)]`、`HashMap` の import の移動、`Value` の説明(§2.1)。仕様書 S5 | 96 / 19 + 1 |
| 7 | (確かめ)§7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy、カバレッジの記録 | コミットなし。記録はコード生成の記録に書く | — |

合計 **6 コミット**(7 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

- コミット 1 は 466 行のファイルの移動を含むが、差は先頭の import(3 行)と `Value::` の 5 行だけ。レビューは `git show -M --color-moved` で、移動と差を分けて見られる。
- コミット 1〜4 の途中では、`tests/common/mod.rs` の `#![allow(dead_code)]` があるので、使われなくなった補助が残っても警告は出ない。`src/tests/mod.rs` の補助は、各コミットで使うものだけを置く(コミット 1 の `read_resolve.rs` は 7 つの補助をすべて使う)。
- コミット 5 の前に、`git grep -n "testing::\|read_tree\|opposite" -- crates` が `lib.rs` と `types.rs` だけであることを確かめる(移し忘れがあればビルドで止まる)。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」(cli のライブラリの公開範囲の大きい部分、converter の公開の入口の形、converter のほかの src と単体テスト)。
- 言語に依らない仕様書と、凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- 版の更新、タグとリリース。
- core、runtime、cli のコードとテスト。

## 10. 決めてほしい点

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | 木を比べず、誤りや受理だけを確かめるテスト(`read_resolve.rs` の「誤り」16 件と「受理」3 件、`nfr_security.rs` の `ランダムな入力と切り詰めた入力で_パニックしない`。計 20 件)の行き先 | **補助 `yaml()`・`toml()` か `Value` を使うテストは、すべて単体テストへ移す**(45 件)。確かめの行は一字も変えない | (B)木や値を比べるテスト(25 件)だけを移し、誤りと受理だけのテスト(20 件)は統合テストに残して、公開の `convert_text` で書き直す(`yaml(x)` を `convert_text(x, TargetFormat::Toml, "t.yaml")` に) | 推奨は機械的で、確かめる中身が変わらないことが差を見るだけで分かる(Q2 の「移すのは機械的」と同じ向き)。代案は公開の入口で確かめる統合テストが増えるが、(1)誤りは読み取りの段で決まるので結果は同じでも、それを読み手が確かめる必要がある、(2)受理(`is_ok`)の 3 件と `ランダム…` の読み戻しの計 4 件は、書き出しの段も通るので確かめる中身が変わる(強くなるが、同じではない)、(3)`read_resolve.rs` が 2 つのファイルに分かれる。公開の入口は、残る統合テスト 19 件と cli の `convert` のテストが確かめ続ける | 決定: 補助 `yaml()`・`toml()` か `Value` を使うテストは、すべて単体テストへ移す(45 件)(推奨のとおり。ユーザー、2026-11-24) |
| D2 | 移す先の形 | **`src/tests/` のディレクトリ**(`lib.rs` の最後に `#[cfg(test)] mod tests;`。下位のファイル名を元の統合テストと同じにする) | (B)`lib.rs` の中の `#[cfg(test)] mod tests { … }` に全部を書く(`lib.rs` が約 1,200 行になる)。(C)読み取り・展開・書き出しの各モジュール(`resolve.rs` など)の既存の `mod tests` に分けて入れる | 推奨は、元のファイルと 1 対 1 で、`git` が移動として追え、言語に依らない仕様書の根拠の欄(`read_resolve::…` など)がそのまま正しい。ソースの検査(`src/*.rs` の直下だけを読む)の対象の外になるが、中身はテストだけ。(B)は検査の対象に入るが、`lib.rs` の製品のコードが読みにくくなり、根拠の欄の名前が合わなくなる。(C)はテストの意味でまとめ直す判断が要り、機械的でない。`src/` の直下のファイル(`src/read_resolve_tests.rs` など)は、ソースの検査で `.unwrap()` が製品のコードとみなされて止まるので選べない | 決定: `src/tests/` のディレクトリ。`lib.rs` の最後に `#[cfg(test)] mod tests;`、下位のファイル名は元の統合テストと同じ(推奨のとおり。ユーザー、2026-11-24) |
| D3 | `opposite` の単体テスト(§4.5) | **消す**(件数 −1) | `opposite` を `#[cfg(test)]` の中に残し、テストも残す | 確かめる対象の関数を本番が使わない。代案は件数が変わらないが、使われない関数とそのテストが残る。往復のテストの補助は、`match` で反対の形式を求める(2 行) | 決定: 消す(推奨のとおり。ユーザー、2026-11-24)。個々のテストの削除(§4.5)は設計の承認で認めてもらう(承認待ち) |
| D4 | 言語に依らない仕様書の 11 章の根拠の欄(`read_resolve::…`、`convert_roundtrip::…`、`convert_ac::ac_7_1_1_…` など) | **直さない**(D2 の推奨の形で、ファイル名とテスト名が同じなので正しいまま) | 移したテストの根拠に「区分: 単体」を足す | 根拠の欄は「参照実装のテストの名前」を示すだけで、受け入れテストそのもの(外から実行して確かめる)は変わらない。区分は受け入れテストの確かめ方の区分で、参照実装のテストの置き場所の区分ではない | 決定: 直さない(推奨のとおり。ユーザー、2026-11-24) |

注(D1 の件数): 木と誤りの両方を確かめる 2 件(`最上位はマップでなければならない`、`要素数ちょうど100000は受理し_1つ多いと大きすぎる`)は「木」に数える(§4.1)。45 件 = 木 25(`read_resolve` 9、`convert_roundtrip` 13、`convert_ac` 2、`nfr_security` 1)+ 受理 4(`read_resolve` 3、`nfr_security` 1)+ 誤り 16(`read_resolve`)。どちらの案でも件数とテスト名は同じで、パスだけが違う(代案では 20 件が統合テストのパスのまま)。

D1〜D4 は、ユーザーが決めた(すべて推奨のとおり。2026-11-24)。

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-24)**: §4 の T 表(T1〜T7。移す 45 件、名前は変えない)、§4.5 の消すテスト 1 件(D3 の決定による個々の削除)。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた(字下げした項目は、親の項目に含める)。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | `testing` モジュールと `TargetFormat::opposite` が本番のコードから消え、参照が 0 件 | BR1.1、BR1.2 |
| SM2 | 木を比べるテストは src の単体テストへ移り、確かめ方と期待の値をそのまま残す | BR3.1、BR3.2、BR3.3 |
| SM3 | `Value` の `PartialEq` は `#[cfg(test)]` の中にだけあり、本番の成果物に入らない | BR2.1 |
| SM4 | 仕様書 rust の `testing` と `opposite` の記述が実装と合う(検索で 0 件) | BR4.1、BR4.2 |
| SM5 | 確かめる手段: 移す・変えるテストの一覧(補助と `types.rs` の単体テストの扱いを含む)、終わったと判断する基準、名前を変えるかどうかの一覧 | BR9.2、BR9.3、BR9.6 |
| SM6 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM7 | 既存のテストの扱い(移す・変える・消すを検索で網羅し、消すものは理由を添えて承認を取る)と件数(基準 − 消したテスト + 足したテスト) | BR8.1、BR9.2、BR9.3 |
| SM8 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
