# セキュリティの要件: u3-unset-env

出典: 要件(`requirements.md` の FR3.2、FR3.3、NFR4、NFR5、NFR6)、機能の仕様(`functional-spec.md` の 4)、ビジネスルール(`rules.md` の BR2.1、BR2.2、BR3.2)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)。

`unset_env` は、秘密(トークン、プロキシの設定など)をジョブに渡さないための機能である。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR6.1 | 消すと書いた名前は、親の環境にあれば、ジョブの環境に確実に無い(UNIX 系は完全一致、Windows は大文字小文字を区別しない) | 実バイナリで、親に置いた変数がジョブの環境に無いことを確かめる(UNIX と Windows の CI) | FR3.2、BR3.2 |
| NFR6.2 | 重なりの誤り(`env`、`.env`)の表示に、値を含めない。新しい 2 つの理由の文は固定の文 | `nfr_security.rs` で、目印の値が表示に出ないこと | NFR6、BR2.1、BR2.2 |
| NFR6.3 | `--dry-run` は消す名前だけを出し、親の環境の値を読まない・出さない | `--dry-run` の出力に親の値が出ないこと | NFR6 |
| NFR5.1 | `unsafe`・`cfg` を足さない。消す仕組みは std の `Command::env_remove`(OS ごとの違いは std に任せる) | `nfr_source.rs` | NFR5 |
| NFR4.1 | 新しい依存を加えない | `Cargo.lock` が変わらない | NFR4 |
