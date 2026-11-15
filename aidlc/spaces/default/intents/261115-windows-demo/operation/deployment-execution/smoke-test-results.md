# Smoke Test の結果

デプロイをしていないので、デプロイ後の smoke test は無い。代わりに、手元の Windows 11 で、次を確かめた(Build and Test の結果。`construction/build-and-test/test-results.md`)。

| 確認 | 結果 |
|---|---|
| `target\release\conductor.exe --version` | `conductor 0.3.0` |
| `demo\windows\run-demo.cmd`(場面 0〜6) | 最後まで完了。終了コードは期待どおり |
| `windows_demo`(5 件) | 成功 |

CI(windows-2022)での確認は、`main` の push のあとになる。
