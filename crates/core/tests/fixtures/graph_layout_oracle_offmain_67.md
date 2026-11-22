# レイアウトの正解データ(両方の飛び越し・分岐を含む、ジョブ 6〜7 個の形): graph-layout-unify

この文書は、`conductor graph` の text の図のうち、**「両方の飛び越し」か分岐を、1 つ以上含む、ジョブ 6〜7 個の形**の、**正解の図**である(深い入れ子の確認用)。

**注意: この図は、`git log --graph` の出力ではない。** 規則(`functional-spec.md` の WF2)の参照実装 `layout-reference.py` の `layout` から生成した。

- 形の表し方: `jobs=<ジョブの数> skips=<辺>`。`a>d` は「`a` の成功が `d` へ飛ぶ(警告は次のジョブへ)」。`a>>d` は「`a` が、成功でも警告でも `d` へ(両方の飛び越し)」。`a>c,d` は「`a` の成功が `c`、警告が `d` へ(分岐)」。`END` は終点。
- 作り方: `construction/code-generation/gen_offmain_67.py` で、乱数の種 20261122 から、ジョブ 6 か 7 個で、「両方の飛び越し」か分岐を 1 つ以上含む形を、重複なく 300 個、生成順に並べた(`python3 gen_offmain_67.py <layout-reference.py> <この文書>`)。
- **この文書のすべての例に、1 文字も違わず一致すること**が、レーンの配置の合格の条件である。

### jobs=7 skips=b>END c>g e>g,END f>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | |     e
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=a>f b>g,END e>END f>END
```text
*   a
|\
* \     b
 \ \ \
* | | | c
* | | | d
* | | |   e
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | | f
|\| |
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e c>>f d>END e>g,END
```text
* a
*   b
|\
* \   c
 \ \
* \ \   d
 \ \ \
  |_|/
 /| |
* | |     e
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>>e b>d c>END d>>f
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| |
* | |   d
 \ \ \
  |_|/
 /| |
* | | e
|/ /
* | f
* | g
|/
o @end
```
### jobs=6 skips=a>f b>e,END c>>e d>>f
```text
*   a
|\
* \     b
 \ \ \
* | | | c
 \| | |
* | | |   d
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=7 skips=a>c,e c>g
```text
*     a
 \ \
* | | b
 / /
* |   c
|\ \
* | | d
| |/
|/|
* | e
* | f
|/
* g
o @end
```
### jobs=7 skips=b>>f c>>f d>>END e>g,END
```text
* a
*   b
 \
* | c
 \|
* |   d
 \ \
* \ \     e
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=6 skips=a>f,END b>>d c>f d>f,END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \   c
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | |     d
 \ \ \ \
  | |/ /
  |/| /
  | |/
* / / e
 / /
* / f
|/
o @end
```
### jobs=6 skips=a>c,f b>e c>e,END
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | |     c
 \ \ \ \
  | |/ /
  |/| |
* / | | d
 / / /
* | / e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c,END c>e,f d>f,END e>>END
```text
*     a
 \ \
* | | b
 / /
* |     c
 \ \ \
* \ \ \     d
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |/ /
  |/| /
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=a>END c>f d>f e>>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>c,f b>e,g c>g,END e>g
```text
*     a
 \ \
* \ \     b
 \ \ \ \
  |_|/ /
 /| | |
* | | |     c
 \ \ \ \ \
  | |_|/ /
  |/| | |
* | | | | d
  |_|/ /
 /| | |
* | | | e
 \| | |
  |_|/
 /| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=6 skips=b>e,f d>>END e>END
```text
* a
*     b
 \ \
* | | c
* | |   d
 \ \ \
  |/ /
 /| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=7 skips=a>>c b>d d>>END f>END
```text
*   a
 \
* \   b
 \ \
  |/
 /|
* | c
|/
*   d
 \
* | e
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=a>d,g b>END c>>e d>>f e>g,END f>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \   c
 \ \ \ \
  |_|/ /
 /| | |
* | | |   d
 \ \ \ \
  |/ / /
 /| | |
* | | |     e
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |_|/
  |/| /
  | |/
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=d>f,END e>END
```text
* a
* b
* c
*     d
 \ \
* \ \   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=b>d c>>f d>f,END
```text
* a
*   b
|\
* \   c
 \ \
  |/
 /|
* |     d
 \ \ \
  | |/
  |/|
* / | e
 / /
* / f
|/
o @end
```
### jobs=7 skips=a>>g d>>g
```text
*   a
 \
* | b
* | c
* | d
 \|
* | e
* | f
 /
* g
o @end
```
### jobs=7 skips=b>END c>f,END d>f
```text
* a
*   b
|\
* \     c
 \ \ \
  | |/
* | | d
 \| |
* | | e
 / /
* | f
* | g
|/
o @end
```
### jobs=7 skips=d>>END e>g,END f>>END
```text
* a
* b
* c
*   d
 \
* \     e
 \ \ \
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
 /
o @end
```
### jobs=7 skips=a>e,g d>>g e>g f>>END
```text
*     a
 \ \
* | | b
* | | c
* | |   d
 \ \ \
  |/ /
 /| /
  |/
* | e
|\|
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>c,END b>END c>>f d>>END
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| /
  |/
* |   c
 \ \
* \ \   d
 \ \ \
  | |/
  |/|
* | | e
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=b>d,END c>f d>>f e>>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| |
* | | d
 \| |
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=b>g,END c>END e>g,END f>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  | |/
  |/|
* | | d
* | |     e
 \ \ \ \
  | |_|/
  |/| /
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=a>c,END b>d
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | | c
|/ /
* | d
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>>e b>END c>e d>f,END e>>END
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
  | |/
  |/|
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
 /
o @end
```
### jobs=7 skips=b>f,g c>f d>f
```text
* a
*     b
 \ \
* | | c
 \| |
* | | d
 \| |
* | | e
 / /
* / f
|/
* g
o @end
```
### jobs=6 skips=a>c,e b>>END c>e,END e>>END
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | |     c
 \ \ \ \
  | |_|/
  |/| /
  | |/
* / / d
 / /
* | e
 \|
* | f
 /
o @end
```
### jobs=6 skips=a>>END b>d,e
```text
*   a
 \
* \     b
 \ \ \
* | | | c
 / / /
* / / d
 / /
* | e
* | f
 /
o @end
```
### jobs=7 skips=a>d,g b>e,g c>f,g d>g,END f>>END
```text
*     a
 \ \
* \ \     b
 \ \ \ \
  | | |/
  | |/|
* | | |     c
 \ \ \ \ \
  | | |/ /
  | |/| /
  |_|_|/
 /| | |
* | | |     d
 \ \ \ \ \
  | |_|/ /
  |/| | /
  |_|_|/
 /| | |
* | | | e
  |_|/
 /| |
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=a>f b>>f c>g f>>END
```text
*   a
|\
* | b
 \|
* |   c
 \ \
* | | d
* | | e
  |/
 /|
* |   f
 \ \
  |/
 /|
* | g
 /
o @end
```
### jobs=6 skips=a>f,END c>>e d>END e>>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
* \ \ \   d
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>>END b>d,e c>f,g d>>f e>END f>END
```text
*   a
 \
* \     b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  |_|/ / /
 /| | | |
* | | | | d
 \| | | |
  |_|/ /
 /| | |
* | | |   e
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | f
 \| |
  |/
 /|
* | g
 /
o @end
```
### jobs=7 skips=b>f c>e,f d>f f>>END
```text
* a
*   b
|\
* \     c
 \ \ \
  | |/
* | |   d
 \ \ \
  |/ /
 /| /
  |/
* / e
|/
*   f
 \
* | g
 /
o @end
```
### jobs=6 skips=b>>f c>e d>>f
```text
* a
*   b
 \
* \   c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| /
  |/
* / e
 /
* f
o @end
```
### jobs=6 skips=b>>f c>e,END
```text
* a
*   b
 \
* \     c
 \ \ \
* | | | d
 / / /
* | / e
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=b>f,END
```text
* a
*     b
 \ \
* | | c
* | | d
* | | e
 / /
* / f
|/
o @end
```
### jobs=7 skips=a>>g b>>g c>g d>g,END e>g f>END
```text
*   a
 \
* | b
 \|
* | c
 \|
* |     d
 \ \ \
  | |/
  |/|
* | | e
 \| |
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=a>e b>>d c>e,END d>f,END e>>END
```text
*   a
|\
* \   b
 \ \
* \ \     c
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=b>>d e>g,END f>END
```text
* a
*   b
 \
* | c
 /
* d
*     e
 \ \
* \ \   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=c>f,g e>END
```text
* a
* b
*     c
 \ \
* | | d
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=c>e,f d>f e>END
```text
* a
* b
*     c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| /
  |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d c>END e>>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
* | e
 \|
* | f
 /
o @end
```
### jobs=7 skips=a>f,END c>>g e>g,END f>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
* | | | d
* | | |     e
 \ \ \ \ \
  | |/ / /
  |/| / /
  | | |/
  | |/|
  |_|/
 /| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=c>e,END e>>END
```text
* a
* b
*     c
 \ \
* | | d
 / /
* | e
 \|
* | f
 /
o @end
```
### jobs=6 skips=a>c,d d>END e>END
```text
*     a
 \ \
* | | b
 / /
* / c
|/
*   d
|\
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c,e c>>f d>f,END e>END
```text
*     a
 \ \
* | | b
 / /
* |   c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>>e c>>END d>f,END e>END
```text
*   a
 \
* | b
* |   c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>END b>f,g c>f e>g f>>END
```text
*   a
|\
* \     b
 \ \ \
* | | | c
 \| | |
* | | | d
* | | |   e
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=a>>END c>e,END d>g,END e>g,END f>END
```text
*   a
 \
* | b
* |     c
 \ \ \
  | |/
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |     e
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
 /
o @end
```
### jobs=7 skips=a>>e b>>f c>e,g d>f,END
```text
*   a
 \
* \   b
 \ \
* \ \     c
 \ \ \ \
  | |_|/
  |/| |
* | | |     d
 \ \ \ \ \
  |_|/ / /
 /| | | /
  | |_|/
  |/| |
* / | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>>d d>g e>g,END f>>END
```text
*   a
|\
* | b
 \|
* | c
 /
*   d
|\
* \     e
 \ \ \
  | |/
  |/|
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=b>e c>g d>END e>g f>>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| | |/
| |/|
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=b>>e c>END e>g,END
```text
* a
*   b
 \
* \   c
 \ \
* | | d
  |/
 /|
* |     e
 \ \ \
  | |/
* / / f
 / /
* / g
|/
o @end
```
### jobs=6 skips=a>c b>e c>e,END d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |     c
 \ \ \
  | |/
  |/|
* | |   d
 \ \ \
  |/ /
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>>d b>d c>f e>>END
```text
*   a
 \
* | b
 \|
* |   c
 \ \
  |/
 /|
* | d
* |   e
 \ \
  |/
 /|
* | f
 /
o @end
```
### jobs=7 skips=a>>d b>d d>>g e>>g f>>END
```text
*   a
 \
* | b
 \|
* | c
 /
*   d
 \
* | e
 \|
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>END b>f,END c>e,f d>f,END
```text
*   a
|\
* \     b
 \ \ \
  | |/
* | |     c
 \ \ \ \
  | |/ /
* | | |     d
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |/ /
  |/| /
  | |/
* / / e
 / /
* / f
|/
o @end
```
### jobs=6 skips=a>>f b>e,f d>>END e>END
```text
*   a
 \
* \     b
 \ \ \
  | |/
* | | c
* | |   d
 \ \ \
  |/ /
 /| |
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=c>>e e>g
```text
* a
* b
*   c
 \
* | d
 /
*   e
|\
* | f
|/
* g
o @end
```
### jobs=7 skips=a>>END b>d c>e,END d>f f>END
```text
*   a
 \
* \   b
 \ \
* \ \     c
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |   d
 \ \ \
  |/ /
 /| |
* | | e
 / /
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=a>c c>>g d>>END f>END
```text
*   a
|\
* | b
|/
*   c
 \
* \   d
 \ \
* | | e
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>END c>>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
 \ \ \
  |_|/
 /| |
* | |   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d,e c>f e>>g
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  |/ /
 /| |
* | | d
| |/
|/|
* |   e
 \ \
  |/
 /|
* | f
 /
* g
o @end
```
### jobs=7 skips=a>c b>f,END d>g,END f>>END
```text
*   a
|\
* \     b
 \ \ \
  |_|/
 /| |
* | | c
* | |     d
 \ \ \ \
  | | |/
  | |/|
* | | | e
  |_|/
 /| |
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=a>END d>END e>END f>>END
```text
*   a
|\
* | b
* | c
* | d
|\|
* | e
|\|
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=a>>e c>g d>g,END e>g f>END
```text
*   a
 \
* | b
* |   c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | | e
|\| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=a>e,f b>d c>f
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \   c
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | d
  |/
 /|
* | e
|/
* f
o @end
```
### jobs=7 skips=a>g,END b>d c>e,f d>f,END f>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  |_|/ / /
 /| | | |
* | | | |     d
 \ \ \ \ \ \
  |_|/ / / /
 /| | / / /
  | |/ / /
  |/| | /
  | | |/
  | |/|
* / | | e
 / / /
* | / f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>d b>f,END d>END e>END
```text
*   a
|\
* \     b
 \ \ \
* | | | c
  |_|/
 /| |
* | |   d
|\ \ \
| | |/
| |/|
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=c>>END
```text
* a
* b
*   c
 \
* | d
* | e
* | f
 /
o @end
```
### jobs=6 skips=a>f b>>END c>f e>END
```text
*   a
|\
* \   b
 \ \
* \ \   c
 \ \ \
  | |/
  |/|
* | | d
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=b>f e>g f>>END
```text
* a
*   b
|\
* | c
* | d
* |   e
|\ \
| |/
|/|
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=b>f c>>g e>g,END f>END
```text
* a
*   b
|\
* \   c
 \ \
* | | d
* | |     e
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=a>d b>END d>f,END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* |     d
 \ \ \
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=d>>END f>END
```text
* a
* b
* c
*   d
 \
* | e
* | f
 \|
* | g
 /
o @end
```
### jobs=6 skips=a>d b>>d
```text
*   a
|\
* | b
 \|
* | c
 /
* d
* e
* f
o @end
```
### jobs=7 skips=a>e,f b>g c>g,END f>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  | |/ / /
  |/| | |
* | | | | d
  |_|/ /
 /| | |
* | | | e
| |_|/
|/| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d,f d>f,END f>END
```text
*   a
|\
* \     b
 \ \ \
* | | | c
 / / /
* | |     d
 \ \ \ \
  | |/ /
  |/| /
  | |/
* / / e
 / /
* | f
|\|
* | g
|/
o @end
```
### jobs=6 skips=a>d b>e,END c>>END e>END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \   c
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=7 skips=a>e,END b>>e c>>END e>g,END f>>END
```text
*     a
 \ \
* | | b
 \| |
* | |   c
 \ \ \
  | |/
  |/|
* | | d
  |/
 /|
* |     e
 \ \ \
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=a>>f b>>f c>g e>END
```text
*   a
 \
* | b
 \|
* |   c
 \ \
* | | d
* | |   e
 \ \ \
  |_|/
 /| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f d>>f f>END
```text
*   a
|\
* | b
* | c
* | d
 \|
* | e
 /
*   f
|\
* | g
|/
o @end
```
### jobs=6 skips=a>>END b>>f c>f,END e>END
```text
*   a
 \
* \   b
 \ \
* \ \     c
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | | d
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
 /
o @end
```
### jobs=7 skips=a>e,g e>>END
```text
*     a
 \ \
* | | b
* | | c
* | | d
 / /
* |   e
 \ \
* | | f
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>e,f b>d,f c>f,END d>f
```text
*     a
 \ \
* \ \     b
 \ \ \ \
  | | |/
  | |/|
* | | |     c
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |/ /
  |/| |
* | | | d
 \| | |
  |_|/
 /| |
* | | e
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>g b>>g d>>END f>END
```text
*   a
|\
* | b
 \|
* | c
* |   d
 \ \
* | | e
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>d,e c>>e d>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  |/ /
 /| /
  |/
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>>f b>>d c>END e>END
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| |
* | | d
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>e,END b>g c>END d>>f e>>END f>>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \   c
 \ \ \ \
  | |_|/
  |/| |
* | | |   d
 \ \ \ \
  |_|_|/
 /| | |
* | | |   e
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | / f
 \| |
  |/
 /|
* | g
 /
o @end
```
### jobs=7 skips=b>>END d>f,END e>g,END f>>END
```text
* a
*   b
 \
* | c
* |     d
 \ \ \
  | |/
* | |     e
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
 /
o @end
```
### jobs=7 skips=a>d,END c>>END e>>END f>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  |/ /
 /| /
  |/
* | d
* | e
 \|
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=b>>e c>f f>END
```text
* a
*   b
 \
* \   c
 \ \
* | | d
  |/
 /|
* | e
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>>g c>END d>f,END
```text
*   a
 \
* | b
* |   c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
* / / / e
 / / /
* | / f
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>>f b>f,END c>>e d>f e>g,END f>END
```text
*   a
 \
* \     b
 \ \ \
  | |/
  |/|
* | |   c
 \ \ \
* \ \ \   d
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | |     e
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=b>>f d>END
```text
* a
*   b
 \
* | c
* |   d
 \ \
* | | e
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>c,f c>>END d>f f>>END
```text
*     a
 \ \
* | | b
 / /
* |   c
 \ \
* \ \   d
 \ \ \
  | |/
  |/|
* / | e
 / /
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=b>END c>e,g e>g f>END
```text
* a
*   b
|\
* \     c
 \ \ \
* | | | d
 / / /
* | | e
|\| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=7 skips=d>f,g
```text
* a
* b
* c
*     d
 \ \
* | | e
 / /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>d b>>d c>f d>END e>g f>END
```text
*   a
|\
* | b
 \|
* |   c
 \ \
  |/
 /|
* |   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=a>END c>e,f d>END e>END
```text
*   a
|\
* | b
* |     c
 \ \ \
* \ \ \   d
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>>END d>f
```text
*   a
|\
* | b
* |   c
 \ \
* \ \   d
 \ \ \
  |_|/
 /| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=7 skips=a>c,END f>>END
```text
*     a
 \ \
* | | b
 / /
* | c
* | d
* | e
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=a>>d b>e c>e,g e>g,END f>END
```text
*   a
 \
* \   b
 \ \
* \ \     c
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | | d
|/ /
* |     e
 \ \ \
  | |/
  |/|
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=b>f d>f,END e>END
```text
* a
*   b
|\
* | c
* |     d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>>END c>e,f d>END
```text
*   a
 \
* | b
* |     c
 \ \ \
* \ \ \   d
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | e
  |/
 /|
* | f
* | g
 /
o @end
```
### jobs=7 skips=a>e b>e,g c>g,END e>END
```text
*   a
|\
* \     b
 \ \ \
  | |/
  |/|
* | |     c
 \ \ \ \
  | |_|/
  |/| |
* | | | d
  |_|/
 /| |
* | |   e
|\ \ \
| | |/
| |/|
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>END d>g,END f>>END
```text
* a
* b
*   c
|\
* \     d
 \ \ \
  | |/
* | | e
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=c>>f d>g f>>END
```text
* a
* b
*   c
 \
* \   d
 \ \
* | | e
  |/
 /|
* |   f
 \ \
  |/
 /|
* | g
 /
o @end
```
### jobs=6 skips=a>>END b>e,END c>>END d>f,END
```text
*   a
 \
* \     b
 \ \ \
  | |/
* | |   c
 \ \ \
  | |/
  |/|
* | |     d
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | | e
 / /
* / f
 /
o @end
```
### jobs=6 skips=a>c b>e,f c>>f e>END
```text
*   a
|\
* \     b
 \ \ \
  |_|/
 /| |
* | |   c
 \ \ \
  | |/
  |/|
* | | d
  |/
 /|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=7 skips=a>f b>d,e c>f,g d>f,g e>g,END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  |_|/ / /
 /| | | /
  | |_|/
  |/| |
* | | |     d
 \ \ \ \ \
  | |/ / /
  |/| / /
  | |/ /
  |_|/
 /| |
* | |     e
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* / | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>>d f>END
```text
*   a
 \
* | b
* | c
 /
* d
* e
*   f
|\
* | g
|/
o @end
```
### jobs=6 skips=a>>f b>>END d>END e>END
```text
*   a
 \
* \   b
 \ \
* | | c
* | | d
 \| |
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>END c>>f d>f,g e>g,END f>>END
```text
*   a
|\
* | b
* |   c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
  |/| |
* | | |     e
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |/ /
  |/| /
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
 /
o @end
```
### jobs=6 skips=a>d b>>END c>e
```text
*   a
|\
* \   b
 \ \
* \ \   c
 \ \ \
  |_|/
 /| |
* | | d
|/ /
* | e
* | f
|/
o @end
```
### jobs=7 skips=a>f b>>e c>f,END d>>f e>g,END
```text
*   a
|\
* \   b
 \ \
* \ \     c
 \ \ \ \
  | |_|/
  |/| |
* | | | d
 \| | |
  |_|/
 /| |
* | |     e
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>f,END c>g d>>f e>END
```text
*   a
|\
* \     b
 \ \ \
  | |/
* | |   c
 \ \ \
* \ \ \   d
 \ \ \ \
  | |/ /
  |/| |
* | | |   e
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=b>>e c>e
```text
* a
*   b
 \
* | c
 \|
* | d
 /
* e
* f
o @end
```
### jobs=6 skips=a>>c b>>d d>f,END e>END
```text
*   a
 \
* \   b
 \ \
  |/
 /|
* | c
|/
*     d
 \ \
* \ \   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>g b>f c>>g d>g e>>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
 \ \ \
  | |/
  |/|
* | | d
 \| |
* | |   e
 \ \ \
  |_|/
 /| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g,END b>>END d>g e>END f>>END
```text
*     a
 \ \
* \ \   b
 \ \ \
  | |/
  |/|
* | | c
* | |   d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  | |/
  |/|
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>c e>g,END f>END
```text
*   a
|\
* | b
|/
* c
* d
*     e
 \ \
* \ \   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=a>>e b>f e>>END
```text
*   a
 \
* \   b
 \ \
* | | c
* | | d
  |/
 /|
* |   e
 \ \
  |/
 /|
* | f
 /
o @end
```
### jobs=6 skips=b>f d>>f
```text
* a
*   b
|\
* | c
* | d
 \|
* | e
 /
* f
o @end
```
### jobs=7 skips=a>c,f b>>g c>>g e>g
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | | c
 \| |
* | | d
* | | e
 \| |
  |/
 /|
* | f
|/
* g
o @end
```
### jobs=6 skips=b>>f c>END d>f,END e>END
```text
* a
*   b
 \
* \   c
 \ \
* \ \     d
 \ \ \ \
  | |_|/
  |/| /
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=d>>f e>END
```text
* a
* b
* c
*   d
 \
* \   e
 \ \
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>d,END c>g e>g
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  |/ /
 /| |
* | | d
* | | e
|\| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=6 skips=a>END b>d,END c>f e>>END
```text
*   a
|\
* \     b
 \ \ \
  | |/
* | |   c
 \ \ \
  |/ /
 /| |
* | | d
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
 /
o @end
```
### jobs=7 skips=a>>f b>d e>>g f>END
```text
*   a
 \
* \   b
 \ \
* | | c
 / /
* | d
* |   e
 \ \
  |/
 /|
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=b>e d>f,END e>END
```text
* a
*   b
|\
* | c
* |     d
 \ \ \
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>g c>g,END d>END e>>g
```text
*   a
|\
* | b
* |     c
 \ \ \
  | |/
  |/|
* | |   d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  | |/
  |/|
* / | f
 / /
* / g
|/
o @end
```
### jobs=7 skips=a>f,g b>d c>>f e>END f>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \   c
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | | d
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
|\| |
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=a>>c d>f e>END
```text
*   a
 \
* | b
 /
* c
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e,END b>>f d>f,END
```text
*     a
 \ \
* \ \   b
 \ \ \
* | | | c
* | | |     d
 \ \ \ \ \
  | |/ / /
  |/| / /
  | | |/
  | |/|
  |_|/
 /| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=b>>f c>END d>>END e>END
```text
* a
*   b
 \
* \   c
 \ \
* | | d
 \| |
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>>d c>f d>END f>>END
```text
*   a
 \
* | b
* |   c
 \ \
  |/
 /|
* |   d
|\ \
* | | e
| |/
|/|
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=b>d,g c>f,g d>f,END f>END
```text
* a
*     b
 \ \
* \ \     c
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |     d
 \ \ \ \
  | |/ /
  |/| |
* / | | e
 / / /
* | / f
|\| |
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>>f d>END f>>END
```text
* a
*   b
 \
* | c
* |   d
 \ \
* | | e
  |/
 /|
* | f
 \|
* | g
 /
o @end
```
### jobs=6 skips=a>c,END b>f
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | | c
* | | d
* | | e
|/ /
* / f
|/
o @end
```
### jobs=7 skips=a>>f b>>g c>f d>g e>g f>>END
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
  | |/
  |/|
* | |   d
 \ \ \
  | |/
  |/|
* | | e
 \| |
  |/
 /|
* |   f
 \ \
  |/
 /|
* | g
 /
o @end
```
### jobs=7 skips=a>c,g b>g,END
```text
*     a
 \ \
* \ \     b
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* | | c
* | | d
* | | e
* | | f
|/ /
* / g
|/
o @end
```
### jobs=6 skips=a>c,d b>e c>e d>f,END
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | | c
|\| |
| |/
|/|
* |     d
 \ \ \
  |_|/
 /| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>>END b>d,e c>>e d>END
```text
*   a
 \
* \     b
 \ \ \
* \ \ \   c
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | |   d
 \ \ \
  |/ /
 /| /
  |/
* | e
* | f
 /
o @end
```
### jobs=6 skips=a>f,END c>f e>END
```text
*     a
 \ \
* | | b
* | | c
 \| |
* | | d
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=b>e c>e,f
```text
* a
*   b
|\
* \     c
 \ \ \
  | |/
  |/|
* / | d
 / /
* / e
|/
* f
o @end
```
### jobs=7 skips=a>>d b>>f e>g,END
```text
*   a
 \
* \   b
 \ \
* | | c
  |/
 /|
* | d
* |     e
 \ \ \
  |_|/
 /| |
* | | f
 / /
* / g
|/
o @end
```
### jobs=6 skips=a>f c>f d>>f
```text
*   a
|\
* | b
* | c
|\|
* | d
 \|
* | e
 /
* f
o @end
```
### jobs=7 skips=b>>g c>g d>f,END
```text
* a
*   b
 \
* | c
 \|
* |     d
 \ \ \
* | | | e
 / / /
* | / f
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>f,END c>e,END d>END f>END
```text
*   a
|\
* \     b
 \ \ \
  | |/
  |/|
* | |     c
 \ \ \ \
  | | |/
  | |/|
* | | |   d
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | / e
  |/
 /|
* | f
|\|
* | g
|/
o @end
```
### jobs=7 skips=c>f,END
```text
* a
* b
*     c
 \ \
* | | d
* | | e
 / /
* | f
* | g
|/
o @end
```
### jobs=6 skips=a>>END b>END c>>f e>END
```text
*   a
 \
* | b
 \|
* |   c
 \ \
* | | d
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
 /
o @end
```
### jobs=7 skips=c>g d>f,g e>END f>END
```text
* a
* b
*   c
|\
* \     d
 \ \ \
  | |/
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
|\| |
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=a>f b>d,e c>END d>f e>END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \   c
 \ \ \ \
  |/ / /
 /| | |
* | | |   d
|\ \ \ \
| |_|/ /
|/| | /
| | |/
| |/|
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>d b>f d>>g e>>g f>>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* |   d
 \ \
* | | e
 \| |
  |/
 /|
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>>c b>d d>>f
```text
*   a
 \
* \   b
 \ \
  |/
 /|
* | c
|/
*   d
 \
* | e
 /
* f
o @end
```
### jobs=7 skips=c>e,g e>g,END f>END
```text
* a
* b
*     c
 \ \
* | | d
 / /
* |     e
 \ \ \
  | |/
  |/|
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=a>>d e>>END
```text
*   a
 \
* | b
* | c
 /
* d
*   e
 \
* | f
 /
o @end
```
### jobs=7 skips=c>f d>>g
```text
* a
* b
*   c
|\
* \   d
 \ \
* | | e
  |/
 /|
* | f
|/
* g
o @end
```
### jobs=6 skips=d>f e>>END
```text
* a
* b
* c
*   d
|\
* \   e
 \ \
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=c>e d>f,END e>END
```text
* a
* b
*   c
|\
* \     d
 \ \ \
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>g c>g d>f e>>END f>>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
* \ \   e
 \ \ \
  |/ /
 /| |
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=c>>e d>END
```text
* a
* b
*   c
 \
* \   d
 \ \
  |/
 /|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c,d b>>f c>f d>END e>END
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| |
* | | c
|\| |
| |/
|/|
* |   d
|\ \
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=7 skips=a>f,g b>>f e>>END f>END
```text
*     a
 \ \
* | | b
 \| |
* | | c
* | | d
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
|\| |
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=a>>END d>END e>END
```text
*   a
 \
* | b
* | c
* | d
 \|
* | e
 \|
* | f
 /
o @end
```
### jobs=6 skips=a>f b>d,e c>f,END d>END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  |_|/ / /
 /| | | /
  | |_|/
  |/| |
* | | |   d
|\ \ \ \
| | |/ /
| |/| /
| |_|/
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=7 skips=a>c c>g d>f,END e>END
```text
*   a
|\
* | b
|/
*   c
|\
* \     d
 \ \ \
* \ \ \   e
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=b>d,f c>f d>f e>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| /
  |/
* | d
|\|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>END e>>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
| |/
|/|
* | e
 \|
* | f
 /
o @end
```
### jobs=7 skips=a>c b>END c>f,g e>>g
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |     c
 \ \ \
* | | | d
* | | |   e
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=6 skips=a>>END b>>d c>f d>END e>END
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| |
* | |   d
 \ \ \
  | |/
  |/|
* | | e
 \| |
  |/
 /|
* | f
 /
o @end
```
### jobs=7 skips=a>>c b>g d>>g e>>g f>END
```text
*   a
 \
* \   b
 \ \
  |/
 /|
* | c
* | d
 \|
* | e
 \|
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>>f e>END
```text
*   a
 \
* | b
* | c
* | d
* |   e
 \ \
  |/
 /|
* | f
* | g
|/
o @end
```
### jobs=6 skips=a>d b>d c>f,END d>f,END e>>END
```text
*   a
|\
* | b
|\|
* |     c
 \ \ \
  |_|/
 /| |
* | |     d
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=b>e c>f d>END e>>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>e,g c>g d>f
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  | |/
  |/|
* | |   d
 \ \ \
  |_|/
 /| |
* | | e
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=b>f f>>END
```text
* a
*   b
|\
* | c
* | d
* | e
|/
*   f
 \
* | g
 /
o @end
```
### jobs=7 skips=a>d b>d c>>END d>>f
```text
*   a
|\
* | b
|\|
* |   c
 \ \
  |/
 /|
* |   d
 \ \
* | | e
 / /
* | f
* | g
|/
o @end
```
### jobs=6 skips=b>>e d>END e>END
```text
* a
*   b
 \
* | c
* |   d
 \ \
  |/
 /|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e,f b>f c>f,END d>f e>END
```text
*     a
 \ \
* \ \   b
 \ \ \
  | |/
  |/|
* | |     c
 \ \ \ \
  | |/ /
  |/| |
* | | | d
 \| | |
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>>e c>>f d>f,g e>>END
```text
*   a
 \
* | b
* |   c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
  |/
 /|
* | g
 /
o @end
```
### jobs=7 skips=a>d,e b>f,END c>>g d>>END e>>END
```text
*     a
 \ \
* \ \     b
 \ \ \ \
* \ \ \ \   c
 \ \ \ \ \
  |_|_|/ /
 /| | | |
* | | | |   d
 \ \ \ \ \
  | |_|/ /
  |/| | /
  |_|_|/
 /| | |
* | | | e
 \| | |
  |_|/
 /| |
* | | f
  |/
 /|
* | g
 /
o @end
```
### jobs=6 skips=a>d b>d,e d>f,END e>>END
```text
*   a
|\
* \     b
 \ \ \
  | |/
  |/|
* / | c
 / /
* |     d
 \ \ \
  |_|/
 /| |
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>>END d>f,g
```text
*   a
 \
* | b
* | c
* |     d
 \ \ \
* | | | e
 / / /
* / / f
 / /
* / g
 /
o @end
```
### jobs=7 skips=b>>END c>f d>END e>g,END
```text
* a
*   b
 \
* \   c
 \ \
* \ \   d
 \ \ \
  | |/
  |/|
* | |     e
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | | f
 / /
* / g
 /
o @end
```
### jobs=7 skips=a>END b>e c>e,END d>g,END f>>END
```text
*   a
|\
* \   b
|\ \
* \ \     c
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | | e
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
 /
o @end
```
### jobs=7 skips=b>e d>f,g e>g,END f>END
```text
* a
*   b
|\
* | c
* |     d
 \ \ \
  |_|/
 /| |
* | |     e
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=a>e d>>END
```text
*   a
|\
* | b
* | c
* |   d
 \ \
  |/
 /|
* | e
* | f
|/
o @end
```
### jobs=6 skips=d>f,END
```text
* a
* b
* c
*     d
 \ \
* | | e
 / /
* / f
|/
o @end
```
### jobs=7 skips=a>>d b>d,e d>>END e>>END f>>END
```text
*   a
 \
* \     b
 \ \ \
  | |/
  |/|
* / | c
 / /
* |   d
 \ \
  |/
 /|
* | e
 \|
* | f
 \|
* | g
 /
o @end
```
### jobs=6 skips=b>d,END c>>END d>f,END e>>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| /
  |/
* |     d
 \ \ \
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=b>d,f c>END e>>g
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| |
* | | d
* | |   e
 \ \ \
  |_|/
 /| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=6 skips=c>f d>END e>>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=b>>e c>>f d>END e>END
```text
* a
*   b
 \
* \   c
 \ \
* \ \   d
 \ \ \
  |_|/
 /| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=7 skips=a>f,END b>d,f d>f,g e>g f>END
```text
*     a
 \ \
* \ \     b
 \ \ \ \
  | |/ /
* / / / c
 / / /
* | |     d
 \ \ \ \
  | |/ /
  |/| |
* | | |   e
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=b>e,f c>e,END d>END e>>END
```text
* a
*     b
 \ \
* \ \     c
 \ \ \ \
  | |/ /
  |/| |
* | | |   d
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | / e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d,e c>e e>>END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \   c
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* / / d
|/ /
* | e
 \|
* | f
 /
o @end
```
### jobs=7 skips=c>>g
```text
* a
* b
*   c
 \
* | d
* | e
* | f
 /
* g
o @end
```
### jobs=7 skips=a>d,END b>END c>END d>f,g e>END
```text
*     a
 \ \
* \ \   b
 \ \ \
  | |/
  |/|
* | | c
 \| |
  |/
 /|
* |     d
 \ \ \
* \ \ \   e
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d,e b>>d c>END e>END
```text
*     a
 \ \
* | | b
 \| |
* | |   c
 \ \ \
  |/ /
 /| |
* | | d
| |/
|/|
* | e
|\|
* | f
* | g
|/
o @end
```
### jobs=6 skips=b>e c>>e d>>f e>>END
```text
* a
*   b
|\
* | c
 \|
* |   d
 \ \
  |/
 /|
* |   e
 \ \
  |/
 /|
* | f
 /
o @end
```
### jobs=6 skips=b>d,e c>f,END d>f,END
```text
* a
*     b
 \ \
* \ \     c
 \ \ \ \
  |_|/ /
 /| | |
* | | |     d
 \ \ \ \ \
  | |/ / /
  |/| / /
  | |/ /
  |_|/
 /| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=c>END d>f e>>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* \ \   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=a>>c b>>e d>>f
```text
*   a
 \
* \   b
 \ \
  |/
 /|
* | c
* |   d
 \ \
  |/
 /|
* | e
 /
* f
o @end
```
### jobs=6 skips=a>d,e b>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* | | | c
  |/ /
 /| |
* | | d
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=7 skips=b>g c>f,g d>f f>>END
```text
* a
*   b
|\
* \     c
 \ \ \
  | |/
* | | d
 \| |
* | | e
 / /
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=b>e,g e>g,END
```text
* a
*     b
 \ \
* | | c
* | | d
 / /
* |     e
 \ \ \
  | |/
  |/|
* / | f
 / /
* / g
|/
o @end
```
### jobs=7 skips=b>e d>g,END e>g
```text
* a
*   b
|\
* | c
* |     d
 \ \ \
  |_|/
 /| |
* | | e
|\| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>>d e>g,END f>END
```text
*   a
|\
* \   b
 \ \
* | | c
 / /
* | d
* |     e
 \ \ \
  |_|/
 /| |
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=a>c,d b>>d c>e
```text
*     a
 \ \
* \ \   b
 \ \ \
  |/ /
 /| /
  |/
* |   c
|\ \
| |/
|/|
* | d
|/
* e
* f
o @end
```
### jobs=7 skips=c>>e d>f e>>g
```text
* a
* b
*   c
 \
* \   d
 \ \
  |/
 /|
* |   e
 \ \
  |/
 /|
* | f
 /
* g
o @end
```
### jobs=6 skips=a>d b>>e d>f
```text
*   a
|\
* \   b
 \ \
* | | c
  |/
 /|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=b>d,f c>e,END e>>END
```text
* a
*     b
 \ \
* \ \     c
 \ \ \ \
  |_|/ /
 /| | |
* | | | d
|/ / /
* | / e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=a>>d b>>f c>END e>END
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
  |_|/
 /| |
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f,END b>>e c>e,END d>f e>>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  | |/ / /
  |/| | /
  | | |/
  | |/|
* | | |   d
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>f,END b>>e f>>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* | | | c
* | | | d
 / / /
* / / e
 / /
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=c>e,f d>>g e>>g
```text
* a
* b
*     c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| |
* | | e
 \| |
  |/
 /|
* | f
|/
* g
o @end
```
### jobs=7 skips=c>f,g d>f,g
```text
* a
* b
*     c
 \ \
* \ \     d
 \ \ \ \
  | |/ /
  |/| /
  | |/
* / / e
 / /
* / f
|/
* g
o @end
```
### jobs=7 skips=d>END e>>g f>>END
```text
* a
* b
* c
*   d
|\
* \   e
 \ \
* \ \   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=b>d,f c>e,f d>f,END e>END
```text
* a
*     b
 \ \
* \ \     c
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>d c>>f d>f,END e>g
```text
*   a
|\
* | b
* |   c
 \ \
  |/
 /|
* |     d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>>END c>>e d>>g e>END f>END
```text
*   a
|\
* | b
 \|
* |   c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| |
* | |   e
 \ \ \
  | |/
  |/|
* | | f
 \| |
  |/
 /|
* | g
 /
o @end
```
### jobs=6 skips=a>>e b>END c>f d>>f e>END
```text
*   a
 \
* \   b
 \ \
* \ \   c
 \ \ \
* | | | d
 \| | |
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=e>>g
```text
* a
* b
* c
* d
*   e
 \
* | f
 /
* g
o @end
```
### jobs=7 skips=b>>e c>END d>END f>>END
```text
* a
*   b
 \
* \   c
 \ \
* | | d
 \| |
  |/
 /|
* | e
* | f
 \|
* | g
 /
o @end
```
### jobs=6 skips=b>f c>f,END d>>f e>END
```text
* a
*   b
|\
* \     c
 \ \ \
  | |/
  |/|
* | | d
 \| |
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=c>g e>g,END f>>END
```text
* a
* b
*   c
|\
* | d
* |     e
 \ \ \
  | |/
  |/|
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=a>g b>>d c>>g d>>g e>END
```text
*   a
|\
* \   b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| /
  |/
* | d
 \|
* |   e
 \ \
* | | f
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=b>f,END d>f,END e>END
```text
* a
*     b
 \ \
* | | c
* | |     d
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=7 skips=a>c,f c>e d>END
```text
*     a
 \ \
* | | b
 / /
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=6 skips=b>d,f c>END d>>END e>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  |/ /
 /| |
* | | d
 \| |
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=b>f,g c>>e e>>g f>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
* | | | d
 / / /
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f d>f,g e>END
```text
*   a
|\
* | b
* | c
* |     d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=a>c,e c>e,END d>f,END e>END
```text
*     a
 \ \
* | | b
 / /
* |     c
 \ \ \
  | |/
  |/|
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=b>f,END c>END d>>END e>>END f>>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  | |/
  |/|
* | | d
 \| |
* | | e
 \| |
  |/
 /|
* | f
 \|
* | g
 /
o @end
```
### jobs=6 skips=a>>f b>e c>e e>END
```text
*   a
 \
* \   b
 \ \
* | | c
 \| |
* | | d
 / /
* |   e
 \ \
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=b>END c>f,g d>f e>END f>>END
```text
* a
*   b
|\
* \     c
 \ \ \
* | | | d
 \| | |
* | | |   e
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=b>d d>g e>g,END f>>END
```text
* a
*   b
|\
* | c
|/
*   d
|\
* \     e
 \ \ \
  | |/
  |/|
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=b>f c>e,END d>END f>END
```text
* a
*   b
|\
* \     c
 \ \ \
* \ \ \   d
 \ \ \ \
  |/ / /
 /| / /
  |/ /
* | / e
| |/
|/|
* | f
|\|
* | g
|/
o @end
```
### jobs=6 skips=a>>f c>>END d>>f e>END
```text
*   a
 \
* | b
* |   c
 \ \
* \ \   d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=b>END c>END d>>f e>>END
```text
* a
*   b
|\
* | c
|\|
* |   d
 \ \
* \ \   e
 \ \ \
  |/ /
 /| /
  |/
* / f
|/
o @end
```
### jobs=6 skips=a>d,e c>>END d>f,END e>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  |/ /
 /| |
* | |     d
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>f b>f,END c>f,g d>f e>g,END
```text
*   a
|\
* \     b
 \ \ \
  | |/
  |/|
* | |     c
 \ \ \ \
  | |/ /
  |/| |
* | | | d
 \| | |
* | | |     e
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |/ /
  |/| /
  | |/
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>END c>f,g d>g f>>END
```text
* a
*   b
|\
* \     c
 \ \ \
* \ \ \   d
 \ \ \ \
  | |/ /
  |/| |
* | | | e
  |/ /
 /| |
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=7 skips=b>d,g d>>END e>>END
```text
* a
*     b
 \ \
* | | c
 / /
* |   d
 \ \
* | | e
 \| |
* | | f
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=c>e,END e>END
```text
* a
* b
*     c
 \ \
* | | d
 / /
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>>END e>>END
```text
* a
*   b
 \
* | c
* | d
* | e
 \|
* | f
 /
o @end
```
### jobs=6 skips=a>c c>e,END d>f,END
```text
*   a
|\
* | b
|/
*     c
 \ \
* \ \     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* / / e
|/ /
* / f
|/
o @end
```
### jobs=7 skips=c>>END d>f
```text
* a
* b
*   c
 \
* \   d
 \ \
* | | e
 / /
* | f
* | g
 /
o @end
```
### jobs=6 skips=b>e,f c>>e e>>END
```text
* a
*     b
 \ \
* | | c
 \| |
* | | d
 / /
* |   e
 \ \
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>e c>END d>g,END f>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \     d
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | | e
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=7 skips=a>f b>>d d>f,END e>>END f>END
```text
*   a
|\
* \   b
 \ \
* | | c
 / /
* |     d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* | f
|\|
* | g
|/
o @end
```
### jobs=7 skips=a>>f b>>g f>END
```text
*   a
 \
* \   b
 \ \
* | | c
* | | d
* | | e
  |/
 /|
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>>f d>>END e>g f>END
```text
* a
*   b
|\
* \   c
 \ \
* \ \   d
 \ \ \
* \ \ \   e
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=7 skips=a>g,END b>d d>>f e>END
```text
*     a
 \ \
* \ \   b
 \ \ \
* | | | c
 / / /
* | |   d
 \ \ \
* \ \ \   e
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | f
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>d b>e,f c>e,END d>f,END e>END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \     c
 \ \ \ \ \
  | |/ / /
  |/| | /
  |_|_|/
 /| | |
* | | |     d
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |_|/
  |/| /
  | |/
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>>c c>e,g d>f
```text
*   a
 \
* | b
 /
*     c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| |
* | | e
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=e>g,END
```text
* a
* b
* c
* d
*     e
 \ \
* | | f
 / /
* / g
|/
o @end
```
### jobs=6 skips=a>d b>f,END c>END e>>END
```text
*   a
|\
* \     b
 \ \ \
* \ \ \   c
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | | d
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=6 skips=a>e,END c>END d>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  | |/
  |/|
* | | d
 \| |
  |/
 /|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>>END d>END
```text
*   a
 \
* | b
* | c
* | d
 \|
* | e
* | f
 /
o @end
```
### jobs=6 skips=a>d,e b>>END c>f
```text
*     a
 \ \
* \ \   b
 \ \ \
* \ \ \   c
 \ \ \ \
  |_|/ /
 /| | |
* | | | d
| |_|/
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=7 skips=a>END b>d,END c>g d>>f e>>END
```text
*   a
|\
* \     b
 \ \ \
  | |/
* | |   c
 \ \ \
  |/ /
 /| |
* | |   d
 \ \ \
* \ \ \   e
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>>END b>d,END e>>END f>>END
```text
*   a
 \
* \     b
 \ \ \
  | |/
* / / c
 / /
* | d
* | e
 \|
* | f
 \|
* | g
 /
o @end
```
### jobs=7 skips=b>g c>e,END f>>END
```text
* a
*   b
|\
* \     c
 \ \ \
* | | | d
 / / /
* | | e
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=c>e,f d>f
```text
* a
* b
*     c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| /
  |/
* / e
|/
* f
o @end
```
### jobs=7 skips=a>d b>e,f d>>END e>g,END
```text
*   a
|\
* \     b
 \ \ \
* | | | c
  |_|/
 /| |
* | |   d
 \ \ \
  |/ /
 /| |
* | |     e
 \ \ \ \
  | |/ /
  |_|/
 /| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>g,END c>e d>>END e>g,END f>>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
* \ \ \   d
 \ \ \ \
  |/ / /
 /| | /
  | |/
  |/|
* | |     e
 \ \ \ \
  | |_|/
  |/| /
  | |/
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=a>f b>>f c>f
```text
*   a
|\
* | b
 \|
* | c
 \|
* | d
* | e
 /
* f
o @end
```
### jobs=7 skips=c>e d>>g e>>END
```text
* a
* b
*   c
|\
* \   d
 \ \
  |/
 /|
* |   e
 \ \
* | | f
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=b>>g c>>f d>g,END
```text
* a
*   b
 \
* \   c
 \ \
* \ \     d
 \ \ \ \
  | |_|/
  |/| |
* | | | e
  |_|/
 /| |
* | | f
 / /
* / g
|/
o @end
```
### jobs=6 skips=b>d,e
```text
* a
*     b
 \ \
* | | c
 / /
* / d
|/
* e
* f
o @end
```
### jobs=7 skips=a>>e b>>e c>e,f e>g
```text
*   a
 \
* | b
 \|
* |     c
 \ \ \
  | |/
  |/|
* / | d
 / /
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=c>>e d>g,END e>g f>>END
```text
* a
* b
*   c
 \
* \     d
 \ \ \
  |_|/
 /| |
* | | e
|\| |
* | |   f
 \ \ \
  |/ /
 /| /
  |/
* / g
|/
o @end
```
### jobs=6 skips=b>e c>>f d>>END e>END
```text
* a
*   b
|\
* \   c
 \ \
* \ \   d
 \ \ \
  |_|/
 /| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c,d c>f
```text
*     a
 \ \
* | | b
 / /
* |   c
|\ \
| |/
|/|
* | d
* | e
|/
* f
o @end
```
### jobs=6 skips=b>e,END c>e d>>END e>END
```text
* a
*     b
 \ \
* | | c
 \| |
* | |   d
 \ \ \
  |/ /
 /| /
  |/
* | e
|\|
* | f
|/
o @end
```
### jobs=7 skips=a>>g c>>END f>END
```text
*   a
 \
* | b
* |   c
 \ \
* | | d
* | | e
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>c b>f c>>e d>f,END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
 \ \
* \ \     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
  |/|
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=a>e b>>f d>f,g e>END
```text
*   a
|\
* \   b
 \ \
* | | c
* | |     d
 \ \ \ \
  | |/ /
  |/| /
  |_|/
 /| |
* | |   e
|\ \ \
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=b>e,END c>e,END d>f,END
```text
* a
*     b
 \ \
* \ \     c
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |     d
 \ \ \ \
  |_|/ /
 /| | /
  | |/
* / / e
|/ /
* / f
|/
o @end
```
### jobs=7 skips=a>END b>f,g e>g,END f>END
```text
*   a
|\
* \     b
 \ \ \
* | | | c
* | | | d
* | | |     e
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |/ /
  |/| /
  | |/
* | |   f
|\ \ \
| |/ /
|/| /
| |/
* / g
|/
o @end
```
### jobs=6 skips=b>e,END c>>END d>>END
```text
* a
*     b
 \ \
* \ \   c
 \ \ \
  | |/
  |/|
* | | d
 \| |
  |/
 /|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>e,END d>f,END e>END
```text
*   a
|\
* \   b
|\ \
* \ \     c
 \ \ \ \
  | | |/
  | |/|
* | | |     d
 \ \ \ \ \
  |_|/ / /
 /| | / /
  | |_|/
  |/| /
  | |/
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=7 skips=b>f,END d>END
```text
* a
*     b
 \ \
* | | c
* | |   d
 \ \ \
  | |/
  |/|
* | | e
  |/
 /|
* | f
* | g
|/
o @end
```
### jobs=6 skips=a>e c>e,END
```text
*   a
|\
* | b
* |     c
 \ \ \
  | |/
  |/|
* / | d
 / /
* | e
* | f
|/
o @end
```
### jobs=6 skips=b>e,END d>f
```text
* a
*     b
 \ \
* | | c
* | |   d
 \ \ \
  |/ /
 /| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d,e d>f
```text
*   a
|\
* \     b
 \ \ \
* | | | c
 / / /
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d,END b>d,END c>>f d>>END e>>END
```text
*     a
 \ \
* \ \     b
 \ \ \ \
  | |/ /
  |/| /
  | |/
* | |   c
 \ \ \
  |/ /
 /| |
* | |   d
 \ \ \
  | |/
  |/|
* | | e
 \| |
  |/
 /|
* | f
 /
o @end
```
### jobs=6 skips=c>END e>>END
```text
* a
* b
*   c
|\
* | d
* | e
 \|
* | f
 /
o @end
```
### jobs=7 skips=c>e,f d>g f>END
```text
* a
* b
*     c
 \ \
* \ \   d
 \ \ \
  |/ /
 /| |
* | | e
| |/
|/|
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=6 skips=a>>e c>>END d>f e>>END
```text
*   a
 \
* | b
* |   c
 \ \
* \ \   d
 \ \ \
  |_|/
 /| |
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* / f
 /
o @end
```
### jobs=7 skips=a>f,g c>g d>>f e>>g f>>END
```text
*     a
 \ \
* | | b
* | |   c
 \ \ \
  | |/
  |/|
* | |   d
 \ \ \
  | |/
  |/|
* | |   e
 \ \ \
  |/ /
 /| /
  |/
* |   f
 \ \
  |/
 /|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>>g e>END f>END
```text
*   a
|\
* | b
* | c
 \|
* | d
* |   e
 \ \
* | | f
 \| |
  |/
 /|
* | g
|/
o @end
```
### jobs=6 skips=a>END c>>f d>END e>>END
```text
*   a
|\
* | b
* |   c
 \ \
* \ \   d
 \ \ \
  | |/
  |/|
* | | e
 \| |
  |/
 /|
* | f
|/
o @end
```
### jobs=7 skips=a>e b>END d>>END f>>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
 \| |
  |/
 /|
* | e
* | f
 \|
* | g
 /
o @end
```
### jobs=6 skips=a>f,END b>>d d>f
```text
*     a
 \ \
* \ \   b
 \ \ \
* | | | c
 / / /
* | | d
 \| |
* | | e
 / /
* / f
|/
o @end
```
### jobs=7 skips=a>>d d>END f>>END
```text
*   a
 \
* | b
* | c
 /
*   d
|\
* | e
* | f
 \|
* | g
 /
o @end
```
