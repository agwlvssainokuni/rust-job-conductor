# プラクティスの確認: 質問

前回確定した `team.md` の内容を既定とし、コードと CI から確かめられなかった点だけを聞きます。

## Q1. [開発者] コミットとマージの流れ

変更は、どう `main` に入れていますか。履歴は、1 人の作者で、マージコミットがなく、`main` だけです。

- A. `main` に直接コミットする(今のやり方を続ける)
- B. プルリクエストを作って、レビューしてから入れる
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [開発者] 機能の作り方

今の進め方は「機能を 1 つずつ順に作る」で、「最初に、全体を端から端まで動かす最小の版を作って、部品がつながることを確かめる」(薄い縦切りの先行)はしていません。今回の 4 つの機能(環境変数と `.env`、`graph`、`--dry-run`、`init`)でも、同じにしますか。

- A. 同じにする(1 つずつ順に作り、薄い縦切りは先に作らない)
- B. 今回は、最初に薄い縦切りを作る
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [品質] テストの方針

今は「実装してからテストを書く」で、行カバレッジ 80% を下限にし、単体テストと、バイナリを実行する統合テストを書いています。今回も、同じにしますか。

- A. 同じにする(実装してからテスト。カバレッジ 80%。単体と統合)
- B. 同じにして、性能・堅牢性・セキュリティの確認も、機能ごとに加える
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [開発者] 依存の追加

`.env` の読み込みや、`graph` の図の出力のために、新しい外部のライブラリを足してよいですか。今は、C の部品のコンパイルを要する依存を禁止しています。

- A. 足さない(必要なものは、自前で書く)
- B. 必要なら足してよい(ただし、C のコンパイルを要するものは除く)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q5. [リリース] 後回しにしていること

次の 4 つは、今は後回しです: (1) 配布物の署名と公証、(2) Intel Mac の動作確認、(3) 依存の脆弱性検査の強制、(4) 起動時間の計測の強制。今回も、同じにしますか。

- A. 今回も、4 つとも後回しにする(今のとおり)
- B. 一部を、今回の範囲に入れる(「その他」に、入れるものを書く)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q6. [開発者] 主な開発環境の OS

主に使っている開発の OS は、どれですか(その OS では、ローカルでテストを実行します。ほかの OS は CI で確かめます)。

- A. macOS
- B. Linux
- C. Windows
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q7. [開発者] 規則として確定するもの

CI やテストで、すでに強制されている次の制約を、「規則」として確定しますか。確定したものは、これからの作業で必ず守ります。

必ず守る(ALWAYS):
- 行カバレッジ 80% 以上を保つ。
- 最小の Rust(1.100)でビルドとテストが通ることを保ち、引き上げるときは README と一致させる。
- リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って、人が確認してから公開する。
- `cargo fmt` と `cargo clippy -D warnings` を通してからマージする。
- 変更は小さな単位に保ち、小さな単位でレビューする。

決してしない(NEVER):
- C のコンパイルを要する依存や、`-sys` クレートを、実行時の依存に加えない。
- `unsafe` を、runtime の `platform/unix.rs` の外に書かない(cli では禁止)。
- `cfg(unix)` と `cfg(windows)` を、`platform` の外に置かない。
- エラーの表示に、入力の値を含めない。ステートに、環境変数を書かない。
- テスト専用の補助バイナリを、リリースの成果物に含めない。

- A. すべて、規則として確定する
- B. 一部だけ確定する(「その他」に、外すものを書く)
- C. 規則としては確定しない
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
