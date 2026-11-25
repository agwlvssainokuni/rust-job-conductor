# セキュリティのテストの手順(Security Test Instructions)

- cli の製品のコードの検査(`tests/nfr_source.rs`)は 6 件とも通った。`unsafe` の追加なし、`cfg(unix)`・`cfg(windows)` は `platform` の中だけ、ネットワークのライブラリなし。
- 公開範囲: `run_job_with_retry` は非公開、`escape_name`・`escape_full` は `pub(crate)`(`mod report` が非公開なので、今の実効の範囲と同じ)、`spawn_watcher` は `pub(crate)`(runtime と同じ形)。cli のライブラリの外に見える範囲は変わらない。
- 誤りの表示に入力の値を含めない: 表示のエスケープの関数は中身を変えずに移した。エスケープの表のテスト(`src/tests/escape_table.rs`)は、`use` の 1 行のほかは変えずに通る。
- 新しい依存なし: `Cargo.toml` と `Cargo.lock` の差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
