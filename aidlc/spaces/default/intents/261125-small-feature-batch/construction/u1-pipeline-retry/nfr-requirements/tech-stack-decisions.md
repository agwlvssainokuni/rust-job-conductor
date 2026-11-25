# 技術の選択: u1-pipeline-retry

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の NFR4、NFR7)、機能の仕様(`functional-spec.md` の 1)、ビジネスルール(`rules.md` の BR2.2)、契約(`contract-summary.md` の C3)、team.md の Code Style。

新しい技術は選ばない。今の技術スタックのまま作る。

| 項目 | 選択 | 理由 |
|---|---|---|
| 言語 | Rust(edition 2024、MSRV 1.100) | 今のまま |
| 定義の読み取り | 既存の `toml`(位置つきの木 `DeTable`) | `[pipeline.retry]` と `retry = false` は、今の木の走査(`walk`)で読める。新しい依存は要らない(NFR4.1) |
| 置き場所 | core の `definition/walk`(`retry` の子モジュール)と、`types.rs` の文書 | 検証は core の責務。型の形は変えない(`entities.md`) |
| 誤りの種類 | 既存の `DefinitionErrorKind`(`RetryNotTable`、`RetryExitCodesOverlap` など)を使い、新しい種類を足さない | 種類の文がそのまま当てはまり、既定を書かない定義の表示を変えずに済む(BR4.1) |
| テスト | 各モジュールの `mod tests` と、cli の `tests/` の実バイナリのテスト、`cargo-llvm-cov` | team.md の Testing Posture |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` が通る | NFR7 |
