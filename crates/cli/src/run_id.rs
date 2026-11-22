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

//! run ID の解決(FR1.2〜FR1.4、FR4.3)。引数の解釈(`args`)の外で、`--run-id` の指定と環境変数から、
//! この実行の run ID を決める。
//!
//! | 関数 | 使う場所 |
//! |------|----------|
//! | [`resolve_run_id`] | 定義を読んだあと(`run`・`pipeline`・`--dry-run`・`graph --state`)と、`watch`・`reset` |
//! | `aux_run_id`(クレートの中) | `watch`・`reset`(定義を検証せず、環境変数の名前だけをゆるく読む) |

use std::path::Path;

use conductor_core::{DEFAULT_RUN_ID_ENV, RunId};

use crate::ctx::Ctx;
use crate::deps::Env;
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// run ID を決める(FR1.2〜FR1.4)。
///
/// - `option` が `Some`(`--run-id` あり): 環境変数は見ない。空なら run ID なし(固定)。
/// - `option` が `None`: 環境変数 `env_name` を読み、未定義と空は run ID なし。
///
/// `--run-id` の値は、引数の段階で検査済みだが、ここでも検査する(環境変数の値と同じ関数)。
/// 規則に反する値は、`invalidValue`(対象は、どこから来たか=オプションか環境変数かだけ。値は含めない)。
pub fn resolve_run_id(
    option: Option<&str>,
    env_name: &str,
    env: &dyn Env,
) -> Result<Option<RunId>, UsageError> {
    let (value, subject) = match option {
        Some(v) => (Some(v.to_string()), "--run-id"),
        None => (env.var(env_name), "run ID (environment variable)"),
    };
    match value.as_deref() {
        None | Some("") => Ok(None),
        Some(v) => RunId::parse(v)
            .map(Some)
            .map_err(|_| UsageError::with_subject(UsageErrorKind::InvalidValue, subject)),
    }
}

/// `watch`・`reset` が使う run ID を決める(FR4.3)。
///
/// `--run-id` があれば、定義を読まない。なければ、定義の `[pipeline].run_id_env` だけをゆるく読み
/// (読めない・キーがない・型や形が違う、のときは既定の名前)、その環境変数を見る。
pub(crate) fn aux_run_id(
    ctx: &Ctx<'_>,
    cmd: &CliCommand,
    path: &Path,
) -> Result<Option<RunId>, UsageError> {
    if cmd.run_id.is_some() {
        return resolve_run_id(cmd.run_id.as_deref(), DEFAULT_RUN_ID_ENV, ctx.env);
    }
    let env_name = ctx.runtime.run_id_env(path);
    resolve_run_id(
        None,
        env_name.as_deref().unwrap_or(DEFAULT_RUN_ID_ENV),
        ctx.env,
    )
}
