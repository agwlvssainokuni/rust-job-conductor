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

//! 値の読み取り(Walk の一部。型は BR1.5、`command` は BR1.13)。
//!
//! 文字列、真偽値、終了コードのリスト、`command` を取り出す `Walker` のメソッドと、整数の値を
//! 取り出す自由関数を持つ。親のモジュールと、兄弟の子モジュールから呼ばれる。

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use super::Walker;
use crate::definition::integer_value;
use crate::error::DefinitionErrorKind as Kind;
use crate::types::Command;

impl Walker<'_> {
    /// 文字列の値を取り出す。必須で無ければ、親の位置で `typeMismatch`。
    pub(super) fn string(
        &mut self,
        table: &DeTable<'_>,
        key: &'static str,
        parent: usize,
        required: bool,
    ) -> Option<(String, usize)> {
        let Some(value) = table.get(key) else {
            if required {
                self.type_error(parent, format!("必須の値 `{key}` がありません"));
            }
            return None;
        };
        match value.get_ref() {
            DeValue::String(s) => Some((s.to_string(), value.span().start)),
            _ => {
                self.type_error(value.span().start, format!("`{key}` は文字列で書きます"));
                None
            }
        }
    }

    /// 真偽値の値を取り出す。無ければ `false`。型が違えば `typeMismatch`(値は `false` とみなす)。
    pub(super) fn boolean(&mut self, table: &DeTable<'_>, key: &'static str) -> bool {
        let Some(value) = table.get(key) else {
            return false;
        };
        match value.get_ref() {
            DeValue::Boolean(b) => *b,
            _ => {
                self.type_error(
                    value.span().start,
                    format!("`{key}` は真偽値(`true` または `false`)で書きます"),
                );
                false
            }
        }
    }

    /// 終了コードのリストを取り出す。型の誤りが 1 つでもあれば `None`(誤りは、要素ごとに報告する)。
    pub(super) fn exit_codes(
        &mut self,
        table: &DeTable<'_>,
        key: &'static str,
    ) -> Option<(Vec<i32>, usize)> {
        let (key_span, value) = table.get_key_value(key)?;
        let key_offset = key_span.span().start;
        let DeValue::Array(items) = value.get_ref() else {
            self.type_error(
                value.span().start,
                format!("`{key}` は整数のリストで書きます"),
            );
            return None;
        };
        let mut codes = Vec::with_capacity(items.len());
        let mut ok = true;
        for item in items.iter() {
            match exit_code_of(item) {
                Some(c) => codes.push(c),
                None => {
                    ok = false;
                    self.type_error(
                        item.span().start,
                        format!("`{key}` の要素は、整数(32 ビットの範囲)で書きます"),
                    );
                }
            }
        }
        ok.then_some((codes, key_offset))
    }

    /// `command` を取り出す。文字列(`Line`)または文字列の配列(`Argv`)。
    ///
    /// 配列は、空でなく、先頭の要素が空・空白だけでないこと。2 番目以降の空の文字列は有効。
    /// 誤りのメッセージに、入力の値を入れない。
    pub(super) fn command(&mut self, table: &DeTable<'_>, parent: usize) -> Option<Command> {
        let Some(value) = table.get("command") else {
            self.type_error(parent, "必須の値 `command` がありません");
            return None;
        };
        let start = value.span().start;
        match value.get_ref() {
            DeValue::String(s) => {
                if s.trim().is_empty() {
                    self.push(start, Kind::EmptyField, "`command` が空です");
                    None
                } else {
                    Some(Command::Line(s.to_string()))
                }
            }
            DeValue::Array(items) => {
                if items.is_empty() {
                    self.push(start, Kind::EmptyField, "`command` の配列が空です");
                    return None;
                }
                let mut argv = Vec::with_capacity(items.len());
                let mut ok = true;
                for item in items.iter() {
                    match item.get_ref() {
                        DeValue::String(s) => argv.push(s.to_string()),
                        _ => {
                            ok = false;
                            self.type_error(
                                item.span().start,
                                "`command` の配列の要素は、文字列で書きます",
                            );
                        }
                    }
                }
                if !ok {
                    return None;
                }
                if argv[0].trim().is_empty() {
                    self.push(
                        items[0].span().start,
                        Kind::EmptyField,
                        "`command` の配列の先頭の要素が空です",
                    );
                    return None;
                }
                Some(Command::Argv(argv))
            }
            _ => {
                self.type_error(start, "`command` は文字列、または文字列の配列で書きます");
                None
            }
        }
    }
}

/// 終了コードの値を `i32` で取り出す(整数でない、または `i32` に収まらなければ `None`)。
pub(super) fn exit_code_of(item: &Spanned<DeValue<'_>>) -> Option<i32> {
    integer_of(item).and_then(|v| i32::try_from(v).ok())
}

/// 整数の値を `i64` で取り出す(整数でない、または `i64` に収まらなければ `None`)。
pub(super) fn integer_of(item: &Spanned<DeValue<'_>>) -> Option<i64> {
    match item.get_ref() {
        DeValue::Integer(i) => integer_value(i),
        _ => None,
    }
}
