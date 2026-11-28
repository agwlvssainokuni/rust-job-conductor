## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no
**Date:** 2026-11-28T08:49:33Z

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | docs/spec/rust/12-rebuild-guide.md > 各段の「作るもの(cli)」の行(117、127、137、179、211 行目) | 版ごとの履歴の行が `args.rs` を書いたまま残っている。履歴として残す方針で、76 行目には v0.10.4 の分割の注記があるが、他の行には無い。誤りではない。 | 対応は不要。読み手の迷いが気になるなら、各行ではなく 12 章の冒頭に 1 行の注記を足す。 | New |

### 検査の結果

- **移すだけのコミット**: 10 個とも、行の多重集合で比べた。増減は次のものだけだった。
  - `mod`・`use` の行
  - ファイルの冒頭(ライセンスと `//!`)
  - 可視性の変更(`fn` を `pub(super) fn` にした)
  - 移した先に合わせて変えた `use` の並び
  - 17ac829 は名前の変更のみで、行の増減は 0。
- **`build_command`(5e0e031)**: 誤りの順は元と同じ。
  - 順序は、字句(知らないオプション)、コマンド名、「使えない」と重複、`--all` と `--run-id` の組み合わせ、値(`--assume`、`--format`、`--interval`、`--run-id`)、位置引数の数、コマンドごとの位置引数。
  - `run_id` の `take` は `count > 0` に置き換わっている。値の検査は `fill_options` に残っている。
- **`render`(f903860)**: 動きは同じ。
  - `language()` は `OnceCell` で 1 回だけ判定する。
  - 固定の文(`SignalSetupFailed`、`Version`)と `pass_through` は言語を判定しない。
  - `transition` は、`skipped` が空でないときだけ言語を引く。
  - `watch_frame` は `self.render` を呼び直している。
  - 色を判定するのは `watch_table` だけ。
- **公開の範囲と本番のコード**:
  - 新しく増えた `pub` は、移した先で元の可視性を保ったものか、`pub(super)` のものだけ。
  - テストのための `pub` は無い。
  - 本番に `unwrap`・`expect`・`panic!`・`unreachable!`・`#[path]`・`include!`・`#[allow]`・`cfg(unix)`・`cfg(windows)` は増えていない。
- **触ってはいけない場所**:
  - `env.rs`・`retry.rs` の差分は 0 行。
  - `walk/mod.rs` のキーの表は変わっていない(差分に出るのは呼び出しの行のみ)。
- **team.md**: 差分は付録 A の 1 行だけ(使い方の表の場所)。
- **仕様書(2c284c3)**: 新しいファイルの表と注記が入っている。もう無いパスを今の形として書いた箇所は見つからなかった。12 章の履歴の `args.rs` は残してある。
- **版(3852f7a)**: ad775bc と同じ 22 ファイル。
- **人の判断 A(334c574)**: 変えたのは、`with_all_events` の 2 つの変種(`ResetAllKept`、`DefinitionValid`)と、例外の条件(`ResetAllKept`)だけ。
- **実行した検査**: `cargo fmt --check` と `cargo clippy -D warnings` は通った。`cargo test --workspace --locked` は 1449 件が成功、失敗は 0 件だった。

### Summary

分割は、移すコミットで中身が変わっておらず、分けた 2 つの関数でも誤りの順・言語の後回し・`watch_frame` の呼び直しが保たれている。公開の範囲も、触ってはいけない場所も、team.md も、版も、計画どおり。Critical と Major の指摘は無く、READY とする。
