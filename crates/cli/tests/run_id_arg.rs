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

//! `--run-id` の指定の型 `RunIdArg`(internal-cleanup-2 の BR4.1。FR1.2〜FR1.4)。

use conductor_cli::{RunIdArg, UsageErrorKind};
use conductor_core::RunId;

#[test]
fn run_id_arg_は空を固定_規則どおりを値_規則違反を値を含めない誤りにする() {
    // 空は「run ID なしに固定」。書かれた値は空文字。
    let empty = RunIdArg::parse("").expect("空は有効");
    assert_eq!(empty, RunIdArg::Empty);
    assert_eq!(empty.as_str(), "");

    // 規則どおりの値は、検査済みの RunId を持ち、書かれた値をそのまま返す。
    let long_ok = "a".repeat(64);
    for ok in ["r1", "build-2026.11_a", long_ok.as_str()] {
        let arg = RunIdArg::parse(ok).expect("規則どおり");
        assert_eq!(arg, RunIdArg::Id(RunId::parse(ok).expect("規則どおり")));
        assert_eq!(arg.as_str(), ok);
    }

    // 規則に反する値は invalidValue。対象は `--run-id` で、値は含めない。
    let long_bad = "z".repeat(65);
    for bad in ["a/b", "..", ".x", "a b", "日本語", long_bad.as_str()] {
        let e = RunIdArg::parse(bad).expect_err("規則違反");
        assert_eq!(e.kind, UsageErrorKind::InvalidValue, "{bad}");
        assert_eq!(e.subject.as_deref(), Some("--run-id"), "{bad}");
    }
}
