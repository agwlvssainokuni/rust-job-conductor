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

//! 図(`conductor graph`)の構造と配置(Unit u2-graph。entities.md の `CoreGraph`)。
//!
//! 定義と(任意の)状態の重ね合わせだけから、図の構造([`FlowGraph`])と、レーンの配置
//! ([`GraphLayout`])を作る。ここにあるものは、すべて純粋な関数で、ファイルにもプロセスにも
//! 環境変数にも触れない。文字への変換(`*`、`|`、`[OK]`、日本語と英語の注記、Mermaid の記法)は、
//! `conductor-cli` が行う。環境変数は、この型のどれにも持たない(FR1.12、BR3.3)。
//!
//! | モジュール | 内容 |
//! |------------|------|
//! | [`edges`] | 辺の取り出し(BR1.1〜BR1.5)。実行と同じ「遷移の解決」を使う |
//! | [`overlay`] | 図の構造と、状態の重ね合わせ(BR2.1、BR4.1、BR4.2) |
//! | [`layout`] | 「対応する形」の判定(BR2.4)と、レーンの配置(BR2.1〜BR2.7) |

pub mod edges;
pub mod layout;
pub mod overlay;

/// 辺が、どの結果のときに通るか。エラーには辺がない(ジョブが止まるため。BR1.3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Success,
    Warning,
}

/// 辺の行き先。最後のジョブが完了したときの行き先も `End`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeTarget {
    /// ジョブ(ID)。
    Job(String),
    /// 終点(`@end`)。
    End,
}

/// 1 つのジョブから出る、1 つの辺(契約 C2 の edges。BR1.1〜BR1.4)。
///
/// 成功と警告の行き先が同じなら、1 つにまとめ、`kinds` に両方を持つ(BR1.2)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEdge {
    /// 元のジョブの ID。
    pub from_job: String,
    /// 行き先。
    pub to: EdgeTarget,
    /// この辺を通る結果(1 つ以上。成功、警告の順)。
    pub kinds: Vec<EdgeKind>,
    /// 元と行き先のあいだにあるジョブの ID(定義の記述順。BR1.4)。行き先が次のジョブなら空。
    pub skipped_jobs: Vec<String>,
    /// 定義に遷移として書かれた辺か。書かずに、記述順の次へ進む辺は `false`。
    pub explicit: bool,
}

/// `--state` で重ねる状態。成功と警告は `Ok` にまとめる(BR4.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Ok,
    Skipped,
    Error,
    Pending,
}

/// `--state` で重ねる入力(BR4.1、BR4.2)。ステートの内容から作る。環境変数も、コマンドも持たない。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StateOverlay {
    /// ジョブごとの状態。
    pub states: Vec<(String, NodeState)>,
    /// 再開位置。完了していないステートで、再開位置があるときだけ。
    pub resume_job_id: Option<String>,
}

/// 図の節。ジョブ 1 つ、または終点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    /// ジョブの ID。終点では `None`。
    pub job_id: Option<String>,
    pub is_end: bool,
    /// `--state` のときだけ。
    pub state: Option<NodeState>,
    /// 再開位置のジョブか。
    pub is_resume: bool,
}

/// 図の構造。節は、定義の記述順で、最後に終点(BR2.1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowGraph {
    pub nodes: Vec<GraphNode>,
    /// 元のジョブの記述順。同じ元の中では、行き先が近い辺が先(BR1.2)。
    pub edges: Vec<FlowEdge>,
}

/// レーンの図の 1 マス(1 文字分)。表示の文字は、`conductor-cli` が決める(BR2.2)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneGlyph {
    /// 空白。
    Empty,
    /// ジョブ(`*`)。
    Node,
    /// 終点(`o`)。
    EndNode,
    /// 線(`|`)。
    Line,
    /// 分岐(`\`)。
    Fork,
    /// 合流(`/`)。
    Join,
}

/// 図の 1 行。ノードの行、または、線をつなぐ行(分岐、合流)。
///
/// ノードの行では、`glyphs` の長さが、ラベル(ジョブ ID、`@end`)を置く位置(BR2.5)。
/// ノードの行以外では、末尾の空白を持たない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneRow {
    /// 左から 1 文字ずつ。
    pub glyphs: Vec<LaneGlyph>,
    /// ノードの行のときだけ。`FlowGraph.nodes` の位置。
    pub node_index: Option<usize>,
}

/// レーンの配置の結果。全ノードが、ちょうど 1 回ずつ、記述順(最後が終点)に現れる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneLayout {
    pub rows: Vec<LaneRow>,
}

/// 図の配置。「対応する形」(BR2.4)ならレーンの図、そうでなければ、辺の一覧(BR2.8)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphLayout {
    Lanes(LaneLayout),
    EdgeList,
}
