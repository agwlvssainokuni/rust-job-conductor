# 規則: v0.10.2 の仕様書の整理

出典: 要件(`requirements.md`、承認済み。受け入れたレビューの指摘 R-01〜R-04)、設計の質問(`functional-design-questions.md` の Q1・Q2)、`code-quality-assessment.md`、`docs/spec/language-agnostic/11-quality-and-build.md` の AC の体系(v0.9.0 の 4.17〜4.20 と AC-297・355・395 の書き方)。

```yaml
rules:
  - id: BR1.1
    statement: 仕様書に書く事柄は、実装の該当の行から写す。数・名前・理由の文・表示の文・--help の全文は、実装(主に crates/core/src/definition/walk/、crates/core/src/error.rs、crates/core/src/envfile/、crates/runtime/src/state/reader.rs、crates/runtime/src/error.rs、crates/cli/src/args.rs、crates/cli/src/report/messages.rs、crates/cli/src/report/watch.rs)と一致させ、推測で書かない
    category: constraint
    applies_to: docs/spec/rust/、docs/spec/language-agnostic/
    trigger: 記述を書くとき
    logic: IF 書く事柄に実装の根拠の行が無い THEN 書かずに、差として報告する
    violation: レビューで差し戻す
    source: NFR2
  - id: BR1.2
    statement: v0.10 の機能(要件の FR1 の一覧)を、rust の 00〜08・10〜12 章に、今の章と同じ細かさで書く。09 章は版の記述だけ
    category: policy
    applies_to: docs/spec/rust/
    trigger: 作業のとき
    logic: 要件の FR1.1〜FR1.11 の章ごとの項目をすべて満たす
    violation: レビューで差し戻す
    source: FR1.1, FR1.2, FR1.3, FR1.4, FR1.5, FR1.6, FR1.7, FR1.8, FR1.9, FR1.10, FR1.11
  - id: BR1.3
    statement: 実装と逆のことを書いた記述(status が contradicts)は、書き足しではなく書き換える。対象の行は functional-spec.md の 3 章の一覧で、作業の後にその各行が書き換わったことを確かめる。その時点の形を書いた履歴の記述(history)は残す
    category: constraint
    applies_to: docs/spec/rust/、docs/spec/language-agnostic/
    trigger: 作業のとき
    logic: IF 一覧の行が書き換わっていない THEN 未完了。IF 履歴の記述を書き換えた THEN 戻す
    violation: レビューで差し戻す
    source: FR1.12, NFR3
  - id: BR2.1
    statement: language-agnostic にも、v0.10 の機能を今の章と同じ細かさで、言語に依らない言い方で書く。v0.10.1 の内部の入口は、計画の模擬の結果が実行の判断と一致することと性能の目安として書く
    category: policy
    applies_to: docs/spec/language-agnostic/
    trigger: 作業のとき
    logic: 要件の FR2.1 を満たす。型名やモジュール名に頼らない
    violation: レビューで差し戻す
    source: FR2.1
  - id: BR2.2
    statement: 新しい AC は、11 章に機能ごとの新しい節(4.21 以降、見出しに「(v0.10.0 以降)」)を足し、AC-519 からの続きの番号を振る。既存の AC の番号は変えず、欠番も作らない
    category: constraint
    applies_to: docs/spec/language-agnostic/11-quality-and-build.md、12-rebuild-guide.md の AC の範囲の表
    trigger: AC を足すとき
    logic: 新しい番号は AC-519 から 1 ずつ増やす。12 章の段の表に、新しい AC の範囲を書く
    violation: レビューで差し戻す
    source: FR2.2
  - id: BR2.3
    statement: 実装と逆になった既存の AC は、番号を残して中身を今の実装に書き換え、文に「(v0.10.0 で変更)」を添え、根拠の列のテストの名前を今の名前に直す。対象は functional-spec.md の 3.2 の一覧と、作業の中の洗い出しで見つけたもの
    category: constraint
    applies_to: docs/spec/language-agnostic/11-quality-and-build.md
    trigger: 作業のとき
    logic: IF 書き換えた AC に「(v0.10.0 で変更)」が無い、またはテストの名前が今のテストに無い THEN 直す
    violation: レビューで差し戻す
    source: FR2.2
  - id: BR2.4
    statement: language-agnostic の誤りの数を 34 種(定義)と 14 種(.env)に直し、版の記述を v0.10.2 に合わせる
    category: constraint
    applies_to: docs/spec/language-agnostic/
    trigger: 作業のとき
    logic: functional-spec.md の 4 章の確かめのコマンドで、今の数を指す 33 種・13 種が残らない(履歴を除く)
    violation: レビューで差し戻す
    source: FR2.3, FR2.4
  - id: BR3.1
    statement: team.md の Testing Posture の「テストの種類」の 1 行の数だけを今の値に書き換え、数え方を括弧で添える。他の文と節は変えない。書き換えはコード生成の計画の承認を経て行う
    category: constraint
    applies_to: aidlc/spaces/default/memory/team.md
    trigger: コード生成のとき
    logic: IF その 1 行の外に差分がある THEN 戻す
    violation: レビューで差し戻す
    source: FR3.1, FR3.2
  - id: BR4.1
    statement: crates/runtime/src/lib.rs の「呼び出しの順序」のドキュメントコメントを、v0.10.0 の後の形(試行の開始で Fast で書く、run_job の今の引数、ログ、試行の終わり)に直す。コードは変えない
    category: policy
    applies_to: crates/runtime/src/lib.rs
    trigger: コード生成のとき
    logic: ドキュメントコメントの差分だけで、cargo test と cargo doc が通る
    violation: レビューで差し戻す
    source: FR4.1
  - id: BR5.1
    statement: すべてを片づけてから、版を 0.10.2 にする(v0.10.1 の 152c4a1 と同じ 22 ファイルの範囲)。件名は「版を 0.10.2 にする」の 1 コミット
    category: policy
    applies_to: 4 クレートの Cargo.toml、Cargo.lock、crates/cli/tests/binary_basic.rs、docs/spec/ の版の表記
    trigger: 最後
    logic: IF 他の変更が残っている THEN 版を上げない
    violation: 順序の誤りとして差し戻す
    source: FR5.1
  - id: BR6.1
    statement: crates/ の変更は、ドキュメントコメント(BR4.1)と版(BR5.1)と版の期待のテストだけにする。conductor の動きは変えない
    category: constraint
    applies_to: crates/
    trigger: 作業のとき
    logic: IF crates/ に、ドキュメントコメントと版の外の差分がある THEN 戻す
    violation: 既存のテストが落ちる、またはレビューで差し戻す
    source: NFR1, NFR5
  - id: BR6.2
    statement: 作業は小さな単位でコミットする。1 コミットは、rust の 1 章、language-agnostic の 1 章(11 章は、既存の AC の書き換えと新しい節を別のコミットに分ける)、team.md、lib.rs、版のどれか 1 つ
    category: policy
    applies_to: コード生成の作業
    trigger: コミットのとき
    logic: IF 1 コミットが 2 つ以上の章にまたがる THEN 分ける(版の更新の 1 コミットを除く)
    violation: レビューで差し戻す
    source: NFR4
  - id: BR6.3
    statement: CI(ubuntu・macos・windows の test、msrv-test、lint、coverage、docs-check)が通る
    category: validation
    applies_to: リポジトリ全体
    trigger: push の後
    logic: IF どれかのジョブが落ちる THEN 直す
    violation: Build and Test で不合格
    source: NFR4
```

## 規則のまとめ

| ID | 種類 | 対象 | 要約 | 出典 |
|---|---|---|---|---|
| BR1.1 | constraint | 両仕様書 | 実装の行から写す | NFR2 |
| BR1.2 | policy | rust | v0.10 の機能を章ごとに書く | FR1.1〜FR1.11 |
| BR1.3 | constraint | 両仕様書 | 逆の記述は書き換え、履歴は残す | FR1.12、NFR3 |
| BR2.1 | policy | language-agnostic | 同じ細かさで、言語に依らず書く | FR2.1 |
| BR2.2 | constraint | 11・12 章 | 新しい節に AC-519 から | FR2.2 |
| BR2.3 | constraint | 11 章 | 逆の既存 AC は番号を残して書き換え | FR2.2 |
| BR2.4 | constraint | language-agnostic | 34 種・14 種、版 | FR2.3、FR2.4 |
| BR3.1 | constraint | team.md | 1 行の数だけ | FR3.1、FR3.2 |
| BR4.1 | policy | runtime lib.rs | 呼び出しの順序の文書 | FR4.1 |
| BR5.1 | policy | 版 | 最後に 0.10.2 | FR5.1 |
| BR6.1 | constraint | crates/ | 動きを変えない | NFR1、NFR5 |
| BR6.2 | policy | コミット | 1 章ずつ | NFR4 |
| BR6.3 | validation | CI | すべて通る | NFR4 |
