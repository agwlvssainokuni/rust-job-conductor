# 信頼性の要件: u3-unset-env

出典: 要件(`requirements.md` の FR3.5、NFR1、NFR2、NFR3)、機能の仕様(`functional-spec.md` の 2、3、5)、ビジネスルール(`rules.md` の BR4.2、BR4.3)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)、team.md の Testing Posture。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.1 | `unset_env` を書かない定義は、v0.9.9 と同じに動く(足す変数、起動、表示、誤り) | 今のテストが、型の組み立ての直しを除いて変えずに通る | FR3.5、BR4.2 |
| NFR1.2 | U2 の環境の統合の出力(足す変数)の形を変えず、消す名前を足すだけにする(C1 の「前の形を壊さない」) | `env_merge.rs` の今のテストが変えずに通る | C1 |
| NFR2.1 | test-after。core(検証、統合、重なり)→ runtime(消す)→ cli(差し替え口と表示)の順に、層ごとに作ってからテストを書く | 規則ごとに正しい場合と誤りの場合のテスト | NFR2 |
| NFR2.2 | 実バイナリのテストで、親から消えること、`env` と `.env` の重なりで 64、`--dry-run` の表示を確かめる | `binary_env.rs` に足したテスト | NFR2 |
| NFR2.3 | 行カバレッジ 80% 以上を保つ。テストの都合で本番を曲げない | 確認のコマンド、本番にテスト専用の項目が無い | NFR2、project.md |
| NFR3.1 | 3 つの OS と MSRV の CI が通る。Windows の大文字小文字の扱いを Windows の CI で確かめる | CI | NFR3 |
| NFR3.2 | 更新したデモで、デモのテストが通る | `unix_demo.rs`、`windows_demo.rs` | NFR3、BR4.3 |
