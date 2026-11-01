"""Keeps the free-licence, native candidates (Commons categories in cats.json) and writes the download plan.

Usage: python3 plan.py candidates.json plan.json
"""
import json, sys, collections
FREE = ("CC-BY-SA-4.0", "CC-BY-SA-3.0", "CC-BY-4.0", "CC-BY-3.0", "CC-Zero")
cats = json.load(open("cats.json"))
cands = json.load(open(sys.argv[1]))
def licence(cs):
    for c in cs:
        if c.startswith("Category:CC-"):
            return c[9:]
    return "?"
plan = {}
for word, cs in cands.items():
    ok = []
    for c in cs:
        cc = cats.get(c["title"], [])
        if any("Non-native" in x or "non-native" in x for x in cc):
            continue
        lic = licence(cc)
        if lic not in FREE:
            continue
        ok.append(dict(c, licence=lic))
    if ok:
        plan[word] = ok
json.dump(plan, open(sys.argv[2], "w"), ensure_ascii=False, indent=0)
print(len(plan), "words;", collections.Counter(v[0]["licence"] for v in plan.values()), collections.Counter(v[0]["speaker"] for v in plan.values()).most_common(8))
