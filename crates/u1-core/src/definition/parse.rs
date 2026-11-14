//! 読み取り(Parse)。文字列を、位置の情報(バイトの範囲)を保った、型を決めない文書の木にする。
//!
//! 文法の誤りは `syntax` として返す。表示する理由は固定の文で、ライブラリのエラー文は使わない(NFR4.6)。

use toml::Spanned;
use toml::de::DeTable;

use crate::error::{DefinitionError, DefinitionErrorKind, Locate};

/// 文書の木(根のテーブル)。
pub type Document<'i> = Spanned<DeTable<'i>>;

/// TOML として読み取る。失敗は `syntax`(行つき)。パニックしない(NFR4.7)。
pub fn parse_document<'i>(
    text: &'i str,
    file: &str,
    locate: &Locate,
) -> Result<Document<'i>, DefinitionError> {
    DeTable::parse(text).map_err(|e| {
        let line = e.span().map(|s| locate.line_of(s.start));
        DefinitionError::new(
            file,
            line,
            DefinitionErrorKind::Syntax,
            "TOML の文法に誤りがあります",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<usize, DefinitionError> {
        let locate = Locate::new(text);
        parse_document(text, "f.toml", &locate).map(|d| d.get_ref().len())
    }

    #[test]
    fn 正しい_toml_は読める() {
        assert_eq!(parse("a = 1\nb = 2\n").unwrap(), 2);
    }

    #[test]
    fn 文法の誤りは_syntax_で行がつく() {
        let e = parse("a = 1\nb = = 2\n").unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::Syntax);
        assert_eq!(e.line, Some(2));
    }

    #[test]
    fn 理由に入力の値もライブラリの文も入らない() {
        let e = parse("secret_value_xyz = = 1\n").unwrap_err();
        assert_eq!(e.reason, "TOML の文法に誤りがあります");
        assert!(!e.to_string().contains("secret_value_xyz"));
    }

    #[test]
    fn キーの重複は_syntax() {
        let e = parse("a = 1\na = 2\n").unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::Syntax);
    }
}
