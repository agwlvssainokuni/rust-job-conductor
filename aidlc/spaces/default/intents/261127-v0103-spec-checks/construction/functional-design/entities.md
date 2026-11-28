# 実体: v0.10.3 の仕様書の粗の直しと数の検査

出典: 要件(`requirements.md`、承認済み。直すと決めたレビューの指摘 R-01〜R-06)、設計の質問(`functional-design-questions.md` の Q1 C、まとめの確かめ済み)、引き渡し(`inception/reverse-engineering/developer-scan.md` の 2.1・2.2)。refactor の作業で Units とドメインの設計は無い。本番のコードのデータの形は変えない。実体は、足すテストが扱う値の形として表す。

```yaml
entities:
  - name: SpecNumber
    description: 検査する 1 つの数(例 定義の誤りの種類)。実装から取り出す値と、仕様書の中の現れ方の規則を持つ。
    attributes:
      - name: name
        type: string
        required: true
        unique: true
      - name: implementation_source
        type: string
        required: true
        constraints: 値を取り出すソースのファイルと、取り出し方(列挙の変種を数える、配列の要素を数える、定数の値を読む、scene の数を数える)
      - name: anchors
        type: list
        required: true
        constraints: 必ず見る決まった場所(仕様書のファイル、見出しや表、取り方=語か表の行か)。1 つ以上
      - name: pattern
        type: string
        required: true
        constraints: 仕様書の全体で、この数を書いた語を探す規則(数の部分を取り出す)。決まった場所の外も見る(Q1 C)
    relationships:
      - target: Anchor
        cardinality: one-to-many
  - name: Anchor
    description: 決まった場所の 1 つ。見つからない、または数が取り出せないときは失敗にする(R-05)。
    attributes:
      - name: file
        type: string
        constraints: docs/spec/rust/ か docs/spec/language-agnostic/ の章のファイル
      - name: locator
        type: string
        constraints: 見出しの文字列か、表の直前の見出し(行番号は使わない。R-02)
      - name: method
        type: enum
        allowed: [word, table_rows, both]
        constraints: 語から数を読むか、表の行を数えるか、両方か(R-01)。両方なら、どちらかが実装と違えば失敗
  - name: HistoryMarker
    description: 版の注記。同じ行にあれば、その行の数は履歴として除く(R-02)。
    attributes:
      - name: patterns
        type: list
        constraints: "v0.x.x まで、v0.x.x 以前、v0.x.x の時点、段 N の時点、v0.x.x までは(行の中のどこでも)"
  - name: TeamMdEdit
    description: team.md の 1 か所の書き換え(FR2)。
    attributes:
      - name: line_hint
        type: integer
        constraints: 今の行番号(目安。R-06)
      - name: current_text
        type: string
        required: true
        constraints: 今の文の一部(対象の識別に使う。R-06)
      - name: new_text
        type: string
        required: true
```

## まとめ

- `SpecNumber` が検査の 1 項目。実装の値はソースを文字列として読んで取り出す(本番は変えない。NFR2)。
- `Anchor` は必ず見る場所で、見つからなければ失敗(空振りしない)。`pattern` は仕様書の全体を見る規則で、`HistoryMarker` のある行は除く(Q1 C)。
- `TeamMdEdit` は team.md の書き換えの 1 か所。行番号と今の文の一部の両方で指す。
