# 意図の記述(Intent Statement): graph の text 表示の改善 その 4

## Problem Statement

- `conductor graph` の text 表示は、飛び越しの線が交差する形に、同じ行き先、「両方の飛び越し」、分岐が、グラフのどこかに混ざると、レーンの図ではなく「辺の一覧」を出す。[desc]
- 解決したいのは、この「混ざる形」のうち、範囲(下の Initial Scope Signal)で選んだものを、レーンの図で描くことである。[desc] [Q2]

## Target Customer

- `conductor graph` の text 表示を読んで、パイプラインの流れを確かめる人。[desc]

## Success Metrics

- 範囲の形(交差に、同じ行き先が混ざる形)が、レーンの図で出る。[Q1] [Q2]
- ほかの形(通常の飛び越し、同じ行き先、両方の飛び越し、分岐、交差)の出力は、1 文字も変わらない。[Q1] [desc]
- README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」の記述が、実装に合う。[Q1]

## Initiative Trigger

- 前の作業(交差する飛び越し)に続く、辺の一覧に落ちる形を減らす作業である。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 交差に、同じ行き先(合流)が混ざる形だけ。`git log --graph` の出力を正解データにする。交差に「両方の飛び越し」や分岐が混ざる形と、同じ行き先を含む分岐は、従来どおり辺の一覧(後回し)。[Q2]
- 描き方は、設計の段階で、代表的な形の図を作り、ユーザーが見て承認してから、コードを書く。`--format mermaid`、`--state` の重ね表示、`-v` の書式の仕様は変えない。git のソースは読まない(出力の観察だけから規則を立てる)。[Q3] [desc]
- 制約: 新しい外部ライブラリ(依存)を足さない。[memory:M1]

## Assumptions & Open Questions

None.
