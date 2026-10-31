"""List top frequency forms not covered by draft lemmas (with possible verb lemma)."""
import sys, glob, os, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
draft_dir, topn = sys.argv[1], int(sys.argv[2])
rows = []
for fn in sorted(glob.glob(os.path.join(draft_dir, "d*.txt"))):
    for line in open(fn, encoding="utf-8"):
        line = line.rstrip("\n")
        if line:
            rows.append(line.split("|"))
top = []
for i, line in enumerate(open("/home/claude/data/es_50k.txt", encoding="utf-8")):
    if i >= topn:
        break
    top.append(line.split()[0])
topset = set(top)
form2lem = collections.defaultdict(set)
for line in open("/home/claude/data/kaikki/forms.tsv", encoding="utf-8"):
    p = line.rstrip("\n").split("\t")
    if len(p) >= 2 and p[1] in topset:
        form2lem[p[1]].add(p[0])
covered = set()
lemset = {r[0] for r in rows}
for r in rows:
    es = r[0]
    covered |= {es, es + "s", es + "es", es[:-1] + "a", es[:-1] + "as", es[:-1] + "os"}
for f in top:
    pass
out = []
for f in top:
    if f in covered:
        continue
    lems = form2lem.get(f, set())
    if lems & lemset:
        continue
    out.append(f + ("<" + ",".join(sorted(lems)[:2]) + ">" if lems else ""))
print(" ".join(out))
