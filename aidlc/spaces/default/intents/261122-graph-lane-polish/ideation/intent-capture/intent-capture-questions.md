# 意図の把握の質問(Intent Capture Questions): graph の text 表示の改善

## Sources

- [desc] Initial description: "conductor graph の text 表示の改善(v0.9.0 の次): 辺の一覧に落ちる 2 つの形をレーンの図で描く。(1) 同じ行き先へ飛ぶ辺が 2 つ: 合流の / を新しい * の行にせず、既にある | の線に合流させる。(2) 成功と警告のどちらも次のジョブを飛び越す: 上下の | なしで * の行のみで表す。交差する飛び越しは対象外(別の intent)で、従来どおり辺の一覧のまま。新しい依存は足さない。決定事項の詳細は、メモリの project_graph_text_next_intent.md を参照。"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 何が出来上がれば「できた」と言えるか

背景: 今は 2 つの形(同じ行き先へ飛ぶ辺が 2 つ、成功と警告のどちらも次のジョブを飛び越す)で、レーンの図の代わりに「辺の一覧」が出ます。完了の基準によって、テストと文書の範囲が決まります。

- A. 2 つの形が、決めた描き方のレーンの図で出る。ほかの形の出力は、1 文字も変わらない
- B. A に加えて、README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」の記述も、実装に合わせて直す
- C. 今は決められない
- X. Other (please specify)

[Answer]: B

## Q2. [プロダクト] この版で、やらないこと

背景: 交差する飛び越しは、別の intent にすると決めました(説明に明記)。それ以外に、この版で扱わないものがあるかを確かめます。

- A. 交差する飛び越しだけを扱わない(従来どおり辺の一覧)。`--format mermaid` と `--state` の重ね表示の仕様は変えない
- B. A に加えて、`-v` の詳細の行の書式も変えない
- C. 特にない(説明のとおり)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 描き方の決め方

背景: 同じ行き先と両方の飛び越しは、`git log --graph` の出力では取れない描き方(こちらで決める形)です。機能の設計で、描き方の例を作って確かめます。

- A. 設計の段階で、正解の例(形ごとの図)を作り、ユーザーが見て承認してから、コードを書く
- B. 例は作らず、説明の記述どおりに実装し、結果の図を最後に見せる
- X. Other (please specify)

[Answer]: A
