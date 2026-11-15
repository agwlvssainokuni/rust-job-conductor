# Deployment Log(Windows 向けデモ)

## Sources

- Initial description: Windows用のデモをつくってほしい。 [desc]
- Workflow-selected scope: express [scope]
- Build and Test の結果(`construction/build-and-test/test-results.md`)
- 質問: 「この作業を、どう公開しますか」→ リリースしない(`main` の push だけ) [Q1]

## 実行の記録(2026-11-15)

| 順 | 操作 | 結果 |
|----|------|------|
| 1 | リリース(版の更新、タグ付け、タグの push) | **行わない**(Q1: A)。配布する実行ファイルのコードは変わっておらず、デモはリリースの配布物に入らないため |
| 2 | `main` の push | **人が実施する**。この作業では行っていない |

- デプロイの対象となる環境は、この作業では無い(単一バイナリの CLI。ステージング環境への自動デプロイは該当しない。`team.md`)。
- 既存の `release.yml` と `ci.yml` は、変えていない。次にタグ `v*` を付けたときに、これまでと同じように動く。

## 未確認のこと

- `windows_demo`(Windows 専用のテスト 5 件)の、CI(windows-2022)での実行は、`main` を push するまで確かめられない。手元の Windows 11 では、5 件とも成功している(Build and Test)。

## ロールバック

- 公開していないので、利用者への影響はない。
- 取り消すときは、この作業のコミット(`git log` の `windows-demo` の各コミット)を `git revert` する。追加したファイルは `demo/windows/` と `crates/cli/tests/windows_demo.rs` だけで、`demo/README.md` の変更は、`git checkout f5b2460 -- demo/README.md` で元に戻せる。

## Assumptions & Open Questions

None.
