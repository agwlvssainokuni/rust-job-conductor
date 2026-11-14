//! 構造の検査と定義の組み立て(Walk)。
//!
//! 位置を保つ文書の木を自分で走査し、許されたキー(BR1.6)、型(BR1.5)、構造の制約(BR1.13)、
//! 遷移(BR1.1〜BR1.3)、終了コード(BR1.4)、名前の一致(BR1.8)を検査する。
//! 誤りは、すべて集めて返す。型の誤りで検査できなくなった子の後続の検査は、その子だけ省略する。
//! 理由の文は、固定の文から組み立てる。入力に由来するのは、キー名(エスケープと切り詰めを行う)だけである。

use std::collections::HashMap;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::error::{DefinitionError, DefinitionErrorKind as Kind, Locate, quote_key};
use crate::types::{JobDefinition, PipelineDefinition, Transition, TransitionOn, TransitionTarget};

const ROOT_KEYS: &[&str] = &["format_version", "pipeline", "jobs"];
const PIPELINE_KEYS: &[&str] = &["name", "description"];
const JOB_KEYS: &[&str] = &[
    "id",
    "command",
    "warning_exit_codes",
    "success_exit_codes",
    "transitions",
];
const TRANSITION_KEYS: &[&str] = &["on", "next"];

/// 遷移先に書く、終了を表す値。
const END_TARGET: &str = "@end";

struct RawTransition {
    on: Option<TransitionOn>,
    next: Option<(String, usize)>,
}

struct RawJob {
    id: Option<(String, usize)>,
    command: Option<String>,
    warning: Option<(Vec<i32>, usize)>,
    success: Option<(Vec<i32>, usize)>,
    transitions: Vec<RawTransition>,
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

    fn pipeline_section(
        &mut self,
        root: &DeTable<'_>,
        expected_name: Option<&str>,
    ) -> Option<(String, Option<String>)> {
        let Some(value) = root.get("pipeline") else {
            self.type_error(0, "必須の値 `pipeline` がありません");
            return None;
        };
        let start = value.span().start;
        let table = match value.get_ref() {
            DeValue::Table(t) => t,
            DeValue::Array(_) => {
                self.type_error(
                    start,
                    "`pipeline` は 1 つだけで、テーブルで書きます(1 ファイルに 1 パイプライン)",
                );
                return None;
            }
            _ => {
                self.type_error(start, "`pipeline` はテーブルで書きます");
                return None;
            }
        };
        self.check_keys(table, PIPELINE_KEYS);
        let name = self.string(table, "name", start, true);
        let description = self.string(table, "description", start, false);
        let (name, name_offset) = name?;
        if name.trim().is_empty() {
            self.push(name_offset, Kind::EmptyField, "`name` が空です");
            return None;
        }
        if let Some(expected) = expected_name
            && expected != name
        {
            self.push(
                name_offset,
                Kind::PipelineNameMismatch,
                "指定されたパイプライン名が、定義の `name` と一致しません",
            );
        }
        Some((name, description.map(|(d, _)| d)))
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
        for item in items.iter() {
            let offset = item.span().start;
            let DeValue::Table(t) = item.get_ref() else {
                self.type_error(offset, "`transitions` の要素はテーブルで書きます");
                continue;
            };
            self.check_keys(t, TRANSITION_KEYS);
            let on = self.transition_on(t, offset);
            let next = self.string(t, "next", offset, true);
            if let Some(on) = on
                && out.iter().any(|r| r.on == Some(on))
            {
                self.push(
                    offset,
                    Kind::DuplicateTransition,
                    "同じ結果(`on`)の遷移が 2 つ以上あります",
                );
            }
            out.push(RawTransition { on, next });
        }
        out
    }

    fn transition_on(&mut self, t: &DeTable<'_>, parent: usize) -> Option<TransitionOn> {
        let (text, offset) = self.string(t, "on", parent, true)?;
        match text.as_str() {
            "success" => Some(TransitionOn::Success),
            "warning" => Some(TransitionOn::Warning),
            "error" => {
                self.push(
                    offset,
                    Kind::ErrorTransition,
                    "エラー結果(`error`)には遷移先を書けません",
                );
                None
            }
            _ => {
                self.type_error(offset, "`on` は `success` または `warning` で書きます");
                None
            }
        }
    }

    fn job(&mut self, value: &Spanned<DeValue<'_>>) -> RawJob {
        let start = value.span().start;
        let mut raw = RawJob {
            id: None,
            command: None,
            warning: None,
            success: None,
            transitions: Vec::new(),
            complete: false,
        };
        let DeValue::Table(table) = value.get_ref() else {
            self.type_error(start, "`jobs` の要素はテーブルで書きます");
            return raw;
        };
        let errors_before = self.errors.len();
        self.check_keys(table, JOB_KEYS);
        raw.id = self.string(table, "id", start, true);
        let command = self.string(table, "command", start, true);
        if let Some((c, offset)) = command {
            if c.trim().is_empty() {
                self.push(offset, Kind::EmptyField, "`command` が空です");
            } else {
                raw.command = Some(c);
            }
        }
        raw.warning = self.exit_codes(table, "warning_exit_codes");
        raw.success = self.exit_codes(table, "success_exit_codes");
        raw.transitions = self.transitions(table);
        raw.complete = self.errors.len() == errors_before;
        raw
    }

    /// ジョブごとの検査のうち、ほかのジョブとの関係を見るもの(ID、遷移先、終了コード)。
    fn cross_check(&mut self, jobs: &[RawJob]) {
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
                if next == END_TARGET {
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
        if let (Some((warning, offset)), Some((success, _))) = (&job.warning, &job.success) {
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

fn exit_code_of(item: &Spanned<DeValue<'_>>) -> Option<i32> {
    match item.get_ref() {
        DeValue::Integer(i) => {
            let v = i64::from_str_radix(i.as_str(), i.radix()).ok()?;
            i32::try_from(v).ok()
        }
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
    let pipeline = w.pipeline_section(root, expected_pipeline_name);

    let raw_jobs: Vec<RawJob> = match root.get("jobs") {
        None => {
            w.push(0, Kind::NoJobs, "ジョブ(`jobs`)が 1 つもありません");
            Vec::new()
        }
        Some(value) => match value.get_ref() {
            DeValue::Array(items) if items.is_empty() => {
                w.push(
                    value.span().start,
                    Kind::NoJobs,
                    "ジョブ(`jobs`)が 1 つもありません",
                );
                Vec::new()
            }
            DeValue::Array(items) => items.iter().map(|item| w.job(item)).collect(),
            _ => {
                w.type_error(value.span().start, "`jobs` はテーブルの配列で書きます");
                Vec::new()
            }
        },
    };
    w.cross_check(&raw_jobs);

    if w.errors.is_empty() {
        if let Some((name, description)) = pipeline {
            if let Some(jobs) = build_jobs(&raw_jobs) {
                return Ok(PipelineDefinition {
                    name,
                    description,
                    format_version,
                    jobs,
                });
            }
        }
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

fn build_jobs(raw: &[RawJob]) -> Option<Vec<JobDefinition>> {
    raw.iter()
        .map(|job| {
            if !job.complete {
                return None;
            }
            let mut transitions = Vec::new();
            for t in &job.transitions {
                let (next, _) = t.next.as_ref()?;
                let target = if next == END_TARGET {
                    TransitionTarget::End
                } else {
                    TransitionTarget::Job(next.clone())
                };
                transitions.push(Transition {
                    on: t.on?,
                    next: target,
                });
            }
            Some(JobDefinition {
                id: job.id.as_ref()?.0.clone(),
                command: job.command.clone()?,
                warning_exit_codes: job
                    .warning
                    .as_ref()
                    .map(|(c, _)| c.clone())
                    .unwrap_or_default(),
                success_exit_codes: job.success.as_ref().map(|(c, _)| c.clone()),
                transitions,
            })
        })
        .collect()
}
