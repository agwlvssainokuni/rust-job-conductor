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

//! 単体テストの共通の補助(偽の `Env` など)。
//!
//! 統合テストの `tests/common/mod.rs` の同じ名前のものの写し(本体は変えない)。単体テストが使う
//! ものだけを置く。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::deps::Env;

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
    pub fn with(vars: &[(&str, &str)]) -> Self {
        Self {
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Self::default()
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
