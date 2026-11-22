# 参照実装 `layout-reference.py` の `layout` で、1 つの形の図を出す(テストの期待を確かめる補助)。
# 使い方: python3 print_layout.py <layout-reference.py> <ジョブの数> '<skips>'(例: 'a>>d b>e c>e,END')
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("layout_reference", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
n = int(sys.argv[2])
label = sys.argv[3] if len(sys.argv) > 3 else "-"
skip = {}
if label.strip() != "-":
    for p in label.split():
        both = ">>" in p
        a, b = p.split(">>") if both else p.split(">")
        tg = [n if x == "END" else ord(x) - 97 for x in b.split(",")]
        if both:
            skip[ord(a) - 97] = ("s", tg[0])
        elif len(tg) == 1:
            skip[ord(a) - 97] = ("n", tg[0])
        else:
            t1, t2 = sorted(tg)
            skip[ord(a) - 97] = ("b", t1, t2)
lines = [line.rstrip() for line in module.layout(n, skip)]
print("\n".join(lines))
# Rust の文字列の書き方(`\` と改行をエスケープ)。
print('"' + "\\n".join(line.replace("\\", "\\\\") for line in lines) + '\\n"')
