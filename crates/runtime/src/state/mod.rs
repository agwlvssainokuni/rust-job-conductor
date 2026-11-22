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

//! ステート(StateStore)。ファイルに触れるのは、`reader`、`writer` だけで、
//! `consistency` は、入出力のない純粋な処理である。ステートの更新(`initial_state`、`apply_*`)は、
//! `conductor-core` の `flow::record` にある(`lib.rs` で再公開する)。

pub(crate) mod consistency;
pub(crate) mod reader;
pub(crate) mod reset_all;
pub(crate) mod writer;
