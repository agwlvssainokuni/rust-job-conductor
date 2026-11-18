## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-18T22:25:46Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/cli/src/retry.rs > run_job_with_retry, the notice before wait | If the interrupt signal is raised after the job exits (exit code in the retry list) but before the wait starts, the retry notice ("retry in N seconds") is printed and the wait then returns Interrupted at once. The user sees a notice for a retry that never happens. Exit code and resume point are correct. | Optionally check `signal.signal_number()` before showing the notice, or accept and note it. Not blocking. | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo fmt --all -- --check | PASS (no output) | Shell-edited files are not damaged in formatting. |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS (Finished, 0 warnings) | Clean. |
| cargo test --workspace --locked | PASS: every "test result" line is ok, 0 failed, 1 ignored (existing) | Existing tests are green with the new code. |
| git diff 7c2d9d8 HEAD --stat (crates) | 18 files under crates/ | Matches the 18 paths in source-manifest.json exactly. |
| Removed lines in the crates diff | Only the changed import lists, the two `run_job` calls, `Level::Error =>` (now `Level::Error \| Level::Notice =>`) and the `pub use` line | No existing assertion changed. Test-side changes are additions only (fake `wait`, `waits`/`interrupt_wait`, one `with_all_events` entry). |
| Cargo.toml / Cargo.lock diff | empty | No new crate. |
| grep unsafe / cfg( in retry.rs, wait.rs, flow/retry.rs | no hits | No unsafe and no OS-specific code outside platform/. |
| License headers | Present on all new files checked (retry.rs x2, wait.rs, and the three new test files) | OK. |

Design check against BR2.1-BR2.16 (by reading the code and the test names):

- Count: `retries_done >= policy.max` gives Stop, so there are at most max+1 runs. Tests cover the boundary.
- Wait time: fixed uses interval. With backoff, the first wait is min(interval, cap), later waits are saturating `last*multiplier` capped. This matches BR2.3 as written. Tests cover the 10/20/40/60/60 series and overflow.
- Retry conditions: `is_retry_target` requires started, not interrupted, not start_failure, not signal-killed, and exit code in the list. A job's own 127 is retried only if listed. No policy gives Stop, which is the old behaviour (BR2.9).
- Interruption in a wait: returns an execution with started=false, interrupted=true and signal_number set. The existing classify and record path gives exit 128+n, resume at that job, and no error record. Tests cover the first and second wait, and `pipeline`.
- A signal raised before the wait is seen at the start of `wait_for`, and by the fake.
- State is written once with the final outcome. The retry loop never touches state, and a test asserts write counts.
- Display of the job result happens in the caller after the loop, once. `run` and `pipeline` both call the single `run_job_with_retry`.
- Notice: `Level::Notice` goes to stderr without -v. The text is a fixed sentence plus integers plus the id escaped by `escape_name`. English and Japanese are tested. Error-level events still go through `error_line`, so the `conductor: error:` convention is untouched. The `show` match arm was widened only for Notice.

Not verified: CI on Linux and Windows, and the claimed 799 total test count and 97.8% coverage (not re-measured).

### Summary

I found no defects against BR2.1-BR2.16. Format, lint and the full test suite pass locally, and the manifest, no-new-crate, no-unsafe and no-changed-assertion claims all hold. One Minor cosmetic edge case is noted and is not blocking.
