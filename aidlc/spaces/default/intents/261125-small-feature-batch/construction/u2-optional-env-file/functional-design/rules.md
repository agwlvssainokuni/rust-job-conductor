# ビジネスルール: u2-optional-env-file(無くてもよい `.env`)

出典: 要件(`requirements.md` の FR2.1〜FR2.5、FR8.1、NFR1、NFR6)、ユニットの定義(`unit-of-work.md` の U2)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md` の CoreDefinition、CoreEnvFile、RuntimeEnvFile)、契約(`contract-summary.md` の C1、C3)、機能の設計の質問(`functional-design-questions.md` の Q1、Q2)。規則の番号はこのユニットの中のもの。

```yaml
rules:
  - id: BR1.1
    statement: "`env_file` の項目は、文字列か表 `{ path, required }`。1 つでも配列でも書け、配列には文字列と表を混ぜて書ける"
    category: validation
    applies_to: EnvFileRef
    trigger: 定義の検証で `[pipeline]` と `[job]` の `env_file` を読むとき
    logic: >
      IF 文字列 THEN EnvFileRef(path = 文字列, required = true)。IF 表 THEN BR1.2 で読む。
      IF 配列 THEN 各要素を同じく読む。ELSE 型の誤り(TypeMismatch)、理由
      「`env_file` は、文字列かテーブル、またはその配列で書きます」。配列の要素が文字列でも表でもなければ、
      理由「`env_file` の配列の要素は、文字列かテーブルで書きます」
    violation: 定義の誤り(64)
    source: FR2.1

  - id: BR1.2
    statement: "表の `path` は必須で空でない文字列、`required` は省略可の真偽。ほかのキーは書けない"
    category: validation
    applies_to: EnvFileRef
    trigger: BR1.1 で表を読むとき
    logic: >
      書けるキーは path と required だけ(ほかは UnknownKey)。path が無い・文字列でない → TypeMismatch、
      空の文字列 → EmptyField(今の文字列の項目と同じ理由の文)。required が真偽でない → TypeMismatch
    violation: 定義の誤り(64)。理由の文に値を含めない
    source: FR2.1

  - id: BR1.3
    statement: "`required` を省いた表と、文字列は、必ず要るファイルとして扱う"
    category: policy
    applies_to: EnvFileRef.required
    trigger: 定義の組み立て
    logic: IF 文字列 OR 表で required を省いた THEN required = true
    violation: 該当なし
    source: FR2.4

  - id: BR2.1
    statement: 同じパスの文字列を複数の場所に書いたときは、1 つにまとめ、どれか 1 つでも必ず要るなら必ず要る
    category: calculation
    applies_to: EnvFileTarget
    trigger: 読む対象の一覧を作るとき(`env_file_refs`)
    logic: >
      読む範囲(今のまま: `run` は `[pipeline]` とそのジョブ、`pipeline`・`validate`・`--dry-run`・`graph` は
      `[pipeline]` とすべてのジョブ)の EnvFileRef を書いた順に集め、パスの文字列で重複を除く。
      除くときは required を論理和にする
    violation: 該当なし
    source: FR2.2, FR2.4

  - id: BR2.2
    statement: 解決したパスが同じになる項目も、どれか 1 つでも必ず要るなら必ず要る
    category: calculation
    applies_to: 読み取り(RuntimeEnvFile)
    trigger: "`.env` を読むとき"
    logic: >
      別の文字列(例 `a.env` と `./a.env`)が同じパスに解決されるときは、今のまま 1 回だけ読む。
      無いかどうかの扱いは、そのパスに解決されるすべての項目の required の論理和で決める
    violation: 該当なし
    source: FR2.2, FR2.4

  - id: BR2.3
    statement: 必ず要らないファイルが無ければ、読まずに進み、何も表示しない
    category: policy
    applies_to: 読み取り(RuntimeEnvFile)
    trigger: "`.env` を読むとき"
    logic: >
      IF パスが無い(今の FileMissing に当たる場合)AND required = false THEN 誤りにしない。
      その文字列を読み取りの結果の引き当てに載せない(統合で何も足さない)。
      標準出力・標準エラーに何も足さず、終了コードも変えない
    violation: 該当なし
    source: FR2.2

  - id: BR2.4
    statement: 必ず要らないファイルでも、あって読めない・通常のファイルでない・大きすぎる・UTF-8 でない・書式の誤りは、今までどおり 64
    category: validation
    applies_to: 読み取り(RuntimeEnvFile)
    trigger: "`.env` を読むとき"
    logic: IF パスが無い以外の失敗 THEN required によらず、今と同じ種類の誤り(Unreadable、TooLarge、NotUtf8、書式の誤り)
    violation: ".env の誤り(64)"
    source: FR2.3

  - id: BR2.5
    statement: 必ず要るファイルが無ければ、今までどおり 64
    category: validation
    applies_to: 読み取り(RuntimeEnvFile)
    trigger: "`.env` を読むとき"
    logic: IF パスが無い AND required = true THEN FileMissing(今のまま)
    violation: ".env の誤り(64)"
    source: FR2.4

  - id: BR3.1
    statement: "`run`・`pipeline`・`validate`・`--dry-run`・`graph` は同じ読み取りを使い、無くてもよいファイルが無くても誤りにしない"
    category: policy
    applies_to: cli の流れ
    trigger: 各コマンドが `.env` を読むとき
    logic: >
      どのコマンドも、今の 1 つの入口(読む対象の一覧 → 読み取り)を通る。`--dry-run` の環境変数の名前の表示は、
      読めたファイルの名前だけになる(無いファイルの分は出ない。特別な印は付けない)
    violation: 該当なし
    source: FR2.5

  - id: BR4.1
    statement: "表の `env_file` を書かない定義は、v0.9.9 と同じに動く(読み取り、誤り、表示)"
    category: constraint
    applies_to: 全体
    trigger: すべての場面
    logic: >
      文字列だけの `env_file` は required = true の EnvFileRef になり、読み取りの結果は今と同じ。
      変わるのは、`env_file` の型の誤りの理由の文に「テーブル」が入ることだけ
    violation: 該当なし
    source: FR2.4

  - id: BR4.2
    statement: README とデモを更新する。デモには置いていないファイルを無くてもよいものとして足す
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: >
      README の `env_file` の説明に表の形と required、無いときの扱い、読めないときは 64 を足す。
      `demo/env.toml` と Windows 版の `env_file` に `{ path = "local.env", required = false }` を足す(ファイルは置かない)。
      デモの出力は変わらない
    violation: 該当なし
    source: FR8.1
```

## まとめ

| ID | 規則 | 種類 | 出典 |
|---|---|---|---|
| BR1.1 | 項目は文字列か表、混ぜて書ける | validation | FR2.1 |
| BR1.2 | 表は path 必須、required 省略可、ほかのキーは不可 | validation | FR2.1 |
| BR1.3 | 文字列と、required を省いた表は必ず要る | policy | FR2.4 |
| BR2.1 | 同じ文字列は 1 つにまとめ、required は論理和 | calculation | FR2.2、FR2.4 |
| BR2.2 | 同じパスに解決されるものも required は論理和 | calculation | FR2.2、FR2.4 |
| BR2.3 | 必ず要らないファイルが無ければ、黙って進む | policy | FR2.2 |
| BR2.4 | あるが読めない等は required によらず 64 | validation | FR2.3 |
| BR2.5 | 必ず要るファイルが無ければ 64 | validation | FR2.4 |
| BR3.1 | 5 つのコマンドで同じ扱い | policy | FR2.5 |
| BR4.1 | 表を書かない定義は今と同じ | constraint | FR2.4 |
| BR4.2 | README とデモ | policy | FR8.1 |
