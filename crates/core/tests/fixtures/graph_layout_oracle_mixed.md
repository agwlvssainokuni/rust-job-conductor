# レイアウトの正解データ(交差と同じ行き先が混ざる形): graph-mixed-shapes

この文書は、`conductor graph` の text の図のうち、**飛び越しの線が交差し、かつ、同じ行き先が混ざる形**の、**正解の出力**である。実際の `git log --graph` の出力から取った(行末の空白は取り除き、終点は `o @end` にした)。

- 形の表し方: `jobs=<ジョブの数> skips=<飛び越しの辺>`。ジョブは `a`、`b`、…。`a>d` は「`a` が成功で `d` へ飛ぶ(警告は次のジョブ `b` へ)」、`b>END` は「`b` が `@end` へ飛ぶ」。
- 範囲: 各ジョブの飛び越しの辺は多くて 1 つ。飛び越しの区間が交差する形が 1 つ以上あり、かつ、行き先が同じ辺が 2 つ以上ある形(ジョブ 4〜6 個。計 487 個)。
- **この文書のすべての例に、1 文字も違わず一致すること**が、レーンの配置の合格の条件である(実装のテストが、この文書を読み込んで確かめる)。このファイルは、ジョブ 6 個以下の分だけ(質問 Q2: A)。ジョブ 7 個の分(3,861 例、約 64 万バイト)は、コード生成のときに、手元で 1 回だけ照合し、ファイルには入れない。
- 作り方(再現できる): ジョブごとに 1 つのコミットを作り、第 1 の親を「次のジョブ」(最後のジョブは終点)、第 2 の親を「飛び越しの行き先」にして、`git log --graph --format=%s --topo-order <a>` を実行した。**git のソースは読んでいない。出力の観察だけから、規則(functional-spec.md の WF2、`layout-reference.py`)を立てた。**

### jobs=4 skips=a>END b>d c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* / d
|/
o @end
```
### jobs=4 skips=a>d b>END c>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|/
o @end
```
### jobs=4 skips=a>d b>d c>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|/
o @end
```
### jobs=4 skips=a>c b>END c>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|/
o @end
```
### jobs=5 skips=b>END c>e d>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=b>e c>END d>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=b>e c>e d>END
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=b>d c>END d>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>END c>e d>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>END b>END c>e d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>END b>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>END b>e c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>END b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>END b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>END b>d c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
* | e
|/
o @end
```
### jobs=5 skips=a>END b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>END b>d c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>e c>END d>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e c>e d>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>END c>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>END c>e
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* / | d
|/ /
* / e
|/
o @end
```
### jobs=5 skips=a>e b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>e b>e d>END
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>e c>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>e c>END d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>e c>e d>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>e b>d c>e
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* / d
|/
* e
o @end
```
### jobs=5 skips=a>e b>d c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>d c>END d>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>d b>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>d b>END c>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
* | e
|/
o @end
```
### jobs=5 skips=a>d b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>d b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>d b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>d b>e c>e
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|/
* e
o @end
```
### jobs=5 skips=a>d b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>d b>d c>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
* | e
|/
o @end
```
### jobs=5 skips=a>d b>d c>END d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>d b>d c>e
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|/
* e
o @end
```
### jobs=5 skips=a>d b>d c>e d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>c b>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>c b>END c>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
* | e
|/
o @end
```
### jobs=5 skips=a>c b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|\|
* | e
|/
o @end
```
### jobs=5 skips=a>c b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
o @end
```
### jobs=5 skips=a>c b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>c b>e c>e
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|/
* e
o @end
```
### jobs=5 skips=a>c b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
o @end
```
### jobs=5 skips=a>c b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
o @end
```
### jobs=6 skips=c>END d>f e>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=c>f d>END e>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=c>f d>f e>END
```text
* a
* b
*   c
|\
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
### jobs=6 skips=c>e d>END e>END
```text
* a
* b
*   c
|\
* \   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>END d>f e>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>END c>END d>f e>END
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>END c>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>END c>f d>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>END c>f d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
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
### jobs=6 skips=b>END c>f d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>END c>e d>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=b>END c>e d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>END c>e d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>f d>END e>END
```text
* a
*   b
|\
* | c
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
### jobs=6 skips=b>f d>f e>END
```text
* a
*   b
|\
* | c
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
### jobs=6 skips=b>f c>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>END d>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>END d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>END d>f
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=b>f c>END d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
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
### jobs=6 skips=b>f c>f e>END
```text
* a
*   b
|\
* | c
|\|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>f d>END
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>f d>END e>END
```text
* a
*   b
|\
* | c
|\|
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
### jobs=6 skips=b>f c>f d>f e>END
```text
* a
*   b
|\
* | c
|\|
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
### jobs=6 skips=b>f c>e d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>e d>f
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=b>f c>e d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>e d>END e>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>e c>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>e c>END d>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=b>e c>END d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>e c>END d>f e>END
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
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>e c>f d>END e>END
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
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>e c>f d>f
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=b>e c>f d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>e c>e d>END
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=b>e c>e d>END e>END
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>e c>e d>f
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=b>e c>e d>f e>END
```text
* a
*   b
|\
* | c
|\|
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>d c>END e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>d c>END d>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=b>d c>END d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=b>d c>END d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=b>d c>f d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
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
### jobs=6 skips=b>d c>f d>f
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=b>d c>f d>f e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
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
### jobs=6 skips=b>d c>e d>END e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END d>f e>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END c>END d>f e>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END c>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END c>f d>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END c>f d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
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
### jobs=6 skips=a>END c>f d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END c>e d>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END c>e d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END c>e d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>END d>f e>END
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>END c>END d>f e>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>END c>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>END c>f d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>END c>f d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
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
### jobs=6 skips=a>END b>END c>f d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>END c>e d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>END c>e d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>END c>e d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>END c>e d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
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
### jobs=6 skips=a>END b>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
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
### jobs=6 skips=a>END b>f c>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f c>f d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
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
### jobs=6 skips=a>END b>f c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|/ / /
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| | /
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>f c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| | /
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
### jobs=6 skips=a>END b>f c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| / /
| |/ /
* / / e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>f c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| / /
| |/ /
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|/ /
|/| | /
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|/ /
|/| | /
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
### jobs=6 skips=a>END b>e c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |/ /
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>e c>e d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
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
### jobs=6 skips=a>END b>e c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
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
### jobs=6 skips=a>END b>d c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|/ /
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>f d>END e>END
```text
*   a
|\
* | b
* | c
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
### jobs=6 skips=a>f d>f e>END
```text
*   a
|\
* | b
* | c
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
### jobs=6 skips=a>f c>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f c>END d>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f c>END d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f c>END d>f
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f c>END d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
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
### jobs=6 skips=a>f c>f e>END
```text
*   a
|\
* | b
* | c
|\|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f c>f d>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f c>f d>END e>END
```text
*   a
|\
* | b
* | c
|\|
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
### jobs=6 skips=a>f c>f d>f e>END
```text
*   a
|\
* | b
* | c
|\|
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
### jobs=6 skips=a>f c>e d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f c>e d>f
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>f c>e d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END d>f
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
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
### jobs=6 skips=a>f b>END c>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>END d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
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
### jobs=6 skips=a>f b>END c>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
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
### jobs=6 skips=a>f b>END c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|/ / /
* | / e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| / /
| |/ /
* | / e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| / /
| |/ /
* | / e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
|/| | /
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>END c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |/ / /
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
### jobs=6 skips=a>f b>f e>END
```text
*   a
|\
* | b
|\|
* | c
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f d>END
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f d>END e>END
```text
*   a
|\
* | b
|\|
* | c
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
### jobs=6 skips=a>f b>f d>f e>END
```text
*   a
|\
* | b
|\|
* | c
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
### jobs=6 skips=a>f b>f c>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>END d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>END d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>END d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>f c>END d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
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
### jobs=6 skips=a>f b>f c>f e>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>f d>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>f d>END e>END
```text
*   a
|\
* | b
|\|
* | c
|\|
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
### jobs=6 skips=a>f b>f c>f d>f e>END
```text
*   a
|\
* | b
|\|
* | c
|\|
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
### jobs=6 skips=a>f b>f c>e e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>e d>END
```text
*   a
|\
* | b
|\|
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
|/
o @end
```
### jobs=6 skips=a>f b>f c>e d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>f c>e d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>f b>f c>e d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e d>f
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>f b>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |/ /
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|/ /
|/| | /
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>e c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
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
### jobs=6 skips=a>f b>e c>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>f b>e c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>f b>e c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>e d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>f b>e c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
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
### jobs=6 skips=a>f b>d d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
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
### jobs=6 skips=a>f b>d c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f b>d c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
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
### jobs=6 skips=a>f b>d c>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
* | e
|/
* f
o @end
```
### jobs=6 skips=a>f b>d c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
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
### jobs=6 skips=a>f b>d c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>f b>d c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
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
### jobs=6 skips=a>f b>d c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>f b>d c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e d>END e>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>END e>END
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
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>END d>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e c>END d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>END d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e c>f d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>f d>f
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e c>f d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>e d>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e c>e d>END e>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e c>e d>f
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e c>e d>f e>END
```text
*   a
|\
* | b
* | c
|\|
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>END d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
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
### jobs=6 skips=a>e b>END c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| | |/ /
| |/| /
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |_|/
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>e
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* / | d
|/ /
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* / | d
|/ /
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
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
### jobs=6 skips=a>e b>END c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f d>f
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |_|/
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|\| | |
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| | |/ /
| |/| /
| |_|/
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>f c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| | |/ /
| |/| /
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>f c>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>f c>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>f d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>f d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>f c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>e
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* / | d
|/ /
* / e
|/
* f
o @end
```
### jobs=6 skips=a>e b>f c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* / | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>e b>f c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| | |/
| |/|
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e d>END
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e d>END e>END
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e d>f
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>e d>f e>END
```text
*   a
|\
* | b
|\|
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>END d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>END d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>END d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>e c>END d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>e c>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>e c>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>f d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>f d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>f d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>e c>f d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>e d>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>e d>END e>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>e c>e d>f
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>e c>e d>f e>END
```text
*   a
|\
* | b
|\|
* | c
|\|
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>e b>d c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>d c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| |
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>e
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* / d
|/
* e
* f
o @end
```
### jobs=6 skips=a>e b>d c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* / d
|/
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>e b>d c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |/ /
|/| /
| |/
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d c>END e>END
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
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d c>END d>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d c>END d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d c>END d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d c>f d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
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
### jobs=6 skips=a>d c>f d>f
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d c>f d>f e>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
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
### jobs=6 skips=a>d c>e d>END e>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>END d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
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
### jobs=6 skips=a>d b>END c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>e e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|/ /
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
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
### jobs=6 skips=a>d b>f d>f
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
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
### jobs=6 skips=a>d b>f c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>f c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
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
### jobs=6 skips=a>d b>f c>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>f c>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>f d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
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
### jobs=6 skips=a>d b>f c>f d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>f c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
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
### jobs=6 skips=a>d b>f c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>e d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>d b>f c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>e c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | |   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>f d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>e c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>e
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|/
* e
* f
o @end
```
### jobs=6 skips=a>d b>e c>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* | d
|/
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>e d>f
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>e c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|\| |
| |/
|/|
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d d>f e>END
```text
*   a
|\
* | b
|\|
* | c
|/
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
### jobs=6 skips=a>d b>d c>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>END d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>END d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>END d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>d c>END d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>d b>d c>f
```text
*   a
|\
* | b
|\|
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
### jobs=6 skips=a>d b>d c>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>f d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>f d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
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
### jobs=6 skips=a>d b>d c>f d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>d c>f d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
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
### jobs=6 skips=a>d b>d c>e
```text
*   a
|\
* | b
|\|
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
### jobs=6 skips=a>d b>d c>e e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* | d
|/
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>e d>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>e d>END e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>d c>e d>f
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>d b>d c>e d>f e>END
```text
*   a
|\
* | b
|\|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c c>END d>f e>END
```text
*   a
|\
* | b
|/
*   c
|\
* \   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c c>f d>END e>END
```text
*   a
|\
* | b
|/
*   c
|\
* \   d
|\ \
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c c>f d>f e>END
```text
*   a
|\
* | b
|/
*   c
|\
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
### jobs=6 skips=a>c c>e d>END e>END
```text
*   a
|\
* | b
|/
*   c
|\
* \   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>END d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>f d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
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
### jobs=6 skips=a>c b>END c>f d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
* | | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>e e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|/ /
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>e d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>f d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
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
### jobs=6 skips=a>c b>f d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>c b>f d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
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
### jobs=6 skips=a>c b>f c>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>END d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| | |/
| |/|
* / | e
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>f c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
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
### jobs=6 skips=a>c b>f c>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
* | e
|/
* f
o @end
```
### jobs=6 skips=a>c b>f c>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>f d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
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
### jobs=6 skips=a>c b>f c>f d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>c b>f c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
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
### jobs=6 skips=a>c b>f c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>e d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* / e
|/
* f
o @end
```
### jobs=6 skips=a>c b>f c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |/ /
|/| /
| |/
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | |   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>e c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* \ \   d
|\ \ \
| |_|/
|/| |
* | | e
|\| |
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>f d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>c b>e c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
* | | d
|\| |
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>e
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|/
* e
* f
o @end
```
### jobs=6 skips=a>c b>e c>e e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* | d
|/
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>e d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>e d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
| |/
|/|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>c b>e c>e d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|\|
* |   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
*   d
|\
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* | d
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>END d>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>END d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|\|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>END d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
* \ \   e
|\ \ \
| |/ /
|/| /
| |/
* / f
|/
o @end
```
### jobs=6 skips=a>c b>d c>f d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
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
### jobs=6 skips=a>c b>d c>f d>f
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* | d
|\|
* | e
|/
* f
o @end
```
### jobs=6 skips=a>c b>d c>f d>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
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
### jobs=6 skips=a>c b>d c>e d>END e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* |   c
|\ \
| |/
|/|
* |   d
|\ \
| |/
|/|
* | e
|\|
* | f
|/
o @end
```
