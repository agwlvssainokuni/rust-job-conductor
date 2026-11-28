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
//! `[job.retry]` と `[pipeline.retry]` の検査と組み立ては `retry`、`env`・`env_file`・`unset_env` の検査は `env` の
//! 子モジュールにある(どちらも `Walker` のメソッドで、ここから呼ぶ。呼ぶ順序は変わらない)。
//!
//! 実効のリトライは、検証の中で 1 度だけ決める(v0.10.0 の BR2.2、BR2.4)。ジョブの表があれば丸ごと
//! それ、`retry = false` なら無し、書いていなければ既定(`[pipeline.retry]`)の複製。組み立て後の定義は
//! 既定を持たない。

mod env;
mod retry;
mod transition;
mod values;

use std::collections::HashMap;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::definition::version::VERSION_KEY;
use crate::error::{DefinitionError, DefinitionErrorKind as Kind, Locate, quote_key};
use crate::run_id::{env_name_rule, is_valid_env_name};
use crate::types::{
    Command, EnvFileRef, EnvVar, JobDefinition, PipelineDefinition, RetryPolicy, Transition,
    TransitionTarget,
};
use retry::{JobRetrySetting, PipelineRetryDefault, build_retry};
use transition::RawTransition;
use values::{exit_code_of, integer_of};

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
    "unset_env",
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
    "unset_env",
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
    /// `unset_env` の名前と位置(`env` との重なりの誤りの位置。U3 の BR2.1)。
    unset_env: Vec<(String, usize)>,
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
    /// `unset_env` の名前と位置(U3 の BR2.1)。
    unset_env: Vec<(String, usize)>,
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
        let unset_env = self.unset_env(table);
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
            unset_env,
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
            unset_env: Vec::new(),
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
        raw.unset_env = self.unset_env(table);
        raw.complete = self.errors.len() == errors_before;
        raw
    }

    /// ジョブごとの検査のうち、ほかのジョブとの関係を見るもの(ID、遷移先、終了コード、
    /// 既定のリトライとの重なり、`unset_env` と `env` の重なり)。
    fn cross_check(
        &mut self,
        pipeline: Option<&PipelineHead>,
        jobs: &[RawJob],
        retry_default: Option<&PipelineRetryDefault>,
    ) {
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
        self.check_unset_env(pipeline, jobs);
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
    w.cross_check(pipeline.as_ref(), &raw_jobs, retry_default.as_ref());

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
            unset_env: names_of(head.unset_env),
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
                unset_env: names_of(job.unset_env),
            })
        })
        .collect()
}

/// `unset_env` の名前と位置から、名前だけを取り出す(並びは保つ)。
fn names_of(unset_env: Vec<(String, usize)>) -> Vec<String> {
    unset_env.into_iter().map(|(name, _)| name).collect()
}
