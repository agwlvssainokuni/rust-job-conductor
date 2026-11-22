# 性能のテストの手順(Performance Test Instructions)

引数の読み取りは、オプションの表(11 行)を名前で引く形になった。引数の数に比例する処理のままで、性能に関わる変更は無い。起動の性能のテスト(`nfr_perf.rs`)は `cargo test --locked --workspace` に含まれ、通った。CI の `startup-bench` は警告のみ。
