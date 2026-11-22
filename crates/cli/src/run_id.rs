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
use crate::types::{CliCommand, RunIdArg, UsageError, UsageErrorKind};

/// run ID を決める(FR1.2〜FR1.4)。
///
/// - `option` が `Some`(`--run-id` あり): 環境変数は見ない。[`RunIdArg::Empty`] なら run ID なし(固定)、
///   [`RunIdArg::Id`] ならその値(引数の段階で検査済みなので、検査し直さない)。
/// - `option` が `None`: 環境変数 `env_name` を読み、未定義と空は run ID なし。規則に反する値は、
///   `invalidValue`(対象は環境変数であることだけ。値は含めない)。
pub fn resolve_run_id(
    option: Option<&RunIdArg>,
    env_name: &str,
    env: &dyn Env,
) -> Result<Option<RunId>, UsageError> {
    match option {
        Some(RunIdArg::Empty) => Ok(None),
        Some(RunIdArg::Id(id)) => Ok(Some(id.clone())),
        None => match env.var(env_name).as_deref() {
            None | Some("") => Ok(None),
            Some(v) => RunId::parse(v).map(Some).map_err(|_| {
                UsageError::with_subject(
                    UsageErrorKind::InvalidValue,
                    "run ID (environment variable)",
                )
            }),
        },
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
        return resolve_run_id(cmd.run_id.as_ref(), DEFAULT_RUN_ID_ENV, ctx.env);
    }
    let env_name = ctx.runtime.run_id_env(path);
    resolve_run_id(
        None,
        env_name.as_deref().unwrap_or(DEFAULT_RUN_ID_ENV),
        ctx.env,
    )
}
