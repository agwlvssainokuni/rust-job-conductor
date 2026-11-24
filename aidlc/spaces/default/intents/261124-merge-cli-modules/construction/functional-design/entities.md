# エンティティ(Entities): cli の本番の小さなモジュールの統合

永続のデータ(ステートファイル)は扱わない。利用者に見える出力も変えない。関数の本体と名前は 1 つも変えない。変わるのは、関数の置き場所(ファイル)、可視性の語、`use`、説明のコメント、仕様書 rust の記述だけ。詳細は `inventory.md`。D1〜D7 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。下の D の箇所は、その決定のとおりの形を書いている。T 表(T1)は、設計の承認で認めてもらう(承認待ち)。

```yaml
entities:
  # ---- 中身が増えるファイル ----
  - name: crates/cli/src/step.rs
    description: 1 つのジョブの共通の手順に、リトライを含む実行 run_job_with_retry(元 retry.rs。非公開。D1。決定: 推奨のとおり。ユーザー、2026-11-25)を run_step の直後に足す(D4。決定: 推奨のとおり。ユーザー、2026-11-25)。use に 6 つの名前を足す。251 行 → 321 行。
  - name: crates/cli/src/report/mod.rs
    description: 表示の本体に、エスケープの包み escape_name・escape_full(pub(crate)。D2。決定: 推奨のとおり。ユーザー、2026-11-25)と上限の定数 MAX_DISPLAY_CHARS(非公開)を push_line の直前に足す(D5。決定: 推奨のとおり。ユーザー、2026-11-25)。モジュールの説明に元の escape.rs の説明を写す。643 行 → 657 行。
  - name: crates/cli/src/aux_commands.rs
    description: シグナルの準備をしない補助のコマンドに、run_graph・read_overlay(元 graph.rs)と run_init(元 init.rs)を足す(D6。決定: 推奨のとおり。ユーザー、2026-11-25)。元のモジュールの説明は各関数の説明へ。118 行 → 222 行。

  # ---- 形が変わるファイル ----
  - name: crates/cli/src/platform/mod.rs
    description: 中継の 3 関数と use 3 つを消し、#[cfg(unix)]/#[cfg(windows)] の pub(crate) use … as start_signal_watcher の 2 行にする(runtime と同じ再公開の形。D3。決定: 推奨のとおり。ユーザー、2026-11-25)。52 行 → 38 行。
  - name: crates/cli/src/platform/unix.rs・windows.rs
    description: spawn_watcher の可視性を pub(super) → pub(crate)(再公開に要る)。名前・本体・説明は変えない。
  - name: crates/cli/src/lib.rs
    description: mod retry; mod graph; mod init; を消し、graph と init の呼び出しを aux_commands:: にする。モジュールの表の「流れ」の行を直す。
  - name: crates/cli/src/run.rs
    description: モジュールの説明の retry::run_job_with_retry を step の run_job_with_retry にする。
  - name: crates/cli/src/report/plan_text.rs・graph_text.rs・watch.rs
    description: use super::escape::… を use super::escape_full; / use super::escape_name; にする。
  - name: crates/cli/src/tests/escape_table.rs(単体テスト)
    description: use の 1 行だけ(T1)。テストの名前・本体は変えない。

  # ---- 無くなるファイル(4) ----
  - name: crates/cli/src/retry.rs
    description: 中身は step.rs へ。
  - name: crates/cli/src/report/escape.rs
    description: 中身は report/mod.rs へ。
  - name: crates/cli/src/graph.rs、crates/cli/src/init.rs
    description: 中身は aux_commands.rs へ。

  # ---- 仕様書の記述(実装に揃える) ----
  - name: docs/spec/rust の 02・04・05・07・08・12 章
    description: モジュールの表、crate の中の関数の表、シーケンスの図、実装の所在、部品の表、エスケープの章の見出し、再構築の手引きの作るものを、まとめた後のファイルとモジュールに揃える(S1〜S24)。版の履歴の注記は足さない(D7。決定: 足さない。推奨のとおり。ユーザー、2026-11-25)。
  - name: docs/spec/language-agnostic
    description: 変えない(ファイル名もモジュールのパスも書いていない。関数の名前は変わらない)。
```

## 利用者に見える変わること

なし。`conductor` の出力、誤りの表示、終了コード、ステートファイル、`--dry-run` の出力、シグナルの扱い(UNIX 系の 4 つのシグナル、Windows の Ctrl+C、登録はプロセスで 1 回)、`conductor --version`(`conductor 0.9.8`)は変わらない。配布する実行ファイルの振る舞いも変わらない(関数の本体を変えないため)。
