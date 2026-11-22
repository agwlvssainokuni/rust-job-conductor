## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/core/src/graph/layout.rs > doc comment of `skip_targets` (line 44) | The comment still names `is_crossing_class`, which this change removed and renamed to `is_ordinary_skip_graph`. It also describes the "range that allows crossing" as skipping conditions 2 and 3, but the early return is now the ordinary graph (BR1.1). | Change the comment to `is_ordinary_skip_graph` and the "ordinary skip graph" wording. | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-graph-mixed-shapes/construction/functional-design/rules.md > BR1.4 | BR1.4 does not say that a duplicate pair whose target equals the arriving node is excluded from the first settle pass. The Python reference has this exclusion (`rem[a]!=tgt`) and the Rust code implements it (`lanes[a] == node`). The behaviour is correct but the rule text is incomplete. | Add the exclusion to the BR1.4 text, or leave the Python reference as the sole normative source. | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test --locked -p conductor-core | PASS (every test binary ok, including graph_layout_oracle 8, graph_mixed_shapes 6, graph_perf 4) | The oracle test loads all five fixtures, with count asserts of 537, 1,517, 871, 940 and 487. It compares every example character by character, and the lanes output is not vacuous. |
| cargo test --locked -p conductor-cli | PASS | graph_report, binary_graph and docs_graph pass, including the new M2 README example check. |
| cargo clippy --workspace --all-targets --locked -D warnings, cargo fmt --check | PASS (clean) | Meets the lint gate. |
| Diff scan for dependencies, unsafe, cfg, git-internal names | No Cargo change, no unsafe, no cfg. Nothing in the diff quotes or paraphrases git internals. | BR5.1 and the dependency and unsafe rules hold. |

**Details of the review**

- Python reference match. I compared `ordinary_layout`, `settle_duplicates` and `move_rows` line by line against `layout-reference.py`; no differences found. Points checked:
  - The settle exclusions (target == arriving node, and a pair straddling the arriving lane with `k` fixed at call time).
  - The 1-row and 2-row MOVE rows: the number of `_`, the first duplicate, and the `d>=1` outer `/` rule via `outer_start` and `reached_next`.
  - The slash rules `OneRow`, and `DuplicateTwoRows` with q==0 only. `ArrivalTwoRows` and `None` map to `|`.
  - `after_fork` reset, the join row and the fork row.
  - The END handling: `lanes` is empty after the arrival at the end node.
  - The arrival of the first of several arriving lanes: `rest` drops every lane with target == i, and `move_rows` for `d=0` ignores `next`, as in the Python (`nxt=None`).
- Optimized duplicate search. I found no counter-example. For each `a`, the Python takes the smallest `b>a` with the same target that does not straddle `k`. If the next same-target lane straddles `k` (`a<k<b`), every later same-target lane also straddles it, so skipping `a` is equivalent. A target equal to the node is excluded in both. `a` is scanned in ascending order and the first valid pair is taken, as in the Python. Each move removes a lane, so there are at most n moves, each O(n). That makes O(n^2) at 1,000 jobs, and the perf test (under 1 second) passes.
- Old path. `ordinary == false` implies some job is a both-skip or a branch. `skip_targets` then pushes onto a strictly decreasing, unique stack, so crossing is already rejected. An arrival lane is therefore always the innermost one, and `debug_assert_eq!(index, open.len()-1)` can never fire. It is justified. The 871 branch oracle and the unchanged 537, 1,517 and 940 oracles pass. The removal of `crossing_rows` only drops the unreachable `k>=1` branch.
- Functional-design review. The old/new path interface is resolved by `SkipPlan.ordinary` and the dispatch in `layout_graph`. BR1.3 and BR1.4 now match the Python reference, apart from the BR1.4 wording point above.
- Tests and fixtures. The edge-list fixtures replaced in graph_report and binary_graph (a goes to e with both success and warning kinds) stay in the edge-list class. The tests still assert the edge-list text, `-v` details and the `--state` overlay. The one-off check of 4,348 and 4,757 cases is reported in code-summary (BR4.1).
- Docs. The README and the spec statements about falling back to the edge list for same-target plus crossing were updated. A docs_graph test now guards the stale phrase. No stale counts found.

### Summary

The implementation matches the Python reference and BR1.1 to BR5.1, and the old both-skip and branch path is unchanged in behaviour. Tests, clippy and fmt pass. Only two minor documentation points remain, so the verdict is READY.
