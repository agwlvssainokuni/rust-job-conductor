//! TOML の読み取り(Read)。
//!
//! `toml` のライブラリで読み、素の木(日時の値に印を付ける)に写す。深い入力は、ライブラリの
//! 再帰の制限により、異常終了せず失敗で返る(T2。テストで確かめる)。読み取り後に、明示の
//! スタックで深さを数え、32 を超えたら `syntax`。空の TOML は、空のマップ。

use crate::error::{ConversionError, ConversionErrorKind, reason};
use crate::raw::{MAX_DEPTH, RawKind, RawNode, RawScalar, RawTree, Tag};

fn syntax(file: &str, line: Option<usize>, why: &str) -> ConversionError {
    ConversionError::new(file, line, None, ConversionErrorKind::Syntax, why)
}

/// バイトの位置から、行番号(1 始まり)を求める。
fn line_at(text: &str, offset: usize) -> usize {
    let end = offset.min(text.len());
    1 + text.as_bytes()[..end]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
}

/// 写し取りの途中の、開いている表または配列。
struct Frame<'a> {
    node: usize,
    kids: Vec<(Option<&'a str>, &'a toml::Value)>,
    next: usize,
}

fn kids_of(value: &toml::Value) -> Option<Vec<(Option<&str>, &toml::Value)>> {
    match value {
        toml::Value::Table(t) => Some(t.iter().map(|(k, v)| (Some(k.as_str()), v)).collect()),
        toml::Value::Array(a) => Some(a.iter().map(|v| (None, v)).collect()),
        _ => None,
    }
}

fn scalar_of(value: &toml::Value) -> RawScalar {
    match value {
        toml::Value::String(s) => RawScalar::Str(s.clone()),
        toml::Value::Integer(n) => RawScalar::Int(*n),
        toml::Value::Float(f) => RawScalar::Float(*f),
        toml::Value::Boolean(b) => RawScalar::Bool(*b),
        // 表と配列は、ここへ来ない。日時だけが残る
        _ => RawScalar::Datetime,
    }
}

/// TOML の文字列を読んで、素の木にする。
pub(crate) fn read(text: &str, file: &str) -> Result<RawTree, ConversionError> {
    let table: toml::Table = text.parse().map_err(|e: toml::de::Error| {
        let line = e.span().map(|s| line_at(text, s.start));
        syntax(file, line, reason::SYNTAX)
    })?;
    let root_value = toml::Value::Table(table);

    let mut nodes: Vec<RawNode> = vec![RawNode {
        kind: RawKind::Map {
            pairs: Vec::new(),
            tag: Tag::None,
        },
        line: 0,
    }];
    let mut stack: Vec<Frame<'_>> = Vec::new();
    stack.push(Frame {
        node: 0,
        kids: kids_of(&root_value).unwrap_or_default(),
        next: 0,
    });

    while let Some(top) = stack.last_mut() {
        if top.next >= top.kids.len() {
            stack.pop();
            continue;
        }
        let (key, value) = top.kids[top.next];
        top.next += 1;
        let parent = top.node;

        let key_node = key.map(|k| {
            nodes.push(RawNode {
                kind: RawKind::Scalar {
                    scalar: RawScalar::Str(k.to_string()),
                    tag: Tag::None,
                },
                line: 0,
            });
            nodes.len() - 1
        });

        let kids = kids_of(value);
        if kids.is_some() && stack.len() >= MAX_DEPTH {
            return Err(syntax(file, None, reason::TOO_DEEP));
        }
        let kind = match (&kids, value) {
            (Some(_), toml::Value::Table(_)) => RawKind::Map {
                pairs: Vec::new(),
                tag: Tag::None,
            },
            (Some(_), _) => RawKind::Seq {
                items: Vec::new(),
                tag: Tag::None,
            },
            (None, v) => RawKind::Scalar {
                scalar: scalar_of(v),
                tag: Tag::None,
            },
        };
        nodes.push(RawNode { kind, line: 0 });
        let child = nodes.len() - 1;
        match (&mut nodes[parent].kind, key_node) {
            (RawKind::Map { pairs, .. }, Some(k)) => pairs.push((k, child)),
            (RawKind::Seq { items, .. }, None) => items.push(child),
            _ => {}
        }
        if let Some(kids) = kids {
            stack.push(Frame {
                node: child,
                kids,
                next: 0,
            });
        }
    }
    Ok(RawTree { nodes, root: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested_arrays(n: usize) -> String {
        format!("a = {}{}\n", "[".repeat(n), "]".repeat(n))
    }

    #[test]
    fn syntax_error_has_line() {
        let e = read("a = 1\nb = [1,\nc = 3\n", "t.toml").unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Syntax);
        assert!(e.line.is_some());
        let e = read("a = 1\nb = = 2\n", "t.toml").unwrap_err();
        assert_eq!(e.line, Some(2));
    }

    #[test]
    fn empty_toml_is_empty_map() {
        let tree = read("", "t").unwrap();
        assert!(
            matches!(&tree.nodes[tree.root].kind, RawKind::Map { pairs, .. } if pairs.is_empty())
        );
        assert!(read("# コメントだけ\n", "t").is_ok());
    }

    #[test]
    fn datetime_is_marked() {
        let tree = read("d = 1979-05-27T07:32:00Z\ns = \"x\"\n", "t").unwrap();
        let has_datetime = tree.nodes.iter().any(|n| {
            matches!(
                &n.kind,
                RawKind::Scalar {
                    scalar: RawScalar::Datetime,
                    ..
                }
            )
        });
        assert!(has_datetime);
    }

    #[test]
    fn depth_32_ok_and_33_syntax() {
        // 最上位のテーブルを 1 として、32 段(テーブル 1 + 配列 31)
        assert!(read(&nested_arrays(31), "t").is_ok());
        let e = read(&nested_arrays(32), "t").unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Syntax);
        assert_eq!(e.reason, reason::TOO_DEEP);
    }

    #[test]
    fn nested_tables_depth() {
        let key = |n: usize| vec!["a"; n].join(".");
        // ドットのキー `a.a. ... = 1`: キーが n 個のとき、表は、最上位 + (n - 1) 段
        assert!(read(&format!("{} = 1\n", key(32)), "t").is_ok());
        assert_eq!(
            read(&format!("{} = 1\n", key(33)), "t").unwrap_err().reason,
            reason::TOO_DEEP
        );
        // `a = {a={a=1}}`: インラインの表が n 段のとき、最上位 + n 段
        let inline = |n: usize| format!("a = {}1{}\n", "{a=".repeat(n), "}".repeat(n));
        assert!(read(&inline(31), "t").is_ok());
        assert_eq!(read(&inline(32), "t").unwrap_err().reason, reason::TOO_DEEP);
    }

    #[test]
    fn very_deep_inputs_fail_without_abort() {
        let handle = std::thread::spawn(|| {
            let mut failed = Vec::new();
            for n in [1000usize, 100_000] {
                failed.push(read(&nested_arrays(n), "t").is_err());
                failed.push(read(&format!("a = {}", "[".repeat(n)), "t").is_err());
                failed.push(
                    read(&format!("a = {}1{}", "{a=".repeat(n), "}".repeat(n)), "t").is_err(),
                );
                failed.push(read(&format!("{} = 1", vec!["a"; n].join(".")), "t").is_err());
                failed.push(read(&format!("[{}]\nx = 1", vec!["a"; n].join(".")), "t").is_err());
            }
            failed
        });
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(handle.join());
        });
        let failed = rx
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("30 秒以内に終わる")
            .expect("パニックしない");
        assert!(failed.iter().all(|&f| f), "{failed:?}");
    }

    #[test]
    fn i64_range_is_handled_by_the_library() {
        assert!(read("n = 9223372036854775807\nm = -9223372036854775808\n", "t").is_ok());
        let e = read("n = 9223372036854775808\n", "t").unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Syntax);
    }
}
