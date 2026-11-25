# 機能の設計の質問: u4-graph-dot(`graph --format dot`)

出典: ユニットの定義(`unit-of-work.md` の U4)、要件(`requirements.md` の FR4.1〜FR4.4、7 章の未決の事項)、契約(`contract-summary.md` の C5)、設計の決定(ADR-006)、今のコード(`crates/cli/src/report/graph_mermaid.rs`、`graph_text.rs`)。

すでに決まっていること(質問しない): DOT は text・mermaid と同じ構造(`FlowGraph`)から描く(ADR-006)。出力は標準出力に `digraph` を 1 回で書く(C5)。ジョブの数と辺の数は text・mermaid と一致する(FR4.1)。値は DOT の文字列として escape する(FR4.3)。`--format` の値が `text`・`mermaid`・`dot` 以外なら今と同じ使い方の誤り(FR4.4)。節の名前は mermaid と同じく連番(`n0`、`n1`…、終点は `nend`)にしてジョブの ID はラベルに書き、主線の辺はラベルなし、それ以外の辺は種類(成功・警告)をラベルにする。上から下へ並べる(mermaid の `TD` と同じ)。

## Q1. [利用者] `--state` のとき、ジョブの状態を DOT でどう見分けるか

要件のレビューで「状態をどの属性で表すか」が未決になっています(R-04)。今の text と mermaid は、ラベルの先頭に記号(`[OK] `、`[NG] `、`[--] `、`[  ] `)を付け、再開位置は mermaid では太い線で示しています。

A. mermaid と同じ: ラベルの先頭に同じ記号を付け、再開位置は太い枠(`penwidth=3`)。色は使わない
B. A に加えて、状態ごとに塗りの色を付ける(例: OK は薄い緑、NG は薄い赤、飛ばしは灰色)。白黒でも記号で見分けられる
C. 記号は付けず、色だけで見分ける
X. Other (please specify)

[Answer]: A

## Q2. [利用者] `-v` のとき、DOT に何を出すか

要件は「`-v` と `--state` に、text・mermaid と同じく対応する」としています(FR4.2)。ところが今は、text の `-v` は各ジョブのコマンドとリトライの要約を出すのに対し、mermaid は `-v` を無視しています(図の中に詳細を入れない)。

A. mermaid と同じく `-v` は影響しない(図は同じ。`-v` を付けても誤りにはしない)
B. text の `-v` と同じ情報(コマンドとリトライの要約)を、各ジョブの節のラベルに複数行で足す
C. text の `-v` と同じ情報を、各ジョブの節の `tooltip` 属性に入れる(図の見た目は変えず、SVG などで見られる)
X. Other (please specify)

[Answer]: A
