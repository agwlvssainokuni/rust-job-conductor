## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T15:46:37Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u5-log-dir/nfr-design/security-design.md > 1. 安全な名前 項目 3(Windows の予約名) | 予約名の判定が「置き換えた後の名前(`safe_log_stem` の出力)全体が `CON` などと一致するとき」だけになっている。実際のファイル名は `<stem>.<時刻>.<番号>.log` で、Windows は最初の `.` より前の部分が予約名なら拡張子が付いていても装置として扱う。このため ID が `NUL.x` や `CON.build` のとき、stem は予約名と一致せず、できるファイル名 `NUL.x.20261125T...Z.1.log` が装置名になる。ジョブ ID は空でなく `@` で始まらなければ何でも書けるので(`definition/walk/mod.rs` の `cross_check`)、この入力は起こりうる。NUL のときはログが黙って捨てられ、NFR6.3 の「名前が安全であること」も満たせない。 | 予約名の判定を「最初の `.` より前の部分(大文字小文字を問わない)が予約名と一致するか」に変える。判定の対象は stem ではなく、最終のファイル名の先頭部分とする。確かめ方の表に `NUL.x`→`_NUL.x`、`con.build`→`_con.build` の対を足す。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u5-log-dir/nfr-design/security-design.md > 1. 安全な名前(名前の長さ) | ファイル名の長さの上限(多くの FS で 255 バイト)を扱っていない。ID は長さの制限が無く、`.<時刻>.<番号>-<n>.log` を足すと上限を超えうる。超えたときの結果(127 になる)は暗黙の挙動で、設計に書かれていない。 | stem の最大の長さ(バイトで)を決めて切り詰めるか、長すぎるときは「作れない(BR5.1、127)」になると明記して、そのテストを足す。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u5-log-dir/nfr-design/security-design.md > 1. 安全な名前 冒頭と項目 2 | 「次の 2 つを足す」とあるが、列挙は 4 項目あり、項目 1 は機能の設計の BR3.2 をほぼ繰り返している(`:`、`*` なども含む)。項目 2 の「末尾の `.`」は、名前が `.<時刻>` で続くため最終のファイル名では末尾に来ず、効果の説明が無い。何が BR3.2 への追加で何が再掲かが読み取りにくい。 | 「BR3.2 に足す」ものと再掲を分けて書く。末尾の `.` と空白の置き換えは、stem を単体の関数として検査するための規則であることを書く。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u5-log-dir/nfr-design/reliability-design.md > 2. 読み取りのスレッドの止め方 | 中断(シグナル)で子が終わった場合に、BR4.5 の最大 2 秒の待ちを同じように行うのか、待ちを省くのかが書かれていない。孫がパイプを持つ状態で中断すると、終了が最大 2 秒遅れる。 | 中断のときも 2 秒の待ちを通る(または省く)ことを一文で決め、`signals_real.rs` の追加の確かめ方に期待する所要時間を書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (この段階の検証ツールの指定なし) | 実行せず | 上流の `functional-spec.md`、`rules.md`、`contract-summary.md` の C6、`definition/walk/mod.rs` のジョブ ID の検査、`deps.rs` の `run_job` を手で突き合わせた。cfg は `platform` 内、新しい依存なし、失敗の表示にジョブの出力を含めない、の 3 点は設計の範囲で守られている。 |

### Summary

設計は要件と機能の設計にほぼ対応しており、0600 の作成時指定、「無いときだけ作る」、固定の枠、待ち切りの止める印、失敗の伝え方(LogOutcome)は実装できる水準にある。Major は 1 件(Windows の予約名が拡張子付きで素通りする)で、規則どおり READY とするが、コード生成の前に直すのが望ましい。
