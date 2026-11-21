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

//! `conductor init` の流れ(U4 の WF2。BR1.1〜BR1.6、BR3.1、BR3.2)。
//!
//! ひな形を作り、ファイルに排他的に書く(または標準出力に出す)。定義、`.env`、ステートは読まず、
//! シグナルの準備もしない(BR1.1)。引数の誤り(68)は、`parse_args` が、ファイルに触れる前に返している。

use std::path::Path;

use conductor_runtime::{CreateError, CreateOutcome};

use crate::ctx::Ctx;
use crate::exit::{EXIT_EXISTS, EXIT_OK, EXIT_STATE};
use crate::report::DisplayEvent;
use crate::report::init_template::render_template;
use crate::types::{CliCommand, InitTarget};

/// `conductor init` を実行し、終了コードを返す。
pub(crate) fn run_init(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let text = render_template(ctx.reporter.language(), &cmd.init.pipeline_name);
    match &cmd.init.target {
        InitTarget::Stdout => {
            // ひな形だけを 1 回で書く。`--force` は意味がないので、無視する(BR1.5)。
            if ctx.show(&DisplayEvent::InitTemplate(&text)) {
                EXIT_OK
            } else {
                ctx.show(&DisplayEvent::InitStdoutFailed);
                EXIT_STATE
            }
        }
        InitTarget::File(path) => {
            let path = Path::new(path);
            match ctx
                .runtime
                .create_file(path, text.as_bytes(), cmd.init.force)
            {
                Ok(CreateOutcome::Created) => {
                    ctx.show(&DisplayEvent::InitCreated { path });
                    EXIT_OK
                }
                Ok(CreateOutcome::Replaced) => {
                    ctx.show(&DisplayEvent::InitReplaced { path });
                    EXIT_OK
                }
                Err(CreateError::AlreadyExists) => {
                    ctx.show(&DisplayEvent::InitExists { path });
                    EXIT_EXISTS
                }
                Err(error) => {
                    ctx.show(&DisplayEvent::InitFailed { path, error });
                    EXIT_STATE
                }
            }
        }
    }
}
