# 性能の設計: u2-optional-env-file

出典: 性能の要件(`performance-requirements.md` の NFR1.3)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3)、契約(`contract-summary.md` の C1)。

## 設計

- `env_file_refs` は今と同じく、見たパスの文字列の集合(`HashSet`)で重複を除く。重複を見つけたときは、一覧の中の位置を引ける表(文字列 → 位置)で required を論理和にする。手間は項目の数に比例する。
- `load_env_files` は、解決したパスでまとめる前処理を 1 回行う。今の「見たパスの並び」を、解決したパス → (required, 読んだ結果) の対応にする。項目の数に比例する手間を保つため、並びの線形探索ではなく表で引く(今の線形探索を表に替えることで、項目が多いときに今より遅くならない)。

## 確かめ方

- `crates/core/tests/env_refs_perf.rs` が、今の上限のまま通る。必要なら、表の項目を混ぜた入力のケースを 1 つ足し、同じ上限で確かめる。
