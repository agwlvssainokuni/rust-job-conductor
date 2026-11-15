# Cross-Unit Traceability

`aidlc engine sensor-traceability --stage build-and-test` の結果(2026-11-15)。

## 判定: 不合格(未対応 1 件)

| ID | 状態 | 担当 | 対応先 |
|----|------|------|--------|
| NFR5(CI で行カバレッジ 80% 未満なら失敗) | 未対応 | ci-pipeline(この計画の後ろのステージ) | CI のワークフロー。いまは、ローカルの実測(97.66%)だけがある |

## 対応済み

上記を除く、すべての要件(FR1.1〜FR7.2)と非機能の要件(NFR1〜NFR4、NFR6)は、受け入れ基準(AC)または Unit ごとの NFR を通して対応済み。内訳:

- **FR1.x(定義・引数)、FR2.x(個別実行)、FR3.x(一括実行)、FR4.x(判断・ステート)、FR5.1(watch)、FR6.1(reset)、FR7.x(convert):** u1-core(判断)、u2-runtime(実行・ステート)、u3-cli(流れ・表示)、u4-converter(変換)の AC で対応。
- **NFR1(性能)、NFR2(配布・互換)、NFR3(移植性)、NFR4(堅牢性・安全)、NFR6(表示):** 各 Unit の NFR の表で対応(コマンドの出力にある対応先は、u1〜u4 の各 `traceability.json`)。

## 未対応の要素の扱い

NFR5 は、承認のときに、人に示す。CI Pipeline のステージで、CI を作ったときに対応する。
