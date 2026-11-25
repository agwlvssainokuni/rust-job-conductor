# セキュリティの要件: u2-optional-env-file

出典: 要件(`requirements.md` の NFR4、NFR5、NFR6)、機能の仕様(`functional-spec.md` の 6)、ビジネスルール(`rules.md` の BR1.2、BR2.3、BR2.4)、契約(`contract-summary.md` の C1、C3)、技術スタック(`technology-stack.md`)、project.md の Forbidden。

`.env` は秘密を含みうる。このユニットは「無くてもよいファイル」を足すだけで、中身の読み方と扱い(値を表示しない、`Debug` は伏せ字)は変えない。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR6.1 | 表の `env_file` の誤り(型、知らないキー、空の path)の理由の文は固定の文で、`path` の値や `required` の値を含めない(キー名だけは今の規則どおりエスケープして出す) | 新しい誤りの表示に、目印の値が出ないことを `nfr_security.rs` で確かめる | NFR6、BR1.2 |
| NFR6.2 | 無くてもよいファイルが無かったことは、どこにも表示しない(パスも出さない) | 無いときの標準出力・標準エラーが空(今と同じ)であることを実バイナリのテストで確かめる | NFR6、BR2.3 |
| NFR6.3 | 無くてもよいファイルでも、あるのに読めない・通常のファイルでない・大きすぎるものは 64 で止め、黙って読み飛ばさない(取り違えで秘密が欠けた状態のまま動かない) | 各場合が 64 になる単体テストと実バイナリのテスト | NFR6、BR2.4 |
| NFR5.1 | `unsafe` と `cfg` を足さない。読み取りの失敗の見分けは今の `read_regular_file` の結果(Missing、Unreadable、TooLarge)だけで行う | `nfr_source.rs`(runtime、cli)と core の `forbid(unsafe_code)` | NFR5 |
| NFR4.1 | 新しい依存を加えない | `Cargo.lock` が変わらない。CI の `build-check` | NFR4 |

## 脅威の検討

| 脅威 | 対策 |
|---|---|
| 必ず要るつもりのファイルを `required = false` と書き間違え、秘密が欠けたまま動く | 既定は必ず要る(BR1.3)。同じファイルをどこかで必ず要るとすれば、必ず要るが優先(BR2.1、BR2.2)。README で既定と優先を示す |
| 名前付きパイプや特殊なファイルを置いて読みを止める | 今の `read_regular_file` が通常のファイルでないものを開かずに Unreadable にする。`required = false` でも同じ(NFR6.3) |
| 表示から `.env` のパスや中身が漏れる | NFR6.1、NFR6.2 |
