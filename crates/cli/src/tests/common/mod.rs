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
//! 偽の `Env`・`Sink` と `state_error` は、統合テストと共有するファイル(`tests/common/fakes.rs`)を
//! 写しを持たずに `#[path]` で読む。`fakes.rs` は、この親のモジュールの `Env`・`Sink`・`Language`
//! を使う。

pub mod kinds;

#[path = "../../../tests/common/fakes.rs"]
mod fakes;

pub use fakes::{FakeEnv, FakeSink, state_error};

use crate::deps::{Env, Sink};
use crate::types::Language;
