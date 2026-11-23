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

//! 素の木(展開前の構造)。YAML と TOML の読み取りの結果を、同じ形で持つ。
//!
//! ノードは、配列(アリーナ)に並べ、子は添字で指す。このため、木を捨てるときも再帰しない。
//! アンカーとエイリアスは、展開せずに、そのまま持つ(エイリアスは、参照先のノードの添字を持つ)。

/// 入れ子の深さの上限(NFR4.4)。最上位のマップを 1 として数え、マップ・配列が 1 段ずつ。
pub(crate) const MAX_DEPTH: usize = 32;

/// 展開後の要素数の上限(NFR4.3)。展開のために訪れたノードを、すべて数える。
pub(crate) const MAX_NODES: usize = 100_000;

/// 展開後のスカラーの文字の総バイト数の上限(NFR4.3 を、メモリの上限としても働かせる。16 MiB)。
///
/// 展開で訪れたすべてのスカラー(キーと値、捨てられるマージの項目、エイリアスの参照のたびの
/// 複製を含む)の文字のバイト数を足し、超えたら `tooLarge`。ノード数の上限(`MAX_NODES`)だけでは、
/// 大きな文字列を何度も参照する入力(数十 KB で数百 MB に展開される)を止められない。
pub(crate) const MAX_EXPANDED_BYTES: usize = 16 * 1024 * 1024;

/// YAML のタグの分類(BR1.6)。標準のタグ(`!!str` など)と、それ以外を分ける。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tag {
    /// タグがない。
    None,
    /// YAML のコアスキーマの標準のタグ。
    Core(CoreTag),
    /// 独自のタグ、または、標準でも対象にしないタグ(`!!binary` など)。
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoreTag {
    Str,
    Int,
    Float,
    Bool,
    Null,
    Map,
    Seq,
    /// コアスキーマにない標準のタグ(`!!binary`、`!!set` など)。
    Unsupported,
}

/// スカラー。YAML のスカラーは、文字列のまま持ち、解決は Resolve で行う(YAML 1.2 コアスキーマ)。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RawScalar {
    /// YAML のスカラー。`plain` は、引用符も `|` `>` もない書き方。
    Yaml {
        text: String,
        plain: bool,
    },
    /// TOML の、解決済みの値。
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    /// TOML の日時(YAML に同じ型がない)。
    Datetime,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RawKind {
    Scalar {
        scalar: RawScalar,
        tag: Tag,
    },
    Seq {
        items: Vec<usize>,
        tag: Tag,
    },
    /// (キーのノード, 値のノード)の並び。入力に現れる順。
    Map {
        pairs: Vec<(usize, usize)>,
        tag: Tag,
    },
    /// エイリアス。参照先のノードの添字。
    Alias(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawNode {
    pub kind: RawKind,
    /// 行番号(1 始まり)。TOML の木では `None`(位置を持たない)。
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawTree {
    pub nodes: Vec<RawNode>,
    pub root: usize,
}

impl RawTree {
    pub(crate) fn line_of(&self, node: usize) -> Option<usize> {
        self.nodes[node].line
    }
}
