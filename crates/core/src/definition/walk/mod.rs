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

//! 構造の検査と定義の組み立て(Walk)。
//!
//! 位置を保つ文書の木を自分で走査し、許されたキー(BR1.6)、型(BR1.5)、構造の制約(BR1.13)、
//! 遷移(BR1.1〜BR1.3)、終了コード(BR1.4)、名前の一致(BR1.8)を検査する。
//! 誤りは、すべて集めて返す。型の誤りで検査できなくなった子の後続の検査は、その子だけ省略する。
//! 理由の文は、固定の文から組み立てる。入力に由来するのは、キー名(エスケープと切り詰めを行う)だけである。
//!
//! `[job.retry]` と `[pipeline.retry]` の検査と組み立ては `retry`、`env` と `env_file` の検査は `env` の
//! 子モジュールにある(どちらも `Walker` のメソッドで、ここから呼ぶ。呼ぶ順序は変わらない)。
//!
//! 実効のリトライは、検証の中で 1 度だけ決める(v0.10.0 の BR2.2、BR2.4)。ジョブの表があれば丸ごと
//! それ、`retry = false` なら無し、書いていなければ既定(`[pipeline.retry]`)の複製。組み立て後の定義は
//! 既定を持たない。

mod env;
mod retry;

use std::collections::HashMap;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::definition::integer_value;
use crate::definition::version::VERSION_KEY;
use crate::error::{DefinitionError, DefinitionErrorKind as Kind, Locate, quote_key};
use crate::run_id::{env_name_rule, is_valid_env_name};
use crate::types::{
    Classification, Command, EnvFileRef, EnvVar, JobDefinition, PipelineDefinition, RetryPolicy,
    Transition, TransitionOn, TransitionTarget,
};
use retry::{JobRetrySetting, PipelineRetryDefault, build_retry};

const ROOT_KEYS: &[&str] = &[VERSION_KEY, "pipeline", "job"];
const PIPELINE_KEYS: &[&str] = &[
    "name",
    "description",
    "run_id_env",
    "remove_on_complete",
    "env",
    "env_file",
    "retry",
    "log_dir",
];
const JOB_KEYS: &[&str] = &[
    "id",
    "command",
    "warning_exit_codes",
    "success_exit_codes",
    "transitions",
    "retry",
    "env",
    "env_file",
];
const TRANSITION_KEYS: &[&str] = &["on", "next"];

/// ジョブが 1 つもないときの理由(`job` が無い、または空の配列)。
const NO_JOBS_REASON: &str = "ジョブ(`job`)が 1 つもありません";

/// `[pipeline]` から取り出した値。
struct PipelineHead {
    name: String,
    description: Option<String>,
    run_id_env: Option<String>,
    remove_on_complete: bool,
    env: Vec<EnvVar>,
    env_file: Vec<EnvFileRef>,
    log_dir: Option<String>,
}

struct RawTransition {
    on: Option<TransitionOn>,
    next: Option<(String, usize)>,
}

/// `success_exit_codes` の読み取りの結果。
enum SuccessCodes {
    /// 書いていない(省略時は `[0]`)。
    Absent,
    /// 書いて誤り(ほかの値との重なりを調べない)。
    Invalid,
    /// 正しい値(位置は、ほかの誤りの場所に使わないので持たない)。
    Valid(Vec<i32>),
}

struct RawJob {
    /// ジョブの表(`[[job]]`)の始まりの位置(既定との重なりの誤りの位置。v0.10.0 の BR3.2)。
    start: usize,
    id: Option<(String, usize)>,
    command: Option<Command>,
    warning: Option<(Vec<i32>, usize)>,
    success: SuccessCodes,
    transitions: Vec<RawTransition>,
    /// ジョブの `retry`(無し・`false`・表。v0.10.0 の BR2.1)。
    retry: JobRetrySetting,
    env: Vec<EnvVar>,
    env_file: Vec<EnvFileRef>,
    /// 構造の誤りがなく、定義に組み立てられるか。
    complete: bool,
}

struct Walker<'a> {
    file: &'a str,
    locate: &'a Locate,
    errors: Vec<(usize, DefinitionError)>,
}

impl<'a> Walker<'a> {
    fn push(&mut self, offset: usize, kind: Kind, reason: impl Into<String>) {
        let line = self.locate.line_of(offset);
        self.errors.push((
            offset,
            DefinitionError::new(self.file, Some(line), kind, reason),
        ));
    }

    fn type_error(&mut self, offset: usize, reason: impl Into<String>) {
        self.push(offset, Kind::TypeMismatch, reason);
    }

    /// 許されたキー以外を `unknownKey` にする。
    fn check_keys(&mut self, table: &DeTable<'_>, allowed: &[&str]) {
        for (key, _) in table.iter() {
            if !allowed.contains(&key.get_ref().as_ref()) {
                self.push(
                    key.span().start,
                    Kind::UnknownKey,
                    format!("キー {} は定義にありません", quote_key(key.get_ref())),
                );
            }
        }
    }

    /// 文字列の値を取り出す。必須で無ければ、親の位置で `typeMismatch`。
    fn string(
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
    fn boolean(&mut self, table: &DeTable<'_>, key: &'static str) -> bool {
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
    fn exit_codes(&mut self, table: &DeTable<'_>, key: &'static str) -> Option<(Vec<i32>, usize)> {
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
    fn command(&mut self, table: &DeTable<'_>, parent: usize) -> Option<Command> {
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

    /// `[pipeline]` を読む。既定のリトライ(`[pipeline.retry]`)は `PipelineHead` とは別に返し、
    /// `name` の誤りで既定の検査が省かれないようにする(v0.10.0 の BR1.1、BR3.1)。
    fn pipeline_section(
        &mut self,
        root: &DeTable<'_>,
        expected_name: Option<&str>,
    ) -> (Option<PipelineHead>, Option<PipelineRetryDefault>) {
        let Some(value) = root.get("pipeline") else {
            self.type_error(0, "必須の値 `pipeline` がありません");
            return (None, None);
        };
        let start = value.span().start;
        let table = match value.get_ref() {
            DeValue::Table(t) => t,
            DeValue::Array(_) => {
                self.type_error(
                    start,
                    "`pipeline` は 1 つだけで、テーブルで書きます(1 ファイルに 1 パイプライン)",
                );
                return (None, None);
            }
            _ => {
                self.type_error(start, "`pipeline` はテーブルで書きます");
                return (None, None);
            }
        };
        let retry_default = self.pipeline_retry(table);
        let head = self.pipeline_head(table, start, expected_name);
        (head, retry_default)
    }

    /// `[pipeline]` の、既定のリトライ以外の値を読む。
    fn pipeline_head(
        &mut self,
        table: &DeTable<'_>,
        start: usize,
        expected_name: Option<&str>,
    ) -> Option<PipelineHead> {
        self.check_keys(table, PIPELINE_KEYS);
        let name = self.string(table, "name", start, true);
        let description = self.string(table, "description", start, false);
        let run_id_env = self.string(table, "run_id_env", start, false);
        if let Some((env_name, offset)) = &run_id_env
            && !is_valid_env_name(env_name)
        {
            self.push(
                *offset,
                Kind::InvalidEnvName,
                concat!("`run_id_env` は、", env_name_rule!(), "の名前で書きます"),
            );
        }
        let remove_on_complete = self.boolean(table, "remove_on_complete");
        let env = self.env_table(table);
        let env_file = self.env_file(table);
        let log_dir = self.log_dir(table, start);
        let (name, name_offset) = name?;
        if name.trim().is_empty() {
            self.push(name_offset, Kind::EmptyField, "`name` が空です");
            return None;
        }
        if expected_name.is_some_and(|expected| expected != name) {
            self.push(
                name_offset,
                Kind::PipelineNameMismatch,
                "指定されたパイプライン名が、定義の `name` と一致しません",
            );
        }
        Some(PipelineHead {
            name,
            description: description.map(|(d, _)| d),
            run_id_env: run_id_env.map(|(e, _)| e),
            remove_on_complete,
            env,
            env_file,
            log_dir,
        })
    }

    /// `[pipeline].log_dir` を読む(v0.10.0 の BR1.1)。文字列でなければ `typeMismatch`、空なら
    /// `emptyField`。書かなければ `None`。
    fn log_dir(&mut self, table: &DeTable<'_>, start: usize) -> Option<String> {
        let (dir, offset) = self.string(table, "log_dir", start, false)?;
        if dir.is_empty() {
            self.push(offset, Kind::EmptyField, "`log_dir` が空です");
            return None;
        }
        Some(dir)
    }

    fn transitions(&mut self, job: &DeTable<'_>) -> Vec<RawTransition> {
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

    fn job(&mut self, value: &Spanned<DeValue<'_>>) -> RawJob {
        let start = value.span().start;
        let mut raw = RawJob {
            start,
            id: None,
            command: None,
            warning: None,
            success: SuccessCodes::Absent,
            transitions: Vec::new(),
            retry: JobRetrySetting::Absent,
            env: Vec::new(),
            env_file: Vec::new(),
            complete: false,
        };
        let DeValue::Table(table) = value.get_ref() else {
            self.type_error(start, "`job` の要素はテーブルで書きます");
            return raw;
        };
        let errors_before = self.errors.len();
        self.check_keys(table, JOB_KEYS);
        raw.id = self.string(table, "id", start, true);
        raw.command = self.command(table, start);
        raw.warning = self.exit_codes(table, "warning_exit_codes");
        raw.success = match self.exit_codes(table, "success_exit_codes") {
            Some((codes, _)) => SuccessCodes::Valid(codes),
            None if table.contains_key("success_exit_codes") => SuccessCodes::Invalid,
            None => SuccessCodes::Absent,
        };
        raw.transitions = self.transitions(table);
        raw.retry = self.job_retry(table);
        raw.env = self.env_table(table);
        raw.env_file = self.env_file(table);
        raw.complete = self.errors.len() == errors_before;
        raw
    }

    /// ジョブごとの検査のうち、ほかのジョブとの関係を見るもの(ID、遷移先、終了コード、
    /// 既定のリトライとの重なり)。
    fn cross_check(&mut self, jobs: &[RawJob], retry_default: Option<&PipelineRetryDefault>) {
        // ジョブIDの検査と、一意の索引
        let mut index: HashMap<&str, usize> = HashMap::new();
        for (i, job) in jobs.iter().enumerate() {
            let Some((id, offset)) = &job.id else {
                continue;
            };
            if id.trim().is_empty() || id.starts_with('@') {
                self.push(
                    *offset,
                    Kind::InvalidJobId,
                    "ジョブID(`id`)は、空でなく、`@` で始まらない文字列で書きます",
                );
                continue;
            }
            if index.contains_key(id.as_str()) {
                self.push(
                    *offset,
                    Kind::DuplicateJobId,
                    "同じジョブID(`id`)が、すでに定義されています",
                );
            } else {
                index.insert(id.as_str(), i);
            }
        }

        for (i, job) in jobs.iter().enumerate() {
            for t in &job.transitions {
                let (Some(_), Some((next, offset))) = (t.on, &t.next) else {
                    continue;
                };
                if next == TransitionTarget::END_LABEL {
                    continue;
                }
                match index.get(next.as_str()) {
                    None => self.push(
                        *offset,
                        Kind::UnknownTarget,
                        "遷移先(`next`)のジョブが、定義にありません",
                    ),
                    Some(&target) if target <= i => self.push(
                        *offset,
                        Kind::BackwardOrSelfTarget,
                        "遷移先(`next`)は、記述順で後ろのジョブまたは `@end` で書きます",
                    ),
                    Some(_) => {}
                }
            }
            self.check_exit_codes(job);
            self.check_retry(job, retry_default);
        }
    }

    fn check_exit_codes(&mut self, job: &RawJob) {
        let zero_as_warning = match &job.warning {
            Some((codes, offset)) if codes.contains(&0) => {
                self.push(
                    *offset,
                    Kind::ZeroAsWarning,
                    "0 は警告の終了コード(`warning_exit_codes`)にできません",
                );
                true
            }
            _ => false,
        };
        if let (Some((warning, offset)), SuccessCodes::Valid(success)) =
            (&job.warning, &job.success)
        {
            let overlaps = warning
                .iter()
                .any(|w| success.contains(w) && !(zero_as_warning && *w == 0));
            if overlaps {
                self.push(
                    *offset,
                    Kind::OverlappingExitCodes,
                    "`warning_exit_codes` と `success_exit_codes` に、同じ値があります",
                );
            }
        }
    }
}

/// 終了コードの値を `i32` で取り出す(整数でない、または `i32` に収まらなければ `None`)。
fn exit_code_of(item: &Spanned<DeValue<'_>>) -> Option<i32> {
    integer_of(item).and_then(|v| i32::try_from(v).ok())
}

/// 整数の値を `i64` で取り出す(整数でない、または `i64` に収まらなければ `None`)。
fn integer_of(item: &Spanned<DeValue<'_>>) -> Option<i64> {
    match item.get_ref() {
        DeValue::Integer(i) => integer_value(i),
        _ => None,
    }
}

/// 木を検査し、誤りがなければ `PipelineDefinition` を返す。誤りは、すべて集めて返す(位置の順)。
///
/// 版の確認(`format_version`)は、この関数より前に済んでいる前提で、`format_version` には
/// 確認済みの版を渡す。
pub fn walk(
    root: &DeTable<'_>,
    format_version: u32,
    file: &str,
    locate: &Locate,
    expected_pipeline_name: Option<&str>,
) -> Result<PipelineDefinition, Vec<DefinitionError>> {
    let mut w = Walker {
        file,
        locate,
        errors: Vec::new(),
    };
    w.check_keys(root, ROOT_KEYS);
    let (pipeline, retry_default) = w.pipeline_section(root, expected_pipeline_name);

    let raw_jobs: Vec<RawJob> = match root.get("job") {
        None => {
            w.push(0, Kind::NoJobs, NO_JOBS_REASON);
            Vec::new()
        }
        Some(value) => match value.get_ref() {
            DeValue::Array(items) if items.is_empty() => {
                w.push(value.span().start, Kind::NoJobs, NO_JOBS_REASON);
                Vec::new()
            }
            DeValue::Array(items) => items.iter().map(|item| w.job(item)).collect(),
            _ => {
                w.type_error(value.span().start, "`job` はテーブルの配列で書きます");
                Vec::new()
            }
        },
    };
    w.cross_check(&raw_jobs, retry_default.as_ref());

    // 既定のリトライを 1 度だけ組み立てる。書いていなければ `Some(None)`、書いたのに値が欠けていれば
    // `None`(定義を組み立てない。v0.10.0 の BR2.2)。
    let default_policy = match retry_default {
        None => Some(None),
        Some(d) => d.into_policy().map(Some),
    };
    if w.errors.is_empty()
        && let Some(head) = pipeline
        && let Some(default_policy) = default_policy
        && let Some(jobs) = build_jobs(raw_jobs, default_policy.as_ref())
    {
        return Ok(PipelineDefinition {
            name: head.name,
            description: head.description,
            run_id_env: head.run_id_env,
            remove_on_complete: head.remove_on_complete,
            format_version,
            job: jobs,
            env: head.env,
            env_file: head.env_file,
            log_dir: head.log_dir,
        });
    }

    let mut errors = w.errors;
    if errors.is_empty() {
        // 到達しない想定の防御。誤りがないのに組み立てられなかった場合も、失敗として返す。
        errors.push((
            0,
            DefinitionError::new(file, None, Kind::TypeMismatch, "定義を組み立てられません"),
        ));
    }
    errors.sort_by_key(|(offset, _)| *offset);
    Err(errors.into_iter().map(|(_, e)| e).collect())
}

/// 走査の結果から、ジョブの定義を組み立てる(値は移す。組み立てられないジョブが 1 つでもあれば `None`)。
///
/// `default` は組み立てた既定のリトライ(`[pipeline.retry]`)。`retry` を書かないジョブに複製して入れる
/// (v0.10.0 の BR2.2)。
fn build_jobs(raw: Vec<RawJob>, default: Option<&RetryPolicy>) -> Option<Vec<JobDefinition>> {
    raw.into_iter()
        .map(|job| {
            if !job.complete {
                return None;
            }
            let mut transitions = Vec::new();
            for t in job.transitions {
                let (next, _) = t.next?;
                let target = if next == TransitionTarget::END_LABEL {
                    TransitionTarget::End
                } else {
                    TransitionTarget::Job(next)
                };
                transitions.push(Transition {
                    on: t.on?,
                    next: target,
                });
            }
            Some(JobDefinition {
                id: job.id?.0,
                command: job.command?,
                warning_exit_codes: job.warning.map(|(c, _)| c).unwrap_or_default(),
                success_exit_codes: match job.success {
                    SuccessCodes::Valid(codes) => Some(codes),
                    SuccessCodes::Absent | SuccessCodes::Invalid => None,
                },
                transitions,
                // 実効のリトライ(v0.10.0 の BR2.2): 表はその表を丸ごと(書いたのに値が欠けていれば
                // 組み立てない)、`retry = false` は `None`、書かなければ既定の複製(既定も無ければ `None`)。
                retry: match job.retry {
                    JobRetrySetting::Table(raw) => Some(build_retry(raw)?),
                    JobRetrySetting::Disabled => None,
                    JobRetrySetting::Absent => default.cloned(),
                },
                env: job.env,
                env_file: job.env_file,
            })
        })
        .collect()
}
