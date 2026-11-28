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

//! オプションの値の解釈(ArgParser の一部)。
//!
//! `--assume`、`--format`、`--interval`、`--run-id`、`--to` の値を検査して、型にする。誤りの文には、
//! 入力の値を含めない(対象は、オプションの名前だけ)。親のモジュールの組み立てから呼ばれる。

use std::ffi::OsString;

use conductor_converter::TargetFormat;
use conductor_core::Assumption;

use super::Opt;
use crate::types::{
    GraphFormat, MAX_FOLLOW_INTERVAL_SECS, MIN_FOLLOW_INTERVAL_SECS, RunIdArg, UsageError,
    UsageErrorKind as Kind,
};

/// `--assume` の値の検査(BR2.1)。`<ジョブID>=<終了コード>`。最初の `=` で分ける。
/// 誤りの文には、入力の値を含めない(対象は `--assume` だけ)。
pub(super) fn parse_assumptions(values: Vec<OsString>) -> Result<Vec<Assumption>, UsageError> {
    let invalid = || UsageError::with_subject(Kind::InvalidValue, Opt::Assume.subject());
    let mut out: Vec<Assumption> = Vec::with_capacity(values.len());
    for value in values {
        let text = value.into_string().map_err(|_| invalid())?;
        let (job_id, code) = text.split_once('=').ok_or_else(invalid)?;
        let code: i32 = code.parse().map_err(|_| invalid())?;
        if !Assumption::exit_code_in_range(code) {
            return Err(invalid());
        }
        if out.iter().any(|a| a.job_id == job_id) {
            return Err(invalid());
        }
        out.push(Assumption::new(job_id, code));
    }
    Ok(out)
}

/// `--format` の値の検査。`text`、`mermaid`、`dot` のどれか(BR1.1)。誤りの文に、値は含めない。
pub(super) fn parse_graph_format(value: &OsString) -> Result<GraphFormat, UsageError> {
    match value.to_str() {
        Some("text") => Ok(GraphFormat::Text),
        Some("mermaid") => Ok(GraphFormat::Mermaid),
        Some("dot") => Ok(GraphFormat::Dot),
        _ => Err(UsageError::with_subject(
            Kind::InvalidValue,
            Opt::Format.subject(),
        )),
    }
}

/// `--interval` の値の検査(v0.10.0 の U7 の BR1.2)。1〜3600 の整数の秒。数字だけを受け付ける(符号、小数点、
/// 空白、空は誤り)。誤りの文に、値は含めない(対象は `--interval` だけ)。
pub(super) fn parse_interval(value: &OsString) -> Result<u32, UsageError> {
    let invalid = || UsageError::with_subject(Kind::InvalidValue, Opt::Interval.subject());
    let text = value.to_str().ok_or_else(invalid)?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let secs: u32 = text.parse().map_err(|_| invalid())?;
    if (MIN_FOLLOW_INTERVAL_SECS..=MAX_FOLLOW_INTERVAL_SECS).contains(&secs) {
        Ok(secs)
    } else {
        Err(invalid())
    }
}

/// `--run-id` の値の検査。空は有効(run ID なしに固定する)。UTF-8 でない値、規則に反する値は、
/// `invalidValue`(対象は `--run-id` で、値は含めない)。
pub(super) fn parse_run_id_option(value: OsString) -> Result<RunIdArg, UsageError> {
    let text = value
        .into_string()
        .map_err(|_| UsageError::with_subject(Kind::InvalidValue, Opt::RunId.subject()))?;
    RunIdArg::parse(&text)
}

/// `--to` の値の検査。`yaml` か `toml`(大文字小文字を区別する)。UTF-8 でない値、ほかの値は、
/// `invalidValue`(対象は `--to` で、入力の値は含めない。BR3.1)。
pub(super) fn parse_target(value: OsString) -> Result<TargetFormat, UsageError> {
    match value.to_str() {
        Some("yaml") => Ok(TargetFormat::Yaml),
        Some("toml") => Ok(TargetFormat::Toml),
        _ => Err(UsageError::with_subject(
            Kind::InvalidValue,
            Opt::To.subject(),
        )),
    }
}
