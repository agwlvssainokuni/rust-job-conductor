## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T22:13:48Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Critical | crates/u4-converter/src/resolve.rs:154-160 (count_node), :195 (scalar_value call in enter); crates/u4-converter/src/scalar.rs:134-148 | NFR4.3 ("展開でメモリを使い切らない") is enforced only as a node count (100,000), not as a size in bytes. Every alias visit clones the whole scalar text into a new Value::Str. A scalar anchor holding a large string, aliased many times, uses far more memory than the node cap implies. I ran this against the crate: a 35 KB input (a 20,000-byte anchored string aliased 5,000 times) converted successfully to a 100 MB output and reached 324 MB RSS. A 1 MiB input (one ~500 KB string plus ~100,000 `*s` aliases, all under the 100,000-node limit) would need about 50 GB. The same amplification applies to keys: deliver() clones the pending key (resolve.rs:263), then entries/seen clone it again. The design (security-design §1.4) says a large map referenced many times must not use time and memory outside the 100,000 limit, and the summary and traceability.json mark NFR4.3 OK. The test `alias_bomb_is_too_large_without_exhausting_memory` (read_resolve.rs:139) only aliases 1-byte scalars. The 30 s timeout only detects hangs, so no test can catch this. | Add a byte budget to Resolve. Sum the bytes of every string produced (scalar values and keys) and return `tooLarge` when the sum exceeds a fixed cap, for example a small multiple of the 1 MiB input limit. Check the budget before the clone. Count bytes in the same place as the node count, including merge-discarded items. Add tests: a large anchored string aliased until it exceeds the cap gives `tooLarge`, the boundary on each side is covered, and a large anchored key is covered too. Correct the NFR4.3 entry in traceability.json and code-summary.md. | New |
| R-02 | Major | crates/u4-converter/src/intake.rs:37-60 (metadata, then size check, then File::open) | BR1.12 fixes the order as 2) unreadable (no permission, not a regular file) before 3) tooLarge. The code checks the size from `metadata` first and only opens the file afterwards. A file over 1 MiB that cannot be opened (mode 000) returns `tooLarge`, not `unreadable`. The permission path is never run in a test, so nothing detects this. | Open the file first and take the size from the handle's metadata, or map the open failure before the size check. Then check size and bounded read. Add a UNIX-only test (`#[cfg(unix)]`) for a mode-000 file under 1 MiB and one over 1 MiB, both expecting `unreadable`. | New |
| R-03 | Minor | crates/u4-converter/src/intake.rs:184-190 (io_errors_map_to_fixed_reasons); code-summary.md deviation 4 | The real permission-denied test was replaced by a mapping test, citing NFR3.1. NFR3.1 forbids OS-specific branches in the product code, not a `#[cfg(unix)]` test. The replacement leaves the Intake permission path and the BR1.12 2-before-3 order (previous finding) with no coverage. | Restore a `#[cfg(unix)]` permission test, or record in the summary that the path is untested and why it was accepted. | New |
| R-04 | Minor | crates/u4-converter/src/lib.rs:55-56, :123; crates/u4-converter/src/error.rs:17, :138, :53-60 | The public API is wider than contract C4 (`convert` with its input and output). `pub mod error` also exposes `sanitize`, `MAX_KEY_DISPLAY_CHARS`, and `ConversionErrorKind::{code, ALL}`. The summary lists only `testing` and `intake` as additions. u3-cli can start depending on these. | Make `sanitize` and `MAX_KEY_DISPLAY_CHARS` `pub(crate)`, or list every extra public item in the summary as an intended addition to C4. Keep `testing` and `intake` documented as is. | New |
| R-05 | Minor | crates/u4-converter/src/resolve.rs:733-741 (duplicate key found in Resolve) | A duplicate key that only appears after alias or merge expansion is reported as `syntax` at step 8. BR1.12 lists duplicate keys under step 5 (before multiple documents and rootNotMap). A multi-document or non-map input with such a duplicate can return a different kind than the rule says. The summary notes the Resolve-time duplicate, but not that it contradicts the order. | State in the rules or the summary that duplicates created by expansion are ordered at step 8, or move the check. The first option is cheaper, and the second is not practical without expanding first. | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test -p conductor-converter --locked | PASS: 47 + 8 + 10 + 12 (+1 ignored helper) + 26 = 103 run | Tests pass, but none checks the byte size of expanded strings, so they cannot detect the Critical finding. |
| cargo fmt --all -- --check | PASS (no output) | Clean. |
| cargo clippy -p conductor-converter --all-targets --locked -- -D warnings | PASS | Clean. |
| Probe crate (scratchpad, path dependency): alias of a 20,000-byte string x 5,000 | Converted OK, 100 MB output, 324 MB RSS | Reproduces the Critical finding. |
| Probe crate: TOML dotted keys, table headers, and `[[...]]` with 40 to 100,000 parts | `syntax` for every case, no crash | Confirms the TOML library's recursion limit and that the Drop of its tree is safe. |
| Probe crate: anchor redefinition, `<<` with own key, `!!float`/`!!int` tags, `<<`/yes/on/""/multi-line quoting both ways | Results as expected | BR1.2, BR1.7 and BR1.11 behave as designed. |
| Dependency `rust-version` fields in the local registry (saphyr-parser 0.3.4, getrandom 0.4.3, serde_spanned 1.1.3, syn, thiserror, winnow) | All declare 1.85 or lower | Consistent with the MSRV 1.85 claim. A real `cargo +1.85` build was not run (stable only, as the summary says). |

### Summary

The design is followed closely: the explicit stacks, the depth limits (32 on read and after expansion), the node count (100,000 accepted, 100,001 `tooLarge`), the quoting and float round trips, and the sorted TOML key order (allowed by BR1.3 and security-design §2, and flagged honestly) all hold up under probing. The blocker is that NFR4.3 limits nodes, not bytes: a small file with one large aliased string exhausts memory, and the existing test only uses 1-byte scalars. The BR1.12 order of unreadable before tooLarge is also violated for unopenable large files.
