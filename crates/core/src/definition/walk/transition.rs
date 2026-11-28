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

//! 遷移(`transitions`)の読み取り(Walk の一部。BR1.1〜BR1.3)。
//!
//! ジョブの走査の中で、親のモジュールから呼ばれる。遷移先の検査(ほかのジョブとの関係)は
//! `cross_check` で行う。

use toml::de::{DeTable, DeValue};

use super::{TRANSITION_KEYS, Walker};
use crate::error::DefinitionErrorKind as Kind;
use crate::types::{Classification, TransitionOn};

pub(super) struct RawTransition {
    pub(super) on: Option<TransitionOn>,
    pub(super) next: Option<(String, usize)>,
}

impl Walker<'_> {
    pub(super) fn transitions(&mut self, job: &DeTable<'_>) -> Vec<RawTransition> {
        let Some(value) = job.get("transitions") else {
            return Vec::new();
        };
        let DeValue::Array(items) = value.get_ref() else {
            self.type_error(
                value.span().start,
                "`transitions` はテーブルの配列で書きます",
            );
            return Vec::new();
        };
        let mut out: Vec<RawTransition> = Vec::new();
        // 前に同じ結果(`on`)の遷移があったか。種類ごとの真偽値(`Success`、`Warning` の 2 つ)で持つ。
        let mut seen = [false; 2];
        for item in items.iter() {
            let offset = item.span().start;
            let DeValue::Table(t) = item.get_ref() else {
                self.type_error(offset, "`transitions` の要素はテーブルで書きます");
                continue;
            };
            self.check_keys(t, TRANSITION_KEYS);
            let on = self.transition_on(t, offset);
            let next = self.string(t, "next", offset, true);
            if let Some(on) = on {
                let slot = match on {
                    TransitionOn::Success => &mut seen[0],
                    TransitionOn::Warning => &mut seen[1],
                };
                if *slot {
                    self.push(
                        offset,
                        Kind::DuplicateTransition,
                        "同じ結果(`on`)の遷移が 2 つ以上あります",
                    );
                }
                *slot = true;
            }
            out.push(RawTransition { on, next });
        }
        out
    }

    fn transition_on(&mut self, t: &DeTable<'_>, parent: usize) -> Option<TransitionOn> {
        let (text, offset) = self.string(t, "on", parent, true)?;
        if let Some(on) = TransitionOn::parse(&text) {
            return Some(on);
        }
        if text == Classification::Error.as_str() {
            self.push(
                offset,
                Kind::ErrorTransition,
                "エラー結果(`error`)には遷移先を書けません",
            );
        } else {
            self.type_error(offset, "`on` は `success` または `warning` で書きます");
        }
        None
    }
}
