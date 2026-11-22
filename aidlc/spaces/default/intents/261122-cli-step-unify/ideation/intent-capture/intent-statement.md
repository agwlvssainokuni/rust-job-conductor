# 意図の記述(Intent Statement): run と pipeline の重複をなくすリファクタリング

## Problem Statement

- `crates/cli/src/run.rs` と `pipeline.rs` では、1 つのジョブを実行する手順(ステートの取り出し、スキップの記録、初期ステートの書き込み、リトライつきの実行、結果の分類、記録)が、ほぼ同じ文で 2 か所にある。[desc]
- 機能を足すたびに、2 か所を揃えて直す必要があり、ずれる恐れがある。[desc]
- 解決したいのは、この手順を 1 つの関数にまとめ、`pipeline` にはループと中断の扱いだけを残すことである。動きは変えない。[desc] [Q1]

## Target Customer

- このコマンドラインツールのコードを保守し、機能を足す開発者。[desc]

## Success Metrics

- 1 つのジョブを処理する手順が 1 つの関数にまとまり、`run` と `pipeline` はそれを呼ぶだけになる(`pipeline` にはジョブのループと、ジョブのあいだの中断の扱いだけが残る)。[Q1] [desc]
- 利用者から見える動き(出力、終了コード、ステートの書き込みの順序と内容)は 1 文字も変わらない。既存のテストを変えずに、すべて通る。[Q1] [Q3] [desc]
- 仕様書(`docs/spec/rust` の実行の流れの章)の run と pipeline の手順の記述が、共通の手順として書き直される。完了の基準: その章で、run と pipeline の手順が共通の手順を指し、同じ手順を 2 回書いていない。[Q1]
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、行カバレッジ 80% 以上、MSRV 1.100 でのテストが通る(チームの決まり)。[memory:M1] [desc]

## Initiative Trigger

- 機能を順に足してきた跡を調べた結果、run と pipeline の重複が、最も効果の大きい候補だった(ユーザーが、小さな refactor の intent に分けて、ここから始めると決めた)。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 共通の手順に含めるもの: ステートの取り出し、スキップの記録、初期ステートの書き込み、リトライつきの実行、結果の分類と報告、記録、完了の判断(すべて終わったらステートを消す)。`--dry-run`(模擬)は範囲の外。[desc] [Q2]
- 範囲: run と pipeline の重複だけ。`--dry-run` の模擬のステートの更新、run ID の解決とステートの一致の検査などの、ほかの重複は別の intent にする。[Q2]
- 確かめ方: 既存のテストを変えずに、すべて通ることで確かめる。テストは、共通の関数の単体のテストを最小限足すだけ。[Q3]
- 変更は小さな単位に保ち、小さな単位でレビューする。[memory:M1]

## Assumptions & Open Questions

None.
