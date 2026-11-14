# Entities(u4-converter)

上流: unit-of-work(U4 u4-converter の責務)、components(Converter)、contract-summary(C4)、requirements(FR7)、stories(US7.1)。この Unit は、定義ファイルを YAML と TOML のあいだで変換する(Converter)。Converter は、永続するエンティティを所有しない(components.md の Entity Ownership)。ここに置くのは、呼び出しの入力・出力・失敗の形(値の型)だけである。技術に依存しない記述であり、言語やデータ形式は定めない。

```yaml
entities:
  - name: TargetFormat
    owner: Converter
    description: 変換後の形式。入力の形式は、これと反対の形式とみなす(規則 BR1.1)
    attributes:
      - { name: value, type: enum, required: true, allowed: [yaml, toml] }

  - name: ConversionRequest
    owner: Converter
    description: 変換の呼び出しの入力(契約 C4 の input)
    attributes:
      - { name: inputPath, type: text, required: true, constraints: 変換する入力ファイルのパス }
      - { name: targetFormat, type: TargetFormat, required: true }

  - name: ConvertedText
    owner: Converter
    description: 変換の結果の文字列(契約 C4 の output)。書き出し先(標準出力かファイルか)と書き出しの失敗は、Cli の責務
    attributes:
      - { name: text, type: text, required: true, constraints: 変換後の形式で、文法として正しい。コメントと記述順は保持しない }

  - name: ConversionError
    owner: Converter
    description: 変換の失敗の報告(契約 C4 の failure)。変換は行われず、出力は作られない
    attributes:
      - { name: file, type: text, required: true }
      - { name: line, type: integer, required: false, constraints: 文法の誤りで、位置が分かるときのみ }
      - { name: keyPath, type: text, required: false, constraints: 表せない値のとき、その値の場所(キーの経路)。値そのものは含めない }
      - { name: kind, type: enum, required: true, allowed: [fileMissing, unreadable, tooLarge, syntax, unrepresentable, rootNotMap] }
      - { name: reason, type: text, required: true, constraints: 固定の文。入力の値と、ライブラリのエラー文は含めない }
```

## 要約

- 値の「生成」は、すべて Converter が行う。永続する状態はなく、入力のファイルを読み、文字列を返すだけである。
- 変換の途中で使う、キーと値の木(文字列、整数、浮動小数、真偽、配列、マップ)は、内部の表現であり、他の Unit に公開しない。
- 失敗の `kind`: `fileMissing`(ファイルがない)、`unreadable`(読めない・UTF-8 でない)、`tooLarge`(大きさの上限を超えた)、`syntax`(入力の文法の誤り)、`unrepresentable`(相手の形式で意味を変えずに表せない値がある)、`rootNotMap`(最上位がキーと値の集まり(マップ)でない)。

## Assumptions & Open Questions

- `keyPath` の書式(例: `jobs[2].transitions[0]`)は、Code Generation で決める。 [assumption]
- 論理的な型(text、integer など)は、技術に依存しない記述であり、実装の型への対応は Code Generation で決める。 [assumption]
