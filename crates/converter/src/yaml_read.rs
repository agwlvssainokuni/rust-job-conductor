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

//! YAML の読み取り(Read)。
//!
//! パーサーのイベントの列を、**再帰を使わず、明示のスタックで**読み、展開しない構造
//! (素の木。アンカーとエイリアスの参照を、そのまま持つ)を作る(D2、NFR4.4、NFR4.7)。
//! この段階で、文法の誤り、重複するキー、再帰するアンカー、展開前の構造の深さ 32 の超過を
//! `syntax`(行つき)として返す。文書の数も数える(最初の文書の木だけを残す)。
//! 深さは、スタックに積むときに数え、33 段目で打ち切る。

use std::collections::{HashMap, HashSet};

use saphyr_parser::{Event, Parser, ScalarStyle};

use crate::error::{ConversionError, ConversionErrorKind, reason};
use crate::raw::{CoreTag, MAX_DEPTH, RawKind, RawNode, RawScalar, RawTree, Tag};
use crate::scalar::{Resolved, yaml_resolved};

/// 読み取りの結果。
#[derive(Debug)]
pub(crate) struct YamlRead {
    /// 文書の数。
    pub documents: usize,
    /// 最初の文書の素の木(文書がなければ `None`)。
    pub tree: Option<RawTree>,
}

/// キーの重複の判定に使う、解決した後の同一性。
#[derive(Debug, Hash, PartialEq, Eq)]
enum KeyId {
    Null,
    Bool(bool),
    Int(i64),
    Float(u64),
    Other(String),
    Str(String),
}

fn key_id(text: &str, plain: bool, tag: Tag) -> KeyId {
    match yaml_resolved(text, plain, tag) {
        Ok(Resolved::Null) => KeyId::Null,
        Ok(Resolved::Bool(b)) => KeyId::Bool(b),
        Ok(Resolved::Int(n)) => KeyId::Int(n),
        Ok(Resolved::Float(f)) => KeyId::Float(if f.is_nan() {
            f64::NAN.to_bits()
        } else {
            f.to_bits()
        }),
        Ok(Resolved::Str(s)) => KeyId::Str(s),
        Ok(Resolved::IntOutOfRange) | Ok(Resolved::FloatOutOfRange) | Err(_) => {
            KeyId::Other(text.to_string())
        }
    }
}

/// 開いているコレクション。
struct Open {
    node: usize,
    anchor: usize,
    is_map: bool,
    /// マップで、キーを受け取って、値を待っているとき、そのキーのノード。
    pending_key: Option<usize>,
    seen: HashSet<KeyId>,
}

#[derive(Default)]
struct Doc {
    nodes: Vec<RawNode>,
    anchors: HashMap<usize, usize>,
    open_anchors: HashSet<usize>,
    stack: Vec<Open>,
    root: Option<usize>,
}

fn syntax(file: &str, line: usize, why: &'static str) -> ConversionError {
    ConversionError::new(file, Some(line), None, ConversionErrorKind::Syntax, why)
}

fn tag_of(tag: &Option<std::borrow::Cow<'_, saphyr_parser::Tag>>) -> Tag {
    match tag {
        None => Tag::None,
        Some(t) if t.is_yaml_core_schema() => Tag::Core(match t.suffix.as_str() {
            "str" => CoreTag::Str,
            "int" => CoreTag::Int,
            "float" => CoreTag::Float,
            "bool" => CoreTag::Bool,
            "null" => CoreTag::Null,
            "map" => CoreTag::Map,
            "seq" => CoreTag::Seq,
            _ => CoreTag::Unsupported,
        }),
        Some(_) => Tag::Other,
    }
}

impl Doc {
    /// 作ったノードを、いまのコレクションの子にする(スタックが空なら、文書の根)。
    fn attach(&mut self, child: usize, file: &str, line: usize) -> Result<(), ConversionError> {
        let (parent, is_map, pending) = match self.stack.last() {
            None => {
                self.root = Some(child);
                return Ok(());
            }
            Some(top) => (top.node, top.is_map, top.pending_key),
        };
        if !is_map {
            if let RawKind::Seq { items, .. } = &mut self.nodes[parent].kind {
                items.push(child);
            }
            return Ok(());
        }
        match pending {
            None => {
                // キーを受け取った。スカラーのキーは、重複を確かめる
                let id = self.scalar_key_id(child);
                if let Some(top) = self.stack.last_mut() {
                    top.pending_key = Some(child);
                    if let Some(id) = id
                        && !top.seen.insert(id)
                    {
                        return Err(syntax(file, line, reason::DUPLICATE_KEY));
                    }
                }
            }
            Some(key) => {
                if let Some(top) = self.stack.last_mut() {
                    top.pending_key = None;
                }
                if let RawKind::Map { pairs, .. } = &mut self.nodes[parent].kind {
                    pairs.push((key, child));
                }
            }
        }
        Ok(())
    }

    /// キーのノード(エイリアスなら参照先)がスカラーのとき、その同一性。
    fn scalar_key_id(&self, node: usize) -> Option<KeyId> {
        let mut n = node;
        if let RawKind::Alias(target) = &self.nodes[n].kind {
            n = *target;
        }
        match &self.nodes[n].kind {
            RawKind::Scalar {
                scalar: RawScalar::Yaml { text, plain },
                tag,
            } => Some(key_id(text, *plain, *tag)),
            _ => None,
        }
    }

    fn push_node(&mut self, kind: RawKind, line: usize) -> usize {
        self.nodes.push(RawNode {
            kind,
            line: Some(line),
        });
        self.nodes.len() - 1
    }
}

/// コレクションを開く。深さは、スタックに積むときに数え、33 段目で打ち切る。
fn open_collection(
    doc: &mut Doc,
    is_map: bool,
    anchor: usize,
    tag: Tag,
    file: &str,
    line: usize,
) -> Result<(), ConversionError> {
    if doc.stack.len() >= MAX_DEPTH {
        return Err(syntax(file, line, reason::TOO_DEEP));
    }
    let kind = if is_map {
        RawKind::Map {
            pairs: Vec::new(),
            tag,
        }
    } else {
        RawKind::Seq {
            items: Vec::new(),
            tag,
        }
    };
    let node = doc.push_node(kind, line);
    if anchor > 0 {
        doc.anchors.insert(anchor, node);
        doc.open_anchors.insert(anchor);
    }
    doc.attach(node, file, line)?;
    doc.stack.push(Open {
        node,
        anchor,
        is_map,
        pending_key: None,
        seen: HashSet::new(),
    });
    Ok(())
}

/// YAML の文字列を、文書の終わりまで読む。
pub(crate) fn read(text: &str, file: &str) -> Result<YamlRead, ConversionError> {
    let mut documents = 0usize;
    let mut first: Option<RawTree> = None;
    let mut doc = Doc::default();

    for item in Parser::new_from_str(text) {
        let (event, span) =
            item.map_err(|e| syntax(file, e.marker().line(), reason::YAML_SYNTAX))?;
        let line = span.start.line();
        match event {
            Event::Nothing | Event::StreamStart => {}
            Event::StreamEnd => break,
            Event::DocumentStart(_) => {
                documents += 1;
                doc = Doc::default();
            }
            Event::DocumentEnd => {
                if documents == 1
                    && let Some(root) = doc.root
                {
                    first = Some(RawTree {
                        nodes: std::mem::take(&mut doc.nodes),
                        root,
                    });
                }
                doc = Doc::default();
            }
            Event::Scalar(value, style, anchor, tag) => {
                let kind = RawKind::Scalar {
                    scalar: RawScalar::Yaml {
                        text: value.into_owned(),
                        plain: style == ScalarStyle::Plain,
                    },
                    tag: tag_of(&tag),
                };
                let node = doc.push_node(kind, line);
                if anchor > 0 {
                    doc.anchors.insert(anchor, node);
                }
                doc.attach(node, file, line)?;
            }
            Event::Alias(id) => {
                if doc.open_anchors.contains(&id) {
                    return Err(syntax(file, line, reason::RECURSIVE_ANCHOR));
                }
                let Some(&target) = doc.anchors.get(&id) else {
                    return Err(syntax(file, line, reason::UNKNOWN_ANCHOR));
                };
                let node = doc.push_node(RawKind::Alias(target), line);
                doc.attach(node, file, line)?;
            }
            Event::SequenceStart(anchor, tag) => {
                open_collection(&mut doc, false, anchor, tag_of(&tag), file, line)?;
            }
            Event::MappingStart(anchor, tag) => {
                open_collection(&mut doc, true, anchor, tag_of(&tag), file, line)?;
            }
            Event::SequenceEnd | Event::MappingEnd => {
                if let Some(open) = doc.stack.pop()
                    && open.anchor > 0
                {
                    doc.open_anchors.remove(&open.anchor);
                }
            }
        }
    }
    Ok(YamlRead {
        documents,
        tree: first,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_of(text: &str) -> (ConversionErrorKind, Option<usize>) {
        let e = read(text, "t.yaml").unwrap_err();
        (e.kind, e.line)
    }

    fn deep_flow(n: usize) -> String {
        format!("{}{}", "[".repeat(n), "]".repeat(n))
    }

    #[test]
    fn 文法の誤りは行を持つ() {
        let (k, line) = kind_of("a: 1\nb: [1, 2\nc: 3\n");
        assert_eq!(k, ConversionErrorKind::Syntax);
        assert!(line.is_some());
    }

    #[test]
    fn 重複するキーは行つきの文法の誤り() {
        let (k, line) = kind_of("a: 1\nb: 2\na: 3\n");
        assert_eq!((k, line), (ConversionErrorKind::Syntax, Some(3)));
        // 引用符の有無が違っても、同じ文字列のキーは重複
        assert_eq!(kind_of("a: 1\n\"a\": 2\n").0, ConversionErrorKind::Syntax);
        // 型が違うキー(整数 1 と文字列 "1")は、重複ではない
        assert!(read("1: a\n\"1\": b\n", "t").is_ok());
        // 入れ子のマップごとに別
        assert!(read("a: {x: 1}\nb: {x: 2}\n", "t").is_ok());
    }

    #[test]
    fn 再帰するアンカーは文法の誤り() {
        let (k, _) = kind_of("a: &r [*r]\n");
        assert_eq!(k, ConversionErrorKind::Syntax);
        let (k, _) = kind_of("a: &r\n  b: *r\n");
        assert_eq!(k, ConversionErrorKind::Syntax);
        // 閉じたあとの参照は、再帰ではない
        assert!(read("a: &r [1]\nb: *r\n", "t").is_ok());
    }

    #[test]
    fn 知らないエイリアスは文法の誤り() {
        assert_eq!(kind_of("a: *nothing\n").0, ConversionErrorKind::Syntax);
    }

    #[test]
    fn 深さ32は受理し_33は文法の誤り() {
        // 最上位のマップを 1 として、32 段(マップ 1 + 配列 31)
        let ok = format!("a: {}", deep_flow(31));
        assert!(read(&ok, "t").is_ok());
        let ng = format!("a: {}", deep_flow(32));
        let e = read(&ng, "t").unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Syntax);
        assert_eq!(e.reason, reason::TOO_DEEP);
        // ブロックの配列(`- - - ...`)でも同じ
        let block = |n: usize| format!("{}x\n", "- ".repeat(n));
        assert!(read(&block(32), "t").is_ok());
        assert_eq!(read(&block(33), "t").unwrap_err().reason, reason::TOO_DEEP);
    }

    #[test]
    fn 文書の数を数え_最初の木を残す() {
        let r = read("a: 1\n", "t").unwrap();
        assert_eq!(r.documents, 1);
        assert!(r.tree.is_some());
        let r = read("---\na: 1\n---\nb: 2\n", "t").unwrap();
        assert_eq!(r.documents, 2);
        let tree = r.tree.unwrap();
        assert!(matches!(tree.nodes[tree.root].kind, RawKind::Map { .. }));
        let r = read("", "t").unwrap();
        assert_eq!(r.documents, 0);
        assert!(r.tree.is_none());
        // 2 つめの文書の文法の誤りも、見つける(文書の終わりまで読む)
        assert_eq!(kind_of("a: 1\n---\nb: [1\n").0, ConversionErrorKind::Syntax);
    }

    #[test]
    fn スカラーは文字と書き方とタグを保つ() {
        let r = read(
            "a: yes\nb: 'x'\nc: !!str 1\nd: !foo 2\ne: !!binary aGk=\n",
            "t",
        )
        .unwrap();
        let tree = r.tree.unwrap();
        let RawKind::Map { pairs, .. } = &tree.nodes[tree.root].kind else {
            panic!("map");
        };
        let value = |i: usize| &tree.nodes[pairs[i].1].kind;
        assert!(matches!(
            value(0),
            RawKind::Scalar { scalar: RawScalar::Yaml { text, plain: true }, tag: Tag::None } if text == "yes"
        ));
        assert!(matches!(
            value(1),
            RawKind::Scalar {
                scalar: RawScalar::Yaml { plain: false, .. },
                ..
            }
        ));
        assert!(matches!(
            value(2),
            RawKind::Scalar {
                tag: Tag::Core(CoreTag::Str),
                ..
            }
        ));
        assert!(matches!(
            value(3),
            RawKind::Scalar {
                tag: Tag::Other,
                ..
            }
        ));
        assert!(matches!(
            value(4),
            RawKind::Scalar {
                tag: Tag::Core(CoreTag::Unsupported),
                ..
            }
        ));
    }

    #[test]
    fn アンカーとエイリアスは展開せずに残す() {
        let r = read("a: &x [1]\nb: *x\n", "t").unwrap();
        let tree = r.tree.unwrap();
        let RawKind::Map { pairs, .. } = &tree.nodes[tree.root].kind else {
            panic!("map");
        };
        assert!(matches!(tree.nodes[pairs[1].1].kind, RawKind::Alias(t) if t == pairs[0].1));
    }
}
