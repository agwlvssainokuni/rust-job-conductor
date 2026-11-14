# User Stories Assessment

上流: requirements(FR1〜FR7、NFR1〜NFR6)、team-practices(テストの進め方・Given/When/Then の方針)。

## Decision

**Execute**(User Stories を実施する)。

## Rationale

- 利用者が複数いる。既存の直列ジョブスケジューラ上でバッチジョブを運用する、組織内の運用担当者・開発者と、ローカルでパイプラインをデバッグ・テストする開発者である。(intent-statement の Target Customer)
- 遷移ルール、スキップ、再開、ステート管理など、振る舞いの条件が多く、受け入れ基準を Given/When/Then で個別に確認できる形にする価値が高い。(requirements の FR1〜FR4)
- 画面を持たない CLI だが、利用者が実際に呼び出して得る結果(終了コード、スキップ、再開)が価値の中心であり、利用者視点のストーリーに向いている。(requirements)

## Factors Considered

- プロジェクトの種類: 新規の単一バイナリ CLI(greenfield)。
- 利用者に直接見える範囲: コマンドの実行結果、終了コード、表示。Web UI / GUI は対象外。(intent-statement)
- 複雑さの兆候: 結果の分類、遷移、スキップ、再開、定義変更の検知など、状態を持つ振る舞いが多い。

## Where Stories Add the Most Value

- 個別実行モードでの分岐・スキップ・再開(FR2、FR3、FR4)。スケジューラ側から見ると終了コードが 0 / 非0 の2値のみであるため、境界条件を受け入れ基準で明確にする必要がある。
- 定義ファイルの検証(FR1)。実行前に誤りを拒否する振る舞いが多数ある。
- 補助コマンド(`watch`、`reset`、`convert`)。これらは独立して提供でき、優先度(Must / Should)が分かれる。

## Assumptions & Open Questions

None.
