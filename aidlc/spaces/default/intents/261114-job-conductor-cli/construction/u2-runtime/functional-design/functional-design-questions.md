# Functional Design(u2-runtime)— 質問

u2-runtime は、ジョブの直接起動とシグナルの転送(JobRunner)、ステートの保持・照合・初期化(StateStore)を受け持ちます。要件と契約(C3、E3)で決まっていないが、動きを左右する4点を確認します。

## Q1. [ステート] ステートファイルの名前と場所

AC4.1.1 は「実行ディレクトリの近傍」と定めています。別の定義ファイル(`--config` で指定)に、別のステートが対応する必要があります。

- A. 定義ファイルと同じディレクトリに、`<定義ファイル名>.state`(例: `conductor.toml.state`)を置く。定義ファイルのパスで、ステートが決まる(既定の `conductor.toml` なら、実行ディレクトリに置かれる)(推奨)
- B. 実行ディレクトリの `.conductor/` の下に、定義ファイルの絶対パスから作った名前で置く(定義ファイルの場所が読み取り専用でも使える。名前は人が読みにくい)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [ステート] ステートファイルの形式

契約 E3 は「書式の版つき」とだけ定めています。

- A. TOML にする。定義ファイルと同じ形式で、人が読める。`toml` のライブラリを、u1-core・u4-converter と共有する(推奨)
- B. JSON にする(別のライブラリが増える)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [起動] コマンド文字列の分け方

定義の `command` は、1つの文字列で、コマンド名と引数に分けて、シェルを介さずに起動します。引用符で囲んだ引数(例: `--msg='hello world'`)の分け方を決めます。

- A. 3つの OS で同じ規則にする。空白で区切り、`'...'` と `"..."` で空白を含む引数を書ける。`"..."` の中は `\"` と `\\` だけ解釈する。変数の展開、`*` の展開、`|` や `;` の解釈はしない。閉じていない引用符は、定義の誤りにする(推奨)
- B. OS ごとの規則にする(Windows は Windows の規則、それ以外は POSIX の規則)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q4. [シグナル] Windows でのシグナルの扱い

Windows には、UNIX のシグナルがありません。「`conductor` が受け取ったシグナルをジョブにも転送する」(AC2.5.6)と、「シグナルで終了したら 128 + 番号」(AC2.1.5)を、Windows でどうするかを決めます。

- A. Windows では、Ctrl+C(SIGINT 相当、番号 2)だけを対象にする。受け取ったら、ジョブに転送(同じコンソールのジョブには、OS が配る)し、終了を待ち、エラーとして 130 で終了する。ジョブが、異常な終了コード(例外による終了)で終わった場合は、シグナルではなく、そのままの終了コードとして扱う(推奨)
- B. Windows では、シグナルの転送と、シグナルによる終了の扱いを、行わない(中断された場合は、ジョブの終了を待つだけ)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
