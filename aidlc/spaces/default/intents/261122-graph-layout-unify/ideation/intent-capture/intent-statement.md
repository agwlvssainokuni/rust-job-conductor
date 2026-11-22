# 意図の記述(Intent Statement): graph の配置のリファクタリング

## Problem Statement

- `conductor graph` の配置(core の `graph/layout.rs`)は、通常の飛び越しだけのグラフ用の「線の並びと寄せ」の仕組みと、「両方の飛び越し」と分岐を含むグラフ用の古い経路(`lane_layout` と、その特例)の、2 つを持つ。[desc]
- 古い経路は、交差と同じ行き先を受けつけないので、「両方の飛び越し」や分岐が、交差や同じ行き先と組み合わさる形は、辺の一覧になる。[desc]
- 解決したいのは、配置を 1 つの仕組みにし、組み合わせの形も図にすることである。[desc] [Q1]

## Target Customer

- `conductor graph` の text 表示を読んで、パイプラインの流れを確かめる人と、この配置のコードを保守する開発者。[desc]

## Success Metrics

- 配置が 1 つの仕組みになり、古い経路(`lane_layout` と、その特例)がなくなる。[Q1] [desc]
- 既存の正解データ(537、1,517、871、940、487 例)の出力は、1 文字も変わらない。[Q1] [desc]
- 「両方の飛び越し」や分岐が、交差・同じ行き先と組み合わさる形が、図になる(設計の段階で承認した代表的な図と、機械的な関係で確かめる)。[Q1] [Q2] [Q3]
- README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の配置と「辺の一覧に切り替わる形」の記述が、1 つの仕組みに合わせて書き直される。[Q1]

## Initiative Trigger

- 一つずつ形を足してきた結果、配置のコードが 2 つの経路に分かれ、同じ規則を別々に持つ状態になった(ユーザーの懸念)。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 「両方の飛び越し」と分岐の、交差と同じ行き先との組み合わせを、すべて図にする(1 つの仕組みで自然に描ける形)。規則が決まらない形は、辺の一覧のまま。[Q2]
- 描き方は、設計の段階で、代表的な形の図を作り、ユーザーが見て承認してから、コードを書く。`--format mermaid`、`--state`、`-v` の仕様は変えない。git のソースは読まない。[Q3] [desc]
- 制約: 新しい外部ライブラリ(依存)を足さない。[memory:M1]

## Assumptions & Open Questions

None.
