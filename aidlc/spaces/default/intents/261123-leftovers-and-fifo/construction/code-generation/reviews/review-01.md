## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/core/tests/env_refs_perf.rs:LIMIT(40 ms) | P3 の上限 40 ms に対し、手元(macOS、debug)の中央値は 10 ms 前後(各回 8〜18 ms)で、余裕は約 4 倍しかない。P1(41 ms 対 350 ms、約 8 倍)と P2(240 ms 対 1,600 ms、約 6.7 倍)より狭い。CI の coverage ジョブ(llvm-cov の計測つき)と Windows の runner が重なると、この倍率に近づくおそれがある。直す前の値は約 620 ms なので、上限を 2 倍にしても退行を見分けられる。 | CI で P3 が揺れるようなら、上限を 80 ms 程度に上げる。いまの値のままでも、中央値の方式で単発の外れ値には耐える。マージの妨げにはしない。 | New |
| R-02 | Minor | crates/runtime/tests/state_read.rs:within_5s と crates/converter/tests/convert_ac.rs:名前付きパイプの入力は開かずに_unreadable | 名前付きパイプのテストは、別スレッドで呼び 5 秒で打ち切る。止まっても、テストは落ちて終わる。ただし、止まったスレッドは残るので、プロセスの終了までブロックしたままになる(テストの失敗の後始末としては許せる)。`mkfifo` コマンドが無い環境では `expect` で落ちる。macOS では全件通った。Linux の CI では実行していない。 | 必須ではない。`mkfifo` の失敗の文言は読み取れるので、このままでよい。Linux の CI で止まらないことは、初回の CI で確かめる。 | New |
| R-03 | info | aidlc/spaces/default/intents/261123-leftovers-and-fifo/construction/functional-design/functional-spec.md:§4 T25 の行 | 設計の凍結の後に、T25(converter の `tests/nfr_security.rs:328` の `&e.reason` を `e.reason` に)の 1 行だけが追記された(765444f)。ほかの差分は無い。理由は、`reason` を `&'static str` にすると clippy の `needless_borrows_for_generic_args` に当たるため。利用者の承認(2026-11-23)が code-summary に記録されている。確かめる中身は変わらない。 | 対応は要らない。記録のとおりで、範囲は妥当。 | New |
| R-04 | info | crates/cli/tests/common/mod.rs:state_error の reason | `state_error` の理由が `"テスト用の固定の理由"` から `"reason"` に変わった。T15 と D12(承認済み)の範囲。理由の文を確かめるテストは無い(設計に記録がある)。期待の文字列の変更は ★T1b、T22、T23 に収まり、これは対象外のテスト用の値だけ。 | 対応は要らない。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| cargo test --workspace --locked | PASS(全クレートの全テストが ok、失敗 0) | 件数は code-summary の 1,161 と矛盾しない。P1 41 ms、P2 240 ms、P3 10 ms(手元の中央値) |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | 警告なし |
| cargo fmt --all -- --check | PASS | 差分なし |
| 追加行の検索(unwrap、expect、panic!、unreachable!、todo!、unsafe、cfg) | runtime の製品のコードに該当なし | 該当したのは、bounded_read.rs の単体テスト 2 件と、split.rs のテスト補助の expect、`#[cfg(test)]` だけ(すべてテストの中)。unsafe と src の cfg(unix)・cfg(windows) の追加なし |
| Cargo.lock の差分 | 4 クレートの版 0.9.7 から 0.9.8 だけ | 新しい依存なし |
| 版の更新の範囲(7239aa2 との比較) | Cargo.toml 4 件、Cargo.lock、binary_basic.rs の T22、仕様書の版の表記 | 同じ範囲。v0.9.7 までの動きを指す注記の版は上げていない |
| traceability.json | 28 件の ID がすべて上流の文書に実在、target はすべて実在するファイル | `upstream_ids` と `coverage` が 1 対 1 で一致。`reverse` は空 |

### 観点ごとの確認

1. **段 1 で利用者から見える動き**: `escape_path` から `escape_full` への変更は名前だけで、中身(`escape_display(input, None)`)は同じ。呼ぶ側の置き換えも全件が同じ対象(切らない所)に限られる。`StateContext` への置き換えは、`reset_hint` が `ReadRunId` だけ run ID 用の案内にし、ほかは通常の案内にするので、`with_run_id` の真偽と同じ結果になる。`replace_via_temp` は、一時ファイルの作成、書き込み、閉じる、rename、失敗時の削除の順が元の 2 か所と同じで、誤りの種類の写し方(`CreateTemp` は `classify_kind`、`WriteOrRename` は PermissionDenied か OtherIo)も元と同じ。`bounded_read` は、`.env` の読み込みの 1〜4 をそのまま移したもので、ステートに足した形 B は設計(2b)どおり。`inspect_run_state` は `parse_state_for` から `parse_state_raw_for` に変わるが、完了のステートは元でも `Ok(None)` かつ中身ありで `Ours` となり、結果は同じ。計算量の変更(env の大文字小文字、遷移の重複、`env_file_refs`)は、確認の位置と順が元と同じで、誤りの報告の順と内容は変わらない(P2 のテストが行番号の順と件数 `2 * (TRANSITIONS - 1)` で確かめる)。`run_job` から `job` を外した変更は、本物の起動が元から使わない引数を消しただけ。偽物は `CONDUCTOR_JOB_ID` から取り、無ければパニックで落とす(core の `merge_env` がいつも足す。全テストが通る)。`ConversionError.reason` の `&'static str` 化は、コンパイルが通り、表示の組み立て(`escape_full(e.reason)`)は同じ文を出す。`build_retry` の変更だけは、値が欠けたときの動きが変わる(黙って `None` にするのをやめ、ジョブを組み立てない)。ただし、設計の記述では、誤りがあれば前の段で失敗するので、この枝には来ない。
2. **段 2**: converter(`read_input`)とステートは、パスの種別 → 開く → 開いたものの種別と大きさ → 上限 + 1 バイトの順で、無い、通常でない(名前付きパイプ、ディレクトリ、文字デバイスへのリンク)、権限なし(小、大)、上限超過、UTF-8 でないの各形が、設計の表と合う。`/dev/zero` へのリンクが `tooLarge` から `unreadable` に変わるのは、設計と仕様書の「v0.9.7 までは…」の注記に記録されている。Windows のディレクトリの特別扱いは、種別を先にパスで見るので不要になり、同じ結果になる。空白だけの先頭の語は `trim().is_empty()` で `Empty`(64)になり、`' a'` のような空白を含むが空白だけでない語は通る。runtime の製品のコードに unwrap・expect・panic! 等は無い。
3. **既存のテストの変更**: 消えたのは §4.6 の 4 つだけ(core の depth.rs の 2 件、converter の yaml_read.rs の 1 件、read_resolve.rs の 1 件)。期待の文字列の変更は、★T1b、T22(版)、T23(`/dev/zero` のステートが `unreadable`)に収まる。code-summary の「計画からのずれ」(record.rs の単体テストの import 1 行、1g-4 を版の更新の後に入れた順序、`create_temp_beside` の非公開化、`is_retry_target` の再公開など)は、いずれも T 表の範囲か、利用者の承認(T25)の範囲で、妥当。
4. **性能のテスト**: 5 回の中央値の方式で、起動の初回の外れ値(P1 の 67 ms、P3 の 18 ms)に耐える。上限は直す前の値(645、2,032、624 ms)の下に置かれ、退行を見分けられる。余裕は P3 が最も狭い(上の minor)。
5. **制約**: 新しい依存なし、unsafe と src の cfg の追加なし、core・converter の src のテストに cfg(unix) の追加なし(cfg(unix) は converter と runtime の `tests/` の名前付きパイプのテストだけ)、コードのコメントに版の番号なし(版の文字列は Cargo と `binary_basic.rs` の期待だけ)。converter は core に依存しない。設計の functional-spec の差分は T25 の 1 行だけ。
6. **traceability.json**: 上の表のとおり。

### Summary

段 1 の振る舞いを変えない整理(名前の変更、共通化、計算量の改善、`run_job` の引数の削除、`&'static str` 化)は、差分の読み取りと全テスト・clippy・fmt の通過で、利用者から見える動きを変えていないと確かめられた。段 2 の形 B と空白だけのコマンド名は、設計と仕様書の注記と合い、製品のコードにパニックの経路は無い。指摘は、P3 の性能の上限の余裕(約 4 倍)と、名前付きパイプのテストを Linux の CI で初めて動かす点の 2 つの minor だけで、いずれもマージを妨げない。
