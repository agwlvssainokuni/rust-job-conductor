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

//! ジョブをまたぐ検査(Walk の一部。ジョブID、遷移先は BR1.1〜BR1.3、終了コードは BR1.4)。
//!
//! すべてのジョブを走査した後に、親のモジュールの入口から 1 度だけ呼ばれる。リトライと
//! `unset_env` の検査は、ここから `retry`・`env` の子モジュールのメソッドを呼ぶ。

use std::collections::HashMap;

use super::{PipelineHead, PipelineRetryDefault, RawJob, SuccessCodes, Walker};
use crate::error::DefinitionErrorKind as Kind;
use crate::types::TransitionTarget;

impl Walker<'_> {
    /// ジョブごとの検査のうち、ほかのジョブとの関係を見るもの(ID、遷移先、終了コード、
    /// 既定のリトライとの重なり、`unset_env` と `env` の重なり)。
    pub(super) fn cross_check(
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
