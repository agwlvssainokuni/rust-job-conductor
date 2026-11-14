# NFR Requirements(u2-runtime)— 質問

u2-runtime は library のため、可用性・スケーラビリティ・可観測性の要件は、この Unit の種類には当てはまりません。ここで決めるのは、安全(NFR4)と、技術の選択です。判断が必要な3点を確認します。

## Q1. [技術] シグナルをジョブに送る手段(UNIX 系)

ジョブにシグナルを送るには、標準のライブラリだけでは足りません(標準では、強制終了しかできない)。プロセスグループの作成(`process_group`)は、標準のライブラリにあります。

- A. `libc` のライブラリを、UNIX 系のコードにだけ使い、`kill` を呼ぶ。C のコンパイラは要らず(宣言だけ)、ワークスペースの依存は、1つ増える(推奨)
- B. `nix` のライブラリを使う(安全な包みだが、依存が増え、大きい)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [安全] ステートファイルの大きさの上限

Functional Design で、仮に 1 MiB を置きました。ジョブの数が多い定義でも、ステートは、1ジョブあたり数十バイトです。

- A. 1 MiB のまま確定する(推奨)
- B. 別の値にする(Other で、値を書く)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [安全] ステートファイルの権限

ステートには、ジョブID、終了コード、定義ファイルのパスだけが入ります。秘密情報は入りません。

- A. 特別な設定をせず、作成時の既定の権限(umask に従う)にする(推奨)
- B. 所有者だけが読み書きできる権限(UNIX 系で 0600)にする。Windows では、既定のまま
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
