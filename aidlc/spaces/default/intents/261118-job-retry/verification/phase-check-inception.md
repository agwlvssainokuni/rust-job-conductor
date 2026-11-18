# Phase Check — Inception → Construction

**判定: 合格(PASS)**

実行した Inception の段階の `traceability.json` を読み、未対応(GAP)、孤立(ORPHAN)、無効な対象、足りない上流の ID がないことを確かめた。

## 確かめた記録

| 段階 | ファイル | 結果 |
|---|---|---|
| requirements-analysis | `inception/requirements-analysis/traceability.json` | 要件 ID 41 件(FR 34(親 8 + 副項目 26)、NFR 7)を記録。`requirements.md` と一致 |
| units-generation | `inception/units-generation/traceability.json` | 上流の ID(FR 26 件)のすべてが `OK`。GAP、ORPHAN、無効な対象はなし。各対象の Unit は、`unit-of-work-story-map.md` のその ID の行に載っている |
| user-stories | (省略した段階) | 対象外 |
| domain-design | (省略した段階) | 対象外 |
| contract-design | (記録を作らない段階。省略した) | 対象外 |

## Unit ごとの要件の対応(units-generation)

| Unit | 対応する FR |
|---|---|
| U1 u1-retry-config | FR1.1〜FR1.9、FR7.1、FR7.2(FR1.2、FR1.6 は U2 と共有) |
| U2 u2-retry-execution | FR2.1、FR2.2、FR3.1〜FR3.6、FR4.1、FR4.2、FR5.1、FR5.2、FR6.1(FR1.2、FR1.6、FR6.2 の一部) |
| U3 u3-retry-display-docs | FR6.2、FR8.1 |

## 残っている未決の事項(Construction の設計の段階で扱う)

`requirements.md` の Open questions に記録済み。

- 設定の項目名と形、最大値・桁あふれの扱い
- 起動できなかった場合(127)と、ジョブ自身の 127 の区別
- リトライの表示の出力先と、`-v` なしで出すか
- 一覧なしで最大回数や間隔だけがある定義、空の一覧、上限が間隔より小さい定義の扱い
