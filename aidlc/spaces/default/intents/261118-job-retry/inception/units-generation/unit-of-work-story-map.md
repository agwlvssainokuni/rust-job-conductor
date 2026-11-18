# Unit of Work Story Map — 正常終了しなかったジョブの自動リトライ

ユーザーストーリーは作っていない(User Stories はスキップした)ため、要件の ID(`requirements.md` の FR と NFR)をキーにする。

## 要件と Unit の対応

| ID | Unit ID | Directory |
|---|---|---|
| FR1.1 | U1 | u1-retry-config |
| FR1.2 | U1;U2 | u1-retry-config;u2-retry-execution |
| FR1.3 | U1 | u1-retry-config |
| FR1.4 | U1 | u1-retry-config |
| FR1.5 | U1 | u1-retry-config |
| FR1.6 | U1;U2 | u1-retry-config;u2-retry-execution |
| FR1.7 | U1 | u1-retry-config |
| FR1.8 | U1 | u1-retry-config |
| FR1.9 | U1 | u1-retry-config |
| FR2.1 | U2 | u2-retry-execution |
| FR2.2 | U2 | u2-retry-execution |
| FR3.1 | U2 | u2-retry-execution |
| FR3.2 | U2 | u2-retry-execution |
| FR3.3 | U2 | u2-retry-execution |
| FR3.4 | U2 | u2-retry-execution |
| FR3.5 | U2 | u2-retry-execution |
| FR3.6 | U2 | u2-retry-execution |
| FR4.1 | U2 | u2-retry-execution |
| FR4.2 | U2 | u2-retry-execution |
| FR5.1 | U2 | u2-retry-execution |
| FR5.2 | U2 | u2-retry-execution |
| FR6.1 | U2 | u2-retry-execution |
| FR6.2 | U2;U3 | u2-retry-execution;u3-retry-display-docs |
| FR7.1 | U1 | u1-retry-config |
| FR7.2 | U1 | u1-retry-config |
| FR8.1 | U3 | u3-retry-display-docs |
| NFR1 | U1;U2;U3 | u1-retry-config;u2-retry-execution;u3-retry-display-docs |
| NFR2 | U1;U2;U3 | u1-retry-config;u2-retry-execution;u3-retry-display-docs |
| NFR3 | U2 | u2-retry-execution |
| NFR4 | U1;U2;U3 | u1-retry-config;u2-retry-execution;u3-retry-display-docs |
| NFR5 | U1;U2;U3 | u1-retry-config;u2-retry-execution;u3-retry-display-docs |
| NFR6 | U1;U2;U3 | u1-retry-config;u2-retry-execution;u3-retry-display-docs |
| NFR7 | U2 | u2-retry-execution |

## 複数の Unit にまたがる行

- FR1.2(一覧がなければリトライしない): u1 が「一覧がない」を設定の値として表し、u2 がそれに従ってリトライしない。
- FR1.6(設定しない定義は、これまでと同じ動作): u1 が定義の読み取りを変えず、u2 が実行の流れを変えない。
- FR6.2(英語と日本語の文言): u2 がコンパイルに必要な最初の文言を書き、u3 がそれを確定する。
- NFR1、NFR2、NFR4、NFR5、NFR6: 3 つの Unit のすべてで守る。NFR2 は、3 つとも `core`・`cli` のソースを変えるため、`nfr_source` のテストの対象になる。

## 各 Unit の中の主な作業

順番を拘束するものではない(作る順番は Delivery Planning が決める)。

- u1: 型の追加、定義の読み取りと検査、誤りの文言、`validate`・`convert` の確認、テスト
- u2: 時間と待機の差し込み口、固定の間隔のリトライ(`run`、`pipeline`)、回ごとに伸ばす方式、待機中のシグナル、表示の種類と最初の文言、テスト
- u3: 文言の確定、表示のレベルの確認、README、テスト

## 網羅の確認

- 列挙したすべての ID(FR 26 件、NFR 7 件)に、Unit が割り当てられている。
- 3 つの Unit のすべてに、行がある。
