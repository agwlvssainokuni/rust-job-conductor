# 設計の決定の記録(ADR): ドメイン設計

出典: `components.md`、ドメイン設計の質問(`domain-design-questions.md`。以下 Q1〜Q4)、要件(`requirements.md`)、プラクティス(`team-practices.md`)。

## ADR-001: `.env` の読み取りと環境変数の層の統合は、core の純粋な部品にする

- **Context** — 要件 FR1.3〜FR1.9 は、`.env` の書式、4 つの層の優先順位、上限、誤りを求める。`graph` と `--dry-run` も、渡される環境変数の名前を使う(FR3.2)。`core` は、定義の取り込み(`intake`)以外でファイルに触れない、という性質を、テストで守っている。
- **Decision** — 書式の検査と層の統合は、`core` の新しい部品 `CoreEnvFile`(文字列と表を受け取る純粋な関数)に置く。ファイルを開いて文字列にする処理は、`runtime` の `RuntimeEnvLoader` に置く。(Q1: A)
- **Consequences** — 良い点: 境界の例を、ファイルなしで細かくテストできる。`graph` と `--dry-run` が、`runtime` に依存せず、同じ統合の結果を使える。`core` の性質を保てる。悪い点: 2 つの部品に分かれ、呼び出す側(`CliFlows`)が、読み取りと統合を順につなぐ。
- **Alternatives Rejected** — B(すべて `runtime`): `graph` と `--dry-run` が、実行の部品に依存する。C(すべて `core`): `core` がファイルに触れる範囲が広がり、今の性質が崩れる。

## ADR-002: `graph` の辺とレーンの配置は、core の純粋な部品にし、整形は cli に置く

- **Context** — 要件 FR2.4 は、図の辺を、実行のときに次のジョブを決める判断と、同じ関数から作ることを求める。`git log --graph` 風のレーンの配置は、純粋な計算である。
- **Decision** — 辺の取り出しとレーンの配置は、`core` の新しい部品 `CoreGraph` に置く(`CoreFlow` の判断を使う)。text と Mermaid の文字列の整形は、`cli` の表示(`CliReport`)に置く。(Q2: A)
- **Consequences** — 良い点: 実行と図が食い違わない。レーンの配置を、定義だけでテストできる。表示の都合(文言、記号)が、`core` に入らない。悪い点: 図の構造の型が、`core` と `cli` の間の契約になる。
- **Alternatives Rejected** — B(整形まで `core`): `core` が表示の都合を持つ。C(すべて `cli`): 辺の判断を作り直し、実行との食い違いが出る。

## ADR-003: `--dry-run` の計画は、core の純粋な部品にする

- **Context** — 要件 FR3.2、FR3.4 は、ステートを読んだ上での、再開位置とジョブごとの扱い、仮定の終了コードによる模擬を求める。これらは、`CoreFlow` の判断(起動前の判断、次の動き)を、組み合わせるだけで決まる。
- **Decision** — 計画を作る処理は、`core` の新しい部品 `CorePlan`(定義、ステートの内容、仮定の終了コードを受け取って、計画を返す純粋な関数)に置く。ステートの読み込みは `runtime`、表示は `cli` に残す。(Q3: A)
- **Consequences** — 良い点: 実行と模擬が、同じ判断から出る。実行の流れに、実行しない分岐を混ぜない。悪い点: 計画の型が、`core` と `cli` の間の契約になる。
- **Alternatives Rejected** — B(`cli` の `pipeline` と `run` に dry-run の分岐を足す): 実行の流れが読みにくくなり、テストもしにくい。

## ADR-004: `init` のひな形の文字列は cli、排他的なファイルの作成は runtime に置く

- **Context** — 要件 FR4.2、FR4.3、FR4.5 は、既存のファイルを壊さない排他的な作成と、ロケールに従うひな形を求める。ファイルに触れる処理は、`runtime` に集める構成である。
- **Decision** — ひな形の文字列は、メッセージと同じ置き場所(`CliReport`)に置く。排他的な作成は、`runtime` の新しい部品 `RuntimeFileCreate` に置く(ステートの書き込みと同じ流儀)。(Q4: A)
- **Consequences** — 良い点: ファイルの操作が `runtime` にまとまる。文字列が、メッセージと同じ場所にまとまる。悪い点: `init` の流れが、`cli` と `runtime` にまたがる。
- **Alternatives Rejected** — B(ひな形も作成も `cli`): ファイルの操作が `cli` に散らばり、責務の置き場所が崩れる。

## ADR-005: 新しい部品は、既存の 3 つのクレートの中の、新しいモジュールにする

- **Context** — 新しい部品が 6 つ(`CoreEnvFile`、`CoreGraph`、`CorePlan`、`RuntimeEnvLoader`、`RuntimeFileCreate`、`InitTemplate`)ある。新しい外部の依存は足さない(project.md の NEVER)。チームは、変更を小さな単位に保つ(project.md の ALWAYS)。
- **Decision** — 新しいクレートは作らず、`conductor-core`、`conductor-runtime`、`conductor-cli` の中の、新しいモジュールとして置く。
- **Consequences** — 良い点: ワークスペースの構成と、リリースの成果物が変わらない。依存の方向(`core` ← `runtime` ← `cli`)を保てる。悪い点: `core` のモジュールが増える(`envfile`、`graph`、`plan`)。
- **Alternatives Rejected** — 新しいクレート(`conductor-graph` など): ワークスペースの設定、CI、リリースの検査(版の一致)に及び、変更が大きくなる。
