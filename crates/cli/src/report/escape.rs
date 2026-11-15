//! 表示の安全(Escape。NFR4.2)。入力に由来する文字列を、端末に出せる形にする。
//!
//! conductor-core(`sanitize`)・conductor-runtime・conductor-converter の、同じ規則の関数と、同じ入力に同じ結果を返す
//! (`tests/escape_table.rs` が、同じ入力と期待値の表で揃えを確かめる)。同じ規則の関数が
//! 4 つの Unit に重複している。共通のクレートへまとめることは、将来の検討事項である。

/// ジョブID・キー名などを表示するときの最大の文字数(NFR4.2)。
pub const MAX_DISPLAY_CHARS: usize = 64;

/// 入力に由来する文字列を、表示用に整える。
///
/// - 制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・双方向の制御文字(U+200B〜U+200F、
///   U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)を、`\u{..}` にする。
///   改行・復帰・タブは、`\n`・`\r`・`\t`。
/// - `limit` があれば、エスケープ後の文字数で、その文字数までに収め、切ったときは末尾に `…` を付ける。
///   `None` は、切らない(パス。エスケープだけ)。
pub fn escape_display(input: &str, limit: Option<usize>) -> String {
    let mut out = String::with_capacity(input.len());
    let mut count = 0usize;
    for c in input.chars() {
        let piece = escape_char(c);
        let len = piece.chars().count();
        if limit.is_some_and(|max| count + len > max) {
            out.push('…');
            return out;
        }
        out.push_str(&piece);
        count += len;
    }
    out
}

/// ジョブID・名前など(64 文字で切る)。
pub fn escape_name(input: &str) -> String {
    escape_display(input, Some(MAX_DISPLAY_CHARS))
}

/// パス(切らない。エスケープだけ)。
pub fn escape_path(input: &str) -> String {
    escape_display(input, None)
}

fn escape_char(c: char) -> String {
    match c {
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        c if c.is_control() || is_format_char(c) => format!("\\u{{{:x}}}", c as u32),
        c => c.to_string(),
    }
}

/// 表示を乱す書式文字・双方向制御文字(ゼロ幅、方向指定、行・段落区切り、BOM など)。
fn is_format_char(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200F}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}
