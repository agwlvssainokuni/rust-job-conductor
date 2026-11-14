# NFR Requirements(u3-cli)— 質問

u3-cli は、`conductor` の実行ファイルそのものです。常駐しない、1回で終わるコマンドのため、スケーラビリティ・可用性・可観測性(ログ、メトリクス、トレース)は、この Unit では、ほとんど当てはまりません(当てはまらない理由を、成果物に書きます)。決める必要があるのは、起動の速さ、入力と端末の安全、シグナルの扱いの安全、そして、技術の選択(引数の解釈、シグナルの受け取り、色)です。3点を確認します。

## Q1. [技術] 引数の解釈の方法

コマンドは5つ、オプションは `--config`、`-v`、`--to`、`--help`、`--version` だけです。使い方の誤りは、終了コード 68 にし、使い方の文言を、日本語と英語で出し分ける必要があります。

- A. `clap` のライブラリを使う(広く使われ、情報が多いが、バイナリが大きくなり、使い方の文言と終了コードの置き換えが要る)
- B. `lexopt` のような、小さな字句の解釈だけのライブラリを使い、コマンドごとの解釈は、自分で書く。文法が小さく、文言と終了コードを、全部自分で決められる(推奨)
- C. ライブラリを使わず、標準の `std::env::args` だけで書く
- X. Other (please specify)

[Answer]: B

## Q2. [技術] シグナルの受け取り

UNIX 系では、SIGHUP・SIGINT・SIGQUIT・SIGTERM のどれを受け取ったかを、番号で知る必要があります。Windows では、Ctrl+C(番号 2)だけです。

- A. UNIX 系は `signal-hook`、Windows は `ctrlc` のライブラリを使う。どちらも、ハンドラの中では、安全な操作(印を立てるだけ)に限る。この Unit に、`unsafe` を足さない(推奨)
- B. `libc` で、シグナルのハンドラを自分で書く(この Unit に、`unsafe` が増える)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [表示] Windows での色

`watch` の状態の列に、端末のときだけ色をつけます(状態は、文字でも表す)。Windows の端末で色の制御文字を使うには、OS ごとの設定が要ります。

- A. 色は、UNIX 系の端末だけにつけ、Windows では、色をつけない(文字だけ)。Windows の OS 固有のコードと、依存を増やさない(推奨)
- B. Windows でも、端末の色の設定を有効にして、色をつける(OS 固有のコードと、依存が増える)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
