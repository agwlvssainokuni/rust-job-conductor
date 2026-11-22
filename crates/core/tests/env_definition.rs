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

//! 定義の新しいキー `env`、`env_file` の読み取りと検査(U1。BR1.1〜BR1.8、FR1.1〜FR1.4、FR1.9、FR1.11、FR1.13)。

mod common;

use common::{SECRET, kinds};
use conductor_core::{DefinitionErrorKind as K, PipelineDefinition, parse_definition};

fn parse(text: &str) -> Result<PipelineDefinition, Vec<(K, String)>> {
    parse_definition(text, "t.toml", None)
        .map_err(|errs| errs.into_iter().map(|e| (e.kind, e.to_string())).collect())
}

fn ok(text: &str) -> PipelineDefinition {
    parse(text).unwrap_or_else(|e| panic!("受理されること: {e:?}"))
}

/// パイプラインの追加部分 `p` と、ジョブ `a` の追加部分 `j` を持つ定義。
fn def(p: &str, j: &str) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n{p}\n[[job]]\nid = \"a\"\ncommand = \"c\"\n{j}\n"
    )
}

#[test]
fn 正しい_env_と_env_file_を書いた順に読める() {
    let text = def(
        "env_file = \".env\"\n[pipeline.env]\nZ = \"1\"\nA = \"two words\"\nM = \"\"\n",
        "env_file = [\"a.env\", \"b.env\"]\n[job.env]\nJ = \"x\"\n",
    );
    // [pipeline.env] の後ろに [[job]] が来るため、job の表は別に確かめる
    let d = ok(&text);
    let names: Vec<_> = d.env.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        ["Z", "A", "M"],
        "書いた順を保つ(キーの並べ替えをしない)"
    );
    assert_eq!(d.env[1].value, "two words");
    assert_eq!(d.env[2].value, "");
    assert_eq!(d.env_file, [".env"]);
    assert_eq!(d.job[0].env_file, ["a.env", "b.env"]);
    assert_eq!(d.job[0].env[0].name, "J");
}

#[test]
fn env_file_は文字列も配列も空の配列も受け付ける() {
    assert_eq!(ok(&def("env_file = \"x.env\"\n", "")).env_file, ["x.env"]);
    assert_eq!(
        ok(&def("env_file = [\"x.env\", \"y.env\"]\n", "")).env_file,
        ["x.env", "y.env"]
    );
    assert!(ok(&def("env_file = []\n", "")).env_file.is_empty());
}

#[test]
fn 新しいキーのない既存の定義は変わらず読めて展開もしない() {
    let d = ok(&def("", ""));
    assert!(d.env.is_empty() && d.env_file.is_empty());
    assert!(d.job[0].env.is_empty() && d.job[0].env_file.is_empty());
    assert_eq!(d.format_version, 1);
    // 値の `${X}` は、そのまま
    let d = ok(&def("[pipeline.env]\nV = \"${HOME}/$X\"\n", ""));
    assert_eq!(d.env[0].value, "${HOME}/$X");
}

#[test]
fn 値が文字列でない_env_file_の要素が文字列でないか空は誤り() {
    assert_eq!(
        kinds(&def("[pipeline.env]\nA = 1\n", "")),
        [K::TypeMismatch]
    );
    assert_eq!(kinds(&def("env = \"x\"\n", "")), [K::TypeMismatch]);
    assert_eq!(kinds(&def("env_file = 3\n", "")), [K::TypeMismatch]);
    assert_eq!(
        kinds(&def("env_file = [\"a\", 2]\n", "")),
        [K::TypeMismatch]
    );
    assert_eq!(kinds(&def("env_file = \"\"\n", "")), [K::EmptyField]);
    assert_eq!(
        kinds(&def("env_file = [\"a\", \"\"]\n", "")),
        [K::EmptyField]
    );
}

#[test]
fn 名前の規則に合わない名前は誤り() {
    for bad in ["\"1A\"", "\"A-B\"", "\"A B\"", "\"\"", "\"日本\""] {
        let text = def(&format!("env = {{ {bad} = \"v\" }}\n"), "");
        assert_eq!(kinds(&text), [K::InvalidEnvName], "{bad}");
    }
    ok(&def("env = { _A1 = \"v\", a_b = \"w\" }\n", ""));
}

#[test]
fn 予約名は大文字小文字を区別せずに拒否する() {
    for name in [
        "CONDUCTOR_X",
        "conductor_x",
        "Conductor_Run_Id",
        "CONDUCTOR_",
    ] {
        let text = def(&format!("env = {{ {name} = \"v\" }}\n"), "");
        assert_eq!(kinds(&text), [K::ReservedEnvName], "{name}");
        let text = def("", &format!("env = {{ {name} = \"v\" }}\n"));
        assert_eq!(kinds(&text), [K::ReservedEnvName], "job {name}");
    }
    // 接頭辞でない名前は許す
    ok(&def(
        "env = { CONDUCTOR = \"v\", XCONDUCTOR_A = \"w\" }\n",
        "",
    ));
}

#[test]
fn 同じ表の中の大文字小文字だけが違う名前は誤りで別の表なら誤りでない() {
    let text = def("env = { foo = \"1\", FOO = \"2\" }\n", "");
    assert_eq!(kinds(&text), [K::EnvNameCaseConflict]);
    // 別の表(pipeline と job)の同じ綴り、大文字小文字違いは誤りでない
    let d = ok(&def("env = { foo = \"1\" }\n", "env = { FOO = \"2\" }\n"));
    assert_eq!(d.env[0].name, "foo");
    assert_eq!(d.job[0].env[0].name, "FOO");
}

#[test]
fn nul_を含む値は誤り() {
    let text = def("env = { A = \"x\\u0000y\" }\n", "");
    assert_eq!(kinds(&text), [K::InvalidEnvValue]);
}

#[test]
fn 誤りの表示に値が含まれない() {
    let cases = [
        def(&format!("env = {{ A = \"{SECRET}\\u0000\" }}\n"), ""),
        def(&format!("env = {{ CONDUCTOR_A = \"{SECRET}\" }}\n"), ""),
        def(&format!("env = {{ a = \"1\", A = \"{SECRET}\" }}\n"), ""),
        def(&format!("env = {{ \"1x\" = \"{SECRET}\" }}\n"), ""),
        def(&format!("env = \"{SECRET}\"\n"), ""),
        def("", &format!("env = {{ A = [\"{SECRET}\"] }}\n")),
    ];
    for text in cases {
        for (_, shown) in parse(&text).unwrap_err() {
            assert!(!shown.contains(SECRET), "{shown}");
        }
    }
}
