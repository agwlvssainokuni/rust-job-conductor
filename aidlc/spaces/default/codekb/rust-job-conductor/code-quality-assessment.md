# Code Quality Assessment

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名 | good | Rust 標準の命名。日本語のテスト名で意図が読める |
| 関数の大きさ | fair | `run_pipeline`(約 140 行)、`build_command`(約 140 行)は長い。ほかは短い |
| エラー処理 | good | パニックしない方針。失敗は戻り値、固定の文で表示(`runtime/lib.rs`) |
| テスト | good | 約 681 件。偽の依存での流れ(`flow_*`)と、実バイナリ(`binary_*`、デモ)の両方。80% を CI が要求 |
| 重複 | fair | 完了後の削除の呼び出しが `run.rs` と `pipeline.rs` の 4 か所に分かれる |
| 死んだコード | good | `clippy -D warnings`。`TODO`/`FIXME` は見つけなかった |

## CI/CD と文書

- CI: lint、build-check(Linux・Windows、最小の Rust 1.85)、test(OS 行列)、msrv-test、coverage(80%)、docs-check(README の必須の節)、startup-bench。タグ `v*` でリリース(下書き)。週次の監査。
- 文書: README(日本語。コマンド、定義、ステート、終了コード、シグナル)、`demo/README.md`、各クレートの `//!` の説明。

## 技術的負債のシグナル(この intent に関わるもの)

- `read_state` が「完了は空」を返す約束が、README・`reader.rs`・`runtime/lib.rs` に書かれていて、`watch` は別の入口を持たない。
- 既存テストの多くが「完了後にステートがない」を前提にする(`developer-scan.md` に一覧)。
- `PIPELINE_KEYS` と `string` ヘルパーのみで、真偽値の取り出しがない。
