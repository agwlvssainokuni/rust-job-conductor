# 要件分析の質問(Windows 向けデモ)

依頼: 「Windows用のデモをつくってほしい。」

現状: `demo/run-demo.sh` と `demo/scripts/*.sh` は `sh` 前提で、Windows では動かない(`demo/README.md` にもそう書かれている)。`conductor` はシェルを通さずにジョブを起動するため、Windows 用には、ジョブのスクリプトと定義ファイルと実行スクリプトを、Windows のものとして用意する必要がある。

## Q1. [開発者] Windows 用のジョブスクリプトと実行スクリプトは、何で書きますか?

理由: 書く言語によって、動かすのに必要な環境が決まる。Windows 10 / 11 には、Windows PowerShell 5.1(`powershell.exe`)が最初から入っている。

A. PowerShell(`.ps1`)で書く。Windows PowerShell 5.1 だけで動くようにする(追加のインストールは要らない)
B. cmd(`.cmd`)で書く。もっとも素朴で、どの Windows でも動く
C. PowerShell 7(`pwsh`)を前提に書く(利用者が別途インストールする)
X. Other (please specify)

[Answer]: B

## Q2. [開発者] 新しいファイルは、どこに置きますか?

理由: 今の UNIX 向けのデモ(`demo/run-demo.sh`、`demo/conductor.toml`、`demo/scripts/*.sh`)は、そのまま残す。置き場所によって、`conductor.toml` を 2 つにするかどうかも決まる。

A. `demo/windows/` に、Windows 用を一式まとめて置く(`run-demo.ps1`、`conductor.toml`、`scripts/*.ps1`)
B. `demo/` の直下に、名前を変えて並べる(`run-demo.ps1`、`conductor.windows.toml`、`scripts/*.ps1`)
X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 利用者への案内は、どこまで更新しますか?

理由: `README.md` は、CI の文書チェックが必要な語句を見ている。変更する範囲が広いほど、確認すべきことが増える。

A. `demo/README.md` だけを更新する(UNIX 版と Windows 版の使い方を並べて書き、「Windows では動かない」の一文は書き換える)
B. `demo/README.md` に加えて、ルートの `README.md` にも、Windows のデモへの案内を 1 行足す
X. Other (please specify)

[Answer]: A
