# Code Summary(u1-core)

17 ステップをすべて完了した。`cargo fmt --all -- --check`、`cargo clippy -p conductor-core --all-targets --locked -- -D warnings`、`cargo test -p conductor-core --locked`(115 件、失敗 0。レビュー 1 回目の修正後)が通った。

## 作成・変更したファイル

- ワークスペースの根: `Cargo.toml`(`members = ["crates/*"]`、resolver 3、edition 2024、`rust-version` 1.85)、`Cargo.lock`(固定)、`.gitignore`(コメントと `target/` を末尾に追加。ほかは変更なし。計画に明記した変更)。
- `crates/u1-core/`(パッケージ名 `conductor-core`): `Cargo.toml`、`src/lib.rs`、`src/types.rs`、`src/error.rs`、`src/definition/{mod,intake,parse,depth,version,walk}.rs`、`src/flow/{mod,decide,classify,next_step,resume}.rs`。
- テスト: `tests/{library_capability,definition_valid,definition_errors,flow_sequences,nfr_security,nfr_performance}.rs`、`tests/fixtures/{valid_pipeline,multiple_errors}.toml`、`src/` 内の単体テスト。

## 主な実装の判断

- **読み取りのライブラリ(Step 1 の確認結果):** `toml` 1.2.1(`default-features = false`、機能は `parse` と `std`)の `toml::de::DeTable` / `DeValue` / `Spanned` を使う。(a) キーと値のバイトの範囲が取れること、(b) 深さ 1000(および 100,000)の入れ子で、異常終了せず `Err` が返ることを、テストで確かめた。別スレッドの代替(D4)は不要。ほかの依存は `thiserror` 2.0.21 と、開発用の `tempfile` 3.27.0。`cargo tree` に `-sys` のクレートはなく、すべて Rust だけ(NFR2.1)。
- 読み取りの流れは、security-design のとおり(Intake → Parse → Depth(32 段)→ VersionCheck → Walk)。検査は自分で走査し、誤りをすべて集めて返す。表示の文は固定で、入力の値とライブラリの文を含まない。
- 起動時間(NFR1.1): ジョブ 10 個の定義を 100 回読んだ p95 は 0.915 ms(基準 20ms。macOS での 1 回の測定)。

## 設計との差(計画からの逸脱と、設計の成果物への反映)

1. `decide_before_run` の引数に、`resume_job_id` と `end_reached` が加わった(BR3.5 の 2・4 に必要)。契約 C2 は `[jobId, definition, jobStates]` のみ。
2. `JobOutcome.classification` は `Option`。起動しなかった中断(BR2.4)では分類しない。entities.md は必須としている。
3. `Decision` は、値と理由を持つ列挙型(`Run`、`Skip(理由)`、`Reject(理由)`)。
4. 終了コードは `i32`。範囲外の値は `typeMismatch`。
5. `[[pipeline]]`(配列)は `typeMismatch`(BR1.12)。空白だけの `command`・パイプライン名は `emptyField`。整数でない `format_version` は `versionUnsupported`。`warning_exit_codes` に 0 があるときは、`zeroAsWarning` だけを返す。
6. security-design の「既存の成果物との差分」は、コードには反映した(`syntax`、`tooLarge`、複数パイプラインの `typeMismatch`)。設計の成果物(entities.md の `kind`、rules.md の BR1.9・BR1.12・BR1.11、functional-spec の W1、security-requirements の NFR4.4)は、計画どおり、書き換えていない。差分として、ここに記録する。

## レビュー(1 回目)で直した点

- Major: 個別実行の `@end` 後の完了判定(上記)。テスト(`tests/flow_sequences.rs`)は、途中の呼び出しごとに `!completed` を確かめ、最後で `completed` を確かめる。pipeline の対のテストも追加。
- Major: let チェーン(Rust 1.88 以降)を書き換え、宣言した `rust-version` 1.85 でコンパイルできるようにした。
- Minor: 上限 + 1 バイトの読み取りを `read_bounded` に分け、上限ちょうど・+1・終わりのない入力でテスト。`escape_char` が書式・双方向制御文字も扱う。深さ 32/33/1000 の `syntax` を `parse_definition` でも確認。`signal_number = None` は 128(仕様に範囲がないため、挙動を文書化)。
- Minor: `skipped_job_ids` は、すでに記録済みのジョブを除く(BR3.2・BR3.3 の「全ジョブ」の文面との差。ここに記録する)。

## 設計の隙間(後の Unit・ステージで解決する)

- **`end_reached` の引き継ぎ(レビュー 2 回目の指摘):** `resume_point_on_interrupt` の `end_reached` は、`skipped(byEnd)` の記録だけから決まる。個別実行で `@end` に着いたあと、最初のスキップを記録する前に中断されると、false が返る。u3-cli は、`end_reached` を、戻り値で上書きせず、積み上げる(一度 true になったら、true のまま)必要がある。u3-cli の設計で確かめる。
- **Rust 1.85 での実機コンパイル:** let チェーンは取り除いたが、rustc 1.97 でしか確かめていない。CI で、最低版(1.85)のビルドを確かめる(CI Pipeline のステージで検討)。

- **契約 C2:** `decide_before_run` に `resume_job_id`・`end_reached`、`classify_and_decide` に `run_mode` が要る。契約の更新は、u3-cli の設計で行う。
- **個別実行の `@end` のあとの完了(レビュー 1 回目で修正):** `RunMode::Individual` で `@end` に着いたときは、`end_reached = true`、`skip_reason_kind = ByEnd`、`skipped_job_ids` は空、`completed = false`(BR3.6 のとおり、以降のジョブはスケジューラから呼ばれたとき、`decide_before_run` が `Skip(AfterEnd)` を返す)。Cli は、呼ばれたジョブだけをスキップとして記録する。最後のジョブを記録したあとの `resume_point_on_interrupt` が `completed = true` を返す。`RunMode::Pipeline` は BR3.3 のとおり、以降のジョブを一括でスキップし、`completed = true`。BR3.3 の「全ジョブ」は pipeline の方式の規則であり、individual は BR3.6 に従う。
- **`signal_number` がないままの中断:** 128 + 0 になる。仕様に、この場合の値がない。u2-runtime の `JobExecution` の生成で、番号を必ず持たせる。
- `NextStep.end_reached` は、`@end` に到達した呼び出しだけ true。以降は、`byEnd` の記録、または `PipelineState` が持つ。
- `DefinitionError.reason` は固定の日本語。表示の言語の切り替え(E1)は、u3-cli が行う。

## テスト(115 件)

| 範囲 | 件数 |
|------|------|
| `src/` の単体テスト | 71 |
| definition_errors | 18 |
| definition_valid | 8 |
| flow_sequences | 9 |
| nfr_security | 5 |
| library_capability | 3 |
| nfr_performance | 1 |

## 実行していないテスト・測定

- **行カバレッジ:** `cargo llvm-cov` が未導入のため、測っていない。80% の判定は Build and Test が、全体のテストで行う。
- **Linux と Windows:** macOS でだけ実行した。他の OS は、CI(CI Pipeline のステージで作成)で確かめる。Windows の主スレッドのスタック(深さ 1000 の入力)は、CI で確認が要る。
- CI のワークフローは、まだない(CI Pipeline のステージで作成)。
