# 機能の設計の質問: u2-optional-env-file(無くてもよい `.env`)

出典: ユニットの定義(`unit-of-work.md` の U2)、要件(`requirements.md` の FR2.1〜FR2.5)、契約(`contract-summary.md` の C1、C3)、コンポーネントの一覧(`components.md` の CoreEnvFile、RuntimeEnvFile)、今のコード(`crates/core/src/definition/walk/env.rs`、`crates/core/src/envfile/mod.rs`、`crates/runtime/src/envfile.rs`)。

すでに決まっていること(質問しない): 表の形は `{ path, required }` で、文字列と混ぜて書ける(FR2.1)。`required = false` のファイルが無ければ読まずに進み、何も表示しない(FR2.2)。あるが読めない・大きすぎる・UTF-8 でない・書式の誤りは今までどおり 64(FR2.3)。`required` を省いた表と文字列は必ず要るファイル(FR2.4)。`--dry-run` と `graph` も同じ扱い(FR2.5)。ディレクトリなど通常のファイルでないものは「あるが読めない」として 64(FR2.3、今の `read_regular_file` の扱い)。

## Q1. [利用者] 同じファイルを、必ず要るものと無くてもよいものの両方で書いたとき

今は、同じパスを何度書いても 1 回だけ読みます。たとえば `[pipeline]` に `env_file = ".env.local"`(必ず要る)と書き、あるジョブに `{ path = ".env.local", required = false }` と書くと、同じファイルに 2 つの扱いが付きます。そのファイルが無いときにどうするかを決めます。

A. 必ず要る扱いを優先する。どこか 1 か所でも必ず要るなら、無いときは 64。両方の書き方は許す
B. 定義の誤り(64)にする。同じパスに違う `required` を書くこと自体を禁じる
C. 書いた場所ごとに別に扱う。必ず要る側の層(ここでは `[pipeline]`)は 64、無くてもよい側の層は読まずに進む(結果としては A と同じく 64)
X. Other (please specify)

[Answer]: A

## Q2. [利用者] デモの見せ方

機能の最後に README とデモを更新します(FR8.1)。環境変数のデモは `demo/env.toml` と Windows 版です。

A. `demo/env.toml`(Windows 版も)の `env_file` に、置いていないファイルを `{ path = "local.env", required = false }` で足し、無くても動くことを見せる。デモのテストは今の出力のまま通る
B. A に加えて、そのファイルを置いたときの動きも、デモの場面として足す(デモのファイルとテストが増える)
X. Other (please specify)

[Answer]: A
