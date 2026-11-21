## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-21T22:23:30Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/core/src/flow/decide.rs > `status_at` and rules 5 and 6 of `decide_before_run` | `status_at` returns the record at the same position when its job_id matches, and otherwise falls back to `record_of` (first match). The result equals the old logic for every state that is in definition order, has fewer records, or has none. It can differ only when job_states holds an earlier duplicate of the same job_id, with a different status, before the position. The state consistency check (set and order of job IDs against the definition, crates/runtime/src/state/consistency.rs) excludes this for real input. The existing unit test covers a missing record ("記録のないジョブは_pending_と同じ") but no test covers job_states in a different order or with duplicates. A regression is implausible for real runs, but the equivalence is only argued, not tested. | Add a small equivalence test: for a handful of states (reordered, missing, with a stray id), compare `decide_before_run` against a copy of the old `record_of`-based logic, or record in the doc comment that the function assumes a state in definition order. | New |
| R-02 | Minor | crates/cli/tests/nfr_robust.rs > the assertion "長い入力は切って表示する" (2,000 raised to 3,000) | The limit was raised because the usage list grew. The test still catches an echo of the 10,000-character input, so no regression is hidden. But the ceiling is now 50% looser than needed and no longer states how much of the output is the fixed usage text. | Tighten it to the measured fixed usage length plus a margin, or assert that the output does not contain the long input. | New |
| R-03 | Minor | crates/cli/tests/windows_demo.rs and demo/windows/run-demo.cmd (cfg(windows) test) | The Windows demo test was changed (43 lines added, 1 removed) but cannot run on this macOS host. It is verified only by the CI `test` matrix. CRLF is preserved (`git ls-files --eol`: i/crlf w/crlf; the diff from c6328f5 is 38 insertions only, no deletions), so the line-ending requirement is met. | Confirm the windows-latest CI run is green before relying on scene 13. State in the summary that it has not been run locally. | New |
| R-04 | Minor | crates/core/src/plan/simulate.rs > `plan_pipeline` `Decision::Reject` branch and `apply_result` | The summary calls the Reject branch unreachable. In practice this holds: every earlier job is either run (success or warning) or stopped, and the state consistency check requires a record for every job. It is still reachable if a record is missing, because `apply_result` silently skips updating a missing record and the next job then sees an unsettled predecessor. The code returns a rejection (66) instead of panicking, which is the safe behaviour. | Soften the wording to "unreachable for a state that passed the consistency check". Optionally add a test that feeds a state with a missing record and expects 66. | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test --workspace --locked | PASS: 972 passed, 0 failed across 85 result lines | The tests for this unit (plan_*, flow_dry_run, binary_dry_run, plan_report, docs_dry_run) and the existing suite are green. `flow_dry_run.rs` runs the real pipeline loop with a fake Runtime and compares the started jobs with the plan's will_run list, so the equivalence claim is not a tautology. |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | No warnings. |
| git diff f2537b8 HEAD on Cargo.toml and Cargo.lock | No changes | No new dependencies. |
| grep for unsafe, cfg(unix) and cfg(windows) in plan/, dry_run.rs and plan_text.rs | No hits | Project rules respected. |
| git ls-files --eol and git diff --stat on run-demo.cmd | CRLF kept; 38 lines added, none removed since c6328f5 | Line endings preserved. |
| source-manifest.json against the changed-path list | All paths listed; `plan/` is listed as a directory covering mod.rs and simulate.rs | Complete. |
| traceability.json OK targets | All referenced files exist | Plausible mapping. |

### Summary

The unit implements BR1.1 to BR6.3 as designed. `--dry-run` goes through `prepare_checked` and never calls the Runtime, signals or the state writer, and the dry-run branches in `pipeline.rs` and `run.rs` return before any real-run code. The real-run order in `ctx.rs` is unchanged (signal setup, then definition, job existence, run ID, `.env`, state, consistency). `status_at` is behaviour-preserving for any state that passes the consistency check. `--assume` errors carry only a kind and the fixed subject, never the job ID or exit code. The existing-test changes are adaptations (the usage-text limit, demo scene counts) with no assertion removed. The four findings are advisory.
