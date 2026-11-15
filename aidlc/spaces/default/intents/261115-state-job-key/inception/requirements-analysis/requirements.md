# Requirements(state-job-key)

## Sources

- Initial description: ステートファイル(.state)のキーを [[jobs]] から [[job]] に改名し、定義ファイルの [[job]] と揃える。ステートの format_version は 1 のまま上げない。古い形式(jobs)の読み込みの互換は持たない(まだ使われていないため、破壊的変更でよい)。 [desc]
- Workflow-selected scope: express [scope]
- 既存コードの調査(`aidlc/spaces/default/codekb/rust-job-conductor/`、この intent の focused scan): ステートの `[[jobs]]` は `conductor-runtime` に閉じている(`state/reader.rs` の `TOP_KEYS` と `walk`、`state/writer.rs` の `serialize`、エラー文面、テストの TOML)。`consistency.rs` と `record.rs` の `.jobs` は、定義側のフィールドで、ステートのキーとは別。README には、ステートのキーの書式の記述はない。 [memory:M1]
- 依頼のやり取り: 「揃えるべき」「まだ全然使われていないので破壊的変更はノープロブレム」「format_version を上げる必要もない」。質問は不要(依頼が、すべての判断を決めている)。

## Intent analysis

定義ファイルのジョブのテーブルは `[[job]]`(単数)だが、ステートファイルは `[[jobs]]`(複数)で、同じ「ジョブの列」を、2 つのファイルで違うキー名で呼んでいる。ステートのキーを `job` に揃える。まだ使われていないため、古い形式との互換は持たず、`format_version` も上げない。

## Functional requirements

- **FR1 書き出し** ステートファイルは、ジョブごとの記録を、`[[job]]` のテーブルの配列として書く。`[[jobs]]` は書かない。
  - 受け入れ基準: ステートを書いて、できたファイルを読むと、`[[job]]` のテーブルがジョブの数だけあり、`jobs` というキーはない。書いて読み戻した値は、書く前と等しい。
- **FR2 読み込み** ステートファイルは、`[[job]]` から読む。古い形式(`[[jobs]]`)は受け付けない。互換の読み込みはしない。
  - 受け入れ基準: `[[job]]` を持つ正しいステートは読める。`[[jobs]]` を持つステートは、壊れたステート(`corrupt`、終了コード 65)として拒否される。
- **FR3 書式の版** ステートの `format_version` は 1 のまま。
  - 受け入れ基準: 書き出したステートの `format_version` は 1。`format_version = 1` で、`[[job]]` のステートが読める。
- **FR4 メッセージ** ステートの誤りの文面で、ステートのキー `jobs` に触れているものは、`job` に直す。
  - 受け入れ基準: ステートの誤りの文面に、ステートのキーとしての「jobs」が残らない(固定の文だけで、入力の値を含まない原則は、変えない)。
- **FR5 文書** README とデモに、ステートのキーとしての `[[jobs]]` の記述があれば、`[[job]]` に直す。なければ、変更しない。
  - 受け入れ基準: README とデモに、ステートのキーとしての `jobs` の記述が残っていない。

## Non-functional requirements

- **NFR1 範囲の限定** 変更は、ステートファイルのキー名だけ。定義ファイルの形式、`conductor-core` の型(`JobState`、`PipelineState`、`job_states` などの Rust のフィールド名)、`conductor-cli` の表示、`convert` の動作は、変えない。
- **NFR2 品質** 行カバレッジ 80% 以上を保つ。既存のテストは、ステートのキーに触れるもの(TOML のリテラルとキー列)の修正を除き、そのまま成功する。`cargo fmt --check`、`cargo clippy -D warnings`、Rust 1.85 でのビルドとテスト、3 OS の CI が成功する。
- **NFR3 版** 利用者に見える形式が変わる(破壊的変更)ため、`Cargo.toml` の `version` を 0.3.0 に上げる。[assumption]

## Constraints

- Rust 1.85 で、ビルドできること(let チェーンなど、1.88 以降の構文は使わない)。
- ライブラリの追加はしない。
- チームの実践(test-after、`rustfmt`、`clippy`、`main` への取り込みで履歴を残す)に従う。

## Assumptions

- 版を 0.3.0 に上げる(0.x の破壊的変更は、マイナーを上げる)。タグ付けと公開は、この作業の範囲外(あとの Deployment Execution で、人が決める)。[assumption]
- 古い形式のステートは、専用のエラー文を持たず、既存の「壊れたステート」(未知のキー)として拒否される(調査からの推定。テストで確かめる)。[assumption]

## Out of scope

- 古い形式(`jobs`)の読み込みの互換。`format_version` の引き上げ。
- Rust の型・フィールド名の変更。定義ファイルの形式の変更。

## Open questions

None.

## Assumptions & Open Questions

None.
