# Build and Test Summary

## 状態

- ビルド(安定版 1.97): 成功。リリースビルド: 成功(`conductor` 約 971 KB)。
- テスト: 564 件成功、失敗 0。整形と clippy: 問題なし。行カバレッジ: 97.66%(下限 80%)。
- **最小の Rust のバージョン 1.85 のビルド: 失敗**(`u1-core` の let チェーン 2 か所)。このため Build and Test は現時点で失敗。`u1-core` の Code Generation に戻して直す(`test-results.md` の Loop-Back Log を参照)。

## 作成したテストの種類

| 種類 | ファイル | 備考 |
|------|----------|------|
| 単体 | 各 Unit の `cargo test` | Code Generation で作成済み(`unit-test-instructions.md`) |
| 統合 | `integration-test-instructions.md` | バイナリの起動、実シグナル、Unit 間の結合 |
| 性能 | `crates/u3-cli/tests/nfr_perf.rs` | 警告のみ。別の指示書は作らない(Standard の方針) |
| セキュリティ | `nfr_source.rs`、`nfr_robust.rs`、各 Unit のテスト | Standard の方針のため、別の指示書は作らない |

## カバレッジの期待

プロジェクト全体と各 Unit で行カバレッジ 80% 以上。実測は全体 97.66%、最小のファイル 91.11%。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Testing Posture / NFR5 | 行カバレッジ 80% 以上(全体) | 97.66% | `test-results.md` の `cargo llvm-cov` | build-and-test | Met |
| BT-02 | unit-test-instructions(u1〜u4) | 各 Unit の範囲で 80% 以上 | すべてのファイルが 91% 以上 | `test-results.md` のファイル別の表 | build-and-test | Met |
| BT-03 | 各 Unit の unit-test-instructions | 全テストが成功 | 564 件成功、失敗 0 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-04 | team.md Code Style | `cargo fmt --check` と `cargo clippy -D warnings` が成功 | 両方 終了コード 0 | `test-results.md` | build-and-test | Met |
| BT-05 | nfr-requirements(各 Unit)rust-version 1.85 | 1.85 でワークスペースがビルドできる | `walk.rs:160` と `:193` で失敗 | `cargo +1.85 build --workspace --locked` | build-and-test(u1-core に戻して修正) | Not Met |
| BT-06 | nfr-requirements tech-stack(Rust だけ、C のビルドなし) | `cc`・`-sys` の依存なし | なし | `cargo tree` | build-and-test | Met |
| BT-07 | u3-cli nfr-requirements NFR1 | 起動のオーバーヘッド p95 100 ms 以内ほか | 21.4 ms ほか(macOS の debug) | `test-results.md` の性能の節 | build-and-test | Met |
| BT-08 | u3-cli nfr-requirements NFR2.1 | リリースのビルドが成功し、単一のバイナリ | 971,056 バイト | `cargo build --release --locked --bin conductor` | build-and-test | Met |
| BT-09 | team.md Testing Posture | Linux と Windows のテストを CI で実行 | CI は未作成 | — | ci-pipeline | N/A(CI Pipeline がこの計画の後ろにあり、CI で、これらのテストをマージ前に実行するようにする) |
| BT-10 | 要件 NFR5 の AC(CI で 80% 未満なら失敗) | CI が 80% 未満で失敗する | CI は未作成 | — | ci-pipeline | N/A(同上) |
| BT-11 | u3-cli code-generation-plan(Windows の Ctrl+C) | Windows の Ctrl+C で中断が働く | 自動テストなし。手動のチェックリストのみ | `crates/u3-cli/tests/WINDOWS_CTRL_C_CHECKLIST.md` | なし(リリース前の手動確認) | Unverified |

## 準備の評価

- build-ready: 安定版で可。最小のバージョンは、上の BT-05 を直したあとで可。
- test-ready: 可(macOS)。Linux・Windows は CI のステージで。
- deployment-ready: まだ。BT-05 の修正、CI(ci-pipeline)、BT-11 の手動確認が先。

## 既知の制限と残りの項目

- BT-05 は、`u1-core` の Code Generation に戻して直す(loop-back 1)。
- BT-11 は、計画が許した手動の確認で、担当するステージがない。リリースの前に人が行うか、受け入れるかを、承認のときに決める。
- 実行していない確認: Linux と Windows の実行、`cfg(windows)` のコードの実行。
- 要件 NFR5(CI がカバレッジで失敗する)は、`cross-unit-traceability.md` で「未対応」と出る。CI Pipeline のステージが担当する。
