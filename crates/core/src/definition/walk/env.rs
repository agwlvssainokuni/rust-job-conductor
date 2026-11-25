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

//! `env` と `env_file` の検査(Walk の一部。BR1.1〜BR1.6。`env_file` の表は v0.10.0 の BR1.1〜BR1.3)。
//!
//! `[pipeline]` とジョブの走査の中で、親のモジュールから呼ばれる。

use std::collections::HashSet;

use toml::de::{DeTable, DeValue};

use super::Walker;
use crate::envfile::{is_reserved_env_name, reserved_env_prefix};
use crate::error::{DefinitionErrorKind as Kind, quote_key};
use crate::run_id::{env_name_rule, is_valid_env_name};
use crate::types::{EnvFileRef, EnvVar};

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
        // 見た名前(ASCII の小文字にしたもの)。大文字小文字だけが違う名前を、項目の数に比例する時間で見つける。
        let mut seen: HashSet<String> = HashSet::new();
        for (key, val) in items {
            let name = key.get_ref().as_ref();
            let at = key.span().start;
            if !is_valid_env_name(name) {
                self.push(
                    at,
                    Kind::InvalidEnvName,
                    format!(
                        concat!("`env` の名前 {} は、", env_name_rule!(), "で書きます"),
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
                        concat!(
                            "`env` の名前 {} は、`",
                            reserved_env_prefix!(),
                            "` で始まるため書けません"
                        ),
                        quote_key(name)
                    ),
                );
                continue;
            }
            if !seen.insert(name.to_ascii_lowercase()) {
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

    /// `env_file` を読む(BR1.6。v0.10.0 の BR1.1〜BR1.3)。無ければ空。
    ///
    /// 項目は文字列か表 `{ path, required }`。1 つでも配列でも書け、配列には両方を混ぜて書ける。
    /// 項目 1 つは、要素 1 つの並びにする。文字列は `required = true` の項目になる。
    pub(super) fn env_file(&mut self, parent: &DeTable<'_>) -> Vec<EnvFileRef> {
        let Some(value) = parent.get("env_file") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        match value.get_ref() {
            DeValue::String(s) => self.env_file_path(s, value.span().start, true, &mut out),
            DeValue::Table(t) => self.env_file_table(t, value.span().start, &mut out),
            DeValue::Array(items) => {
                for item in items.iter() {
                    let at = item.span().start;
                    match item.get_ref() {
                        DeValue::String(s) => self.env_file_path(s, at, true, &mut out),
                        DeValue::Table(t) => self.env_file_table(t, at, &mut out),
                        _ => self.type_error(at, ENV_FILE_ITEM_TYPE_REASON),
                    }
                }
            }
            _ => self.type_error(value.span().start, ENV_FILE_TYPE_REASON),
        }
        out
    }

    /// 表の項目 `{ path, required }` を読む(v0.10.0 の BR1.2、BR1.3)。
    ///
    /// 書けるキーは `path` と `required` だけ。`path` は必須の文字列、`required` は省略可の真偽値で、
    /// 省略時は `true`。理由の文は既存の固定の文(`string`・`boolean`・`check_keys`)を使う。
    fn env_file_table(&mut self, table: &DeTable<'_>, at: usize, out: &mut Vec<EnvFileRef>) {
        self.check_keys(table, ENV_FILE_TABLE_KEYS);
        let path = self.string(table, "path", at, true);
        let required = !table.contains_key("required") || self.boolean(table, "required");
        if let Some((path, path_at)) = path {
            self.env_file_path(&path, path_at, required, out);
        }
    }

    /// パスの検査(文字列と表で共有する。空なら `EmptyField`)。正しければ項目を足す。
    fn env_file_path(&mut self, s: &str, at: usize, required: bool, out: &mut Vec<EnvFileRef>) {
        if s.is_empty() {
            self.push(at, Kind::EmptyField, "`env_file` に空の文字列は書けません");
        } else {
            out.push(EnvFileRef {
                path: s.to_string(),
                required,
            });
        }
    }
}

/// `env_file` の表の項目に書けるキー(v0.10.0 の BR1.2)。
const ENV_FILE_TABLE_KEYS: &[&str] = &["path", "required"];

/// `env_file` が文字列・表・その配列のどれでもないときの理由(v0.10.0 の BR1.1)。
const ENV_FILE_TYPE_REASON: &str = "`env_file` は、文字列かテーブル、またはその配列で書きます";

/// `env_file` の配列の要素が文字列でも表でもないときの理由(v0.10.0 の BR1.1)。
const ENV_FILE_ITEM_TYPE_REASON: &str = "`env_file` の配列の要素は、文字列かテーブルで書きます";
