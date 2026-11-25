# 信頼性の要件: u1-pipeline-retry

出典: 要件(`requirements.md` の NFR1、NFR2、NFR3、FR1.6)、機能の仕様(`functional-spec.md` の 4、5、7)、ビジネスルール(`rules.md` の BR4.1、BR4.2)、契約(`contract-summary.md` の C3)、技術スタック(`technology-stack.md` の CI)、team.md の Testing Posture。

単体の CLI で、稼働率の目標は当てはまらない。信頼性は、今の定義を壊さないこと(後方互換)と、テストと CI で確かめることで保つ。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.1 | `[pipeline.retry]` を書かない定義は、v0.9.9 と同じ結果になる(組み立てた定義、誤りの種類と行、表示)。変わるのは、ジョブの `retry` が表でも `false` でもないときの理由の文だけ | 既定を書かない定義で、組み立てた定義が今と同じであることを単体テストで確かめる | NFR1、FR1.6、BR4.1 |
| NFR1.2 | 今あるテストはすべて通る。理由の文を変えるテストは、その文の変更だけを直す | `cargo test --workspace` が通る | NFR1 |
| NFR2.1 | test-after で、core の検証の層を作った後に、その層の単体テストを書く。各規則(BR1.1〜BR4.1)に、正しい場合と誤りの場合のテストを 1 つ以上置く | 単体テストが規則ごとにある。テスト名は日本語 | NFR2 |
| NFR2.2 | 実バイナリを動かす統合テストで、既定によるリトライ、`retry = false` での打ち消し、重なりの 64 を確かめる | `validate` と `run` の統合テストが通る | NFR2 |
| NFR2.3 | 行カバレッジ 80% 以上を保つ | CI の `coverage`(`--fail-under-lines 80`)が通る | NFR2 |
| NFR2.4 | テストの都合で本番のコードを曲げない(テスト専用の引数・`pub`・関数を足さない)。検証は今の公開の入口(定義の読み込み)から試す | 本番のコードにテスト専用の項目が無い | NFR2、project.md の Forbidden |
| NFR3.1 | ubuntu・macos・windows の CI と、最小の Rust(1.100)の CI が通る | CI の `test`、`build-check`、`msrv-test` が通る | NFR3 |
| NFR3.2 | 更新したデモ(`demo/retry.toml` と Windows 版)で、デモのテストが通る | `unix_demo.rs`、`windows_demo.rs` が通る | NFR3、BR4.2 |

## 失敗のときの動き

- 定義の誤りは、すべて 64 で止まり、どのジョブも動かない(今のまま)。誤りの途中で一部のジョブだけ動くことは無い。
