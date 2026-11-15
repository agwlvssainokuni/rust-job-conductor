# Build and Test Summary

## 状態

- ビルド(安定版 1.97 と最小の 1.85)、リリースビルド、fmt、clippy: すべて成功。
- テスト: 580 件成功、失敗 0(前回 564 件 + 新しい 16 件)。行カバレッジ 97.79%(下限 80%)。
- デモ: 全場面が完走(配列の `command` を含む)。
- 未実行: Windows 専用のテスト 1 件(`cfg(windows)`)の実行。Linux・Windows でのテスト実行(CI で行う)。

## 作成したテストの種類

| 種類 | 内容 |
|------|------|
| 単体 | core 7、runtime 5(+ Windows 専用 1)、cli 2、converter 2。要件ごとに 1 件が目安(Minimal の方針) |
| 統合 | `crates/cli/tests/binary_basic.rs`(実際のバイナリで、配列のジョブの `run` と `pipeline`)。別の指示書は、Minimal の方針により作らない |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Testing Posture | 行カバレッジ 80% 以上(全体) | 97.79% | `test-results.md` の `cargo llvm-cov` | build-and-test | Met |
| BT-02 | unit-test-instructions | 全テストが成功(既存 564 件を含む) | 580 件成功、失敗 0 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-03 | team.md Code Style | `cargo fmt --check` と `cargo clippy -D warnings` が成功 | 両方 終了コード 0 | `test-results.md` | build-and-test | Met |
| BT-04 | requirements NFR3、rust-version 1.85 | 1.85 でビルドとテストが成功 | 成功(580 件) | `cargo +1.85 build/test` | build-and-test | Met |
| BT-05 | requirements NFR1 | 文字列の `command` の既存の定義の動作が変わらない | 既存 564 件が、署名の追従だけの修正で成功 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-06 | requirements FR1〜FR3 | 要件ごとのテストが成功 | core 7、runtime 5、cli 2、converter 2 が成功 | `traceability.json`(code-generation) | build-and-test | Met |
| BT-07 | requirements FR5 | デモが配列の `command` を含めて完走 | 終了コード 0 | `DEMO_SLEEP=0.1 sh demo/run-demo.sh` | build-and-test | Met |
| BT-08 | requirements NFR4 | 版が 0.2.0 | `conductor 0.2.0` | `target/release/conductor --version` | build-and-test | Met |
| BT-09 | tech-stack(Rust だけ) | C のビルド依存がない | なし | `cargo tree` | build-and-test | Met |
| BT-10 | requirements FR2.3(Windows の `.cmd` に配列でメタ文字を渡す) | 続くコマンドが実行されない | 型検査まで。実行は未 | `cargo check --target x86_64-pc-windows-msvc` | なし(この計画は CI Pipeline を持たない。既存の CI が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` が `test (windows-2022)` で実行する) |
| BT-11 | team.md(Linux・Windows のテストを CI で実行) | CI でテストが成功 | — | — | なし(同上) | N/A(同上) |

## 準備の評価

- build-ready: 可。test-ready: 可(macOS。Linux・Windows は既存の CI)。
- deployment-ready: 次のステージ(Deployment Pipeline)で、版 0.2.0 のリリースの手順を確認する。公開の前に、CI(Windows のテストを含む)が成功していること。

## 既知の制限と残りの項目

- BT-10・BT-11 は、push 後の CI の結果で確かめる(この計画は CI Pipeline のステージを持たないため、`N/A`。結果は、あとで人が確認する)。
- `README` に、v0.1.x は配列の定義を拒否することを書いた(後方の互換の注意)。
