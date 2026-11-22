# 意図の記述(Intent Statement): graph の text 表示の改善 その 3

## Problem Statement

- `conductor graph` の text 表示は、飛び越しの線が交差する形(例: `a` の成功が `d`、`b` の成功が `e`)では、レーンの図ではなく「辺の一覧」を出す。[desc]
- 解決したいのは、この形を、`git log --graph` 風のレーンの図(線が交差する図)で描くことである。[desc]

## Target Customer

- `conductor graph` の text 表示を読んで、パイプラインの流れを確かめる人。[desc]

## Success Metrics

- 交差する形が、`git log --graph` と同じ見た目の図で出る。[Q1] [desc]
- ほかの形(通常の飛び越し、同じ行き先、両方の飛び越し、分岐)の出力は、1 文字も変わらない。[Q1] [desc]
- README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」の記述が、実装に合う。[Q1]

## Initiative Trigger

- 前の作業(同じ行き先、両方の飛び越し、分岐)に続く、辺の一覧に落ちる形を減らす作業である。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 通常の飛び越し同士の交差(各ジョブの飛び越しは高々 1 本。行き先が違う)だけ。`git log --graph` の出力を正解データにする。「両方の飛び越し」、分岐、同じ行き先を含む交差は、従来どおり辺の一覧(後回し)。[Q2]
- 描き方は、設計の段階で、git の出力から代表的な形の図を作り、ユーザーが見て承認してから、コードを書く。`--format mermaid`、`--state` の重ね表示、`-v` の書式の仕様は変えない。[Q3]
- 制約: 新しい外部ライブラリ(依存)を足さない。[memory:M1]

## Assumptions & Open Questions

None.
