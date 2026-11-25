# 信頼性の要件: u2-optional-env-file

出典: 要件(`requirements.md` の NFR1、NFR2、NFR3、FR2.4、FR2.5)、機能の仕様(`functional-spec.md` の 4、5、7)、ビジネスルール(`rules.md` の BR4.1、BR4.2、BR3.1)、契約(`contract-summary.md` の C1、C3)、技術スタック(`technology-stack.md`)、team.md の Testing Posture。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.1 | 文字列だけの `env_file` の定義は、v0.9.9 と同じに読み、同じに動く。変わるのは `env_file` の型の誤りの理由の文だけ | 今の `env_definition.rs`、`env_merge.rs`、`flow_env.rs`、`binary_env.rs` が、理由の文を見るものを除いて変えずに通る | NFR1、FR2.4、BR4.1 |
| NFR1.2 | 今あるテストはすべて通る。型を `String` から EnvFileRef に変えたことで書き換えるテストは、組み立て方だけを直し、確かめる内容は変えない | `cargo test --workspace` が通る | NFR1 |
| NFR2.1 | test-after。core(定義の検証、読む対象の一覧)を作ってからその単体テスト、runtime(読み取り)を作ってからそのテスト、cli はテストだけを足す | 規則ごとに正しい場合と誤りの場合のテストがある | NFR2 |
| NFR2.2 | 実バイナリのテストで、無くてもよいファイルが無いときに `run`・`pipeline`・`validate`・`--dry-run` が 0 で何も足さないこと、あるが読めないときは 64 であることを確かめる | `binary_env.rs` に足したテストが通る | NFR2、FR2.5、BR3.1 |
| NFR2.3 | 行カバレッジ 80% 以上を保つ | 確認のコマンド(`cargo llvm-cov ... --fail-under-lines 80`)が通る | NFR2 |
| NFR2.4 | テストの都合で本番のコードを曲げない | 本番にテスト専用の項目が無い | NFR2、project.md |
| NFR3.1 | 3 つの OS と MSRV の CI が通る。パスの「無い」の判定は OS によらず `read_regular_file` の Missing で行う | CI の `test`、`build-check`、`msrv-test` | NFR3 |
| NFR3.2 | 更新したデモで、デモのテストが通る | `unix_demo.rs`(手元)、`windows_demo.rs`(CI) | NFR3、BR4.2 |
