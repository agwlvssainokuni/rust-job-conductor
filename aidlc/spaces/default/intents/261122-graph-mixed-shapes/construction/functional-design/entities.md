# エンティティ(Entities): graph の text 表示の改善 その 4

新しい永続のデータも、新しい公開の型もない。既存の図の型(`crates/core/src/graph/mod.rs`。`LaneGlyph` は、前回、横線 `_` を足した)をそのまま使う。変わるのは、配置(core の `graph/layout.rs`)の内部の持ち方だけ。

```yaml
entities:
  - name: OpenLanes
    description: 配置の途中で持つ、開いている線の並び(core の内部)。内側(列 1)が先頭。前回までは「行き先が、どの 2 本でも違う」前提だったが、今回、同じ行き先の線を、離れて 2 本以上持てる。
    attributes:
      - name: lanes
        logical_type: list of 行き先の節の位置(内側から外側の順)。同じ行き先が、重複して現れてよい
    constraints:
      - 新しい線は、先頭(列 1)に入れる(既存の線は 1 つ外側へずれる)。先頭の線と行き先が同じなら、新しい線を作らず、合流する(前回までの規則と同じ)
      - 同じ行き先の線が 2 本あるとき、「寄せ」(後ろの線を、前の線へ寄せる)で、1 本にする。寄せの行は、`MOVE(c, d)` の形で決まる(functional-spec.md の WF2)
  - name: LaneGlyph
    description: 変更なし(前回、`Underscore` を足した)。
```

要約: 型は変えない。配置の内部の「開いている線の並び」が、重複を許す。
