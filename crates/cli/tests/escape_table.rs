//! `escape_display` の、同じ入力と期待値の表(conductor-core・conductor-runtime・conductor-converter と揃える。NFR4.2)。
//!
//! conductor-core の `sanitize` は公開されているため、同じ入力に同じ結果を返すことを直接確かめる。
//! conductor-runtime・conductor-converter の同じ規則の関数は、非公開のため、各 Unit のテストにある、同じ期待値の表で揃える。

use conductor_cli::report::escape::{escape_display, escape_name, escape_path};

/// (入力, 上限, 期待値)。conductor-core・conductor-converter の単体テストの期待値と同じ規則。
fn table() -> Vec<(String, usize, String)> {
    vec![
        ("a\nb\x1b[0m".into(), 64, "a\\nb\\u{1b}[0m".into()),
        ("k\tx".into(), 64, "k\\tx".into()),
        ("a\rb".into(), 64, "a\\rb".into()),
        ("日本語 abc".into(), 64, "日本語 abc".into()),
        ("a\u{200b}b".into(), 64, "a\\u{200b}b".into()),
        ("a\u{202e}b".into(), 64, "a\\u{202e}b".into()),
        ("a\u{2066}b".into(), 64, "a\\u{2066}b".into()),
        ("a\u{feff}b".into(), 64, "a\\u{feff}b".into()),
        (
            "a\u{2028}b\u{2029}".into(),
            64,
            "a\\u{2028}b\\u{2029}".into(),
        ),
        ("\x00\x7f\u{85}".into(), 64, "\\u{0}\\u{7f}\\u{85}".into()),
        ("k".repeat(64), 64, "k".repeat(64)),
        ("k".repeat(200), 64, format!("{}…", "k".repeat(64))),
        ("\n".repeat(100), 64, format!("{}…", "\\n".repeat(32))),
        ("abcdef".into(), 3, "abc…".into()),
        ("".into(), 64, "".into()),
    ]
}

#[test]
fn 期待値の表のとおりにエスケープされ_切り詰められる() {
    for (input, limit, expected) in table() {
        assert_eq!(escape_display(&input, Some(limit)), expected, "{input:?}");
    }
}

#[test]
fn u1_core_の_sanitize_と同じ結果() {
    for (input, limit, _) in table() {
        assert_eq!(
            escape_display(&input, Some(limit)),
            conductor_core::sanitize(&input, limit),
            "{input:?}"
        );
    }
}

#[test]
fn パスは切らず_エスケープだけ() {
    let long = format!("dir\n/{}", "p".repeat(300));
    let out = escape_path(&long);
    assert!(out.starts_with("dir\\n/"));
    assert!(out.ends_with(&"p".repeat(300)));
    assert_eq!(escape_name(&long).chars().count(), 65);
}

#[test]
fn エスケープ後の文字数で切る() {
    // `\n` は 2 文字になる。64 文字を超えないところで切れる。
    let out = escape_display(&"\n".repeat(100), Some(64));
    assert_eq!(out.chars().count(), 65);
}
