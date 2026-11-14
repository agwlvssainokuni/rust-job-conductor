//! 文書の確認(RootCheck)と、展開と写し取り(Resolve)。
//!
//! 素の木を、**明示のスタックで**(再帰を使わず)、深さ優先・入力に現れる順に、途中の表現
//! `Value` へ写す(D2、NFR4.4、NFR4.7)。この段階で、
//!
//! - アンカー・エイリアス・マージキー(`<<`)の展開(BR1.7)、
//! - 展開のために訪れたノードの数え(上限 100,000。捨てられるマージの項目、同じアンカーを
//!   何度も参照したときの複製も数える)と、展開後の深さ(上限 32)の確認(NFR4.3、NFR4.4)、
//! - 表せない値の最初の1つの記録(BR1.6)
//!
//! を行う。上限の超過は、その時点で打ち切り、記録した表せない値より優先して返す(BR1.12 の 8、9)。
//!
//! 数え方: 展開で訪れたノード(キーのスカラーも含む)を1つずつ数える。エイリアスは、エイリアス
//! 自身ではなく、展開した参照先のノードを数える。マージキー `<<` のキーのノードも数える。

use std::collections::HashSet;

use crate::error::{ConversionError, ConversionErrorKind, PathSeg, format_key_path, reason};
use crate::raw::{CoreTag, MAX_DEPTH, MAX_NODES, RawKind, RawScalar, RawTree, Tag};
use crate::scalar::{Unrep, scalar_value};
use crate::types::Value;
use crate::yaml_read::YamlRead;

/// YAML の読み取りの結果から、最初の文書の素の木を取り出す(RootCheck。BR1.12 の 6、7)。
///
/// 複数の文書は `unrepresentable`、最上位がマップでなければ(空の文書も)`rootNotMap`。
pub(crate) fn root_check(read: YamlRead, file: &str) -> Result<RawTree, ConversionError> {
    if read.documents > 1 {
        return Err(ConversionError::new(
            file,
            None,
            None,
            ConversionErrorKind::Unrepresentable,
            reason::MULTI_DOCUMENT,
        ));
    }
    let root_not_map = |why: &str, line: Option<usize>| {
        ConversionError::new(file, line, None, ConversionErrorKind::RootNotMap, why)
    };
    let Some(tree) = read.tree else {
        return Err(root_not_map(reason::EMPTY_DOCUMENT, None));
    };
    match &tree.nodes[tree.root].kind {
        RawKind::Map { .. } => Ok(tree),
        _ => {
            let line = tree.line_of(tree.root);
            Err(root_not_map(reason::ROOT_NOT_MAP, line))
        }
    }
}

#[derive(Debug, Clone)]
enum Phase {
    /// 次の組を始める前。
    Idle,
    /// キーのノードを読んでいる。
    Key,
    /// キーを受け取った(文字列でなければ `None`)。値に進む前。
    KeyDone(Option<String>),
    /// 値のノードを読んでいる。
    Val(Option<String>),
    /// マージキー `<<` の値のノードを読んでいる。
    MergeVal,
}

enum Frame {
    Seq {
        node: usize,
        next: usize,
        items: Vec<Value>,
    },
    Map {
        node: usize,
        next: usize,
        phase: Phase,
        entries: Vec<(String, Value)>,
        seen: HashSet<String>,
        merges: Vec<Vec<(String, Value)>>,
    },
}

enum Entered {
    Leaf(Value),
    Pushed,
}

enum Next {
    Child(usize),
    Done(Value),
}

struct Walker<'a> {
    tree: &'a RawTree,
    file: &'a str,
    stack: Vec<Frame>,
    count: usize,
    /// 最初に見つけた、表せない値(経路と理由)。
    first_unrep: Option<(Option<String>, &'static str)>,
}

fn is_merge_key(tree: &RawTree, node: usize) -> bool {
    matches!(
        &tree.nodes[node].kind,
        RawKind::Scalar {
            scalar: RawScalar::Yaml { text, plain: true },
            tag: Tag::None,
        } if text == "<<"
    )
}

impl<'a> Walker<'a> {
    fn path(&self) -> Vec<PathSeg> {
        let mut segs = Vec::new();
        for frame in &self.stack {
            match frame {
                Frame::Seq { items, .. } => segs.push(PathSeg::Index(items.len())),
                Frame::Map { phase, .. } => match phase {
                    Phase::Key => {
                        segs.push(PathSeg::NonStringKey);
                        break;
                    }
                    Phase::Val(Some(k)) => segs.push(PathSeg::Key(k.clone())),
                    Phase::Val(None) => segs.push(PathSeg::NonStringKey),
                    Phase::MergeVal => segs.push(PathSeg::Key("<<".to_string())),
                    Phase::Idle | Phase::KeyDone(_) => {}
                },
            }
        }
        segs
    }

    fn in_key(&self) -> bool {
        self.stack.iter().any(|f| {
            matches!(
                f,
                Frame::Map {
                    phase: Phase::Key,
                    ..
                }
            )
        })
    }

    fn error_at(&self, node: usize, kind: ConversionErrorKind, why: &str) -> ConversionError {
        ConversionError::new(
            self.file,
            self.tree.line_of(node),
            format_key_path(&self.path()),
            kind,
            why,
        )
    }

    fn count_node(&mut self, node: usize) -> Result<(), ConversionError> {
        self.count += 1;
        if self.count > MAX_NODES {
            return Err(self.error_at(
                node,
                ConversionErrorKind::TooLarge,
                reason::EXPANDED_TOO_LARGE,
            ));
        }
        Ok(())
    }

    fn record(&mut self, why: &'static str) {
        if self.first_unrep.is_none() {
            self.first_unrep = Some((format_key_path(&self.path()), why));
        }
    }

    fn record_unrep(&mut self, unrep: Unrep) {
        let why = if self.in_key() {
            reason::NON_STRING_KEY
        } else {
            match unrep {
                Unrep::Null => reason::NULL_VALUE,
                Unrep::Tagged => reason::TAGGED,
                Unrep::NumberRange => reason::NUMBER_RANGE,
                Unrep::Datetime => reason::DATETIME,
            }
        };
        self.record(why);
    }

    /// ノードに入る。スカラーは値にし、コレクションはフレームを積む。
    fn enter(&mut self, node: usize) -> Result<Entered, ConversionError> {
        let tree = self.tree;
        let target = match &tree.nodes[node].kind {
            RawKind::Alias(t) => *t,
            _ => node,
        };
        self.count_node(target)?;
        match &tree.nodes[target].kind {
            RawKind::Scalar { scalar, tag } => match scalar_value(scalar, *tag) {
                Ok(v) => Ok(Entered::Leaf(v)),
                Err(u) => {
                    self.record_unrep(u);
                    Ok(Entered::Leaf(Value::Bool(false)))
                }
            },
            RawKind::Seq { tag, .. } => {
                if !matches!(tag, Tag::None | Tag::Core(CoreTag::Seq)) {
                    self.record_unrep(Unrep::Tagged);
                }
                self.push_frame(
                    target,
                    Frame::Seq {
                        node: target,
                        next: 0,
                        items: Vec::new(),
                    },
                )
            }
            RawKind::Map { tag, .. } => {
                if !matches!(tag, Tag::None | Tag::Core(CoreTag::Map)) {
                    self.record_unrep(Unrep::Tagged);
                }
                self.push_frame(
                    target,
                    Frame::Map {
                        node: target,
                        next: 0,
                        phase: Phase::Idle,
                        entries: Vec::new(),
                        seen: HashSet::new(),
                        merges: Vec::new(),
                    },
                )
            }
            // エイリアスの参照先は、実体のノードである(読み取りで保証)。ここへは来ない
            RawKind::Alias(_) => {
                Err(self.error_at(target, ConversionErrorKind::Syntax, reason::SYNTAX))
            }
        }
    }

    fn push_frame(&mut self, node: usize, frame: Frame) -> Result<Entered, ConversionError> {
        if self.stack.len() >= MAX_DEPTH {
            return Err(self.error_at(
                node,
                ConversionErrorKind::Syntax,
                reason::TOO_DEEP_EXPANDED,
            ));
        }
        self.stack.push(frame);
        Ok(Entered::Pushed)
    }

    /// 完了した値を、いちばん内側のフレームに渡す。
    fn deliver(&mut self, value: Value) -> Result<(), ConversionError> {
        let tree = self.tree;
        let Some(top) = self.stack.last_mut() else {
            return Ok(());
        };
        let (node, next, phase) = match top {
            Frame::Seq { items, .. } => {
                items.push(value);
                return Ok(());
            }
            Frame::Map {
                node, next, phase, ..
            } => (*node, *next, phase.clone()),
        };
        let pair = match &tree.nodes[node].kind {
            RawKind::Map { pairs, .. } => pairs.get(next).copied(),
            _ => None,
        };
        match phase {
            Phase::Key => {
                let key = match value {
                    Value::Str(s) => Some(s),
                    _ => None,
                };
                if key.is_none() {
                    // 段階が `Key` のまま記録するため、経路に `(非文字列キー)` が付く
                    self.record(reason::NON_STRING_KEY);
                }
                if let Some(Frame::Map { phase, .. }) = self.stack.last_mut() {
                    *phase = Phase::KeyDone(key);
                }
            }
            Phase::Val(key) => {
                let mut duplicate = false;
                if let Some(Frame::Map {
                    next,
                    phase,
                    entries,
                    seen,
                    ..
                }) = self.stack.last_mut()
                {
                    *next += 1;
                    *phase = Phase::Idle;
                    if let Some(k) = key {
                        if seen.insert(k.clone()) {
                            entries.push((k, value));
                        } else {
                            duplicate = true;
                        }
                    }
                }
                if duplicate {
                    let line = pair.and_then(|(kn, _)| tree.line_of(kn));
                    return Err(ConversionError::new(
                        self.file,
                        line,
                        None,
                        ConversionErrorKind::Syntax,
                        reason::DUPLICATE_KEY,
                    ));
                }
            }
            Phase::MergeVal => {
                let mut maps: Vec<Vec<(String, Value)>> = Vec::new();
                let ok = match value {
                    Value::Map(m) => {
                        maps.push(m);
                        true
                    }
                    Value::Array(list) => {
                        let mut all_maps = true;
                        for item in list {
                            match item {
                                Value::Map(m) => maps.push(m),
                                _ => {
                                    all_maps = false;
                                    break;
                                }
                            }
                        }
                        all_maps
                    }
                    _ => false,
                };
                if !ok {
                    // 段階が `MergeVal` のままなので、経路に `<<` が付く
                    return Err(self.error_at(
                        pair.map_or(node, |(kn, _)| kn),
                        ConversionErrorKind::Unrepresentable,
                        reason::MERGE_NOT_MAP,
                    ));
                }
                if let Some(Frame::Map {
                    next,
                    phase,
                    merges,
                    ..
                }) = self.stack.last_mut()
                {
                    merges.extend(maps);
                    *next += 1;
                    *phase = Phase::Idle;
                }
            }
            Phase::Idle | Phase::KeyDone(_) => {}
        }
        Ok(())
    }

    /// いちばん内側のフレームの、次の子を決める。子がなければ、フレームを閉じて値を返す。
    fn next_child(&mut self) -> Result<Next, ConversionError> {
        let tree = self.tree;
        enum Outcome {
            Child(usize),
            MergeChild { key: usize, value: usize },
            Finish,
            Internal,
        }
        let outcome = match self.stack.last_mut() {
            None => Outcome::Internal,
            Some(Frame::Seq { node, next, .. }) => match &tree.nodes[*node].kind {
                RawKind::Seq { items, .. } if *next < items.len() => {
                    let child = items[*next];
                    *next += 1;
                    Outcome::Child(child)
                }
                _ => Outcome::Finish,
            },
            Some(Frame::Map {
                node, next, phase, ..
            }) => match &tree.nodes[*node].kind {
                RawKind::Map { pairs, .. } => match std::mem::replace(phase, Phase::Idle) {
                    Phase::Idle => match pairs.get(*next) {
                        None => Outcome::Finish,
                        Some(&(k, v)) => {
                            if is_merge_key(tree, k) {
                                *phase = Phase::MergeVal;
                                Outcome::MergeChild { key: k, value: v }
                            } else {
                                *phase = Phase::Key;
                                Outcome::Child(k)
                            }
                        }
                    },
                    Phase::KeyDone(key) => match pairs.get(*next) {
                        Some(&(_, v)) => {
                            *phase = Phase::Val(key);
                            Outcome::Child(v)
                        }
                        None => Outcome::Internal,
                    },
                    other => {
                        *phase = other;
                        Outcome::Internal
                    }
                },
                _ => Outcome::Internal,
            },
        };
        match outcome {
            Outcome::Child(c) => Ok(Next::Child(c)),
            Outcome::MergeChild { key, value } => {
                self.count_node(key)?;
                Ok(Next::Child(value))
            }
            Outcome::Finish => Ok(Next::Done(self.finish_frame())),
            Outcome::Internal => Err(ConversionError::new(
                self.file,
                None,
                None,
                ConversionErrorKind::Syntax,
                reason::SYNTAX,
            )),
        }
    }

    fn finish_frame(&mut self) -> Value {
        match self.stack.pop() {
            Some(Frame::Seq { items, .. }) => Value::Array(items),
            Some(Frame::Map {
                entries,
                mut seen,
                merges,
                ..
            }) => {
                let mut out = entries;
                // 先頭のマップが優先。マップ自身のキーは、マージの内容より優先
                for merged in merges {
                    for (k, v) in merged {
                        if seen.insert(k.clone()) {
                            out.push((k, v));
                        }
                    }
                }
                Value::Map(out)
            }
            None => Value::Map(Vec::new()),
        }
    }

    fn run(&mut self) -> Result<Value, ConversionError> {
        let mut enter = Some(self.tree.root);
        let mut ret: Option<Value> = None;
        loop {
            if let Some(node) = enter.take() {
                if let Entered::Leaf(v) = self.enter(node)? {
                    ret = Some(v);
                }
            }
            if let Some(v) = ret.take() {
                if self.stack.is_empty() {
                    return Ok(v);
                }
                self.deliver(v)?;
            }
            match self.next_child()? {
                Next::Child(c) => enter = Some(c),
                Next::Done(v) => ret = Some(v),
            }
        }
    }
}

/// 素の木を展開して、途中の表現にする(Resolve)。
///
/// 展開の上限の超過(`tooLarge`、`syntax`)は、表せない値(`unrepresentable`)より先に返す。
pub(crate) fn resolve(tree: &RawTree, file: &str) -> Result<Value, ConversionError> {
    let mut walker = Walker {
        tree,
        file,
        stack: Vec::new(),
        count: 0,
        first_unrep: None,
    };
    let value = walker.run()?;
    if let Some((key_path, why)) = walker.first_unrep {
        return Err(ConversionError::new(
            file,
            None,
            key_path,
            ConversionErrorKind::Unrepresentable,
            why,
        ));
    }
    Ok(value)
}
