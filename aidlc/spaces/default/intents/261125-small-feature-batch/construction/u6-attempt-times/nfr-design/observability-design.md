# 観測の設計: u6-attempt-times

出典: 観測の要件(`observability-requirements.md` の NFR1.5、NFR6.4)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3、4)、契約(`contract-summary.md` の C4、C5)。

## 設計

- **所要時間の列(NFR1.5)**: `watch.rs` の `render_table(lang, state, color, now)` に今の時刻を渡す。列は「ジョブ」「状態」「所要時間」「補足」の順。見出しは `messages.rs` に `HEADER_DURATION`(`duration` / `所要時間`)を足す。幅は今の `width` と `pad` でそろえる。書き方は core の `format_duration(秒)`(`17s`、`1m05s`、`2h03m04s`。上の単位が 0 なら省き、下の単位は 2 桁)。
- `watch` の流れ(`aux_commands.rs` の `run_watch`)は、表を作る前に `ctx.runtime.now()` を 1 回取る。U7 の継続表示も、描くたびに同じ関数を使う。
- **65 の表示(NFR6.4)**: 新しい理由は、今の `Corrupt` の表示の経路(`state_kind`、理由の文、`reset_hint`)をそのまま通る。`run`、`pipeline`(`--dry-run` を含む)、`watch`、`graph --state` は同じ読み込みの関数を使うので、表示は同じになる。
- 確かめ方: `watch` の表のテスト(0 秒、59 秒、60 秒、3600 秒、空、英日)。4 つのコマンドの流れのテストで、古い形のステートを読んで 65 と案内が出ること。
