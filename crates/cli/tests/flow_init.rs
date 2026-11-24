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

//! `conductor init` の流れのテスト(偽の `Runtime`。ファイルに触れない)。U4 の BR1.1〜BR1.6、BR3.1、BR3.2。

mod common;

use std::ffi::OsString;
use std::path::PathBuf;

use common::{
    FakeEnv, FakeSignals, FakeSink, Language, ScriptedRuntime, SignalMode, run_with_fakes,
};
use conductor_runtime::{CreateError, CreateOutcome};

/// 日本語のひな形の原本(名前は既定の `my-pipeline`)。単体テストの `src/tests/init_template.rs` も
/// 同じファイルを読む。
const ORIGINAL_JA: &str = include_str!("fixtures/init_ja.toml");

/// 英語のひな形の原本(名前は既定の `my-pipeline`)。単体テストの `src/tests/init_template.rs` も
/// 同じファイルを読む。
const ORIGINAL_EN: &str = include_str!("fixtures/init_en.toml");

/// 期待のひな形。言語 `lang` の原本の `name = "my-pipeline"` を `name = "<name>"` に 1 回だけ置き換える。
/// `name` はエスケープの要らない名前だけ。
fn expected_template(lang: Language, name: &str) -> String {
    let original = match lang {
        Language::En => ORIGINAL_EN,
        Language::Ja => ORIGINAL_JA,
    };
    original.replacen("name = \"my-pipeline\"", &format!("name = \"{name}\""), 1)
}

struct Run {
    code: i32,
    sink: FakeSink,
    runtime: ScriptedRuntime,
    signals: FakeSignals,
}

/// `create_file` の結果を `result` に決めて、`init` を実行する。
fn run_with(
    lang: Language,
    result: Result<CreateOutcome, CreateError>,
    sink: FakeSink,
    args: &[&str],
) -> Run {
    let runtime = ScriptedRuntime::default();
    *runtime.create_override.borrow_mut() = Some(result);
    let env = FakeEnv::for_language(lang);
    let signals = FakeSignals::new(SignalMode::Noop);
    let mut sink = sink;
    let code = run_with_fakes(
        &runtime,
        &mut sink,
        &env,
        &signals,
        args.iter().map(OsString::from).collect(),
    );
    Run {
        code,
        sink,
        runtime,
        signals,
    }
}

fn run(lang: Language, result: Result<CreateOutcome, CreateError>, args: &[&str]) -> Run {
    run_with(lang, result, FakeSink::default(), args)
}

#[test]
fn 既定の名前とパスで_ひな形が書かれ_標準エラーに作成の1行() {
    let r = run(Language::En, Ok(CreateOutcome::Created), &["init"]);
    assert_eq!(r.code, 0);
    let calls = r.runtime.create_calls.borrow();
    assert_eq!(calls.len(), 1);
    let (path, bytes, force) = &calls[0];
    assert_eq!(path, &PathBuf::from("conductor.toml"));
    assert_eq!(
        String::from_utf8_lossy(bytes),
        expected_template(Language::En, "my-pipeline")
    );
    assert!(!force);
    assert_eq!(r.sink.err_text(), "created conductor.toml\n");
    assert!(r.sink.out.is_empty());

    let r = run(Language::Ja, Ok(CreateOutcome::Created), &["init"]);
    assert_eq!(r.code, 0);
    assert_eq!(r.sink.err_text(), "conductor.toml を作成しました\n");
    let calls = r.runtime.create_calls.borrow();
    assert_eq!(
        String::from_utf8_lossy(&calls[0].1),
        expected_template(Language::Ja, "my-pipeline")
    );
}

#[test]
fn 名前と_config_を指定すると_そのパスとエスケープした名前で書かれる() {
    let r = run(
        Language::En,
        Ok(CreateOutcome::Created),
        &["init", "--config", "dir/x.toml", "a\"b"],
    );
    assert_eq!(r.code, 0);
    let calls = r.runtime.create_calls.borrow();
    assert_eq!(calls[0].0, PathBuf::from("dir/x.toml"));
    let text = String::from_utf8_lossy(&calls[0].1);
    assert!(text.contains("name = \"a\\\"b\"\n"), "{text}");
    assert_eq!(r.sink.err_text(), "created dir/x.toml\n");
}

#[test]
fn force_が渡り_置き換えたら置き換えたと表示して_0() {
    let r = run(
        Language::En,
        Ok(CreateOutcome::Replaced),
        &["init", "--force"],
    );
    assert_eq!(r.code, 0);
    assert!(r.runtime.create_calls.borrow()[0].2);
    assert_eq!(r.sink.err_text(), "replaced conductor.toml\n");

    let r = run(
        Language::Ja,
        Ok(CreateOutcome::Replaced),
        &["init", "--force"],
    );
    assert_eq!(r.sink.err_text(), "conductor.toml を置き換えました\n");
}

#[test]
fn 既にあるときは_69_と_既にありますの_1_行で_標準出力は空() {
    let r = run(Language::En, Err(CreateError::AlreadyExists), &["init"]);
    assert_eq!(r.code, 69);
    assert_eq!(
        r.sink.err_text(),
        "conductor: error: conductor.toml already exists (use --force to overwrite)\n"
    );
    assert!(r.sink.out.is_empty());

    let r = run(Language::Ja, Err(CreateError::AlreadyExists), &["init"]);
    assert_eq!(r.code, 69);
    assert_eq!(
        r.sink.err_text(),
        "conductor: error: conductor.toml は既にあります(上書きするには --force)\n"
    );
}

#[test]
fn 作れないときは_65_と_種類ごとの固定の文で_os_の文を含まない() {
    let cases = [
        (
            CreateError::ParentMissing,
            "conductor: error: cannot create conductor.toml: the parent directory does not exist\n",
        ),
        (
            CreateError::IsDirectory,
            "conductor: error: cannot create conductor.toml: it is a directory\n",
        ),
        (
            CreateError::PermissionDenied,
            "conductor: error: cannot create conductor.toml: permission denied\n",
        ),
        (
            CreateError::OtherIo,
            "conductor: error: cannot create conductor.toml: I/O error\n",
        ),
    ];
    for (error, expected) in cases {
        let r = run(Language::En, Err(error), &["init"]);
        assert_eq!(r.code, 65, "{error:?}");
        assert_eq!(r.sink.err_text(), expected, "{error:?}");
        assert!(r.sink.out.is_empty());
    }
    let r = run(Language::Ja, Err(CreateError::ParentMissing), &["init"]);
    assert_eq!(r.code, 65);
    assert_eq!(
        r.sink.err_text(),
        "conductor: error: conductor.toml を作れません: 親のディレクトリがありません\n"
    );
}

#[test]
fn config_ハイフンは標準出力にひな形だけを出し_force_を無視し_ファイルを作らない() {
    for args in [
        &["init", "--config", "-"][..],
        &["init", "--config", "-", "--force"],
    ] {
        let r = run(Language::En, Err(CreateError::OtherIo), args);
        assert_eq!(r.code, 0, "{args:?}");
        assert_eq!(
            r.sink.out_text(),
            expected_template(Language::En, "my-pipeline")
        );
        assert_eq!(r.sink.out_writes, 1, "1 回で書く");
        assert!(r.sink.err.is_empty(), "成功のメッセージは出さない");
        assert!(r.runtime.create_calls.borrow().is_empty());
    }
    let r = run(
        Language::Ja,
        Err(CreateError::OtherIo),
        &["init", "--config", "-", "夜間"],
    );
    assert_eq!(r.sink.out_text(), expected_template(Language::Ja, "夜間"));
}

#[test]
fn 標準出力に書けないときは_65() {
    let sink = FakeSink {
        fail_out: true,
        ..FakeSink::default()
    };
    let r = run_with(
        Language::En,
        Ok(CreateOutcome::Created),
        sink,
        &["init", "--config", "-"],
    );
    assert_eq!(r.code, 65);
    assert_eq!(
        r.sink.err_text(),
        "conductor: error: cannot write the template to standard output\n"
    );
    assert!(r.runtime.create_calls.borrow().is_empty());
}

#[test]
fn 引数の誤りは_ファイルに触れる前に_68() {
    let bad: [&[&str]; 6] = [
        &["init", ""],
        &["init", "   "],
        &["init", "a", "b"],
        &["init", "-v"],
        &["init", "--dry-run"],
        &["init", "--run-id", "r"],
    ];
    for args in bad {
        // 作る先が既にあることにしておく: 68 が勝つ。
        let r = run(Language::En, Err(CreateError::AlreadyExists), args);
        assert_eq!(r.code, 68, "{args:?}");
        assert!(r.runtime.create_calls.borrow().is_empty(), "{args:?}");
        assert!(r.runtime.log().is_empty(), "{args:?}");
        assert!(
            r.sink.err_text().starts_with("conductor: error: "),
            "{args:?}"
        );
        assert!(r.sink.out.is_empty(), "{args:?}");
    }
}

#[test]
fn 定義と環境ファイルとステートを読まず_シグナルも準備しない() {
    for args in [
        &["init"][..],
        &["init", "--force"],
        &["init", "--config", "-"],
    ] {
        let r = run(Language::En, Ok(CreateOutcome::Created), args);
        assert_eq!(r.code, 0, "{args:?}");
        let log = r.runtime.log();
        assert!(
            log.iter().all(|entry| entry == "create"),
            "作成の呼び出し以外はしない: {log:?}"
        );
        assert_eq!(r.runtime.run_calls.get(), 0);
        assert_eq!(r.runtime.run_id_env_calls.get(), 0);
        assert!(r.runtime.env_loads.borrow().is_empty());
        assert_eq!(r.signals.started.get(), 0, "{args:?}");
    }
}
