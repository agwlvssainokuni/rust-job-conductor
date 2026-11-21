## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-21T23:51:53Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/runtime/src/file_create.rs > classify_open_error / classify_kind | The design (BR1.3) says a path ending in `/` is `other_io`. On this macOS run, `conductor init --config nonexist.toml/` returns `parent_missing` ("the parent directory does not exist") although the parent exists, because `symlink_metadata("nonexist.toml/")` returns NotFound and that is mapped to `parent_missing`. `ex.toml/` (an existing file with a trailing slash) and `newf/ --force` correctly give `other_io`. The result depends on the OS (Linux and Windows may differ), and no test pins the trailing-slash case. | Accept as advisory, or add a trailing-slash case to `crates/runtime/tests/file_create.rs` that records the expected kind per OS. If you want the design text exactly, treat a trailing separator as `other_io` before the NotFound mapping. | New |
| R-02 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/construction/u4-init/functional-design/functional-spec.md > section 4 (path display) vs crates/cli/src/report/mod.rs > show_path | The spec says the path is "truncated" and "treated like the definition file path". Existing code does not truncate paths (`report/mod.rs` line 24: names are cut at 64 characters, paths are not). The implementation follows "same as the definition path" (escape only), so the design text contradicts itself. The deviation is reasonable and documented, but a very long `--config` path is printed in full. | Correct the design text to "escape only, not truncated", or confirm the deviation at the gate. No code change is needed. | New |
| R-03 | Minor | crates/runtime/tests/file_create.rs | The cleanup of a partially written new file (and of the temp file after a failed write or rename) is implemented in `create_new_file` and `replace_file`, but no test induces a write failure, so the cleanup branches are unverified. The permission and symlink tests are `#[cfg(unix)]` and skip when the effect is not observable (root, Windows). The code summary states this honestly. | Accept as advisory (a write failure is hard to induce portably). Optionally cover the rename-failure cleanup, for example `--force` onto an existing directory or a trailing-slash path, and assert that no temp file is left. I ran these manually and found no leftover files. | New |
| R-04 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/construction/u4-init/code-generation/code-summary.md > not run | `windows_demo.rs`, `demo/windows/run-demo.cmd` scene 14, ubuntu CI and the MSRV test were not run here. The llvm-cov run could not confirm `unix_demo` (30 s timeout under instrumentation), so the 80% coverage floor was not confirmed by a clean instrumented run in this environment. I did not run llvm-cov either. | Confirm in CI (`test` on windows and ubuntu, `msrv-test`, `coverage --fail-under-lines 80`) before tagging. | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo test` runtime `file_create` (8 tests) | PASS | Exclusive create, force, directory, parent, symlink, permission and race tests pass. The race test asserts one Created and one AlreadyExists over 20 rounds, an invariant that holds regardless of scheduling. |
| `cargo test` cli: `flow_init`, `init_template`, `init_types`, `docs_init`, `binary_init`, `args_grammar`, `report_messages`, `nfr_source` | PASS (all) | No assertion was removed or weakened. `git diff` shows existing tests changed only by added imports, added `Init`, `--force` and init cases, the demo scene count 13 to 14, and the added `InitTemplate` entry in the Japanese exemption list. |
| `cargo clippy --workspace --all-targets --locked -D warnings` | PASS | No warnings. |
| Template vs functional-spec section 3 | PASS | I extracted both fenced toml blocks. The `ja_JP` and `en_US` outputs of `conductor init --config -` differ from the originals only on the `name` line (placeholder replaced by `my-pipeline`). The README example equals the Japanese template. |
| `validate` on generated files | PASS (rc 0) | Names used: `a"b`, `a\b`, a newline, U+007F, a tab, an emoji and 5000 characters. They are escaped to `\"`, `\\`, `\u000A`, `\u007F` as required. |
| Behaviour (manual) | PASS | init gives 0. A second init gives 69 with a fixed message. `--force` gives `replaced`. A directory gives 65 "it is a directory". A missing parent gives 65. A parent that is a file gives 65 `I/O error`. A dangling symlink gives 69, and with `--force` the link itself is replaced. `--config -` writes only the template, ignores `--force` and creates no file. An empty or blank name gives 68, and 68 beats an existing file. `-v`, `--run-id`, `--bogus`, a doubled `--force` and a second positional argument all give 68. |
| Secrecy | PASS | The name never appears in comments, in the empty-name error ("pipeline name") or in success messages. Only the existing `unexpected argument: 'b'` echo remains for a surplus positional argument, which is allowed existing behaviour. |
| Project rules | PASS | `Cargo.toml` and `Cargo.lock` are unchanged. There is no `unsafe` and no `cfg(unix)` or `cfg(windows)` in the new `src` files. `nfr_source.rs` (both crates) scans only `src/`, so `#[cfg(unix)]` inside `tests/` is allowed. |
| Line endings | PASS | `demo/windows/run-demo.cmd` is `i/crlf w/crlf`. `git diff --stat c6328f5 HEAD` shows 56 insertions and no deletions. |
| Manifest and traceability | PASS | The source manifest lists all 26 changed application paths, matching `git diff --name-status 5946f72 HEAD -- . ':!aidlc'`. The traceability OK targets exist. |

### Summary

U4 implements BR1.1 to BR4.2 as written. Both templates equal the design originals apart from the name, generated files pass `validate` even with hostile names, no existing behaviour or test was weakened, and no dependency or `unsafe` rule was violated. Only advisory items remain: the trailing-slash classification (OS-dependent), the design text on path truncation, untested write-failure cleanup, and the Windows, ubuntu and coverage checks that must be confirmed in CI.
