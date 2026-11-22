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

//! `env` と `env_file` の検査(Walk の一部。BR1.1〜BR1.6)。
//!
//! `[pipeline]` とジョブの走査の中で、親のモジュールから呼ばれる。

use toml::de::{DeTable, DeValue};

use super::Walker;
use crate::envfile::is_reserved_env_name;
use crate::error::{DefinitionErrorKind as Kind, quote_key};
use crate::run_id::is_valid_env_name;
use crate::types::EnvVar;

impl Walker<'_> {
    /// `env` の表を読む(BR1.1〜BR1.5)。無ければ空。項目は、書いた順(キーの位置の順)に検査する。
    ///
    /// 誤りの理由は固定の文で、値を含めない。名前は `quote_key` を通す。
    pub(super) fn env_table(&mut self, parent: &DeTable<'_>) -> Vec<EnvVar> {
        let Some(value) = parent.get("env") else {
            return Vec::new();
        };
        let DeValue::Table(table) = value.get_ref() else {
            self.type_error(value.span().start, "`env` は表で書きます");
            return Vec::new();
        };
        let mut items: Vec<_> = table.iter().collect();
        items.sort_by_key(|(k, _)| k.span().start);
        let mut out = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for (key, val) in items {
            let name = key.get_ref().as_ref();
            let at = key.span().start;
            if !is_valid_env_name(name) {
                self.push(
                    at,
                    Kind::InvalidEnvName,
                    format!(
                        "`env` の名前 {} は、英字または `_` で始まり、英数字と `_` だけで書きます",
                        quote_key(name)
                    ),
                );
                continue;
            }
            if is_reserved_env_name(name) {
                self.push(
                    at,
                    Kind::ReservedEnvName,
                    format!(
                        "`env` の名前 {} は、`CONDUCTOR_` で始まるため書けません",
                        quote_key(name)
                    ),
                );
                continue;
            }
            if seen.iter().any(|s| s.eq_ignore_ascii_case(name)) {
                self.push(
                    at,
                    Kind::EnvNameCaseConflict,
                    format!(
                        "`env` の名前 {} は、大文字小文字だけが違う名前が、すでにあります",
                        quote_key(name)
                    ),
                );
                continue;
            }
            seen.push(name.to_string());
            match val.get_ref() {
                DeValue::String(s) => {
                    if s.contains('\0') {
                        self.push(
                            val.span().start,
                            Kind::InvalidEnvValue,
                            format!("`env` の名前 {} の値に、NUL は書けません", quote_key(name)),
                        );
                    } else {
                        out.push(EnvVar {
                            name: name.to_string(),
                            value: s.to_string(),
                        });
                    }
                }
                _ => self.type_error(
                    val.span().start,
                    format!("`env` の名前 {} の値は、文字列で書きます", quote_key(name)),
                ),
            }
        }
        out
    }

    /// `env_file` を読む(BR1.6)。文字列 1 つは、要素 1 つの並びにする。無ければ空。
    pub(super) fn env_file(&mut self, parent: &DeTable<'_>) -> Vec<String> {
        let Some(value) = parent.get("env_file") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        match value.get_ref() {
            DeValue::String(s) => self.env_file_item(s, value.span().start, &mut out),
            DeValue::Array(items) => {
                for item in items.iter() {
                    match item.get_ref() {
                        DeValue::String(s) => self.env_file_item(s, item.span().start, &mut out),
                        _ => self.type_error(
                            item.span().start,
                            "`env_file` の配列の要素は、文字列で書きます",
                        ),
                    }
                }
            }
            _ => self.type_error(
                value.span().start,
                "`env_file` は、文字列、または文字列の配列で書きます",
            ),
        }
        out
    }

    fn env_file_item(&mut self, s: &str, at: usize, out: &mut Vec<String>) {
        if s.is_empty() {
            self.push(at, Kind::EmptyField, "`env_file` に空の文字列は書けません");
        } else {
            out.push(s.to_string());
        }
    }
}
