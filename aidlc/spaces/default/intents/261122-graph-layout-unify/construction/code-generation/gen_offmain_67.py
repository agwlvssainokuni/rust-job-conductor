# ジョブ 6〜7 個の、「両方の飛び越し」か分岐を含む形の正解データを、参照実装 `layout-reference.py` の
# `layout` で生成する(graph-layout-unify の Step 5)。git の出力ではなく、規則から導いた図。
# 使い方: python3 gen_offmain_67.py <layout-reference.py のパス> <出力のパス>
import importlib.util
import random
import sys

SEED = 20261122
COUNT = 300


def load_layout(path):
    spec = importlib.util.spec_from_file_location("layout_reference", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.layout


def name(n, x):
    return "END" if x == n else chr(97 + x)


def random_shape(rng):
    n = rng.choice([6, 7])
    skip = {}
    for i in range(n):
        far = list(range(i + 2, n + 1))
        choices = ["none"]
        if far:
            choices += ["n", "s"]
        if len(far) >= 2:
            choices.append("b")
        kind = rng.choices(choices, weights=[4, 3, 2, 2][: len(choices)])[0]
        if kind == "n":
            skip[i] = ("n", rng.choice(far))
        elif kind == "s":
            skip[i] = ("s", rng.choice(far))
        elif kind == "b":
            t1, t2 = sorted(rng.sample(far, 2))
            skip[i] = ("b", t1, t2)
    return n, skip


def label(n, skip):
    parts = []
    for i in sorted(skip):
        sp = skip[i]
        a = chr(97 + i)
        if sp[0] == "n":
            parts.append(f"{a}>{name(n, sp[1])}")
        elif sp[0] == "s":
            parts.append(f"{a}>>{name(n, sp[1])}")
        else:
            parts.append(f"{a}>{name(n, sp[1])},{name(n, sp[2])}")
    return " ".join(parts) if parts else "-"


HEADER = """# レイアウトの正解データ(両方の飛び越し・分岐を含む、ジョブ 6〜7 個の形): graph-layout-unify

この文書は、`conductor graph` の text の図のうち、**「両方の飛び越し」か分岐を、1 つ以上含む、ジョブ 6〜7 個の形**の、**正解の図**である(深い入れ子の確認用)。

**注意: この図は、`git log --graph` の出力ではない。** 規則(`functional-spec.md` の WF2)の参照実装 `layout-reference.py` の `layout` から生成した。

- 形の表し方: `jobs=<ジョブの数> skips=<辺>`。`a>d` は「`a` の成功が `d` へ飛ぶ(警告は次のジョブへ)」。`a>>d` は「`a` が、成功でも警告でも `d` へ(両方の飛び越し)」。`a>c,d` は「`a` の成功が `c`、警告が `d` へ(分岐)」。`END` は終点。
- 作り方: `construction/code-generation/gen_offmain_67.py` で、乱数の種 20261122 から、ジョブ 6 か 7 個で、「両方の飛び越し」か分岐を 1 つ以上含む形を、重複なく 300 個、生成順に並べた(`python3 gen_offmain_67.py <layout-reference.py> <この文書>`)。
- **この文書のすべての例に、1 文字も違わず一致すること**が、レーンの配置の合格の条件である。
"""


def main():
    layout = load_layout(sys.argv[1])
    rng = random.Random(SEED)
    seen = set()
    blocks = []
    while len(blocks) < COUNT:
        n, skip = random_shape(rng)
        if not any(sp[0] in ("s", "b") for sp in skip.values()):
            continue
        key = (n, label(n, skip))
        if key in seen:
            continue
        seen.add(key)
        lines = [x.rstrip() for x in layout(n, skip)]
        blocks.append(f"### jobs={n} skips={key[1]}\n```text\n" + "\n".join(lines) + "\n```\n")
    with open(sys.argv[2], "w") as out:
        out.write(HEADER + "\n" + "".join(blocks))


if __name__ == "__main__":
    main()
