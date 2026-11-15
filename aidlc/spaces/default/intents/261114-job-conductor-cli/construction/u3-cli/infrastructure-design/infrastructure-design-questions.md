# Infrastructure Design(u3-cli)— 質問

u3-cli は、`conductor` の実行ファイルそのものです。サーバー、クラウドの資源、データベース、ネットワークの構成はなく、「インフラ」は、**配布(リリースの実行ファイルの作り方と置き場所)**と、**CI(GitHub Actions)**だけです。チームの実践(タグを付けると、CI が、3つの OS 向けのリリースの実行ファイルを自動でビルドする)に沿って案を作りました。配布の中身を決める3点を確認します。

## Q1. [配布] 実行ファイルの置き場所

- A. GitHub のリリース(Releases)に、実行ファイルとチェックサムを添付する。利用者は、そこからダウンロードして、PATH の通った場所に置く。crates.io へは、公開しない(推奨)
- B. A に加えて、crates.io にも公開し、`cargo install conductor-cli` でも入れられるようにする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [配布] 実行ファイルの対象(OS と CPU)

Linux には、実行ファイルが、OS の共有ライブラリ(glibc)に依存する形と、依存しない静的な形(musl)があります。要件は、外部のランタイムに依存しない単一のバイナリです。

- A. Linux x86_64(musl の静的リンク)、macOS の Apple シリコン(aarch64)と Intel(x86_64)、Windows x86_64(MSVC)の4つ(推奨)
- B. Linux x86_64(glibc)、macOS、Windows の3つ(Intel の macOS は、Apple シリコンの実行環境で動かす前提)
- C. A に加えて、Linux aarch64 も(5つ)
- X. Other (please specify)

[Answer]: A

## Q3. [配布] 配布の形式

- A. UNIX 系(Linux・macOS)は `.tar.gz`、Windows は `.zip` にまとめ、その中に実行ファイルと LICENSE を入れる。チェックサム(SHA-256)は、まとめたファイルごとに、1つのファイルにまとめて添付する(推奨)
- B. 実行ファイルそのものを、そのまま添付する(まとめない)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
