# NFR Requirements(u1-core)— 質問

u1-core は、ファイルとプロセスにほとんど触れない library(定義の読み込みと、判断のロジック)です。画面も、常駐もしないため、性能・スケーラビリティ・信頼性・可観測性の要件は、この Unit の種類には当てはまりません(起動オーバーヘッドの 100ms は、起動経路全体の要件であり、u3-cli で測ります)。この段階では、セキュリティの要件と、技術の選択(実装言語は Rust と決まっています)を決めます。

## Q1. [開発] 定義ファイル(TOML)の読み取りの方法

- A. 定番のライブラリを使う: シリアライズの `serde` と、TOML の `toml`。型の誤りや知らないキーの検出(BR1.5、BR1.6)を、型に載せやすい。ビルドへの依存は増えるが、実績がある(推奨)
- B. TOML の読み取りを手書きする(外部のライブラリを使わない)。依存は無いが、実装と検証の手間が大きく、誤りやすい
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [開発] Rust の edition と、対応する最小のバージョン

- A. 最新の安定版を使い、edition は 2024 とする。対応する最小のバージョンは、edition 2024 を使える最低のバージョンに置く(推奨)
- B. edition は 2021 とし、少し古いコンパイラにも対応する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [開発] エラーの型の作り方

定義の誤り(DefinitionError)など、エラーの型を作る方法です。

- A. 定番のライブラリ(`thiserror`)を使い、エラーの表示の文を型に付ける(推奨)
- B. 標準のライブラリだけで、手書きする。依存は増えないが、記述が長くなる
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q4. [運用] 定義ファイルの大きさの上限

定義ファイルは、利用者が書く、基本的に信頼できる入力ですが、読み取りには、安全な設定と上限を置くことを勧めます(誤って巨大なファイルを渡す事故を防ぐため)。

- A. 上限を置く(例: 1 MiB)。超えたときは、定義の誤りとして拒否する(推奨)
- B. 上限は置かない
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
