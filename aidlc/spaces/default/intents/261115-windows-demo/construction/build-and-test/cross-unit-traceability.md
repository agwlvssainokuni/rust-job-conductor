# 要件とコードの対応(最終確認)

判定: **合格**。要件 16 件(FR 12、NFR 4)のすべてに、作ったファイルが対応している。対応のない要件はない。

| 要件 | 作ったファイル | 確かめた方法 |
|---|---|---|
| FR1.1 | `demo/windows/run-demo.cmd` | ファイルの配置。UNIX 版は変更なし(`git status`) |
| FR1.2 | `demo/windows/scripts/extract.cmd`(ほか 3 本) | 実機の実行 |
| FR1.3 | `demo/windows/conductor.toml` | テスト「定義は 4 つのジョブと extract の警告の遷移を持つ」 |
| FR2.1 | `demo/windows/run-demo.cmd` | テスト 場面 1〜6、実機の実行 |
| FR2.2 | `demo/windows/scripts/load.cmd`(ほか) | テスト 場面 2・3・4 |
| FR2.3 | `demo/windows/scripts/transform.cmd`(ほか) | 実機の実行(`DEMO_SLEEP=0` と既定) |
| FR2.4 | `demo/windows/run-demo.cmd` | 実機の実行(`DEMO_PAUSE=1`、Enter 6 回) |
| FR2.5 | `demo/windows/run-demo.cmd` | 実機の実行(ビルド済みの実行ファイル) |
| FR2.6 | `demo/windows/run-demo.cmd` | 実行後の `git status` |
| FR2.7 | `demo/windows/run-demo.cmd` | 実機の実行(`DEMO_LANG=en_US`) |
| FR3.1 | `demo/README.md` | 目視(Windows 版の節) |
| FR3.2 | `README.md`(変更なし) | `git status` |
| NFR1 | `demo/windows/run-demo.cmd` | `cmd.exe` だけで実行 |
| NFR2 | `demo/windows/scripts/notify.cmd`(ほか) | ライセンスヘッダと日本語のコメント |
| NFR3 | `crates/cli/tests/windows_demo.rs` | テスト 5 件と、実機の実行 |
| NFR4 | `crates/cli/tests/windows_demo.rs` | 全テスト、`fmt`、`clippy` が通る |
