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

//! `[job.retry]` の検査と組み立て(Walk の一部。BR1.1〜BR1.13)。
//!
//! 読み取り(`retry`)はジョブの走査の中で、ほかの値との関係の検査(`check_retry`)は `cross_check` の中で、
//! 組み立て(`build_retry`)は定義の組み立ての中で、親のモジュールから呼ばれる。

use toml::de::{DeTable, DeValue};

use super::{RawJob, Walker, integer_of};
use crate::definition::duration::parse_duration_secs;
use crate::error::DefinitionErrorKind as Kind;
use crate::types::RetryPolicy;

/// `[job.retry]` に書けるキー(BR1.4)。
const RETRY_KEYS: &[&str] = &[
    "exit_codes",
    "max",
    "interval",
    "multiplier",
    "max_interval",
];

/// `[job.retry]` の読み取り結果。値が正しいものだけが `Some`(位置は、誤りの報告に使う)。
pub(super) struct RawRetry {
    exit_codes: Option<(Vec<i32>, usize)>,
    max: Option<u32>,
    interval: Option<(u64, usize)>,
    multiplier: Option<u32>,
    max_interval: Option<(u64, usize)>,
}

impl Walker<'_> {
    /// `[job.retry]` を読む(BR1.2〜BR1.6、BR1.9〜BR1.12)。書かなければ `None`(BR1.1)。
    ///
    /// 成功・警告の終了コードとの重なり(BR1.7)と、上限と間隔の大小(BR1.13)は、`cross_check` が見る。
    pub(super) fn retry(&mut self, job: &DeTable<'_>) -> Option<RawRetry> {
        let value = job.get("retry")?;
        let start = value.span().start;
        let DeValue::Table(table) = value.get_ref() else {
            self.push(start, Kind::RetryNotTable, "`retry` はテーブルで書きます");
            return None;
        };
        self.check_keys(table, RETRY_KEYS);
        let exit_codes = self.retry_exit_codes(table, start);
        let max = self.retry_integer(table, "max", start, true, 1..=100, Kind::RetryMaxInvalid);
        let interval = self.retry_duration(table, "interval", start, true);
        let multiplier = self.retry_integer(
            table,
            "multiplier",
            start,
            false,
            2..=10,
            Kind::RetryMultiplierInvalid,
        );
        let max_interval = self.retry_duration(table, "max_interval", start, false);
        if table.contains_key("multiplier") != table.contains_key("max_interval") {
            self.push(
                start,
                Kind::RetryBackoffIncomplete,
                "`multiplier` と `max_interval` は、両方を書くか、両方とも書きません",
            );
        }
        Some(RawRetry {
            exit_codes,
            max: max.and_then(|m| u32::try_from(m).ok()),
            interval,
            multiplier: multiplier.and_then(|m| u32::try_from(m).ok()),
            max_interval,
        })
    }

    /// 必須のキーがないとき、`retryMissingKey`(キー名だけを示す。BR1.3)。
    fn retry_missing(&mut self, key: &'static str, parent: usize) {
        self.push(
            parent,
            Kind::RetryMissingKey,
            format!("必須の値 `{key}` がありません"),
        );
    }

    /// `retry.exit_codes` を読む。空でない、1〜255 の整数の一覧。昇順にして重複をまとめる(BR1.5、BR1.6)。
    fn retry_exit_codes(
        &mut self,
        table: &DeTable<'_>,
        parent: usize,
    ) -> Option<(Vec<i32>, usize)> {
        let Some((key, value)) = table.get_key_value("exit_codes") else {
            self.retry_missing("exit_codes", parent);
            return None;
        };
        let DeValue::Array(items) = value.get_ref() else {
            self.push(
                value.span().start,
                Kind::RetryExitCodeInvalid,
                "`exit_codes` は整数の一覧で書きます",
            );
            return None;
        };
        if items.is_empty() {
            self.push(
                value.span().start,
                Kind::RetryExitCodesEmpty,
                "`exit_codes` が空です",
            );
            return None;
        }
        let mut codes = Vec::with_capacity(items.len());
        let mut ok = true;
        for item in items.iter() {
            match integer_of(item).and_then(|v| i32::try_from(v).ok()) {
                Some(c) if (1..=255).contains(&c) => codes.push(c),
                _ => {
                    ok = false;
                    self.push(
                        item.span().start,
                        Kind::RetryExitCodeInvalid,
                        "`exit_codes` の要素は、1 以上 255 以下の整数で書きます",
                    );
                }
            }
        }
        codes.sort_unstable();
        codes.dedup();
        ok.then_some((codes, key.span().start))
    }

    /// `retry` の整数の値を読む。範囲外や整数でない値は `kind` の誤り。
    fn retry_integer(
        &mut self,
        table: &DeTable<'_>,
        key: &'static str,
        parent: usize,
        required: bool,
        range: std::ops::RangeInclusive<i64>,
        kind: Kind,
    ) -> Option<i64> {
        let Some(value) = table.get(key) else {
            if required {
                self.retry_missing(key, parent);
            }
            return None;
        };
        match integer_of(value).filter(|v| range.contains(v)) {
            Some(v) => Some(v),
            None => {
                self.push(
                    value.span().start,
                    kind,
                    format!(
                        "`{key}` は、{} 以上 {} 以下の整数で書きます",
                        range.start(),
                        range.end()
                    ),
                );
                None
            }
        }
    }

    /// `retry` の間隔(`<整数><s|m|h>`)を秒で読む(BR1.10)。
    fn retry_duration(
        &mut self,
        table: &DeTable<'_>,
        key: &'static str,
        parent: usize,
        required: bool,
    ) -> Option<(u64, usize)> {
        let Some(value) = table.get(key) else {
            if required {
                self.retry_missing(key, parent);
            }
            return None;
        };
        let offset = value.span().start;
        let secs = match value.get_ref() {
            DeValue::String(s) => parse_duration_secs(s),
            _ => None,
        };
        if secs.is_none() {
            self.push(
                offset,
                Kind::RetryDurationInvalid,
                format!(
                    "`{key}` は、整数と単位(`s`、`m`、`h`)で、1 秒以上 24 時間以下に書きます(例: `30s`)"
                ),
            );
        }
        secs.map(|s| (s, offset))
    }

    /// リトライの設定のうち、ほかの値との関係を見るもの(BR1.7、BR1.13)。
    pub(super) fn check_retry(&mut self, job: &RawJob) {
        let Some(retry) = &job.retry else {
            return;
        };
        if let Some((codes, offset)) = &retry.exit_codes {
            // 成功の終了コードは、省略時は 0 だけ。書いて誤りだったときは、重なりを調べない。
            let success: Option<&[i32]> = match &job.success {
                Some((c, _)) => Some(c),
                None if !job.success_declared => Some(&[0]),
                None => None,
            };
            let overlaps_success = success.is_some_and(|s| codes.iter().any(|c| s.contains(c)));
            let overlaps_warning = job
                .warning
                .as_ref()
                .is_some_and(|(w, _)| codes.iter().any(|c| w.contains(c)));
            if overlaps_success || overlaps_warning {
                self.push(
                    *offset,
                    Kind::RetryExitCodesOverlap,
                    "`exit_codes` に、成功または警告の終了コードと同じ値があります",
                );
            }
        }
        if let (Some((interval, _)), Some((max_interval, offset))) =
            (&retry.interval, &retry.max_interval)
            && max_interval < interval
        {
            self.push(
                *offset,
                Kind::RetryMaxIntervalTooSmall,
                "`max_interval` は、`interval` 以上で書きます",
            );
        }
    }
}

/// 読み取り結果から `RetryPolicy` を組み立てる。`[job.retry]` がなければ `None`(BR1.1)。
///
/// 誤りがあれば、定義の組み立てに入る前に失敗している。値が欠けていても、`None` にして続ける。
pub(super) fn build_retry(raw: Option<&RawRetry>) -> Option<RetryPolicy> {
    let raw = raw?;
    Some(RetryPolicy {
        exit_codes: raw.exit_codes.as_ref()?.0.clone(),
        max: raw.max?,
        interval_secs: raw.interval?.0,
        multiplier: raw.multiplier,
        max_interval_secs: raw.max_interval.map(|(s, _)| s),
    })
}
