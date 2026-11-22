# 意図の記述(Intent Statement): graph の text 表示の改善 その 2

## Problem Statement

- `conductor graph` の text 表示は、1 つのジョブの成功と警告が、異なる、どちらも記述順の次でない行き先へ進む形(例: `a` の成功が `c`、警告が `d`)では、レーンの図ではなく「辺の一覧」を出す。図として読めない定義が出る。[desc]
- 解決したいのは、この形をレーンの図(`git log --graph` 風の text)で描くことである。[desc]

## Target Customer

- `conductor graph` の text 表示を読んで、パイプラインの流れを確かめる人。[desc]

## Success Metrics

- この形が、レーンの図で出る。[Q1]
- ほかの形(通常の飛び越し、同じ行き先、両方の飛び越し)の出力は、1 文字も変わらない。[Q1] [desc]
- README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」の記述が、実装に合う。[Q1]

## Initiative Trigger

- 前の作業(同じ行き先と両方の飛び越し)に続く、辺の一覧に落ちる形を減らす作業である。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 描き方は、`git log --graph` の出力を正解データにして決める(1 つのジョブが 2 つの親を持つ形)。設計の段階で、形ごとの正解の図を作り、ユーザーが見て承認してから、コードを書く。[desc] [Q3]
- この版でやらないこと: 交差する飛び越しは扱わない(従来どおり辺の一覧。後回し)。`--format mermaid` と `--state` の重ね表示の仕様は変えない。[Q2]
- 制約: 新しい外部ライブラリ(依存)を足さない。[memory:M1]

## Assumptions & Open Questions

None.
