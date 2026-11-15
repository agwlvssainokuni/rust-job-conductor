# Build and Test Summary

## 状態

- 文書の確認: docs-check の 7 語句、目次とリンク(17 個、切れ 0)、run ID の規則の集約、整理前後の事実の突き合わせ、変更の範囲、すべて成功。
- README に書いた手順の実行: クイックスタート、ソースからのビルド、リリースのアーカイブの展開、すべて、書いたとおりに成功。
- 既存の検査(コードは変えていない): fmt、clippy、666 件のテスト、Rust 1.85 のビルド、すべて成功。
- 未確認: Windows のクイックスタートと、Linux・Intel macOS・Windows のアーカイブの実物の確認(記述は `release.yml` から)。

## 作成したテストの種類

文書の変更なので、テストコードは書いていない(変更を 2 つの Markdown に限る要件のため)。確認は、実行したコマンドと、その結果を、`test-results.md` に記録した。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | requirements NFR1 | docs-check の 7 語句が README に残る | すべてある | `test-results.md`(`grep` の繰り返し) | build-and-test | Met |
| BT-02 | requirements FR1.1・FR1.2・NFR3 | 節が計画の順で、目次のリンクが切れていない | 順序は計画どおり。リンク 17 個、切れ 0。`demo/README.md`・`LICENSE` は実在 | `test-results.md`(アンカーのスクリプト) | build-and-test | Met |
| BT-03 | requirements FR2 | run ID の規則が README の 1 か所にだけある | 規則の主要な語句が、それぞれ 1 回 | `test-results.md`(検索) | build-and-test | Met |
| BT-04 | requirements FR8 | 事実の記述を削らず変えない(FR4 を除く) | バッククォートの語句 110 個、終了コードの行、各規則が、すべて残っている | `test-results.md`(`git show` との突き合わせ) | build-and-test | Met |
| BT-05 | requirements FR1.3 | クイックスタートが、書いたとおりに動く | 独立に実行した出力と終了コードが、README と一致 | `test-results.md`(クイックスタート) | build-and-test | Met |
| BT-06 | requirements FR5 | ソースからのビルドの手順が、実際に動く | build・test・`cargo install --bin conductor` が成功。URL は実在 | `test-results.md` | build-and-test | Met |
| BT-07 | requirements FR4.1 | 展開の手順が、実際のアーカイブの構造と合う | v0.6.0 の aarch64-apple-darwin で確認(最上位のディレクトリ、`conductor`、`LICENSE`、SHA-256 が OK) | `test-results.md` | build-and-test | Met |
| BT-08 | requirements FR4.2〜FR4.6・FR6 | 6 項目の修正と、版の注記の統一 | 反映済み(`code-summary.md`)。README を再読して確認 | `code-summary.md`、README | build-and-test | Met |
| BT-09 | requirements FR7 | `demo/README.md` がスクリプトの動作と既定値に一致する | スクリプトと突き合わせて修正(場面 0、`DEMO_SLEEP` の既定、環境変数) | `code-summary.md` | build-and-test | Met |
| BT-10 | requirements NFR2 | 変更が `README.md` と `demo/README.md` だけ | そのとおり | `git diff --stat` | build-and-test | Met |
| BT-11 | requirements NFR4・team.md Code Style | fmt・clippy・テスト・Rust 1.85 が成功のまま | 成功(666 件) | `test-results.md` | build-and-test | Met |
| BT-12 | team.md(行カバレッジ 80% 以上) | 下限を保つ | コードを変えていないので、前回(97.74%)のまま。再測定はしていない | `git diff`(コード変更なし) | build-and-test | Met(コードの変更なしによる) |
| BT-13 | requirements FR4.1・FR1.3(他の OS) | Windows のクイックスタート、Linux・Intel macOS・Windows のアーカイブの実物で、README の手順が動く | 未実行(記述は `release.yml` から) | — | なし(人の確認) | Unverified |
| BT-14 | team.md(Linux・Windows のテストを CI で実行) | CI でテストが成功 | — | — | なし(この計画は CI Pipeline を持たない。既存の CI が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` が実行する) |

## 準備の評価

- build-ready: 可。test-ready: 可。deployment-ready: リリースはしない(版は 0.6.0 のまま)。

## 既知の制限と残りの項目

- **BT-13 は未確認:** Windows のクイックスタートの `cmd /c` への読み替えと、Windows の zip の構造(`release.yml` から読んだだけ)、Linux・Intel macOS のアーカイブの実物は、確認していない。README の記述は、`release.yml` と、実際に確認できた aarch64-apple-darwin のアーカイブに基づく。
- `shasum -a 256 --check --ignore-missing` の可用性と、Gatekeeper の手順は、OS の動作で、コードの外のため、確認できない。
- BT-14 は、push 後の CI の結果で確かめる(README だけの変更なので、`docs-check` が主な検査)。
