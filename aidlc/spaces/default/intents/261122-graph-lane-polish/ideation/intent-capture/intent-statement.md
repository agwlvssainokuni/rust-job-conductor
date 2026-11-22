# 意図の記述(Intent Statement): graph の text 表示の改善

## Problem Statement

- `conductor graph` の text 表示は、2 つの形(同じ行き先へ飛ぶ辺が 2 つある形と、成功と警告のどちらも次のジョブを飛び越す形)では、レーンの図ではなく「辺の一覧」を出す。図として読めない定義が出る。[desc]
- 解決したいのは、この 2 つの形を、レーンの図(`git log --graph` 風の text)で描くことである。[desc]

## Target Customer

- `conductor graph` の text 表示を読んで、パイプラインの流れを確かめる人。[desc]

## Success Metrics

- 2 つの形が、決めた描き方のレーンの図で出る。[Q1]
- ほかの形の出力は変わらない。[Q1]
- README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」の記述が、実装に合う。[Q1]
- 描き方は、形ごとの正解の図を作り、ユーザーが見て承認してから、コードを書く。[Q3]

## Initiative Trigger

- v0.9.0 の `graph` の次の作業として、辺の一覧に落ちる形を減らす。[desc]
- 今回の描き方は `git log --graph` の出力から取れず、こちらで決める。[Q3]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- この版でやらないこと: 交差する飛び越しは扱わない(従来どおり辺の一覧。別の intent)。`--format mermaid` と `--state` の重ね表示の仕様は変えない。[Q2]
- 制約: 新しい外部ライブラリ(依存)を足さない。[memory:M1]

## Assumptions & Open Questions

None.
