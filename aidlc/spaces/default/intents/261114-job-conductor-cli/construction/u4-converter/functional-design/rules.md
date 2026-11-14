# Rules(u4-converter)

上流: requirements(FR7)、contract-summary(C4)、entities(この Unit の entities.md)、stories(US7.1)、この Unit の質問への回答(Q1〜Q3)。各規則は、`BR{group}.{seq}` の ID を持ち、出典(FR / Q)を示す。

```yaml
rules:
  # グループ1: 変換(Converter)
  - { id: BR1.1, statement: 入力の形式は、出力の形式(--to)と反対とみなす。拡張子は見ない, category: policy, applies_to: ConversionRequest, trigger: 変換の呼び出し時, logic: "IF targetFormat が toml THEN 入力を YAML として読む. IF targetFormat が yaml THEN 入力を TOML として読む", violation: 入力が、その形式として読めなければ BR1.4, source: FR7.1、Q2 }
  - { id: BR1.2, statement: キーと値の意味が同じになるように変換する。内容が conductor の定義として正しいかは調べない, category: policy, applies_to: ConvertedText, trigger: 入力が読めたとき, logic: "文字列、整数、浮動小数、真偽値、配列、マップ(キーが文字列)を、相手の形式の同じ意味の表現に写す。キーの名前・値・配列の順序・入れ子の構造は、変えない。定義としての検証(版、キーの種類、遷移など)は行わない(読み込み時に行われる)", violation: 表せない値は BR1.6, source: FR7.1、AC7.1.1、AC7.1.2、Q1 }
  - { id: BR1.3, statement: コメントと、マップのキーの記述順は、保持しない, category: policy, applies_to: ConvertedText, trigger: 変換の出力時, logic: "出力の中のコメントは書かない。マップのキーの順序は、出力の形式の既定に従い、入力と同じとは限らない。配列の要素の順序は保つ", violation: なし, source: FR7.1、AC7.1.1 }
  - { id: BR1.4, statement: 文法が誤っている入力は、変換せず、原因を返す, category: validation, applies_to: ConversionError, trigger: 入力を読むとき, logic: "IF 入力が、BR1.1 で決まった形式の文法として誤っている THEN ConversionError(syntax)(位置が分かれば line つき)を返し、出力を作らない", violation: 出力を作らない, source: FR7.2、AC7.1.3 }
  - { id: BR1.5, statement: 入力が読めないときは、変換せず、原因を返す, category: validation, applies_to: ConversionError, trigger: 入力を開くとき, logic: "IF ファイルがない THEN fileMissing. IF 権限がない・通常のファイルでない・UTF-8 でない THEN unreadable. IF 大きさが上限(1 MiB)を超える THEN tooLarge。いずれも出力を作らない", violation: 出力を作らない, source: FR7.2 }
  - { id: BR1.6, statement: 相手の形式で、意味を変えずに表せない値があるときは、変換せず、原因を返す, category: validation, applies_to: ConversionError, trigger: 値を相手の形式に写すとき, logic: "IF 入力に、次のいずれかがある THEN ConversionError(unrepresentable)(keyPath つき、値は含めない)を返し、出力を作らない: YAML の null、キーが文字列でないマップ、1つのファイルの中の複数の文書、TOML の日時の値(YAML に同じ型がない)、整数が相手の形式で表せる範囲を超える値", violation: 出力を作らない。黙って取り除いたり、近い値に置き換えたりしない, source: FR7.1、Q3 }
  - { id: BR1.7, statement: YAML のアンカーとエイリアスは、展開して変換する, category: policy, applies_to: ConvertedText, trigger: YAML を読むとき, logic: "アンカーで定義された値を参照するエイリアスは、参照先の値に展開した結果を、変換の対象にする。出力にアンカーとエイリアスは残らない", violation: なし, source: Q3 }
  - { id: BR1.8, statement: 最上位は、キーと値の集まり(マップ)でなければならない, category: constraint, applies_to: ConversionError, trigger: 入力を読んだあと, logic: "IF 入力の最上位が、マップでない(YAML の配列や単独の値、空の YAML 文書)THEN ConversionError(rootNotMap)を返し、出力を作らない。空の TOML は、空のマップとして扱い、空のマップを出力する", violation: 出力を作らない, source: FR7.2(TOML の最上位は、マップのみ) }
  - { id: BR1.9, statement: 変換の結果は、変換後の形式として文法的に正しい文字列として返す。書き出しは、呼び出し側(Cli)の責務, category: policy, applies_to: ConvertedText, trigger: 変換の成功時, logic: "text を返す。ファイルや標準出力への書き出し、書き出しの失敗の報告は、この Unit で行わない", violation: なし, source: contract-summary の C4 }
  - { id: BR1.10, statement: エラーの表示の内容は、固定の文と、場所だけで組み立てる, category: constraint, applies_to: ConversionError, trigger: 失敗の報告時, logic: "reason は固定の文。file と、分かれば line、keyPath を示す。入力の値と、ライブラリのエラー文は含めない", violation: なし, source: NFR4 の方針(u1-core の NFR4.6 と同じ) }
```

## 規則の要約

| 規則 | 内容 |
|------|------|
| BR1.1 | 入力の形式は、--to の反対 |
| BR1.2〜BR1.3 | 意味を保つ変換。コメントと順序は保持しない |
| BR1.4〜BR1.5 | 文法の誤り、読めない入力は、変換せず原因を返す |
| BR1.6〜BR1.8 | 表せない値、アンカーとエイリアス、最上位がマップでない入力 |
| BR1.9〜BR1.10 | 結果は文字列で返す(書き出しは Cli)。エラーの表示の組み立て |

## Assumptions & Open Questions

- 大きさの上限 1 MiB(BR1.5)は、u1-core の定義ファイルの上限に合わせた、この Unit の仮の値である。NFR Requirements で確定する。 [assumption]
- BR1.6 の「整数が表せる範囲」は、YAML と TOML のライブラリが扱える範囲に依存する。具体は、Code Generation で確かめる。 [assumption]
- YAML のマージキー(`<<`)は、ライブラリが展開するならその結果を、展開しないなら、通常のキーとして扱う。具体は、Code Generation で確かめる。 [assumption]
