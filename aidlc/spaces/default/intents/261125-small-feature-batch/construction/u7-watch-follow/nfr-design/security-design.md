# セキュリティの設計: u7-watch-follow

出典: セキュリティの要件(`security-requirements.md` の NFR6.1〜NFR6.4、NFR5.1)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、3.1、4)、契約(`contract-summary.md` の C5)。

## 1. 引数の誤り(NFR6.1)

- `--interval` の誤りは、今の使い方の誤りの種類と固定の文(英日)で出し、入力の値を入れない。終了コードは 68。
- 確かめ方: `args_grammar.rs` と実バイナリのテストで、目印の値(`--interval 99999abc` など)が表示に出ないこと。

## 2. 読むだけ(NFR6.2)

- 継続表示はステートを読む関数だけを呼ぶ。書き込み・削除の差し替え口を呼ばない。
- 確かめ方: 流れのテストで偽の書き込みと削除が 0 回。

## 3. 制御の文字(NFR6.3)

- 画面を消す文字列(`ESC[H ESC[2J`)は、`stdout_is_terminal()` が真で、`platform` の「画面を消せる」判定が真のときだけ出す。判定は `platform/mod.rs` に 1 つの関数として置き、UNIX 系は真、Windows は偽(`cfg` は `platform` の中だけ)。
- 確かめ方: 表示のテスト(端末と端末でない場合)と、実バイナリのテスト(パイプの出力に `ESC` が無い)。

## 4. ステートに書く値(NFR6.4)

- リトライの待ちの直し(`reliability-design.md` の 2)は、書く時刻を変えず、書く時を変えるだけ。`nfr_content.rs` を変えずに通す。

## 5. コードの規則(NFR5.1)

- `unsafe` を書かない。`cfg` は `platform` の中だけ。`nfr_source.rs` で確かめる。
