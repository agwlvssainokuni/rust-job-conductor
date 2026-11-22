// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! conductor-converter: conductor の定義ファイルの変換ライブラリ(Unit conductor-converter)。
//!
//! 定義ファイルを、YAML と TOML のあいだで変換する(Converter。FR7、US7.1、契約 C4)。
//! コマンドの受け付け、結果の書き出し、エラーの表示(conductor-cli)と、変換した結果の定義としての
//! 検証(conductor-core。`conductor run` などでの読み込み時)は、このクレートの外にある。
//! このクレートは、conductor-core に依存しない。
//!
//! # 公開する関数(契約 C4)
//!
//! | 関数 | 役割 | 入出力 | 失敗 |
//! |------|------|--------|------|
//! | [`convert`] | 入力ファイルを読み、`--to` の形式に変換する(W1) | [`ConversionRequest`] → [`ConvertedText`] | [`ConversionError`] |
//! | [`convert_text`] | 文字列を変換する(ファイルに触れない入口) | 文字列 → [`ConvertedText`] | 同上 |
//!
//! 入力の形式は、出力の形式(`target`)と反対とみなす。拡張子は見ない(BR1.1)。
//! 変換は、読み戻すと入力と同じ木になること(BR1.11)を、合否の基準とする。コメントと、
//! マップのキーの記述順は、保持しない(BR1.3)。結果は文字列で返し、書き出しは呼び出し側が行う(BR1.9)。
//!
//! # 公開する API の範囲
//!
//! 契約 C4 で conductor-cli が使うものだけを公開する。ほかは、クレートの内部(`pub(crate)`)か、
//! テストのための `#[doc(hidden)]` である。
//!
//! - 関数: [`convert`]、[`convert_text`]
//! - 型: [`ConversionRequest`]、[`ConvertedText`]、[`TargetFormat`]、[`ConversionError`]
//!   (`Display` で表示用の文になる)、[`ConversionErrorKind`]
//! - `#[doc(hidden)]`(安定した API ではない。このクレートのテスト用): `testing`
//!   (`Value`、`read_tree`)、`ConversionErrorKind::code`、`ConversionErrorKind::ALL`
//!
//! # 展開の上限
//!
//! アンカー・エイリアス・マージキーの展開には、3 つの上限がある(NFR4.3、NFR4.4)。
//!
//! - 訪れたノードの数: 100,000 まで(超えれば `tooLarge`)
//! - 訪れたスカラー(キーと値。捨てられるマージの項目と、エイリアスの参照のたびの複製を含む)の
//!   文字の総バイト数: 16 MiB まで(超えれば `tooLarge`)。ノード数だけでは止められない、
//!   大きな文字列を何度も参照する入力が、メモリを使い尽くすのを防ぐ。複製の前に確認する
//! - 展開後の入れ子の深さ: 32 段まで(超えれば `syntax`)
//!
//! 展開でだけ生じる重複キー(エイリアスやマージの展開で、同じキーが重なる)は、展開の時点
//! (下の 8)で `syntax` になる。リテラルで重複するキーは、読み取りの時点(下の 5)で `syntax` になる。
//!
//! # 失敗(BR1.12 の順序)
//!
//! 同時に複数の失敗があるときは、次の順で最初に当てはまるものを返す。出力は作られない。
//!
//! 1. `fileMissing`(ファイルがない)
//! 2. `unreadable`(権限がない・通常のファイルでない)
//! 3. `tooLarge`(入力が 1 MiB を超える)
//! 4. `unreadable`(UTF-8 でない)
//! 5. `syntax`(文法の誤り、重複するキー、再帰するアンカー、展開前の入れ子が 32 段を超える)
//! 6. `unrepresentable`(YAML の複数の文書)
//! 7. `rootNotMap`(最上位がマップでない。空の YAML も)
//! 8. 展開(アンカー・エイリアス・マージキー): 展開後の要素数が 100,000、または文字の総バイト数が
//!    16 MiB を超えれば `tooLarge`、展開後の入れ子が 32 段を超えれば `syntax`、展開でだけ生じる
//!    重複キーは `syntax`、展開できないマージキーは `unrepresentable`
//! 9. `unrepresentable`(null、文字列でないキー、タグつきの値、TOML の日時、`i64` を超える整数)
//!
//! # モジュールの構成(logical-components との対応)
//!
//! | モジュール | 部品 | 責務 |
//! |------------|------|------|
//! | `types` | 途中の表現(Value)と入出力の型 | 入出力に依存しない型 |
//! | `error` | 表示用の組み立て(Describe) | 固定の文だけでエラーを表示する。キー名のエスケープと切り詰め |
//! | `intake` | 取り込み(Intake) | ファイルを開き、上限 + 1 バイトまで読む。UTF-8 の確認 |
//! | `yaml_read` / `toml_read` | 読み取り(Read) | YAML は、イベントの列を明示のスタックで読み、展開しない素の木を作る。TOML は、ライブラリで読む |
//! | `resolve` | 文書の確認(RootCheck)、展開と写し取り(Resolve) | 展開、要素数・文字の大きさ・深さの数え、表せない値の記録 |
//! | `yaml_write` / `toml_write` | 書き出し(Write) | 相手の形式の文字列にする。文字列の引用符 |
//!
//! 流れは Intake → Read → RootCheck → Resolve → Write の順である。入出力に触れるのは
//! `intake` だけで、それ以外は文字列だけで動く純粋な処理である(NFR4.5)。
//!
//! # 守る性質
//!
//! - 外部の入力に対して、パニックしない。YAML の読み取りと展開は、再帰を使わない(NFR4.7)。
//! - エラーの表示に、入力の値、ライブラリのエラー文、ソースの抜粋を含めない(NFR4.6)。
//! - 副作用を持たない。入力ファイルを読む以外の入出力、環境変数の参照、プロセスの起動、
//!   ネットワークはない(NFR4.5)。
//! - `#![forbid(unsafe_code)]`。

#![forbid(unsafe_code)]

mod error;
mod intake;
mod raw;
mod resolve;
mod scalar;
mod toml_read;
mod toml_write;
mod types;
mod yaml_read;
mod yaml_write;

use std::path::Path;

pub use error::{ConversionError, ConversionErrorKind};
pub use types::{ConversionRequest, ConvertedText, TargetFormat};

use types::Value;

/// 入力ファイルを読み、`request.target_format` の形式に変換する(W1)。
///
/// 入力の形式は、`target_format` と反対とみなす(BR1.1)。入出力に触れるのは、入力ファイルの
/// 読み取りだけで、結果は文字列で返す(書き出しは、呼び出し側の責務。BR1.9)。
///
/// # Errors
///
/// 失敗は [`ConversionError`](種類は [`ConversionErrorKind`])で返す。変換は行われず、出力は
/// 作られない。複数の失敗が同時にあるときの、返す種類の順序は、クレートの文書(BR1.12)のとおり。
pub fn convert(request: &ConversionRequest) -> Result<ConvertedText, ConversionError> {
    let file = intake::file_label(&request.input_path);
    let text = intake::read_input(Path::new(&request.input_path))?;
    convert_text(&text, request.target_format, &file)
}

/// 文字列を、`target` の形式に変換する(ファイルに触れない入口)。
///
/// 入力の形式は、`target` と反対とみなす(BR1.1)。`file_label` は、エラーの表示に使う名前。
/// 入力の大きさの上限(1 MiB)は、[`convert`] の取り込みで確認する。この関数は、確認しない。
///
/// # Errors
///
/// [`convert`] の取り込み(1〜4)以外の失敗を、[`ConversionError`] で返す。
pub fn convert_text(
    text: &str,
    target: TargetFormat,
    file_label: &str,
) -> Result<ConvertedText, ConversionError> {
    let value = read_value(text, target, file_label)?;
    let out = match target {
        TargetFormat::Toml => toml_write::write(&value, file_label)?,
        TargetFormat::Yaml => yaml_write::write(&value),
    };
    Ok(ConvertedText { text: out })
}

/// 入力の形式(`target` と反対)として読み、展開した途中の表現にする(Read → RootCheck → Resolve)。
fn read_value(text: &str, target: TargetFormat, file: &str) -> Result<Value, ConversionError> {
    let tree = match target {
        TargetFormat::Toml => {
            let read = yaml_read::read(text, file)?;
            resolve::root_check(read, file)?
        }
        TargetFormat::Yaml => toml_read::read(text, file)?,
    };
    resolve::resolve(&tree, file)
}

/// テストのための公開。安定した API ではない。
#[doc(hidden)]
pub mod testing {
    use super::*;

    pub use crate::types::Value;

    /// `text` を、`format` の形式として読み、展開した途中の表現にする(往復の確認に使う)。
    pub fn read_tree(
        text: &str,
        format: TargetFormat,
        file: &str,
    ) -> Result<Value, ConversionError> {
        // `read_value` の `target` は「変換後の形式」なので、反対の形式を渡す
        let target = match format {
            TargetFormat::Yaml => TargetFormat::Toml,
            TargetFormat::Toml => TargetFormat::Yaml,
        };
        read_value(text, target, file)
    }
}
