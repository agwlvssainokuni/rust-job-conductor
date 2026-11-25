# セキュリティの要件: u6-attempt-times

出典: 要件(`requirements.md` の FR6.3、FR6.8、NFR4、NFR5、NFR6)、機能の仕様(`functional-spec.md` の 3、5)、ビジネスルール(`rules.md` の BR2.2、BR3.1)、契約(`contract-summary.md` の C4)、技術スタック(`technology-stack.md`)。

ステートは利用者の手元に残るファイルなので、時刻のほかの情報を増やさない。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR6.1 | ステートに新しく書くのは、試行ごとの `started_at` と `ended_at` だけ。コマンド、環境変数、ジョブの出力、終了コードの履歴、PID を書かない | `nfr_content.rs` を、この 2 つのキーと時刻の値だけを許すように直し、ほかの目印の値が書かれないことを確かめる | FR6.3、BR2.2 |
| NFR6.2 | 新しい 3 つの壊れたステートの理由の文は固定の文で、ステートの値(時刻、ジョブ ID)やパスの中身を含めない | `nfr_security.rs` またはステートの読み込みのテストで、目印の値が表示に出ないこと | FR6.8、BR3.1 |
| NFR6.3 | 実行中を示す印のファイルやロックを作らない(Q1 の A) | 新しいファイルを作らないこと(`state_location` 以外にファイルが増えない) | Q1 |
| NFR5.1 | `unsafe`・`cfg` を足さない | `nfr_source.rs` | NFR5 |
