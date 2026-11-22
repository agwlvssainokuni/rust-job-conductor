# レイアウトの正解データ(交差する飛び越し): graph-crossing-lanes

この文書は、`conductor graph` の text の図のうち、**飛び越しの線が交差する形**の、**正解の出力**である。実際の `git log --graph` の出力から取った(行末の空白は取り除き、終点は `o @end` にした)。

- 形の表し方: `jobs=<ジョブの数> skips=<飛び越しの辺>`。ジョブは `a`、`b`、…。`a>d` は「`a` が成功で `d` へ飛ぶ(警告は次のジョブ `b` へ)」、`b>END` は「`b` が `@end` へ飛ぶ」。
- 範囲: 次の条件をすべて満たし、線が交差する形だけ(ジョブ 3〜7 個。計 940 個)。(1) 各ジョブの飛び越しの辺は、多くて 1 つ。(2) 飛び越しの行き先が、どの 2 つの辺でも違う。(3) 飛び越しの区間が交差する(元 1 < 元 2 < 行き先 1 < 行き先 2 になる組が、1 つ以上ある)。
- 注意: git は、3 本以上の線が続けて交差するとき、横線 `_` を使う。これまでの図には、`_` はなかった。
- **この文書のすべての例に、1 文字も違わず一致すること**が、レーンの配置の合格の条件である(実装のテストが、この文書を読み込んで確かめる)。このファイルは、ジョブ 7 個以下の分だけ(質問 Q2: A)。ジョブ 8 個の分(3,817 例)は、コード生成のときに、手元で 1 回だけ照合し、ファイルには入れない。
- 作り方(再現できる): ジョブごとに 1 つのコミットを作り、第 1 の親を「次のジョブ」(最後のジョブは終点)、第 2 の親を「飛び越しの行き先」にして、`git log --graph --format=%s --topo-order <a>` を実行した。**git のソースは読んでいない。出力の観察だけから、規則(functional-spec.md の WF2)を立てた。**

### jobs=3 skips=a>c b>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
o @end
```
### jobs=4 skips=b>d c>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
o @end
```
### jobs=4 skips=a>d c>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|/
o @end
```
### jobs=4 skips=a>d b>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
o @end
```
### jobs=4 skips=a>c b>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
o @end
```
### jobs=4 skips=a>c b>d
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
o @end
```
### jobs=4 skips=a>c b>d c>END
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
|/
o @end
```
### jobs=5 skips=c>e d>END
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
|/
o @end
```
### jobs=5 skips=b>e d>END
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
|/
o @end
```
### jobs=5 skips=b>e c>END
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
|/
o @end
```
### jobs=5 skips=b>d c>END
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
|/
o @end
```
### jobs=5 skips=b>d c>e
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
* e
o @end
```
### jobs=5 skips=b>d c>e d>END
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
|/
o @end
```
### jobs=5 skips=a>END b>d c>e
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
* / e
|/
o @end
```
### jobs=5 skips=a>e d>END
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
|/
o @end
```
### jobs=5 skips=a>e c>END
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
|/
o @end
```
### jobs=5 skips=a>e b>END
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
|/
o @end
```
### jobs=5 skips=a>e b>d d>END
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
|/
o @end
```
### jobs=5 skips=a>e b>d c>END
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
|/
o @end
```
### jobs=5 skips=a>d c>END
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
|/
o @end
```
### jobs=5 skips=a>d c>e
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|/
* e
o @end
```
### jobs=5 skips=a>d c>e d>END
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
|/
o @end
```
### jobs=5 skips=a>d b>END
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
|/
o @end
```
### jobs=5 skips=a>d b>END c>e
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
* / e
|/
o @end
```
### jobs=5 skips=a>d b>e
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
* e
o @end
```
### jobs=5 skips=a>d b>e d>END
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
|/
o @end
```
### jobs=5 skips=a>d b>e c>END
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
|/
o @end
```
### jobs=5 skips=a>c c>e d>END
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
|/
o @end
```
### jobs=5 skips=a>c b>END
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
|/
o @end
```
### jobs=5 skips=a>c b>END c>e
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
* / e
|/
o @end
```
### jobs=5 skips=a>c b>e
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
* e
o @end
```
### jobs=5 skips=a>c b>e d>END
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
|/
o @end
```
### jobs=5 skips=a>c b>e c>END
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
|/
o @end
```
### jobs=5 skips=a>c b>d
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
* e
o @end
```
### jobs=5 skips=a>c b>d d>END
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
|/
o @end
```
### jobs=5 skips=a>c b>d c>END
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
|/
o @end
```
### jobs=5 skips=a>c b>d c>e
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
|/
* e
o @end
```
### jobs=5 skips=a>c b>d c>e d>END
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
|/
o @end
```
### jobs=6 skips=d>f e>END
```text
* a
* b
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
### jobs=6 skips=c>f e>END
```text
* a
* b
*   c
|\
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=c>f d>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=c>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=c>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=c>e d>f e>END
```text
* a
* b
*   c
|\
* \   d
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
### jobs=6 skips=b>END c>e d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=b>f e>END
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
* | f
|/
o @end
```
### jobs=6 skips=b>f d>END
```text
* a
*   b
|\
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
### jobs=6 skips=b>f c>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>f c>e e>END
```text
* a
*   b
|\
* \   c
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
### jobs=6 skips=b>f c>e d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=b>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=b>e d>f e>END
```text
* a
*   b
|\
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
### jobs=6 skips=b>e c>END
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
* | f
|/
o @end
```
### jobs=6 skips=b>e c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=b>e c>f
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
|/
* f
o @end
```
### jobs=6 skips=b>e c>f e>END
```text
* a
*   b
|\
* \   c
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
### jobs=6 skips=b>e c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>d d>f e>END
```text
* a
*   b
|\
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
### jobs=6 skips=b>d c>END
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
* | f
|/
o @end
```
### jobs=6 skips=b>d c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=b>d c>f
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
|/
* f
o @end
```
### jobs=6 skips=b>d c>f e>END
```text
* a
*   b
|\
* \   c
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
### jobs=6 skips=b>d c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=b>d c>e
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
* e
* f
o @end
```
### jobs=6 skips=b>d c>e e>END
```text
* a
*   b
|\
* \   c
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
### jobs=6 skips=b>d c>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=b>d c>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=b>d c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>END c>e d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>e c>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>END b>d c>e
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
* | f
|/
o @end
```
### jobs=6 skips=a>END b>d c>e d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>f e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>f d>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>f c>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>f c>e e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>f c>e d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | | e
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>END c>e
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
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
### jobs=6 skips=a>f b>e d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>e c>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>e
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
* / e
|/
* f
o @end
```
### jobs=6 skips=a>f b>d c>e e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>f b>d c>e d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>e d>f e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>e c>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>e c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e c>f
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
|/
* f
o @end
```
### jobs=6 skips=a>e c>f e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>e c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>e b>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>END c>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>f
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
|/
* f
o @end
```
### jobs=6 skips=a>e b>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
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
### jobs=6 skips=a>e b>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>f c>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>e b>d d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>e b>d c>f
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
|/
* f
o @end
```
### jobs=6 skips=a>e b>d c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>e b>d c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d d>f e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>d c>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>d c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d c>f
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
|/
* f
o @end
```
### jobs=6 skips=a>d c>f e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>d c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d c>e
```text
*   a
|\
* | b
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
### jobs=6 skips=a>d c>e e>END
```text
*   a
|\
* | b
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
### jobs=6 skips=a>d c>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>d c>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>d c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>END c>e
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
* | f
|/
o @end
```
### jobs=6 skips=a>d b>END c>e d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>f
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
|/
* f
o @end
```
### jobs=6 skips=a>d b>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
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
### jobs=6 skips=a>d b>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>e
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
* / e
|/
* f
o @end
```
### jobs=6 skips=a>d b>f c>e e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>f c>e d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
* e
* f
o @end
```
### jobs=6 skips=a>d b>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
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
### jobs=6 skips=a>d b>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>d b>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>d b>e c>f
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
|/
* f
o @end
```
### jobs=6 skips=a>d b>e c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>d b>e c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c d>f e>END
```text
*   a
|\
* | b
|/
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
### jobs=6 skips=a>c c>f e>END
```text
*   a
|\
* | b
|/
*   c
|\
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c c>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c c>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>END c>e
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>END c>e d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c b>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
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
### jobs=6 skips=a>c b>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>e
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
* / e
|/
* f
o @end
```
### jobs=6 skips=a>c b>f c>e e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>f c>e d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
* e
* f
o @end
```
### jobs=6 skips=a>c b>e e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c b>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>e c>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c b>e c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>e c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
* e
* f
o @end
```
### jobs=6 skips=a>c b>d e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c b>d d>f e>END
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
* \   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>END d>f
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
|/ /
* / f
|/
o @end
```
### jobs=6 skips=a>c b>d c>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c b>d c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>f d>END
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
| |/
|/|
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>e
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
|/
* e
* f
o @end
```
### jobs=6 skips=a>c b>d c>e e>END
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
|/
*   e
|\
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>e d>END
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
* | f
|/
o @end
```
### jobs=6 skips=a>c b>d c>e d>f
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
|/
* f
o @end
```
### jobs=6 skips=a>c b>d c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
|/
o @end
```
### jobs=7 skips=e>g f>END
```text
* a
* b
* c
* d
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=d>g f>END
```text
* a
* b
* c
*   d
|\
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=d>g e>END
```text
* a
* b
* c
*   d
|\
* \   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=d>f e>END
```text
* a
* b
* c
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=d>f e>g
```text
* a
* b
* c
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=d>f e>g f>END
```text
* a
* b
* c
*   d
|\
* \   e
|\ \
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
### jobs=7 skips=c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=c>g f>END
```text
* a
* b
*   c
|\
* | d
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>g e>END
```text
* a
* b
*   c
|\
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>g d>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>g d>f f>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>f e>END
```text
* a
* b
*   c
|\
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=c>f e>g
```text
* a
* b
*   c
|\
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=c>f e>g f>END
```text
* a
* b
*   c
|\
* | d
* |   e
|\ \
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
### jobs=7 skips=c>f d>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=c>f d>END e>g
```text
* a
* b
*   c
|\
* \   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=c>f d>g
```text
* a
* b
*   c
|\
* \   d
|\ \
* | | e
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=c>f d>g f>END
```text
* a
* b
*   c
|\
* \   d
|\ \
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
### jobs=7 skips=c>f d>g e>END
```text
* a
* b
*   c
|\
* \   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>e e>g f>END
```text
* a
* b
*   c
|\
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=c>e d>END e>g
```text
* a
* b
*   c
|\
* \   d
|\ \
| |/
|/|
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=c>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=c>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>e d>g e>END
```text
* a
* b
*   c
|\
* \   d
|\ \
| |/
|/|
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=c>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=c>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=c>e d>f e>END
```text
* a
* b
*   c
|\
* \   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=c>e d>f e>g
```text
* a
* b
*   c
|\
* \   d
|\ \
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=c>e d>f e>g f>END
```text
* a
* b
*   c
|\
* \   d
|\ \
| |/
|/|
* |   e
|\ \
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
### jobs=7 skips=b>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>END c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>END c>f d>g
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>END c>e d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>END c>e d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>END c>e d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>g f>END
```text
* a
*   b
|\
* | c
* | d
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g e>END
```text
* a
*   b
|\
* | c
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g d>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g d>f f>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
* | | e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>END d>f
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>f f>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | | e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>f d>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>e f>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|/ /
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>e e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|/ /
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>e d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>e d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=b>g c>e d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>g c>e d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>f e>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>f e>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=b>f e>g f>END
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
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>f d>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>f d>END e>g
```text
* a
*   b
|\
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>f d>g
```text
* a
*   b
|\
* | c
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>f d>g f>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
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
### jobs=7 skips=b>f d>g e>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>f c>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>f c>END e>g
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | |   e
|\ \ \
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>f c>END d>g
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>f c>g
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | | e
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>f c>g f>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
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
### jobs=7 skips=b>f c>g e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
* | |   e
|\ \ \
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>f c>g d>END
```text
* a
*   b
|\
* \   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>f c>e e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>f c>e e>g
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>f c>e e>g f>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
|/ /
* |   e
|\ \
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
### jobs=7 skips=b>f c>e d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>f c>e d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>f c>e d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>f c>e d>g f>END
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
### jobs=7 skips=b>f c>e d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e e>g f>END
```text
* a
*   b
|\
* | c
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>e d>END e>g
```text
* a
*   b
|\
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=b>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e d>g e>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=b>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=b>e d>f e>END
```text
* a
*   b
|\
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
* | g
|/
o @end
```
### jobs=7 skips=b>e d>f e>g
```text
* a
*   b
|\
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
* g
o @end
```
### jobs=7 skips=b>e d>f e>g f>END
```text
* a
*   b
|\
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
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
### jobs=7 skips=b>e c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>e c>END e>g
```text
* a
*   b
|\
* \   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>e c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>e c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>e c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>e c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=b>e c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e c>g e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=b>e c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>e c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=b>e c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=b>e c>f e>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>e c>f e>g
```text
* a
*   b
|\
* \   c
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
* g
o @end
```
### jobs=7 skips=b>e c>f e>g f>END
```text
* a
*   b
|\
* \   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
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
### jobs=7 skips=b>e c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>e c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>e c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>e c>f d>g f>END
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
### jobs=7 skips=b>e c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d e>g f>END
```text
* a
*   b
|\
* | c
|/
* d
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d d>g f>END
```text
* a
*   b
|\
* | c
|/
*   d
|\
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d d>g e>END
```text
* a
*   b
|\
* | c
|/
*   d
|\
* \   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d d>f e>END
```text
* a
*   b
|\
* | c
|/
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d d>f e>g
```text
* a
*   b
|\
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
* g
o @end
```
### jobs=7 skips=b>d d>f e>g f>END
```text
* a
*   b
|\
* | c
|/
*   d
|\
* \   e
|\ \
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
### jobs=7 skips=b>d c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>END e>g
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>d c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>d c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>d c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=b>d c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>g e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=b>d c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=b>d c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=b>d c>f e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>f e>g
```text
* a
*   b
|\
* \   c
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
* g
o @end
```
### jobs=7 skips=b>d c>f e>g f>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
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
### jobs=7 skips=b>d c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>d c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>d c>f d>g f>END
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
### jobs=7 skips=b>d c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
* e
* f
* g
o @end
```
### jobs=7 skips=b>d c>e f>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
* e
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e e>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
*   e
|\
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e e>g
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
*   e
|\
* | f
|/
* g
o @end
```
### jobs=7 skips=b>d c>e e>g f>END
```text
* a
*   b
|\
* \   c
|\ \
| |/
|/|
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=b>d c>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=b>d c>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=b>d c>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=b>d c>e d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=b>d c>e d>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END c>f d>g
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END c>e d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END c>e d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>END c>e d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>g c>e d>f
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
|/| | |
* | | | e
|/ / /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>f d>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>f c>g
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>f c>e e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>f c>e d>g
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
|/| | |
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>e d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>e d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>END b>e d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>e c>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>e c>g d>f
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
|/| | |
* | | | e
|/ / /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>e c>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>END b>e c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>e c>f d>g
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
|/| | |
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>g d>f
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
* | | | e
|/ / /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>END b>d c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>f d>g
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
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>e
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>END b>d c>e e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>e d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>END b>d c>e d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>END b>d c>e d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>g f>END
```text
*   a
|\
* | b
* | c
* | d
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g e>END
```text
*   a
|\
* | b
* | c
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g d>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g d>f f>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* | | e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>END d>f
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>f f>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | | e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>f d>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>e f>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|/ /
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>e e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|/ /
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>e d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>e d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>g c>e d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g c>e d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>END d>f
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>END c>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>END c>e
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|/ / /
* | | e
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>END c>e d>f
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
|/| | |
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f f>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | | e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f c>e f>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|/ / /
* / / e
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f c>e e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>f c>e d>END
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
|/| | |
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e f>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|/ /
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|/ /
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>g b>e d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e c>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e c>END d>f
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
|/| | |
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e c>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>g b>e c>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>e c>f d>END
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
|/| | |
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d f>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* | d
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>END d>f
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
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>g b>d c>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>f d>END
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
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>e
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>g b>d c>e f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>e e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>e d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>e d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>g b>d c>e d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>g b>d c>e d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f e>END
```text
*   a
|\
* | b
* | c
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f e>g
```text
*   a
|\
* | b
* | c
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f e>g f>END
```text
*   a
|\
* | b
* | c
* | d
* |   e
|\ \
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
### jobs=7 skips=a>f d>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f d>END e>g
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f d>g
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* | | e
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f d>g f>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
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
### jobs=7 skips=a>f d>g e>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
* \ \   e
|\ \ \
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f c>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f c>END e>g
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | |   e
|\ \ \
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f c>END d>g
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f c>g
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | | e
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f c>g f>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
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
### jobs=7 skips=a>f c>g e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
* | |   e
|\ \ \
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f c>g d>END
```text
*   a
|\
* | b
* |   c
|\ \
* \ \   d
|\ \ \
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f c>e e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f c>e e>g
```text
*   a
|\
* | b
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
* g
o @end
```
### jobs=7 skips=a>f c>e e>g f>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
|/ /
* |   e
|\ \
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
### jobs=7 skips=a>f c>e d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f c>e d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f c>e d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f c>e d>g f>END
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
### jobs=7 skips=a>f c>e d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | | e
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>END e>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | |   e
|\ \ \
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>END d>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>END c>g
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>END c>e
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>END c>e e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>END c>e d>g
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
|/| | |
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | | e
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
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
### jobs=7 skips=a>f b>g e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
* | |   e
|\ \ \
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>g d>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | |   d
|\ \ \
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>g c>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>g c>e
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>g c>e f>END
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
### jobs=7 skips=a>f b>g c>e e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>g c>e d>END
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
|/| | |
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>e e>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|/ /
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>e e>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|/ /
* |   e
|\ \
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
### jobs=7 skips=a>f b>e d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>e d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>e d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>e d>g f>END
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
### jobs=7 skips=a>f b>e d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>e c>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>e c>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>e c>END d>g
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
|/| | |
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>e c>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>e c>g f>END
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
### jobs=7 skips=a>f b>e c>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>e c>g d>END
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
|/| | |
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d e>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d e>g
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>d e>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* | d
* |   e
|\ \
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
### jobs=7 skips=a>f b>d d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>d d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>d d>g f>END
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
### jobs=7 skips=a>f b>d d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>d c>END d>g
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
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>d c>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>d c>g f>END
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
### jobs=7 skips=a>f b>d c>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>g d>END
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
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>e
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
* / e
|/
* f
* g
o @end
```
### jobs=7 skips=a>f b>d c>e f>END
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
* / e
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>e e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>e e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>d c>e e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>f b>d c>e d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>f b>d c>e d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>f b>d c>e d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>f b>d c>e d>g f>END
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
### jobs=7 skips=a>f b>d c>e d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e e>g f>END
```text
*   a
|\
* | b
* | c
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e d>END e>g
```text
*   a
|\
* | b
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e d>g e>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>e d>f e>END
```text
*   a
|\
* | b
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
* | g
|/
o @end
```
### jobs=7 skips=a>e d>f e>g
```text
*   a
|\
* | b
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
* g
o @end
```
### jobs=7 skips=a>e d>f e>g f>END
```text
*   a
|\
* | b
* | c
* |   d
|\ \
| |/
|/|
* |   e
|\ \
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
### jobs=7 skips=a>e c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e c>END e>g
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e c>g e>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>e c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>e c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>e c>f e>END
```text
*   a
|\
* | b
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
* | g
|/
o @end
```
### jobs=7 skips=a>e c>f e>g
```text
*   a
|\
* | b
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
* g
o @end
```
### jobs=7 skips=a>e c>f e>g f>END
```text
*   a
|\
* | b
* |   c
|\ \
* | | d
| |/
|/|
* |   e
|\ \
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
### jobs=7 skips=a>e c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e c>f d>g f>END
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
### jobs=7 skips=a>e c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>END e>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
| |/
|/|
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>END c>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>END c>g d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|_|/
|/| | |
* | | | e
|/ / /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>END c>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>END c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>END c>f d>g
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|_|/
|/| | |
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
| |/
|/|
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>e b>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g c>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g c>END d>f
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|_|/
|/| | |
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g c>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>e b>g c>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>g c>f d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|_|/
|/| | |
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>e b>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f e>g
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
| |/
|/|
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>f e>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
| |/
|/|
* |   e
|\ \
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
### jobs=7 skips=a>e b>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>f d>g f>END
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
### jobs=7 skips=a>e b>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f c>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f c>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>f c>END d>g
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|_|/
|/| | |
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>f c>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>f c>g f>END
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
### jobs=7 skips=a>e b>f c>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>f c>g d>END
```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* \ \ \   d
|\ \ \ \
| |_|_|/
|/| | |
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d e>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
|/ /
* / d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>d d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>d d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>e b>d d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>d d>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>e b>d c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>d c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>d c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>d c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>d c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>e b>d c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>e b>d c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>d c>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>e b>d c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>e b>d c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>e b>d c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>e b>d c>f d>g f>END
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
### jobs=7 skips=a>e b>d c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d e>g f>END
```text
*   a
|\
* | b
* | c
|/
* d
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d d>g f>END
```text
*   a
|\
* | b
* | c
|/
*   d
|\
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d d>g e>END
```text
*   a
|\
* | b
* | c
|/
*   d
|\
* \   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d d>f e>END
```text
*   a
|\
* | b
* | c
|/
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d d>f e>g
```text
*   a
|\
* | b
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
* g
o @end
```
### jobs=7 skips=a>d d>f e>g f>END
```text
*   a
|\
* | b
* | c
|/
*   d
|\
* \   e
|\ \
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
### jobs=7 skips=a>d c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d c>END e>g
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>g e>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>d c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>d c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d c>f e>END
```text
*   a
|\
* | b
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
* | g
|/
o @end
```
### jobs=7 skips=a>d c>f e>g
```text
*   a
|\
* | b
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
* g
o @end
```
### jobs=7 skips=a>d c>f e>g f>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
* |   e
|\ \
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
### jobs=7 skips=a>d c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d c>f d>g f>END
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
### jobs=7 skips=a>d c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|/
* e
* f
* g
o @end
```
### jobs=7 skips=a>d c>e f>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|/
* e
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e e>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|/
*   e
|\
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e e>g
```text
*   a
|\
* | b
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
* g
o @end
```
### jobs=7 skips=a>d c>e e>g f>END
```text
*   a
|\
* | b
* |   c
|\ \
| |/
|/|
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d c>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d c>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>d c>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d c>e d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d c>e d>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>d b>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>END e>g
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>g d>f
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
* | | | e
|/ / /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>END c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>f d>g
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
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>e
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>END c>e e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>e d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>END c>e d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>END c>e d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>d b>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>END d>f
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
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>d b>g c>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>f d>END
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
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>e
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>g c>e f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>e e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>e d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>e d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>d b>g c>e d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>g c>e d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>d b>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f e>g
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>f e>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
* |   e
|\ \
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
### jobs=7 skips=a>d b>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>f d>g f>END
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
### jobs=7 skips=a>d b>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>f c>END d>g
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
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>f c>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>f c>g f>END
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
### jobs=7 skips=a>d b>f c>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>g d>END
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
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>e
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
* / e
|/
* f
* g
o @end
```
### jobs=7 skips=a>d b>f c>e f>END
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
* / e
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>e e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>e e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>f c>e e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>d b>f c>e d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>f c>e d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>f c>e d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>f c>e d>g f>END
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
### jobs=7 skips=a>d b>f c>e d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
* e
* f
* g
o @end
```
### jobs=7 skips=a>d b>e f>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
* e
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e e>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
*   e
|\
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e e>g
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
*   e
|\
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e e>g f>END
```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>d b>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e d>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>d b>e c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>e c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>e c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>e c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>d b>e c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e c>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>d b>e c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>d b>e c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>d b>e c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>d b>e c>f d>g f>END
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
### jobs=7 skips=a>d b>e c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c e>g f>END
```text
*   a
|\
* | b
|/
* c
* d
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c d>g f>END
```text
*   a
|\
* | b
|/
* c
*   d
|\
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c d>g e>END
```text
*   a
|\
* | b
|/
* c
*   d
|\
* \   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c d>f e>END
```text
*   a
|\
* | b
|/
* c
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c d>f e>g
```text
*   a
|\
* | b
|/
* c
*   d
|\
* \   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c d>f e>g f>END
```text
*   a
|\
* | b
|/
* c
*   d
|\
* \   e
|\ \
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
### jobs=7 skips=a>c c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c c>g f>END
```text
*   a
|\
* | b
|/
*   c
|\
* | d
* | e
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>g e>END
```text
*   a
|\
* | b
|/
*   c
|\
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>f e>END
```text
*   a
|\
* | b
|/
*   c
|\
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c c>f e>g
```text
*   a
|\
* | b
|/
*   c
|\
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c c>f e>g f>END
```text
*   a
|\
* | b
|/
*   c
|\
* | d
* |   e
|\ \
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
### jobs=7 skips=a>c c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c c>f d>g f>END
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
### jobs=7 skips=a>c c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e e>g f>END
```text
*   a
|\
* | b
|/
*   c
|\
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c c>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c c>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c c>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c c>e d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c c>e d>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>c b>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>END e>g
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>g d>f
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
* | | | e
|/ / /
* / / f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>END c>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>f d>g
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
* | | | e
| |/ /
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>e
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>END c>e e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>e d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>END c>e d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>END c>e d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>c b>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>END d>f
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
* | | | e
|/ / /
* | / f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>c b>g c>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>f d>END
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
* | | | e
| |/ /
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>e
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>g c>e f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>e e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>e d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>e d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>c b>g c>e d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>g c>e d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f e>g
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>f e>g f>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
* |   e
|\ \
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
### jobs=7 skips=a>c b>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>f d>g f>END
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
### jobs=7 skips=a>c b>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>f c>END d>g
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
* | | | e
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>f c>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>f c>g f>END
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
### jobs=7 skips=a>c b>f c>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>g d>END
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
* | | | e
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>e
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
* / e
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>f c>e f>END
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
* / e
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>e e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>e e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>f c>e e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>c b>f c>e d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>f c>e d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>f c>e d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>f c>e d>g f>END
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
### jobs=7 skips=a>c b>f c>e d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
* e
* f
* g
o @end
```
### jobs=7 skips=a>c b>e f>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
* e
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
*   e
|\
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e e>g
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
*   e
|\
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e e>g f>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
* | d
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e d>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>c b>e c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>e c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>e c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>e c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>e c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e c>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>c b>e c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>e c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>e c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>e c>f d>g f>END
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
### jobs=7 skips=a>c b>e c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
* e
* f
* g
o @end
```
### jobs=7 skips=a>c b>d f>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
* e
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d e>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
*   e
|\
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d e>g
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
*   e
|\
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d e>g f>END
```text
*   a
|\
* \   b
|\ \
| |/
|/|
* | c
|/
* d
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d d>END e>g
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
* \   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>d d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d d>g e>END
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
* \   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>d d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d d>f e>END
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
* \   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d d>f e>g
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
* \   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d d>f e>g f>END
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
* \   e
|\ \
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
### jobs=7 skips=a>c b>d c>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>d c>END d>g
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
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>d c>END d>f
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
|/ /
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>END d>f e>g
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
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>d c>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>g d>END
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
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>g d>f
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
|/ /
* / f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>g d>f f>END
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
|/ /
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>g d>f e>END
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
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>d c>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>f e>g f>END
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
* |   e
|\ \
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
### jobs=7 skips=a>c b>d c>f d>END
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
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>f d>END e>g
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
| |_|/
|/| |
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>d c>f d>g
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
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>f d>g f>END
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
### jobs=7 skips=a>c b>d c>f d>g e>END
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
| |_|/
|/| |
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e
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
|/
* e
* f
* g
o @end
```
### jobs=7 skips=a>c b>d c>e f>END
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
|/
* e
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e e>END
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
|/
*   e
|\
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e e>g
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
|/
*   e
|\
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>e e>g f>END
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
|/
*   e
|\
* \   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>END
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
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>END e>g
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
* |   e
|\ \
* | | f
|/ /
* / g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>g
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
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>e d>g f>END
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
* |   f
|\ \
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>g e>END
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
* |   e
|\ \
* | | f
| |/
|/|
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>f
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
|/
* f
* g
o @end
```
### jobs=7 skips=a>c b>d c>e d>f f>END
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
|/
*   f
|\
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>f e>END
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
* |   e
|\ \
| |/
|/|
* | f
* | g
|/
o @end
```
### jobs=7 skips=a>c b>d c>e d>f e>g
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
* |   e
|\ \
| |/
|/|
* | f
|/
* g
o @end
```
### jobs=7 skips=a>c b>d c>e d>f e>g f>END
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
* |   e
|\ \
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
