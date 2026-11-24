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

//! 単体テストの共通の補助(偽の `Env`・`Sink`、ステートの誤りの組み立て)。
//!
//! 統合テストの `tests/common/mod.rs` の同じ名前のものの写し(本体は変えない)。単体テストが使う
//! ものだけを置く。

pub mod kinds;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::path::Path;

use conductor_runtime::{StateError, StateErrorKind};

use crate::deps::{Env, Sink};
use crate::types::Language;

// ---- Env ----

#[derive(Default)]
pub struct FakeEnv {
    pub vars: HashMap<String, String>,
    pub terminal: bool,
    pub var_calls: Cell<usize>,
    pub terminal_calls: Cell<usize>,
    /// 読まれた環境変数の名前(読んだ順)。
    pub names: RefCell<Vec<String>>,
}

impl FakeEnv {
    /// ロケールの環境変数(`LC_ALL`・`LC_MESSAGES`・`LANG`)が読まれたか(言語の判定の遅延を確かめる)。
    pub fn locale_read(&self) -> bool {
        self.names
            .borrow()
            .iter()
            .any(|n| matches!(n.as_str(), "LC_ALL" | "LC_MESSAGES" | "LANG"))
    }

    /// 色の判定のための問い合わせ(`NO_COLOR` を読んだ、または端末かを問い合わせた)があったか
    /// (色の判定の遅延を確かめる)。
    pub fn color_read(&self) -> bool {
        self.terminal_calls.get() > 0 || self.names.borrow().iter().any(|n| n == "NO_COLOR")
    }

    pub fn with(vars: &[(&str, &str)]) -> Self {
        Self {
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Self::default()
        }
    }

    /// 言語 `lang` のロケール(`LANG`)だけを持つ環境。
    pub fn for_language(lang: Language) -> Self {
        match lang {
            Language::En => Self::with(&[("LANG", "en_US.UTF-8")]),
            Language::Ja => Self::with(&[("LANG", "ja_JP.UTF-8")]),
        }
    }
}

impl Env for FakeEnv {
    fn var(&self, name: &str) -> Option<String> {
        self.var_calls.set(self.var_calls.get() + 1);
        self.names.borrow_mut().push(name.to_string());
        self.vars.get(name).cloned()
    }

    fn stdout_is_terminal(&self) -> bool {
        self.terminal_calls.set(self.terminal_calls.get() + 1);
        self.terminal
    }
}

// ---- Sink ----

#[derive(Default)]
pub struct FakeSink {
    pub out: Vec<u8>,
    pub err: Vec<u8>,
    pub fail_out: bool,
    pub fail_err: bool,
    pub out_writes: usize,
}

impl FakeSink {
    pub fn out_text(&self) -> String {
        String::from_utf8_lossy(&self.out).into_owned()
    }

    pub fn err_text(&self) -> String {
        String::from_utf8_lossy(&self.err).into_owned()
    }
}

impl Sink for FakeSink {
    fn write_out(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.fail_out {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"));
        }
        self.out_writes += 1;
        self.out.extend_from_slice(bytes);
        Ok(())
    }

    fn write_err(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.fail_err {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"));
        }
        self.err.extend_from_slice(bytes);
        Ok(())
    }
}

// ---- ステートの誤り ----

pub fn state_error(path: &Path, kind: StateErrorKind) -> StateError {
    StateError {
        path: path.to_path_buf(),
        kind,
        // 英語の表示を ASCII と確かめるテスト(`report_messages.rs`)とも共有するので、ASCII の文にする。
        reason: "reason",
    }
}
