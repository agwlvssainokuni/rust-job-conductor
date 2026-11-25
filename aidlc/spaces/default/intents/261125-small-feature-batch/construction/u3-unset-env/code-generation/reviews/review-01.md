## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T22:16:13Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u3-unset-env/code-generation/code-generation-plan.md > Step 8 / Touches | 計画の Step 8 は `demo/env.toml`・`demo/windows/env.toml`・`demo/README.md` と 2 つのデモのテストだけを挙げているが、実際には `demo/run-demo.sh`、`demo/windows/run-demo.cmd`、`demo/scripts/show-env.sh`、`demo/windows/scripts/show-env.cmd` も変わっている。変更は理由があり(親に `DEMO_PROXY` を置き、ジョブが表示しないと「消えた」ことをデモで見せられない)、`code-summary.md` の「計画からのずれ」にも書かれている。追加した行は `DEMO_PROXY` の表示 1 行と場面 11 の呼び出しだけで、既存の場面の出力は変えていない。 | ブロックしない。次の計画では、デモの場面に使うスクリプトも Touches に先に挙げる。 | New |
| R-02 | Minor | crates/cli/tests/windows_demo.rs > 場面_11_unset_env_の名前は親に置いても大文字小文字によらず渡らず_dry_run_は消す名前を出す、crates/runtime/tests/runner_basic.rs > 大文字小文字だけが違う名前の消し方は_os_に従う、crates/cli/tests/binary_env.rs > unset_env_の名前は親から引き継いだ環境から消えてほかは残る | Windows 側の経路(親の `demo_proxy` や `Path` が、大文字小文字を区別せずに消えること)は、このレビューでは実行できず、macOS では通っていない分岐である(`code-summary.md` も未実行と明記)。コードを読むと、`env_remove` は std に任せていて、テストの分岐は `cfg!(windows)` でテストの中に閉じており、`nfr_source.rs` の規則(`cfg` は platform の外に置かない。テストの中を除く)に反しない。ただし、`PATH` を子から消す 2 つのテストが Windows の CI で通るかは、まだ確認されていない。 | マージ前に、CI の `test`(windows)で上の 3 つのテストが通ることを確かめる。通らなければ、`PATH` ではなく、テスト用の名前を使う形に直す。 | New |
| R-03 | Minor | README.md > 引き継いだ環境変数を消す(`unset_env`)(v0.10.0 以降) | README の `unset_env` の例(`unset_env = ["HTTP_PROXY"]` など)は、`docs` のテストで定義として読めることを確かめていない(`code-summary.md` の「気になる点」にも記載あり)。例の書き間違いが後でわかるおそれがある。 | 余裕があれば、README の例の定義を `validate` に通すテストを `src/tests/docs.rs` に足す。必須ではない。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo test --workspace --locked`(macOS) | 1,324 件通過、失敗 0 | `code-summary.md` の数と一致。core・runtime・cli の新しいテストを含めて通る |
| `cargo fmt --all -- --check` | 通過 | 書式の問題なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通過 | 警告なし |
| `source-manifest.json` と `git diff --name-only 382a1f6^..a56aaa7` の突き合わせ | 欠けも余分もなし | 変更したファイルの一覧は実際の差分と一致する |

### Summary

重なりの検査(定義の `env` との比べ方と `.env` との比べ方)は、どちらも ASCII の小文字にそろえて比べ、`.env` 側は「ファイルの位置と行」の集合で重複を除くので、同じ `.env` を複数のジョブが読んでも誤りは 1 つになる。検査の範囲は `env_file_refs` と同じ範囲(`run` は 1 ジョブ、それ以外は全ジョブ)で、パイプラインの `unset_env` も各ジョブに加えて調べている。`env_remove` は足す前に呼ばれ、`unset_env` が無ければ空のループで何もしないので、今の動きは変わらない。`--dry-run` の消す名前は定義だけから作り(`merge_unset_names`)、親の環境を読まない。新しい誤りの文は固定で、値も名前も入らない(テストで確認)。本番コードにテスト専用の引数や関数は足されておらず、`cfg` も platform の外に増えていない。ブロックする問題は見つからなかった。残るのは Windows の CI での確認と、デモの追加ファイルを計画に書いていなかったことなど、軽い点だけである。
