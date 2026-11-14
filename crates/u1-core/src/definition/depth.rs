//! 入れ子の深さの検査(Depth)。
//!
//! 読み取りのあとの木を走査して、入れ子の深さを数える。テーブル、テーブルの配列、
//! インラインのテーブル、配列を、それぞれ 1 段として数え(根の文書は数えない)、
//! 32 段を超えたら `syntax` で拒否する(NFR4.7、D4)。
//! 走査は、上限を超えた時点で打ち切るため、再帰の深さは上限 + 1 までである。

use toml::de::{DeTable, DeValue};

use crate::error::{DefinitionError, DefinitionErrorKind, Locate};

/// 入れ子の深さの上限。
pub const MAX_NESTING_DEPTH: usize = 32;

/// 深さの上限を超える要素があれば、その位置(バイト)を返す。
pub fn find_too_deep(root: &DeTable<'_>) -> Option<usize> {
    walk_table(root, 0)
}

fn walk_table(table: &DeTable<'_>, depth: usize) -> Option<usize> {
    for (_, value) in table.iter() {
        if let Some(pos) = walk_value(value.span().start, value.get_ref(), depth) {
            return Some(pos);
        }
    }
    None
}

fn walk_value(start: usize, value: &DeValue<'_>, depth: usize) -> Option<usize> {
    match value {
        DeValue::Table(t) => {
            if depth + 1 > MAX_NESTING_DEPTH {
                return Some(start);
            }
            for (_, v) in t.iter() {
                if let Some(pos) = walk_value(v.span().start, v.get_ref(), depth + 1) {
                    return Some(pos);
                }
            }
            None
        }
        DeValue::Array(a) => {
            if depth + 1 > MAX_NESTING_DEPTH {
                return Some(start);
            }
            for v in a.iter() {
                if let Some(pos) = walk_value(v.span().start, v.get_ref(), depth + 1) {
                    return Some(pos);
                }
            }
            None
        }
        _ => None,
    }
}

/// 深さの検査。超えていれば `syntax`(理由「深すぎる入れ子」)を返す。
pub fn check_depth(root: &DeTable<'_>, file: &str, locate: &Locate) -> Result<(), DefinitionError> {
    match find_too_deep(root) {
        None => Ok(()),
        Some(pos) => Err(DefinitionError::new(
            file,
            Some(locate.line_of(pos)),
            DefinitionErrorKind::Syntax,
            "入れ子が深すぎます(上限は 32 段)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn depth_result(text: &str) -> Result<(), DefinitionError> {
        let locate = Locate::new(text);
        let doc = DeTable::parse(text).expect("読み取れる入力");
        check_depth(doc.get_ref(), "f.toml", &locate)
    }

    fn arrays(depth: usize) -> String {
        format!("a = {}{}\n", "[".repeat(depth), "]".repeat(depth))
    }

    fn inline_tables(depth: usize) -> String {
        format!("a = {}1{}\n", "{b = ".repeat(depth), "}".repeat(depth))
    }

    #[test]
    fn 配列の深さ_32_は受理し_33_は拒否() {
        assert!(depth_result(&arrays(32)).is_ok());
        let e = depth_result(&arrays(33)).unwrap_err();
        assert_eq!(e.kind, DefinitionErrorKind::Syntax);
        assert_eq!(e.line, Some(1));
    }

    #[test]
    fn インラインテーブルの深さ_32_は受理し_33_は拒否() {
        // 外側の a の値が 1 段目。32 段のインラインテーブルが上限。
        assert!(depth_result(&inline_tables(32)).is_ok());
        assert!(depth_result(&inline_tables(33)).is_err());
    }

    #[test]
    fn テーブルの見出しの深さも数える() {
        let ok = format!("[{}]\nx = 1\n", vec!["a"; 32].join("."));
        let ng = format!("[{}]\nx = 1\n", vec!["a"; 33].join("."));
        assert!(depth_result(&ok).is_ok());
        assert!(depth_result(&ng).is_err());
    }

    #[test]
    fn 通常の定義の深さは浅い() {
        let text = "format_version = 1\n[pipeline]\nname = \"p\"\n[[jobs]]\nid = \"a\"\n[[jobs.transitions]]\non = \"success\"\n";
        assert!(depth_result(text).is_ok());
    }

    #[test]
    fn 深い入力でも行番号を返す() {
        let text = format!("x = 1\n\n{}", arrays(40));
        let e = depth_result(&text).unwrap_err();
        assert_eq!(e.line, Some(3));
    }
}
