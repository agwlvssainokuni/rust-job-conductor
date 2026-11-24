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

//! `init` の型(U4 の BR1.1、BR1.5、BR2.2)のテスト。
//!
//! オプションの既定(`InitOptions::default()` と、引数から取り出した既定)は、`args_grammar.rs` の
//! `init_の既定は_conductor_toml_と_my_pipeline_で_force_なし` が確かめる。

use std::ffi::OsString;

use crate::types::InitTarget;

#[test]
fn ハイフンだけが標準出力でそれ以外はファイル() {
    assert_eq!(InitTarget::from_path("-".into()), InitTarget::Stdout);
    for path in ["conductor.toml", "-x", "--", "a/-", ""] {
        assert_eq!(
            InitTarget::from_path(path.into()),
            InitTarget::File(OsString::from(path)),
            "{path}"
        );
    }
}
