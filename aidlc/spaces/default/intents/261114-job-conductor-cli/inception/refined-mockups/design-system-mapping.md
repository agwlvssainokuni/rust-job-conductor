# Design System Mapping(出力表現の取り決め)

上流: wireframes と user-flow(Rough Mockups)、stories、requirements、team-practices。既存のデザインシステムやコンポーネントライブラリはない(画面を持たない CLI であり、Web UI / GUI は初版の対象外)。そのため、出力の見た目を、ひとまとまりの小さな取り決めとして定義し、すべてのコマンドで揃える。

## 1. 出力の要素

| 要素 | 取り決め | 出典 |
|------|----------|------|
| 状態ラベル | 成功(success)・警告(warning)・スキップ(skipped)・エラー(error)・未実行(pending)を、文字で表す。色だけに頼らない | [Q6]、requirements の NFR6 |
| 色 | 端末に出力するときだけ色を付けてよい。端末以外への出力(ファイルへのリダイレクトなど)では、色を自動で無効にする | requirements の NFR6、Rough Mockups の Q6 |
| 一覧(`watch`) | ジョブID、状態、補足の3列で、1回表示して終了する | [Q6] |
| `conductor` 自身の表示 | 既定では出さない。`-v` / `--verbose` で有効にする | [Q1] |
| `conductor` 自身のエラー | 常に表示する。ファイル名・行・原因など、直す手がかりを含める | requirements の FR2.6、FR1.4 |
| 表示の言語 | 環境のロケールに従う(英語と日本語) | [Q5] |

## 2. 他のコマンドとの整合

- 状態ラベルの語は、`watch`、`-v` の表示、エラーの文言のすべてで同じ語を使う。 [assumption]
- ジョブ本体の標準出力・標準エラーは、加工せずにそのまま通す。 requirements の FR2.6

## 3. 既存のデザインシステムとの対応

- 既存のデザインシステム、ブランドの指針、UI コンポーネントはない。 intent-statement の Initial Scope Signal
- 命名規則は、Rust 標準(関数・変数は snake_case など)とし、独自の規則は設けない。 team-practices

## Assumptions & Open Questions

- 状態ラベルの具体的な綴り(英語・日本語のそれぞれ)と、色の割り当ては、設計で定める。 [assumption]
