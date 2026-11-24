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

//! 途中の表現(`Value` の木)を比べる単体テスト(`lib.rs` の最後の宣言)。
//!
//! クレートの中の非公開の `read_value` と `crate::types::Value` を直接使う。下位のモジュールの
//! ファイル名は、元の統合テスト(`tests/` の下)と同じにしている。ここには、下位のモジュールが
//! 共通に使う補助(Value の作り方、時間の上限つきの実行)を置く。

mod convert_ac;
mod convert_roundtrip;
mod read_resolve;

use std::sync::mpsc;
use std::time::Duration;

use crate::types::Value;
use crate::{ConversionError, TargetFormat, read_value};

/// `text` を YAML として読み、展開した木にする(`read_value` の `target` は変換後の形式)。
fn yaml(text: &str) -> Result<Value, ConversionError> {
    read_value(text, TargetFormat::Toml, "t.yaml")
}

/// `text` を TOML として読み、展開した木にする。
fn toml(text: &str) -> Result<Value, ConversionError> {
    read_value(text, TargetFormat::Yaml, "t.toml")
}

fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

fn i(n: i64) -> Value {
    Value::Int(n)
}

fn m(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn a(items: Vec<Value>) -> Value {
    Value::Array(items)
}

/// 暴走の入力のテストの時間の上限(30 秒)。別のスレッドで動かし、超えたら失敗にする。
fn with_timeout<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(30))
        .expect("30 秒以内に終わり、パニックしない")
}
