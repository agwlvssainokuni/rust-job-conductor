# 性能のテストの手順(Performance Test Instructions)

性能に関わる変更は無い(処理の流れの置き場所を変えただけ)。既存の性能のテストが通ることで確かめる。

- `crates/core/tests/plan_perf.rs`(`--dry-run` の模擬)、graph の性能のテスト(段 4a で重複 1 件をまとめた)、`nfr_perf.rs`。
- いずれも `cargo test --locked --workspace` に含まれ、通った。起動の性能(`startup-bench`)は CI で警告のみ。
