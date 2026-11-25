# 要件とユニットの対応

出典: 要件(`requirements.md`)、ユニットの定義(`unit-of-work.md`)。ユーザーストーリーは作っていない(省いた)ため、要件の ID で対応を示す。

| 要件 | Unit ID | Directory |
|---|---|---|
| FR1.1 | U1 | u1-pipeline-retry |
| FR1.2 | U1 | u1-pipeline-retry |
| FR1.3 | U1 | u1-pipeline-retry |
| FR1.4 | U1 | u1-pipeline-retry |
| FR1.5 | U1 | u1-pipeline-retry |
| FR1.6 | U1 | u1-pipeline-retry |
| FR2.1 | U2 | u2-optional-env-file |
| FR2.2 | U2 | u2-optional-env-file |
| FR2.3 | U2 | u2-optional-env-file |
| FR2.4 | U2 | u2-optional-env-file |
| FR2.5 | U2 | u2-optional-env-file |
| FR3.1 | U3 | u3-unset-env |
| FR3.2 | U3 | u3-unset-env |
| FR3.3 | U3 | u3-unset-env |
| FR3.4 | U3 | u3-unset-env |
| FR3.5 | U3 | u3-unset-env |
| FR4.1 | U4 | u4-graph-dot |
| FR4.2 | U4 | u4-graph-dot |
| FR4.3 | U4 | u4-graph-dot |
| FR4.4 | U4 | u4-graph-dot |
| FR5.1 | U5 | u5-log-dir |
| FR5.2 | U5 | u5-log-dir |
| FR5.3 | U5 | u5-log-dir |
| FR5.4 | U5 | u5-log-dir |
| FR5.5 | U5 | u5-log-dir |
| FR6.1 | U6 | u6-attempt-times |
| FR6.2 | U6 | u6-attempt-times |
| FR6.3 | U6 | u6-attempt-times |
| FR6.4 | U6 | u6-attempt-times |
| FR6.5 | U6 | u6-attempt-times |
| FR6.6 | U5; U6 | u5-log-dir; u6-attempt-times |
| FR6.7 | U6 | u6-attempt-times |
| FR6.8 | U6 | u6-attempt-times |
| FR6.9 | U6 | u6-attempt-times |
| FR7.1 | U7 | u7-watch-follow |
| FR7.2 | U7 | u7-watch-follow |
| FR7.3 | U7 | u7-watch-follow |
| FR7.4 | U7 | u7-watch-follow |
| FR7.5 | U7 | u7-watch-follow |
| FR8.1 | U1; U2; U3; U4; U5; U6; U7 | u1-pipeline-retry; u2-optional-env-file; u3-unset-env; u4-graph-dot; u5-log-dir; u6-attempt-times; u7-watch-follow |
| FR8.2 | U7 | u7-watch-follow |

## 複数のユニットにまたがる項目

- **FR6.6**(時刻の書式を自前で書く): 整形と現在の時刻は U5 で作り、ステートでの使い方は U6。
- **FR8.1**(README とデモ): 各ユニットが、その機能の分を更新する。
- **NFR1〜NFR7**: すべてのユニットに当てはまる(後方互換、テスト、3 つの OS、依存、`unsafe` と `cfg`、秘密の保護、品質の検査)。

## ユニットの中の実装の順序

チームの実践(test-after)に従い、各ユニットで core → runtime → cli の層の順に実装し、各層の後にその層のテストを書く。最後に README とデモを更新する。U7 は最後に版を上げる。

## 網羅の確認

- 要件の 41 項目(FR1.1〜FR8.2)は、すべていずれかのユニットに割り当てた。
- 7 つのユニットは、どれも 1 つ以上の要件を持つ。
