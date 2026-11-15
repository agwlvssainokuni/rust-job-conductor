//! 読み取りのライブラリ(`toml::de::DeTable`)の確認(security-design の確認項目)。
//!
//! (a) 要素ごとのバイトの範囲が取れること。
//! (b) 入れ子の深さ 1000 の入力で、異常終了せず失敗が返ること。

use toml::de::{DeTable, DeValue};

#[test]
fn 要素ごとのバイトの範囲が取れる() {
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n";
    let doc = DeTable::parse(text).expect("正しい TOML");
    let root = doc.get_ref();
    let (key, value) = root.get_key_value("format_version").expect("キーがある");
    assert_eq!(&text[key.span()], "format_version");
    assert_eq!(&text[value.span()], "1");
    assert!(matches!(value.get_ref(), DeValue::Integer(_)));
    let pipeline = root.get("pipeline").expect("pipeline がある");
    assert!(pipeline.span().start >= text.find('[').unwrap_or(0));
}

fn 入れ子の配列(depth: usize) -> String {
    format!("a = {}{}", "[".repeat(depth), "]".repeat(depth))
}

fn 入れ子のインラインテーブル(depth: usize) -> String {
    let mut s = String::from("a = ");
    for _ in 0..depth {
        s.push_str("{b = ");
    }
    s.push('1');
    s.push_str(&"}".repeat(depth));
    s
}

#[test]
fn 深さ_1000_の入力でも異常終了しない() {
    // 配列とインラインテーブルを、深さ 1000 と 100000 で試す。
    // 結果は成功でも失敗でもよく、パニックやスタック溢れをしないことを確かめる。
    for depth in [33, 1000, 100_000] {
        let _ = DeTable::parse(&入れ子の配列(depth));
        let _ = DeTable::parse(&入れ子のインラインテーブル(depth));
    }
}

#[test]
fn 深さ_1000_の入力は失敗として返る() {
    assert!(DeTable::parse(&入れ子の配列(1000)).is_err());
    assert!(DeTable::parse(&入れ子のインラインテーブル(1000)).is_err());
}
