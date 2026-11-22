# 意図の記述(Intent Statement): `--dry-run` の模擬の重複をなくすリファクタリング

## Problem Statement

- `crates/core/src/plan/simulate.rs` の `apply_result` が、runtime の `state/record.rs` の `apply_outcome` とほぼ同じ文で、ステートの更新を写している。core は runtime を呼べないため、写しになった。[desc]
- 本番はスキップを `apply_skip` で記録する(そのジョブを `skipped` にし、再開位置と完了を直す)が、模擬はスキップで状態を変えない。模擬に古い再開位置が残るので、まれな形で、後のジョブの判断が本番と違う恐れがある(未確認の見込み。設計の段階の比べる確認で確かめる)。[desc] [Q1]
- 解決したいのは、ステートの更新を core の 1 か所にまとめ、本番と模擬の両方がそれを使うことである。[desc] [Q1] [Q2]

## Target Customer

- このコマンドラインツールのコードを保守する開発者と、`--dry-run` で実行の計画を確かめる利用者。[desc]

## Success Metrics

- ステートの更新の関数(結果の記録と、スキップの記録)が core の 1 か所にあり、runtime(本番)と plan(模擬)がそれを呼ぶ。模擬の写し(`apply_result`)はなくなる。runtime の公開の名前(`apply_outcome`、`apply_skip` など)は、再エクスポートで残す。[Q1] [Q2] [desc]
- 模擬も、スキップのとき、本番と同じ記録を使う。[Q1]
- 既存のテストは変えずにすべて通る。設計の段階で、ランダムな定義とステートで、今の模擬と新しい模擬の `--dry-run` の出力を比べ、違いがあれば、その形を示してユーザーが承認してから進める(承認した形のテストだけを直す)。[Q1] [Q3]
  - 比べる確認の合格の基準: 乱数の種を固定し、`pipeline` と `run` の両方で、各 1 万例以上。スキップの 3 つの理由(遷移、終点の後、実行済み)と、エラーのあとの再開を含む。違いが 0 件、または違う形のすべてをユーザーが承認すること。[Q3]
  - ユーザーが違う形を承認しないときは、Q1 の B(結果の記録だけを共通にし、スキップで状態を変えない今の模擬を残す)に切り替える。[Q1]
- 仕様書(`docs/spec/rust` の実行の流れと `--dry-run` の章)の記述が、共通の更新として書き直される。完了の基準: 更新の規則を 1 回だけ書き、模擬はそれを指す。[Q2]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 模擬の写し(`apply_result`)は、本番の更新とずれないよう手で揃える必要があり、そのことをコードのコメント自身が書いている。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: ステートの更新(結果の記録、スキップの記録)の二重だけ。それ以外の模擬のコードの整理は含めない(変更は小さな単位に保つ)。[desc] [memory:M1]
- 変更は小さな単位に保ち、小さな単位でレビューする。[memory:M1]

## Assumptions & Open Questions

None.
