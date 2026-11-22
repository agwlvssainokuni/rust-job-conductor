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

//! ステート(StateStore)。ファイルに触れるのは、`reader`、`writer`、`reset_all` だけである。照合
//! (`check_consistency`)と、その単体テストは `conductor-core` の `consistency` にある。ステートの更新
//! (`initial_state`、`apply_*`)は、`conductor-core` の `flow::record` にある(どちらも `lib.rs` で
//! 再公開する)。

pub(crate) mod reader;
pub(crate) mod reset_all;
pub(crate) mod writer;
