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

//! `env`、`env_file`、`unset_env` の検査(Walk の一部。BR1.1〜BR1.6。`env_file` の表は v0.10.0 の
//! BR1.1〜BR1.3。`unset_env` は U3 の BR1.1〜BR1.3、BR2.1)。
//!
//! `[pipeline]` とジョブの走査の中で、親のモジュールから呼ばれる。

use std::collections::HashSet;

use toml::de::{DeTable, DeValue};

use super::{PipelineHead, RawJob, Walker};
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

    /// `unset_env` を読む(U3 の BR1.1〜BR1.3)。無ければ空。名前と、その位置を、書いた順に返す。
    ///
    /// 文字列の配列で書く。名前の規則は `env` と同じで、空・規則違反・予約名(`CONDUCTOR_*`)は誤り。
    /// 理由の文は `unset_env` の固定の文で、名前は `quote_key` を通す。完全に同じ名前は誤りにせず、
    /// 最初の 1 つだけを残す(BR1.3)。
    pub(super) fn unset_env(&mut self, parent: &DeTable<'_>) -> Vec<(String, usize)> {
        let Some(value) = parent.get("unset_env") else {
            return Vec::new();
        };
        let DeValue::Array(items) = value.get_ref() else {
            self.type_error(value.span().start, UNSET_ENV_TYPE_REASON);
            return Vec::new();
        };
        let mut out: Vec<(String, usize)> = Vec::new();
        // 見た名前(完全一致)。同じ名前は、最初の 1 つにまとめる(BR1.3)。
        let mut seen: HashSet<&str> = HashSet::new();
        for item in items.iter() {
            let at = item.span().start;
            let DeValue::String(name) = item.get_ref() else {
                self.type_error(at, UNSET_ENV_ITEM_TYPE_REASON);
                continue;
            };
            let name: &str = name.as_ref();
            if name.is_empty() {
                self.push(at, Kind::EmptyField, "`unset_env` に空の文字列は書けません");
                continue;
            }
            if !is_valid_env_name(name) {
                self.push(
                    at,
                    Kind::InvalidEnvName,
                    format!(
                        concat!("`unset_env` の名前 {} は、", env_name_rule!(), "で書きます"),
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
                            "`unset_env` の名前 {} は、`",
                            reserved_env_prefix!(),
                            "` で始まるため書けません"
                        ),
                        quote_key(name)
                    ),
                );
                continue;
            }
            if seen.insert(name) {
                out.push((name.to_string(), at));
            }
        }
        out
    }

    /// `unset_env` の名前が、同じジョブに当てはまる `env` の名前と重なっていないかを調べる(U3 の BR2.1)。
    ///
    /// パイプラインの `unset_env` は、パイプラインの `env` とすべてのジョブの `env` と比べる。ジョブの
    /// `unset_env` は、パイプラインの `env` とそのジョブの `env` と比べる。比べ方は、どの OS でも ASCII の
    /// 大文字小文字を区別しない(OS によって意味が変わる書き方を止める。security-design の 1)。重なれば、
    /// `unset_env` の名前の位置で 1 つの `UnsetEnvConflict`。理由の文は固定で、値も名前も入れない。
    pub(super) fn check_unset_env(&mut self, pipeline: Option<&PipelineHead>, jobs: &[RawJob]) {
        let (pipeline_env, pipeline_unset) = match pipeline {
            Some(head) => (head.env.as_slice(), head.unset_env.as_slice()),
            None => (&[][..], &[][..]),
        };
        let pipeline_names = lowercase_names(pipeline_env);
        if !pipeline_unset.is_empty() {
            let mut all_names = pipeline_names.clone();
            for job in jobs {
                all_names.extend(lowercase_names(&job.env));
            }
            self.push_unset_conflicts(pipeline_unset, |name| all_names.contains(name));
        }
        for job in jobs.iter().filter(|job| !job.unset_env.is_empty()) {
            let job_names = lowercase_names(&job.env);
            self.push_unset_conflicts(&job.unset_env, |name| {
                pipeline_names.contains(name) || job_names.contains(name)
            });
        }
    }

    /// 小文字にした名前が `overlaps` に当たる `unset_env` の名前ごとに、誤りを 1 つ足す(U3 の BR2.1)。
    fn push_unset_conflicts(&mut self, unset: &[(String, usize)], overlaps: impl Fn(&str) -> bool) {
        for (name, at) in unset {
            if overlaps(&name.to_ascii_lowercase()) {
                self.push(*at, Kind::UnsetEnvConflict, UNSET_ENV_CONFLICT_REASON);
            }
        }
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

/// `env` の名前を、ASCII の小文字にした集合にする(U3 の BR2.1。名前の数に比例する時間で引く)。
fn lowercase_names(vars: &[EnvVar]) -> HashSet<String> {
    vars.iter().map(|v| v.name.to_ascii_lowercase()).collect()
}

/// `unset_env` が配列でないときの理由(U3 の BR1.1)。
const UNSET_ENV_TYPE_REASON: &str = "`unset_env` は、文字列の配列で書きます";

/// `unset_env` の配列の要素が文字列でないときの理由(U3 の BR1.1)。
const UNSET_ENV_ITEM_TYPE_REASON: &str = "`unset_env` の要素は、文字列で書きます";

/// `unset_env` の名前が `env` の名前と重なるときの理由(U3 の BR2.1)。値も名前も入れない。
const UNSET_ENV_CONFLICT_REASON: &str = "`unset_env` の名前が、`env` の名前と重なっています";

/// `env_file` の表の項目に書けるキー(v0.10.0 の BR1.2)。
const ENV_FILE_TABLE_KEYS: &[&str] = &["path", "required"];

/// `env_file` が文字列・表・その配列のどれでもないときの理由(v0.10.0 の BR1.1)。
const ENV_FILE_TYPE_REASON: &str = "`env_file` は、文字列かテーブル、またはその配列で書きます";

/// `env_file` の配列の要素が文字列でも表でもないときの理由(v0.10.0 の BR1.1)。
const ENV_FILE_ITEM_TYPE_REASON: &str = "`env_file` の配列の要素は、文字列かテーブルで書きます";
