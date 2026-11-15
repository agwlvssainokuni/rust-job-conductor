# Cross-Unit Traceability

`aidlc engine sensor-traceability --stage build-and-test` の結果(2026-11-15)。

## 判定: 合格(未対応 0 件)

| ID | 状態 | 担当 | 対応先 |
|----|------|------|--------|
| FR1.1〜FR1.8 | 対応済み | code-generation | `crates/cli/src/`(`args.rs`、`ctx.rs`、`aux_commands.rs`、`lib.rs`、`report/`)とそのテスト |
| FR2 | 対応済み | code-generation | `crates/core/src/types.rs`(`job` フィールド)と、`core`・`runtime`・`cli`・`converter` の追従 |
| FR3.1・FR3.2 | 対応済み | code-generation | `README.md` |
| NFR1〜NFR4 | 対応済み | code-generation | 各行の `traceability.json`(code-generation)のとおり |

未対応の要素は、ない。
