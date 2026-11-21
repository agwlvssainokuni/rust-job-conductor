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

//! 図の構造と状態の重ね合わせ(BR2.1、BR4.1、BR4.2)。

use super::NodeState;
use crate::types::JobStatus;

/// ジョブの状態から、図の状態を決める(BR4.1)。成功と警告は `Ok` にまとめる。
pub fn node_state_of(status: JobStatus) -> NodeState {
    match status {
        JobStatus::Success | JobStatus::Warning => NodeState::Ok,
        JobStatus::Skipped => NodeState::Skipped,
        JobStatus::Error => NodeState::Error,
        JobStatus::Pending => NodeState::Pending,
    }
}
