# Rules(u4-converter)

上流: requirements(FR7)、contract-summary(C4)、entities(この Unit の entities.md)、stories(US7.1)、この Unit の質問への回答(Q1〜Q3)。各規則は、`BR{group}.{seq}` の ID を持ち、出典(FR / Q)を示す。

```yaml
rules:
  # グループ1: 変換(Converter)
  - { id: BR1.1, statement: 入力の形式は、出力の形式(--to)と反対とみなす。拡張子は見ない, category: policy, applies_to: ConversionRequest, trigger: 変換の呼び出し時, logic: "IF targetFormat が toml THEN 入力を YAML として読む. IF targetFormat が yaml THEN 入力を TOML として読む", violation: 入力が、その形式として読めなければ BR1.4, source: FR7.1、Q2 }
  - { id: BR1.2, statement: キーと値の意味が同じになるように変換する(合否の基準は BR1.11)。内容が conductor の定義として正しいかは調べない, category: policy, applies_to: ConvertedText, trigger: 入力が読めたとき, logic: "文字列、整数、浮動小数、真偽値、配列、マップ(キーが文字列)を、相手の形式の同じ意味の表現に写す。キーの名前・値・値の型・配列の順序・入れ子の構造は、変えない。YAML は 1.2 のコアスキーマとして読み、`yes` / `no` / `on` / `off` は文字列、`true` / `false` だけが真偽値とする(定義の `on` キーは、文字列のキーとして保つ)。重複するキーは、文法の誤り(syntax)とする。文字列は、読み戻したときに文字列のままになるように、必要なら引用符で囲んで出力する(例: `yes`、`1`、`null`)。TOML の `nan` / `inf` / `-inf` と、YAML の `.nan` / `.inf` / `-.inf` は、互いに写し、`-0.0` の符号も保つ。定義としての検証(版、キーの種類、遷移など)は行わない(読み込み時に行われる)", violation: 表せない値は BR1.6, source: FR7.1、AC7.1.1、AC7.1.2、Q1 }
  - { id: BR1.3, statement: コメントと、マップのキーの記述順は、保持しない, category: policy, applies_to: ConvertedText, trigger: 変換の出力時, logic: "出力の中のコメントは書かない。マップのキーの順序は、出力の形式の既定に従い、入力と同じとは限らない。配列の要素の順序は保つ", violation: なし, source: FR7.1、AC7.1.1 }
  - { id: BR1.4, statement: 文法が誤っている入力は、変換せず、原因を返す, category: validation, applies_to: ConversionError, trigger: 入力を読むとき, logic: "IF 入力が、BR1.1 で決まった形式の文法として誤っている THEN ConversionError(syntax)(位置が分かれば line つき)を返し、出力を作らない", violation: 出力を作らない, source: FR7.2、AC7.1.3 }
  - { id: BR1.5, statement: 入力が読めないときは、変換せず、原因を返す, category: validation, applies_to: ConversionError, trigger: 入力を開くとき, logic: "IF ファイルがない THEN fileMissing. IF 権限がない・通常のファイルでない・UTF-8 でない THEN unreadable. IF 大きさが上限(1 MiB)を超える THEN tooLarge。いずれも出力を作らない", violation: 出力を作らない, source: FR7.2 }
  - { id: BR1.6, statement: 相手の形式で、意味を変えずに表せない値があるときは、変換せず、原因を返す, category: validation, applies_to: ConversionError, trigger: 値を相手の形式に写すとき, logic: "IF 入力に、次のいずれかがある THEN ConversionError(unrepresentable)(keyPath つき、値は含めない)を返し、出力を作らない: YAML の null、キーが文字列でないマップ、1つのファイルの中の複数の文書、TOML の日時の値(YAML に同じ型がない)、整数が相手の形式で表せる範囲を超える値。keyPath は、値の場所のキーの経路(入力に現れる順の深さ優先で、最初に見つかったもの)。キーが文字列でないときは、そのマップの経路に `(非文字列キー)` を付ける。複数の文書のときは、keyPath は付けず、reason で示す。値そのものは含めない", violation: 出力を作らない。黙って取り除いたり、近い値に置き換えたりしない, source: FR7.1、Q3 }
  - { id: BR1.7, statement: YAML のアンカー、エイリアス、マージキーは、展開して変換する。展開の大きさには上限を置く, category: policy, applies_to: ConvertedText, trigger: YAML を読むとき, logic: "アンカーで定義された値を参照するエイリアスは、参照先の値に展開した結果を、変換の対象にする。マージキー(`<<`)は、値がマップ、またはマップの並びのとき、その内容を、そのマップに展開する(同じキーは、マップ自身の値を優先する)。出力にアンカー、エイリアス、`<<` のキーは残らない。展開できないマージキー(値がマップでない)は、unrepresentable とする。再帰するアンカー(自分自身を含む)は、syntax とする。展開後の要素の数が 100,000 を超えるときは tooLarge(reason で展開後であることを示す)、入れ子の深さが 32 を超えるときは syntax とする(u1-core の定義の深さの上限と同じ)", violation: 出力を作らない, source: Q3 }
  - { id: BR1.8, statement: 最上位は、キーと値の集まり(マップ)でなければならない, category: constraint, applies_to: ConversionError, trigger: 入力を読んだあと, logic: "IF 入力の最上位が、マップでない(YAML の配列や単独の値、空の YAML 文書)THEN ConversionError(rootNotMap)を返し、出力を作らない。空の TOML は、空のテーブル(= 空のマップ)という正しい文書であり、空のマップとして扱い、空のマップを出力する。空の YAML は、null の文書であり、マップではないため、rootNotMap とする(非対称は、この形式の意味による)", violation: 出力を作らない, source: FR7.2(TOML の最上位は、マップのみ) }
  - { id: BR1.9, statement: 変換の結果は、変換後の形式として文法的に正しい文字列として返す。書き出しは、呼び出し側(Cli)の責務, category: policy, applies_to: ConvertedText, trigger: 変換の成功時, logic: "text を返す。ファイルや標準出力への書き出し、書き出しの失敗の報告は、この Unit で行わない", violation: なし, source: contract-summary の C4 }
  - { id: BR1.10, statement: エラーの表示の内容は、固定の文と、場所だけで組み立てる, category: constraint, applies_to: ConversionError, trigger: 失敗の報告時, logic: "reason は、kind ごとの固定の文(例: 文法の誤り、読めない、表せない値)。file と、分かれば line、keyPath を示す。AC7.1.3 の「原因」は、kind と、この場所で示す。入力の値と、ライブラリのエラー文は含めない", violation: なし, source: NFR4 の方針(同じ方針を u1-core の定義ファイルの読み取りも採る)、AC7.1.3 }
  - { id: BR1.11, statement: 変換の結果は、読み戻すと入力と同じ木になる(「意味が同じ」の合否の基準), category: constraint, applies_to: ConvertedText, trigger: 変換の成功時, logic: "変換後の文字列を、変換後の形式として読み戻した木が、入力の木と等しい。等しさの定義: マップはキーの集合と各値が等しい(キーの順序は問わない)、配列は要素と順序が等しい、文字列・整数・真偽値は値が等しい、浮動小数は値が等しい(NaN どうしは等しく、0.0 と -0.0 は区別する)。型が変わる(文字列 `1` が整数 1 になるなど)ものは、等しくない", violation: テストの失敗(この条件を満たさない出力は不具合), source: FR7.1、AC7.1.1、AC7.1.2 }
  - { id: BR1.12, statement: 失敗の判定の順序を固定する, category: policy, applies_to: ConversionError, trigger: 変換の途中, logic: "次の順に、最初に当てはまるものを返す。1) fileMissing. 2) unreadable(権限がない・通常のファイルでない). 3) tooLarge(入力が上限を超える). 4) unreadable(UTF-8 でない). 5) syntax(入力の全体を、文書の終わりまで読んだときの文法の誤り、重複するキー、再帰するアンカー、深さの超過). 6) unrepresentable(複数の文書). 7) rootNotMap(最上位がマップでない。最初の文書が対象). 8) アンカー・エイリアス・マージキーの展開(BR1.7。展開後の大きさの超過は tooLarge、展開できないものは unrepresentable). 9) 木の走査による unrepresentable(BR1.6)。同じ段階で複数あるときは、入力に現れる順で、最初のものを返す", violation: なし, source: テストの確定性 }
```

## 規則の要約

| 規則 | 内容 |
|------|------|
| BR1.1 | 入力の形式は、--to の反対 |
| BR1.2〜BR1.3 | 意味を保つ変換。コメントと順序は保持しない |
| BR1.4〜BR1.5 | 文法の誤り、読めない入力は、変換せず原因を返す |
| BR1.6〜BR1.8 | 表せない値、アンカーとエイリアス、最上位がマップでない入力 |
| BR1.9〜BR1.10 | 結果は文字列で返す(書き出しは Cli)。エラーの表示の組み立て |
| BR1.11〜BR1.12 | 読み戻すと同じ木(合否の基準)。失敗の判定の順序 |

## Assumptions & Open Questions

- 大きさの上限 1 MiB(BR1.5)は、u1-core の定義ファイルの上限に合わせた、この Unit の仮の値である。NFR Requirements で確定する。 [assumption]
- BR1.6 の「整数が表せる範囲」は、YAML と TOML のライブラリが扱える範囲に依存する。具体は、Code Generation で確かめる。 [assumption]
- 展開後の要素数 100,000 の上限(BR1.7)は、仮の値である。NFR Requirements で確定する。 [assumption]
- YAML 1.2 のコアスキーマで読む(BR1.2)ことを満たす YAML のライブラリを選べるかは、NFR Requirements の技術選択で確かめる。満たせなければ、この規則を見直す。 [assumption]
