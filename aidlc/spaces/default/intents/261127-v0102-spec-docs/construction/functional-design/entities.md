# 実体: v0.10.2 の仕様書の整理

出典: 要件(`requirements.md`、承認済み。受け入れたレビューの指摘 R-01〜R-04 を含む)、設計の質問(`functional-design-questions.md` の Q1・Q2、まとめの確かめ済み)、コードの知識(`code-quality-assessment.md` の「仕様書と実装の差」)、`docs/spec/language-agnostic/11-quality-and-build.md` の AC の体系。refactor の作業で Units とドメインの設計は無い。この作業の対象は文書なので、実体は「仕様書の要素」として表す。コードのデータの形は変えない。

```yaml
entities:
  - name: SpecChapter
    description: 仕様書の 1 つの章(ファイル)。docs/spec/rust/ と docs/spec/language-agnostic/ にそれぞれ 00〜12 の 13 章。
    attributes:
      - name: edition
        type: enum
        allowed: [rust, language-agnostic]
        required: true
      - name: number
        type: integer
        min: 0
        max: 12
        required: true
      - name: version_claim
        type: string
        constraints: 冒頭などの版の記述。v0.10.2 後は「v0.10.2(v0.10.0 の機能を含む)」の形
    relationships:
      - target: SpecStatement
        cardinality: one-to-many
  - name: SpecStatement
    description: 章の中の 1 つの記述(節、表の行、文)。実装との関係で 3 つに分かれる。
    attributes:
      - name: status
        type: enum
        allowed: [matches, missing, contradicts]
        constraints: matches は今の実装と合う、missing は v0.10 の機能の記述が無い、contradicts は実装と逆
      - name: implementation_ref
        type: string
        required: true
        constraints: 根拠になる実装のファイルと行(例 crates/core/src/definition/walk/mod.rs:47-70)
      - name: history
        type: boolean
        default: false
        constraints: その時点の数や形を書いた履歴の記述(例 rust/12 の 105 行の 33 種)。真なら書き換えない
  - name: AcceptanceCriterion
    description: language-agnostic の 11 章の受け入れの条件(AC)の 1 行。
    attributes:
      - name: id
        type: string
        unique: true
        constraints: AC-<番号>。既存は AC-1〜AC-518。新しいものは AC-519 から続けて振る(Q1 A)
      - name: section
        type: string
        constraints: 既存の節(4.1〜4.20)か、新しい節(4.21 以降、見出しに「(v0.10.0 以降)」)
      - name: change_mark
        type: string
        constraints: 既存の AC の中身を書き換えたときだけ「(v0.10.0 で変更)」を文に添える(Q2 A)
      - name: test_names
        type: string
        constraints: 根拠の列のテストの名前。今のテストの名前と一致させる
    relationships:
      - target: SpecStatement
        cardinality: many-to-one
  - name: VerifiableFact
    description: 数で確かめられる事柄(R-03)。仕様書の記述と、実装から数えた値と、その数え方を持つ。
    attributes:
      - name: name
        type: string
      - name: value
        type: string
      - name: source
        type: string
        constraints: 実装のファイルと行、または数えるコマンド
  - name: TeamTestCount
    description: team.md の Testing Posture の「テストの種類」の 1 行の数(FR3)。
    attributes:
      - name: integration_files
        type: string
        constraints: core 36、runtime 16、cli 26、converter 2(tests/ の直下の .rs)
      - name: src_tests_files
        type: string
        constraints: crates/cli/src/tests/ 12、crates/converter/src/tests/ 4(mod.rs と補助を含むすべての .rs)
      - name: test_attributes
        type: integer
        constraints: "#[test] の行の数。1,460(コード生成の時点で数え直す)"
```

## まとめ

- `SpecChapter` と `SpecStatement` で、2 つの仕様書の 26 章の記述を「合う」「無い」「逆」に分ける。「無い」は書き足し、「逆」は書き換え、`history` が真の記述は残す。
- `AcceptanceCriterion` は language-agnostic の 11 章の AC。新しい AC は AC-519 からの続きで新しい節に置き(Q1 A)、実装と逆になった既存の AC は番号を残して書き換える(Q2 A)。
- `VerifiableFact` は、数で確かめる項目の一覧(`functional-spec.md` の 4 章)。
- `TeamTestCount` は `team.md` の 1 行の数。
