# エンティティ(Entities): converter の testing モジュールの削除

永続のデータ(ステートファイル)は扱わない。利用者に見える出力も変えない。変わるのは、converter のライブラリとしての隠れた公開(`#[doc(hidden)]` の `testing`)と、本番で使われない関数(`TargetFormat::opposite`)と実装(`Value` の `PartialEq`)だけ。公開の入口の型は変えない。単体テストの置き場所は `src/tests/`(D2。決定: 推奨のとおり。ユーザー、2026-11-24)で、本番の成果物には入らない。詳細は `inventory.md`。

```yaml
entities:
  # ---- converter の公開(変えない) ----
  - name: TargetFormat
    description: 変換後の形式(converter。公開。cli が使う)。変種 Yaml / Toml は変えない。メソッド opposite を消す(inventory.md 2)。
    attributes:
      - name: Yaml / Toml
        logical_type: 今のまま
      - name: opposite()
        logical_type: (消す)
  - name: ConversionRequest / ConvertedText / ConversionError / ConversionErrorKind
    description: 変換の入出力と誤り(converter。公開)。変えない。

  # ---- converter の隠れた公開 ----
  - name: testing
    description: テストのための doc(hidden) の公開モジュール(Value の再公開と read_tree)。モジュールごと消す(inventory.md 1)。単体テストはクレートの中から read_value と Value を直接使う。

  # ---- converter の内部 ----
  - name: Value
    description: 途中の表現(converter の内部。types の中の pub。クレートの外には出ない)。変種は変えない。手書きの PartialEq を cfg(test) にする(inventory.md 3)。
    attributes:
      - name: Str / Int / Float / Bool / Array / Map
        logical_type: 今のまま
      - name: Debug / Clone
        logical_type: 今のまま(derive)
      - name: PartialEq
        logical_type: テストのときだけ(cfg(test))。比べ方は今のまま
  - name: read_value
    description: 入力を読み、展開した途中の表現にする非公開の関数(lib.rs)。変えない。単体テストの補助 yaml()・toml() が呼ぶ。
```

## 利用者に見える変わること

なし。`conductor convert` の出力、誤りの表示、終了コード、`conductor --version`(`conductor 0.9.8`)は変わらない。converter のライブラリの、ドキュメントに出ない公開(`testing`)と、公開の型 `TargetFormat` のメソッド `opposite` が無くなる(cli は使っていない)。
