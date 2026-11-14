## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Critical | crates/u4-converter/src/resolve.rs:154-160 (count_node), :195 (scalar_value call in enter); crates/u4-converter/src/scalar.rs:134-148 | NFR4.3 ("展開でメモリを使い切らない") is enforced only as a node count (100,000), not as a size in bytes. Every alias visit clones the whole scalar text into a new Value::Str. A scalar anchor holding a large string, aliased many times, uses far more memory than the node cap implies. I ran this against the crate: a 35 KB input (a 20,000-byte anchored string aliased 5,000 times) converted successfully to a 100 MB output and reached 324 MB RSS. A 1 MiB input (one ~500 KB string plus ~100,000 `*s` aliases, all under the 100,000-node limit) would need about 50 GB. The same amplification applies to keys: deliver() clones the pending key (resolve.rs:263), then entries/seen clone it again. The design (security-design §1.4) says a large map referenced many times must not use time and memory outside the 100,000 limit, and the summary and traceability.json mark NFR4.3 OK. The test `alias_bomb_is_too_large_without_exhausting_memory` (read_resolve.rs:139) only aliases 1-byte scalars. The 30 s timeout only detects hangs, so no test can catch this. | Add a byte budget to Resolve. Sum the bytes of every string produced (scalar values and keys) and return `tooLarge` when the sum exceeds a fixed cap, for example a small multiple of the 1 MiB input limit. Check the budget before the clone. Count bytes in the same place as the node count, including merge-discarded items. Add tests: a large anchored string aliased until it exceeds the cap gives `tooLarge`, the boundary on each side is covered, and a large anchored key is covered too. Correct the NFR4.3 entry in traceability.json and code-summary.md. | Resolved (reviewer) |
| R-02 | Major | crates/u4-converter/src/intake.rs:37-60 (metadata, then size check, then File::open) | BR1.12 fixes the order as 2) unreadable (no permission, not a regular file) before 3) tooLarge. The code checks the size from `metadata` first and only opens the file afterwards. A file over 1 MiB that cannot be opened (mode 000) returns `tooLarge`, not `unreadable`. The permission path is never run in a test, so nothing detects this. | Open the file first and take the size from the handle's metadata, or map the open failure before the size check. Then check size and bounded read. Add a UNIX-only test (`#[cfg(unix)]`) for a mode-000 file under 1 MiB and one over 1 MiB, both expecting `unreadable`. | Resolved (reviewer) |
| R-03 | Minor | crates/u4-converter/src/intake.rs:184-190 (io_errors_map_to_fixed_reasons); code-summary.md deviation 4 | The real permission-denied test was replaced by a mapping test, citing NFR3.1. NFR3.1 forbids OS-specific branches in the product code, not a `#[cfg(unix)]` test. The replacement leaves the Intake permission path and the BR1.12 2-before-3 order (previous finding) with no coverage. | Restore a `#[cfg(unix)]` permission test, or record in the summary that the path is untested and why it was accepted. | Resolved (reviewer) |
| R-04 | Minor | crates/u4-converter/src/lib.rs:55-56, :123; crates/u4-converter/src/error.rs:17, :138, :53-60 | The public API is wider than contract C4 (`convert` with its input and output). `pub mod error` also exposes `sanitize`, `MAX_KEY_DISPLAY_CHARS`, and `ConversionErrorKind::{code, ALL}`. The summary lists only `testing` and `intake` as additions. u3-cli can start depending on these. | Make `sanitize` and `MAX_KEY_DISPLAY_CHARS` `pub(crate)`, or list every extra public item in the summary as an intended addition to C4. Keep `testing` and `intake` documented as is. | Resolved (reviewer) |
| R-05 | Minor | crates/u4-converter/src/resolve.rs:733-741 (duplicate key found in Resolve) | A duplicate key that only appears after alias or merge expansion is reported as `syntax` at step 8. BR1.12 lists duplicate keys under step 5 (before multiple documents and rootNotMap). A multi-document or non-map input with such a duplicate can return a different kind than the rule says. The summary notes the Resolve-time duplicate, but not that it contradicts the order. | State in the rules or the summary that duplicates created by expansion are ordered at step 8, or move the check. The first option is cheaper, and the second is not practical without expanding first. | Resolved (reviewer) |
| R-06 | Minor | crates/u4-converter/src/yaml_write.rs と crates/u4-converter/src/toml_write.rs の文字列の引用符(`double_quoted` と toml の書き出し) | 予算の 16 MiB は、展開後の文字のバイト数の合計で、書き出した結果の大きさではない。制御文字は、書き出しで `u0001` の形(6 バイト)になる。80 KB 程度の入力(`x01` を 20,000 個書いた文字列のエイリアスを 800 回参照)で、出力が約 96 MB、最大メモリが約 136 MB になった。メモリの最大は有限(予算の約 8 倍)で、前回の Critical のような青天井ではない。ただし、`lib.rs` の「予算の数倍」の説明は、書き出しの膨らみを含んでいない | `lib.rs` の「展開の上限」の節に、出力が予算の最大 6 倍ほどになりうること(測定値: 約 136 MB)を書く。または、出力の大きさにも上限を置くかを、16 MiB の確認と同時に、人が決める | New |

> R-01 Reviewer note: `crates/u4-converter/src/resolve.rs` の `charge_bytes` が、訪れたすべてのスカラー(キー、値、エイリアスの参照のたびの複製、マージで捨てられる項目、`<<` のキーの 2 バイト)の文字のバイト数を、複製・確保の前に足し、16 MiB を超えたら `tooLarge`(専用の reason)で打ち切る。前回の入力(20,000 バイトのアンカーを 5,000 回参照、約 35 KB)は、`l[837]` で即座に `tooLarge`(7 ms、最大メモリ約 21 MB)になった。16 MiB の直前(20,000 バイト × 800 回)は受理され、出力 16 MB、最大メモリ約 55 MB。キーの複製(`deliver` の `seen.insert(k.clone())`、`finish_frame`、`path()`)は予算の定数倍に収まる。テスト(境界の 16 MiB ちょうど、マージ経由、大きな文字列の参照)と、`cargo test`(全 112 件成功)、`clippy -D warnings`、`fmt --check` は通る

> R-02 Reviewer note: `intake.rs` の `read_input` は、先にファイルを開き(権限の誤りは `unreadable`)、開いたハンドルの `metadata` で大きさを調べる。1 MiB 超でモード 000 のファイルは `unreadable`(権限)になることを、`mode_000_file_over_1_mib_is_unreadable_not_too_large` と `mode_000_file_is_unreadable` が確かめる(権限が効かない環境では確認を飛ばす)。BR1.12 の 1〜4 の順になった

> R-03 Reviewer note: 権限の実テスト(Unix のみ、`permissions_enforced` で root などを除く)を戻し、`io_reason` の対応表のテストは補助として残した

> R-04 Reviewer note: `lib.rs` の公開は `convert`、`convert_text`、`ConversionRequest`、`ConvertedText`、`TargetFormat`、`ConversionError`、`ConversionErrorKind` と、`#[doc(hidden)] testing` だけ。ほかは `pub(crate)`。`code-summary.md` の記述も合っている

> R-05 Reviewer note: 挙動は変わらず、エイリアスやマージの展開でだけ生じる重複キーは、BR1.12 の 8 の位置で `syntax` になる。この違いは `lib.rs`、`resolve.rs`、`code-summary.md` に書かれ、人が見られる。設計の流れ(リテラルの重複は 5)とは種類が同じで、判明する段階だけが違うため、許容できる

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test -p conductor-converter | PASS(53、8、10、12(1 無視)、29 件) | 予算の境界と、権限のテストを含め、すべて成功 |
| cargo clippy --all-targets -D warnings | PASS | 警告なし |
| cargo fmt --check | PASS | 差分なし |
| 自前の調査(別の場所に作った試験用のバイナリ。リポジトリは変更なし) | 35 KB の入力: `tooLarge` で 7 ms、最大メモリ約 21 MB。予算の直前: 約 55 MB。制御文字で書き出しが膨らむ入力: 出力約 96 MB、約 136 MB | R-01 の修正は有効。残るのは、上の Minor のみ |

### Summary

展開後の文字のバイト数の予算(16 MiB)で、前回の Critical の入力は、確保の前に打ち切られるようになり、R-01 から R-05 はすべて直った。残る Minor は、書き出しの膨らみ(最大約 136 MB)の記述だけで、16 MiB の値自体は、code-summary のとおり人の確認が要る。
