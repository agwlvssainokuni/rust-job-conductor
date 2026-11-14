# Functional Design(u3-cli)— 質問

u3-cli は、`conductor` の実行ファイルそのものです。コマンドとオプションの受け付け、1回の実行の流れの組み立て(u1-core・u2-runtime・u4-converter を呼ぶ唯一の調整役)、シグナルの受け取り、表示(Reporter)を受け持ちます。これまでの Unit の設計で、この Unit に残した契約の更新も、ここで扱います。決める必要がある3点を確認します。

## Q1. [定義ファイル] ジョブを並べる表の名前

Rough/Refined Mockups の定義ファイルの例(確認済みの回答に基づく)は、`[[job]]` と `[[job.transitions]]` です。一方、u1-core の実装は、契約の仮の名前に従って、`[[jobs]]` と `[[jobs.transitions]]` を受け付けます。利用者が書くファイルの形なので、どちらかに揃えます。

- A. モックアップの `[[job]]` / `[[job.transitions]]` に揃える。u1-core の検査(受け付けるキー)とテスト・例を、`job` に直す(推奨)
- B. 実装の `[[jobs]]` / `[[jobs.transitions]]` のままにして、モックアップと文書を直す
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [convert] 変換結果の出力先

`conductor convert <file> --to <format>` の出力先は、設計で決めることになっています。

- A. 標準出力だけに書く。ファイルへは、シェルのリダイレクト(`> out.toml`)で書く。出力先のオプションは足さない(推奨)
- B. 標準出力を既定にしつつ、`-o <パス>` / `--output <パス>` で、ファイルへも書けるようにする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [終了コード] コマンドの使い方の誤り(オプションの誤りなど)の終了コード

`conductor` 自身のエラーには、定義の誤り 64、ステートの誤り 65、拒否 66、変換の誤り 67 を決めてあります。コマンドの使い方の誤り(知らないコマンド、必須の引数がない、`run` 以外に `-v` を付けた、など)は、まだ決めていません。引数の解釈のライブラリの既定は 2 ですが、ジョブ自身の終了コード 2 と区別がつきません。

- A. 68 を使う(使い方の誤り専用の値を、新しく足す)(推奨)
- B. 64 にする(定義の誤りと同じ)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
