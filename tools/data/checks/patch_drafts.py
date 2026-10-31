"""Apply field patches to draft rows: python3 -I patch_drafts.py DRAFT_DIR PATCH_FILE
PATCH_FILE lines: es|pos|field=value|field=value ...  (fields: gender fr en theme pic note band)"""
import sys, glob, os
FIELDS = ["es", "pos", "gender", "fr", "en", "theme", "pic", "note", "band"]
d, pf = sys.argv[1], sys.argv[2]
patches = {}
for line in open(pf, encoding="utf-8"):
    line = line.rstrip("\n")
    if not line.strip():
        continue
    parts = line.split("|")
    patches[(parts[0], parts[1])] = [p.split("=", 1) for p in parts[2:]]
used = set()
for fn in sorted(glob.glob(os.path.join(d, "d*.txt"))):
    out = []
    for line in open(fn, encoding="utf-8"):
        cols = line.rstrip("\n").split("|")
        key = (cols[0], cols[1])
        if key in patches:
            for k, v in patches[key]:
                cols[FIELDS.index(k)] = v
            used.add(key)
        out.append("|".join(cols))
    open(fn, "w", encoding="utf-8").write("\n".join(out) + "\n")
missing = set(patches) - used
print("patched", len(used), "missing", sorted(missing))
