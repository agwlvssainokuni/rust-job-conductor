# セキュリティの設計: u6-attempt-times

出典: セキュリティの要件(`security-requirements.md` の NFR6.1〜NFR6.3、NFR5.1)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3、5)、契約(`contract-summary.md` の C4)。

## 1. ステートに書くもの(NFR6.1)

- 書き出しの `job_table` に足すのは `attempt` の配列だけ。各表は `started_at` と、あれば `ended_at` の 2 つの文字列(`format_rfc3339` の結果。数字、`-`、`:`、`T`、`Z` だけ)。
- 読み込みは知らないキーを `UNKNOWN_KEY` で拒否するので、ほかの値が紛れ込んだステートは読まない。
- 確かめ方: `nfr_content.rs` を「`started_at`・`ended_at` のキーと、RFC 3339 の形の値だけを許す」に直す。コマンド、環境変数、出力の目印が書かれないことは今までどおり確かめる。

## 2. 表示(NFR6.2)

- 新しい 3 つの理由の文(`NO_ATTEMPT`、`BAD_TIME`、`UNFINISHED_ATTEMPT`)は `reason` の固定の文。時刻、ジョブ ID、パスの中身を入れない。表示は今の壊れたステートと同じ(種類の文、理由の文、`reset` の案内)。
- `watch` の所要時間の列は、時刻からの計算の結果だけを出す(時刻そのものは出さない)。
- 確かめ方: ステートの読み込みのテストで、壊れたステートに入れた目印の値(ジョブ ID、時刻)が表示に出ないこと。

## 3. 印のファイルを作らない(NFR6.3)

- 動いている最中の見分けは、最後の試行に `ended_at` が無いことだけで行う(Q1)。ロック、PID、ほかのファイルは作らない。

## 4. コードの規則(NFR5.1)

- `unsafe` と `cfg` を書かない。時刻は U5 の `now()`(std の `SystemTime`)。`nfr_source.rs` で確かめる。
