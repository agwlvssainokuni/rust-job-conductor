# ユニットの依存(Unit Dependency): v0.9.0 の機能拡張

出典: `unit-of-work.md`、コンポーネントの一覧(`domain-design/components.md`)。この文書は、何が何に依存するか(トポロジー)だけを記す。実装の順序や、クリティカルパスは、書かない(Delivery Planning が決める)。

## 1. 依存の DAG

- `u3-dry-run` は、`u1-env-injection` と `u2-graph` に依存する。`--dry-run` の計画が、U1 の統合の結果(渡される環境変数の名前)を使い、U2 が公開する辺の判断の入口を使うため(ユニットの質問 Q3 の依存の方針)。
- `u4-init` は、`u1-env-injection` に依存する。ひな形が、U1 の新しいキー(`[pipeline.env]`)の例を含み、`validate` に通る必要があるため。
- `u1-env-injection` と `u2-graph` は、互いに依存しない。
- 循環はない。

```yaml
units:
  - name: u1-env-injection
    kind: service
    depends_on: []
  - name: u2-graph
    kind: service
    depends_on: []
  - name: u3-dry-run
    kind: service
    depends_on: [u1-env-injection, u2-graph]
  - name: u4-init
    kind: service
    depends_on: [u1-env-injection]
```

## 2. 連携の点(Integration points)

| 連携 | 内容 | 提供するユニット | 使うユニット |
|------|------|------------------|--------------|
| 環境変数の統合の結果 | 渡される環境変数の名前(値は含めない)を返す関数 | u1-env-injection | u3-dry-run |
| 辺の判断の入口 | 次のジョブを決める判断から、辺を取り出す `CoreFlow` の公開された関数 | u2-graph | u3-dry-run |
| 定義の新しいキー | `[pipeline.env]`、`[job.env]`、`env_file` | u1-env-injection | u4-init(ひな形の例) |
| 共有する型 | 定義とステートの型(`CoreTypes`) | u1-env-injection(env と env_file の項目) | u2、u3、u4 |
| メッセージのカタログ | 日本語と英語のメッセージ | 各ユニット | 各ユニット(同じファイルに足す) |

## 3. 論理的な依存がない組(トポロジーだけの記述)

- 論理的な依存がない組: `u1-env-injection` と `u2-graph`。`u3-dry-run` と `u4-init` の間にも、依存はない。
- 注意(共有ファイルによる衝突): 4 つのユニットは、同じファイル(`CliArgs`、`CliReport`、`CliFlows`、メッセージのカタログ、`CoreTypes`)を編集する。論理的な依存がなくても、同時に作ると、同じファイルで衝突する。チームは、機能を 1 つずつ順に作る方針(team-practices.md)で、実際に並行して作る予定はない。順序と並行の扱いは、Delivery Planning が決める。
- `u4-init` の `u1-env-injection` への依存は、ひな形の文字列に `[pipeline.env]` の例を入れ、それが `validate` に通ることを、テストで確かめる点だけにある(実行時や型の依存ではない)。ひな形の例を切り出せば、この依存はなくせる。
