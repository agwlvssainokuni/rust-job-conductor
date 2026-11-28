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

//! 字句(ArgParser の一部。BR1.2)。
//!
//! 字句に分けた引数の型と、字句の段階の誤りの組み立て、位置引数の文字列への変換を持つ。
//! 親のモジュールの `parse_args` と組み立てから使われる。

use std::ffi::OsString;

use super::{Opt, spec_by_long};
use crate::types::{UsageError, UsageErrorKind as Kind};

/// 字句に分けた 1 つの引数。
pub(super) enum Token {
    /// 表にあるオプション。値を取るオプションなら `Some`(値は、字句の段階では検査しない)。
    Opt(Opt, Option<OsString>),
    /// 文法にないオプション(表示用の綴り)。
    Unknown(String),
    Positional(OsString),
}

/// 特別なオプション(コマンドの解釈より先に処理する)。
#[derive(Clone, Copy)]
pub(super) enum Special {
    Help,
    Version,
}

/// `lexopt` の値の取り残し(`UnexpectedValue`)を、使い方の誤りにする。
///
/// `--bogus=x` は知らないオプション、`--verbose=x` は誤った値。
/// 知っている、値を取らないオプションは、表の `takes_value` が偽の行と `-v`(BR1.2)。
pub(super) fn map_unexpected_value(option: &str) -> UsageError {
    let known = option == "-v"
        || option
            .strip_prefix("--")
            .is_some_and(|long| spec_by_long(long).is_some_and(|spec| !spec.takes_value));
    if known {
        UsageError::with_subject(Kind::InvalidValue, option.to_string())
    } else {
        UsageError::with_subject(Kind::UnknownOption, option.to_string())
    }
}

/// 位置引数を、文字列にする。UTF-8 でなければ `invalidValue`。
pub(super) fn to_text(value: OsString) -> Result<String, UsageError> {
    value
        .into_string()
        .map_err(|v| UsageError::with_subject(Kind::InvalidValue, v.to_string_lossy().into_owned()))
}
