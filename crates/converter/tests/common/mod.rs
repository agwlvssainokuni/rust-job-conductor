//! 統合テストの共通の補助(Value の作り方、時間の上限つきの実行)。

#![allow(dead_code)]

use std::sync::mpsc;
use std::time::Duration;

use conductor_converter::testing::{Value, read_tree};
use conductor_converter::{ConversionError, TargetFormat};

pub fn yaml(text: &str) -> Result<Value, ConversionError> {
    read_tree(text, TargetFormat::Yaml, "t.yaml")
}

pub fn toml(text: &str) -> Result<Value, ConversionError> {
    read_tree(text, TargetFormat::Toml, "t.toml")
}

pub fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

pub fn i(n: i64) -> Value {
    Value::Int(n)
}

pub fn m(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

pub fn a(items: Vec<Value>) -> Value {
    Value::Array(items)
}

/// 暴走の入力のテストの時間の上限(30 秒)。別のスレッドで動かし、超えたら失敗にする。
pub fn with_timeout<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(30))
        .expect("30 秒以内に終わり、パニックしない")
}
