# Tech Stack Decisions(u3-cli)

上流: functional-spec、rules、requirements(NFR1〜NFR6)、contract-summary(E1)、team-practices(`rustfmt`・`clippy`、test-after、`cargo test`、行カバレッジ 80%、GitHub Actions、タグでリリース)、質問への回答(Q1〜Q3)、u1-core・u2-runtime・u4-converter の tech-stack-decisions(同じワークスペースの方針)。実装言語は Rust(constraint-register の C4)。各行の ID は、**その選択が満たす要件の ID**(nfr-requirements の他のファイルにある `NFRx.y`)であり、新しい要件を定めない。

## 選択

| ID | 項目 | 選択 | 理由 | 出典 |
|----|------|------|------|------|
| NFR2.1 | 配布の形 | パッケージ `conductor-cli`(`crates/u3-cli/`)。実行ファイルの名前は `conductor`。外部のランタイムに依存せず、OS ネイティブの単一バイナリ。C のコンパイラを必要とするライブラリに依存しない(`libc` は、u2-runtime が使う宣言だけ) | 単一バイナリ、3つの OS 対応のため | NFR2、NFR3 |
| NFR3.1 | OS への依存 | この Unit の OS 固有のコードは、シグナルの受け取りの準備と、標準出力が端末かどうかの判定だけ(Windows の端末の色の設定は、行わない)。`cfg` で分け、1つのモジュールに閉じ込める | OS の違いを、u2-runtime と、この小さなモジュールに閉じ込める | NFR3 |
| — | ワークスペースと edition | u1-core・u2-runtime・u4-converter と同じワークスペース。edition 2024、最小のバージョンは 1.85。依存: `conductor-core`、`conductor-runtime`、`conductor-converter`(パス依存) | 同じバイナリに組み込まれるため | unit-of-work-dependency |
| NFR4.1 | 引数の解釈 | `lexopt`(小さな字句の解釈だけのライブラリ)を使い、コマンドごとの解釈は、この Unit の中で書く [Q1: B]。使い方の文言と、使い方の誤りの終了コード(68)は、自分で決める | 文法が小さく(5つのコマンドと数個のオプション)、日本語・英語の文言と、終了コードを、全部自分で決められる。起動も速い | NFR4、NFR1、rules の BR1.x |
| NFR4.3 | シグナルの受け取り | UNIX 系は `signal-hook` の `Signals`(イテレータ)を専用のスレッドで回し、Windows は `ctrlc` を使う [Q2: A]。最初の番号だけを、合図に書く。この Unit に `unsafe` を置かない(`#![forbid(unsafe_code)]`。`low_level` は使わない) | 実績のあるライブラリで、ハンドラの安全な作りを任せ、`unsafe` を増やさない。番号を知るには、`Signals` のイテレータが必要 | NFR4、rules の BR4.1 |
| NFR6.1 | 端末と色 | 標準ライブラリの `std::io::IsTerminal` で、端末かを判定する(1.70 以降)。色は、ANSI の制御文字を、自分で書く(UNIX 系の端末だけ。Windows は色なし。Q3: A)。色のライブラリは、使わない | 依存を増やさず、Windows の OS 固有のコードを増やさない | NFR6、rules の BR6.5 |
| NFR6.3 | 言語の判定と文言 | 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` を、自分で読み、`ja` で始まれば日本語。文言は、英語と日本語の対を、Rust の定数の表として持つ(翻訳のライブラリは使わない)。判定は、最初の表示が必要になるまで行わない | 起動の速さ(NFR1.4)と、依存の少なさ | NFR6、NFR1、rules の BR6.1 |
| NFR4.7 | パニックの扱い | パニックのフック(`std::panic::set_hook`)で、固定の文を表示し、終了コード 70 で終わる。ビルドは、`panic = "unwind"`(既定)のまま | ジョブの終了コードと区別できる値で、内部の誤りを知らせる | NFR4、rules の BR5.1 |
| NFR1.1、NFR2.1 | リリースのビルドの設定 | `[profile.release]` に、`opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true` を置く(仮。実測で見直す。判断 T4) | 起動の速さと、バイナリの大きさ | NFR1、NFR2 |
| NFR5.1 | テスト | `cargo test` による単体テストと、バイナリを実際に動かす統合テスト(`env!("CARGO_BIN_EXE_conductor")`。追加のライブラリなし。補助の実行ファイルは、u2-runtime のものを使う)。行カバレッジ 80% 以上(`cargo llvm-cov`。Linux で測る)。`tempfile` を開発用の依存とする | チームの実践(test-after、80%、バイナリを実行する統合テスト) | NFR5、team-practices |
| — | 整形と静的検査 | `rustfmt` と `clippy`。CI で失敗したらマージできない | チームの実践 | team-practices |
| NFR4.8 | リリースの作り方 | タグを付けると、GitHub Actions が、3つの OS 向けの実行ファイルを、`cargo build --release --locked -p conductor-cli` でビルドし、SHA-256 のチェックサムとともに公開する(CI Pipeline・Deployment Pipeline のステージで作成) | チームの実践(タグでリリース) | NFR4、team.md |

## 判断の記録(代替案との比較)

### T1 引数の解釈に `lexopt` を使い、コマンドの文法を自分で書く [Q1: B]

- **Context:** コマンド 5 つ、オプション数個。使い方の誤りは、終了コード 68(ジョブの終了コード 2 と区別するため)。使い方の文言は、日本語と英語。
- **Decision:** `lexopt` で、オプションと引数を字句に分け、コマンドの文法を、この Unit のコードで検査する。
- **Consequences:** 良い点は、文言・終了コード・エラーの扱いを、全部制御でき、バイナリが小さく、起動が速い。悪い点は、文法の検査と使い方の文言を、自分で書き、テストする必要がある(Rust に不慣れな開発者には、`clap` より、書く量が多い)。
- **Alternatives Rejected:** (A) `clap`: 使い方の文言と、エラーの終了コードを、置き換える作業が要り、日本語の使い方の文言を、別に用意することになり、バイナリが大きくなる。(C) `std::env::args` だけ: 依存はないが、`--config=path` と `--config path` の両方、`-v` のまとめ書き、`--` の扱いなどを、自分で全部書く必要があり、誤りやすい。

### T2 シグナルは `signal-hook`(UNIX 系)と `ctrlc`(Windows) [Q2: A]

- **Context:** 受け取ったシグナルの番号を知り(UNIX 系)、ハンドラの中で、安全な操作だけをしたい。この Unit に `unsafe` を置きたくない。
- **Decision:** UNIX 系は `signal-hook`(フラグの登録、またはイテレータで番号を得る)、Windows は `ctrlc`(Ctrl+C)。
- **Consequences:** 良い点は、`unsafe` を自分で書かず、実績に頼れる。悪い点は、依存が2つ増え、OS ごとに別のライブラリを使う(`cfg` で分ける)。
- **Alternatives Rejected:** (B) `libc` で、`sigaction` を自分で書く: 依存は増えないが、この Unit に `unsafe` が増え、ハンドラの安全性を、自分で守る必要がある。

### T3 色は、自分で書き、UNIX 系の端末だけ [Q3: A]

- **Context:** `watch` の状態の列に、端末のときだけ色をつける。状態は、文字でも表す(NFR6)。
- **Decision:** ANSI の制御文字を、自分で書く(数種類の色だけ)。Windows では、色をつけない。
- **Consequences:** 良い点は、依存と OS 固有のコードを増やさない。悪い点は、Windows の端末では、色がつかない(状態は、文字で判別できる)。
- **Alternatives Rejected:** (B) Windows でも、端末の色の設定を有効にする: 色の体験は揃うが、OS 固有のコード(端末の設定の API)と、依存が増える。色のライブラリ(`anstyle` など)を使う: 数種類の色のためには、大きい。

### T4 リリースのビルドの設定

- **Context:** 起動の速さ(NFR1)と、配布の大きさ。
- **Decision:** `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true`。`panic` は、既定の `unwind`。
- **Consequences:** 良い点は、速く、小さい。悪い点は、ビルドの時間が増える。
- **Alternatives Rejected:** (1) 既定のまま(`lto` なし): ビルドは速いが、実行ファイルが大きく、起動がわずかに遅い。(2) `panic = "abort"`、`opt-level = "z"`(小ささを優先): さらに小さいが、パニックのフック(終了コード 70。NFR4.7)の後始末が行われず、起動の速さは、`opt-level = 3` の方が有利なため退けた。

### T5 ロケールの判定を、自分で書く

- **Context:** 環境のロケールで、英語と日本語を切り替える。判定は、最初の表示が必要になるまで行いたい(NFR1.4)。
- **Decision:** 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` を、この順に読み、`ja` で始まれば日本語。
- **Consequences:** 良い点は、依存がなく、遅延できる。悪い点は、OS 固有の設定(Windows のユーザーの表示言語)は、見ない(環境変数がなければ、英語)。
- **Alternatives Rejected:** ロケールのライブラリ(`sys-locale` など)を使う案: OS の設定まで見られるが、依存が増え、起動のたびに OS に問い合わせる費用がかかるため退けた。

### T6 パニックの方針

- **Context:** 内部の誤り(パニック)を、ジョブの終了コードと見分けやすくしたい。
- **Decision:** パニックのフックで、固定の文を出し、終了コード 70 で終わる。
- **Consequences:** 良い点は、パニックが、`conductor` の誤りとして、はっきり分かる。悪い点は、70 が、ジョブの終了コードと重なりうる(区別は、標準エラーの固定の文で行う)。
- **Alternatives Rejected:** パニックの既定(終了コード 101)に任せる案: 実装は要らないが、Rust の既定の文言が出て、固定の文にならず、101 も、ジョブの終了コードと重なりうるため退けた。

## 測定できる基準(NFR1.1、NFR2.1、NFR3.1、NFR5.1)

| ID | 基準 | 確認の方法 |
|----|------|------------|
| NFR1.1 | `conductor` 自身のオーバーヘッド(起動前 + ジョブのあと)の p95 が 100ms 以内(ジョブ10個、20 回、3つの OS)。仮の配分: 起動前 50ms(u1-core 20、u2-runtime 20、この Unit 10)、ジョブのあと 50ms | ベンチマークのテスト(統合。performance-requirements の測定の方法)。100ms を超えたら、配分の確定までは警告として記録し、確定のあと、失敗にする |
| NFR2.1 | 依存するライブラリは、C のコンパイルを要しない(ビルドスクリプトで C のコンパイラを呼ぶものがない)。`libc`(u2-runtime が使う宣言)と、OS の API の宣言だけの `windows-sys`・`linux-raw-sys`(`ctrlc` や `tempfile` が使う)は、許可する | `cargo tree --package conductor-cli -e normal,build` に、`cc`、`cmake`、`pkg-config`、`bindgen` が現れない(CI。ホストの OS ごとの違いは、3つの OS の CI で確かめる)。ビルドに C のコンパイラを必要としない |
| NFR3.1 | OS 固有のコードは、この Unit では、シグナルの受け取りの準備と端末の判定のモジュールの中だけ(Windows で端末の色を有効にする設定は、行わない。Q3: A)。3つの OS で、同じテストが通る | ソースの検索(`cfg(unix)`・`cfg(windows)` が、そのモジュールの外にない)。CI の3つの OS のテスト |
| NFR5.1 | 行カバレッジ 80% 以上(Linux で測る。`cfg(windows)` のコードは、Windows の CI のテストの成功で確かめる)。バイナリを実行する統合テストを含む | CI の測定(CI Pipeline のステージで導入) |

## 依存するライブラリの方針

- この Unit が使う外部のライブラリは、`lexopt`、`signal-hook`(UNIX 系)、`ctrlc`(Windows)を基本とする。いずれも、C のコンパイルを要しない(NFR2.1)。
- ライブラリの具体のバージョンは、Code Generation で、その時点の安定版を選び、`Cargo.lock` で固定する。最小の Rust のバージョン(1.85)で、ビルドできることを確かめる(できなければ、止めて、人に確認する)。
- ライブラリを増やすときは、起動オーバーヘッド(NFR1)と、単一バイナリ(NFR2)への影響を確認する。

## Assumptions & Open Questions

- `lexopt`、`signal-hook`、`ctrlc` の最新の版が、最小の Rust のバージョン(1.85)でビルドでき、`signal-hook` が、SIGHUP・SIGINT・SIGQUIT・SIGTERM の番号を取れることは、Code Generation の最初に確かめる(確かめられなければ、止めて、人に確認する)。 [assumption]
- リリースのビルドの設定(`lto = "thin"` など)は、仮の値である。Code Generation で、起動の速さと、バイナリの大きさを、実測して見直す。 [assumption]
- `ctrlc` が、Windows で、Ctrl+C の受け取りに加えて、Ctrl+Break や、コンソールの終了(CTRL_CLOSE_EVENT)の扱いを、どうするかは、Code Generation で確かめる(Functional Design の Q4 は、Ctrl+C だけを対象にする)。 [assumption]
