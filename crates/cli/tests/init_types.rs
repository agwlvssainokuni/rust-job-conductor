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

use std::ffi::OsString;

use conductor_cli::{CliCommand, CommandKind, InitOptions, InitTarget};

#[test]
fn init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml() {
    let options = InitOptions::default();
    assert_eq!(options.pipeline_name, "my-pipeline");
    assert!(!options.force);
    assert_eq!(
        options.target,
        InitTarget::File(OsString::from("conductor.toml"))
    );
}

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

#[test]
fn コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ() {
    let cmd = CliCommand::new(CommandKind::Init);
    assert_eq!(cmd.init_options(), InitOptions::default());
}
