# 規則: v0.10.3 の仕様書の粗の直しと数の検査

出典: 要件(`requirements.md`、承認済み。R-01〜R-06)、設計の質問(`functional-design-questions.md` の Q1 C)、引き渡し(`developer-scan.md` の 2.1・2.2・3・4)、project.md の Forbidden。

```yaml
rules:
  - id: BR1.1
    statement: language-agnostic の 11 章の根拠の列で、テスト名に `_` を残した 14 個(AC-097a、AC-206〜234 の 13 個)を、ほかと同じ空白の書き方に直す。型名・環境変数名・ファイル名などの識別子の `_` は直さない
    category: constraint
    applies_to: docs/spec/language-agnostic/11-quality-and-build.md
    trigger: 作業のとき
    logic: IF 根拠の列に、テストの関数名の `_` を含む書き方が残る THEN 直す
    violation: レビューで差し戻す
    source: FR1.1
  - id: BR1.2
    statement: crates/core/tests/env_types.rs のテスト名の「33 種」を「34 種」に直す。中身は変えない
    category: constraint
    applies_to: crates/core/tests/env_types.rs
    trigger: 作業のとき
    logic: テストの関数名の差分だけ
    violation: レビューで差し戻す
    source: FR1.2
  - id: BR1.3
    statement: AC-430 の前提の `LC_ALL=ja_JP.UTF-8` と `LC_MESSAGES=ja_JP.UTF-8` を、実際のテスト(unix_demo.rs)に合わせて `C` にし、「この前提では、デモがこれらを消したかは表示からは見分けられない」と注記する。rust 版 11 章には AC-430 は無く、unix_demo.rs の説明の行に同じ前提があれば合わせる
    category: constraint
    applies_to: docs/spec/language-agnostic/11-quality-and-build.md、docs/spec/rust/11-quality-and-build.md
    trigger: 作業のとき
    logic: 前提の値がテストの値と一致する
    violation: レビューで差し戻す
    source: FR1.3
  - id: BR1.4
    statement: language-agnostic の 11 章の 2.1 の性能の表で、強制の列が「目安」の 7 行(計画の模擬、図の重ね合わせ、図の辺の取り出し、.env の重複キー、env の表の大文字小文字、遷移の重複、env_file の一覧)は「目安」のまま、各行の列に「参照実装は強制として検査する」を同じ文で書き添える。AC-573 の行(4.16)はすでに書いてあるので、文を表と揃えるだけ
    category: policy
    applies_to: docs/spec/language-agnostic/11-quality-and-build.md
    trigger: 作業のとき
    logic: 7 行すべてに同じ文がある
    violation: レビューで差し戻す
    source: FR1.4
  - id: BR2.1
    statement: team.md の書き換えは、functional-spec.md の 4 章の表の行(行番号の目安と今の文の一部で指す)だけ。数と範囲を書かない古くならない書き方にし、v0.10.0 の intent だけの言い方をどの版にも当てはまる言い方に直す。決まりの中身は変えない。書き換えはコード生成の計画の承認を経て行う
    category: constraint
    applies_to: aidlc/spaces/default/memory/team.md
    trigger: コード生成のとき
    logic: IF 表の外に差分がある、または今の文の一部が見つからない THEN 止めて報告する
    violation: レビューで差し戻す
    source: FR2.1, FR2.2, FR2.3
  - id: BR3.1
    statement: 数の検査のテストは、本番のコードを変えずに、仕様書とソースのファイルを文字列として読む。新しい依存を使わない(正規表現の crate も使わず、文字列の操作で書く)
    category: constraint
    applies_to: crates/cli/tests/spec_numbers.rs
    trigger: テストを書くとき
    logic: IF crates/*/src に差分がある(版を除く)、または Cargo.toml に依存が増える THEN 戻す
    violation: レビューで差し戻す
    source: FR3.1, NFR1, NFR2, NFR4
  - id: BR3.2
    statement: 実装の値は、functional-spec.md の 3.1 の表の取り出し方で、ソースから数える。取り出しに失敗した、または 0 のときは失敗にする
    category: validation
    applies_to: 数の検査のテスト
    trigger: テストのとき
    logic: IF 実装の値が取れない THEN 失敗(どのファイルで取れなかったかを文に出す)
    violation: テストが落ちる
    source: FR3.2, FR3.4
  - id: BR3.3
    statement: 決まった場所(functional-spec.md の 3.1 の表の anchors)は必ず見る。見出しや表が見つからない、数が取れないときは失敗にする。取り方が both のときは、語と表の行の両方が実装と一致しなければ失敗にする
    category: validation
    applies_to: 数の検査のテスト
    trigger: テストのとき
    logic: IF 決まった場所が無い、または数が違う THEN 失敗(数の名前、ファイル、見出し、仕様書の値、実装の値を文に出す)
    violation: テストが落ちる
    source: FR3.2, FR3.4
  - id: BR3.4
    statement: 仕様書の全体を、数ごとの規則(3.1 の pattern)で探し、見つかった数が実装と違えば失敗にする。ただし同じ行に版の注記(3.2)があるものは履歴として除く。全体の探しで 1 件も見つからない数は失敗にする
    category: validation
    applies_to: 数の検査のテスト
    trigger: テストのとき
    logic: IF 注記の無い行で数が違う THEN 失敗(ファイルと行番号と行の一部を文に出す)
    violation: テストが落ちる
    source: FR3.3, FR3.4
  - id: BR3.5
    statement: テストは改行の違い(CRLF)とパスの区切りに依らず、3 つの OS で通る
    category: constraint
    applies_to: 数の検査のテスト
    trigger: テストのとき
    logic: 読んだ文字列の `\r` を除いて行に分ける。パスは Path::join で作る
    violation: CI の windows で落ちる
    source: NFR5
  - id: BR3.6
    statement: テストが効くことを、コード生成の中で一時的に数をずらして確かめる(実装の値の取り出し口、決まった場所、全体の探しのそれぞれで 1 回)。ずらした変更はコミットしない
    category: validation
    applies_to: コード生成の作業
    trigger: テストを書いた後
    logic: それぞれでテストが落ち、文に数の名前とファイルが出ることを code-summary.md に記録する
    violation: 記録が無ければ未完了
    source: FR3.4
  - id: BR4.1
    statement: すべてを片づけてから、版を 0.10.3 にする(10ce84e と同じ 22 ファイルの範囲)。件名は「版を 0.10.3 にする」
    category: policy
    applies_to: 版
    trigger: 最後
    logic: IF 他の変更が残る THEN 版を上げない
    violation: 差し戻す
    source: FR4.1
```

## 規則のまとめ

| ID | 種類 | 要約 | 出典 |
|---|---|---|---|
| BR1.1 | constraint | テスト名の `_` の 14 個を空白に | FR1.1 |
| BR1.2 | constraint | env_types.rs のテスト名を 34 種に | FR1.2 |
| BR1.3 | constraint | AC-430 の LC_ALL と LC_MESSAGES を C に、注記 | FR1.3 |
| BR1.4 | policy | 性能の表の目安の 7 行に「参照実装は強制」 | FR1.4 |
| BR2.1 | constraint | team.md は 4 章の表の行だけ | FR2.1〜FR2.3 |
| BR3.1 | constraint | 本番を変えず、依存を足さず、文字列で読む | FR3.1、NFR1・2・4 |
| BR3.2 | validation | 実装の値が取れなければ失敗 | FR3.2、FR3.4 |
| BR3.3 | validation | 決まった場所は必ず見る | FR3.2、FR3.4 |
| BR3.4 | validation | 全体を探し、注記の無い違いは失敗 | FR3.3、FR3.4 |
| BR3.5 | constraint | 3 つの OS で通る | NFR5 |
| BR3.6 | validation | ずらして落ちることを確かめる | FR3.4 |
| BR4.1 | policy | 最後に 0.10.3 | FR4.1 |
