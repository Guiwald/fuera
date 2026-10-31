"""Compute a lemma-level frequency score for draft lexicon rows.

Usage: python3 -I freq_rank.py DRAFT_DIR OUT_TSV
Draft rows: es|pos|gender|fr|en|theme|pic|note|band  (files d*.txt)
Output: es pos band score rank  (sorted by score desc)
"""
import sys, glob, os, collections

DATA = "/home/claude/data"
draft_dir, out_path = sys.argv[1], sys.argv[2]

rows = []
for fn in sorted(glob.glob(os.path.join(draft_dir, "d*.txt"))):
    with open(fn, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                continue
            rows.append(line.split("|"))

freq = {}
with open(os.path.join(DATA, "es_50k.txt"), encoding="utf-8") as f:
    for line in f:
        p = line.split()
        if len(p) == 2:
            freq[p[0]] = int(p[1])

verb_lemmas = {r[0] for r in rows if r[1] == "v"}
vforms = collections.defaultdict(set)
with open(os.path.join(DATA, "kaikki/forms.tsv"), encoding="utf-8") as f:
    for line in f:
        p = line.rstrip("\n").split("\t")
        if len(p) >= 2 and p[0] in verb_lemmas and " " not in p[1]:
            vforms[p[0]].add(p[1])

ACC = str.maketrans("áéíóú", "aeiou")


def plural(w):
    if w.endswith(("s", "x")) and not w.endswith(("és", "ás", "ís", "ós", "ús")):
        return [w]
    if w[-1] in "aeiouáéó":
        return [w + "s"]
    if w.endswith("z"):
        return [w[:-1] + "ces"]
    base = w
    if w.endswith(("ón", "án", "én", "ín", "ún", "és", "ás", "ís")):
        base = w[:-2] + w[-2].translate(ACC) + w[-1]
    return [base + "es"]


def forms_for(r):
    es, pos = r[0], r[1]
    if pos == "v":
        fs = set(vforms.get(es, set())) | {es}
        return fs
    if pos == "expr" or " " in es:
        return set()
    fs = {es}
    if pos == "n":
        fs |= set(plural(es))
    elif pos in ("adj", "det", "pron"):
        if es.endswith("o"):
            fem = es[:-1] + "a"
            fs |= {fem, es + "s", fem + "s"}
        else:
            fs |= set(plural(es))
        extra = {"bueno": ["buen"], "malo": ["mal"], "grande": ["gran"], "primero": ["primer"],
                 "tercero": ["tercer"], "alguno": ["algún"], "ninguno": ["ningún"],
                 "el": ["la", "los", "las"], "un": ["una", "unos", "unas"],
                 "este": ["esta", "estos", "estas"], "ese": ["esa", "esos", "esas"],
                 "aquel": ["aquella", "aquellos", "aquellas"], "mi": ["mis"], "tu": ["tus"],
                 "su": ["sus"], "cualquier": ["cualquiera"], "lo": ["la", "los", "las"], "le": ["les"],
                 "nosotros": ["nosotras"], "vosotros": ["vosotras"], "ellos": ["ellas"]}
        fs |= set(extra.get(es, []))
    return fs


MANUAL_VFORMS = {
    "hacer": "hacer hago haces hace hacemos hacéis hacen hice hiciste hizo hicimos hicisteis hicieron hacía hacías hacíamos hacían haré harás hará haremos haréis harán haría harías haríamos harían haga hagas hagamos hagáis hagan hiciera hicieras hiciéramos hicieran hiciese haz haced hecho hecha hechos hechas haciendo hacerlo hacerte hacerle hacerme hazlo",
    "valer": "valer valgo vales vale valemos valéis valen valí valió valieron valía valían valdrá valdría valga valgan valiera valido",
}
for _l, _fs in MANUAL_VFORMS.items():
    if _l in verb_lemmas:
        vforms[_l] |= set(_fs.split())

claims = collections.defaultdict(list)
for i, r in enumerate(rows):
    for fm in forms_for(r):
        claims[fm].append(i)

score = [0.0] * len(rows)
for fm, idxs in claims.items():
    c = freq.get(fm, 0)
    if not c:
        continue
    exact = [i for i in idxs if rows[i][0] == fm and rows[i][1] != "v"]
    nonverb = [i for i in idxs if rows[i][1] != "v"]
    if exact:
        tgt = exact
    elif nonverb and len(nonverb) < len(idxs):
        tgt = nonverb
    else:
        tgt = idxs
    for i in tgt:
        score[i] += c / len(tgt)

order = sorted(range(len(rows)), key=lambda i: -score[i])
with open(out_path, "w", encoding="utf-8") as f:
    for rank, i in enumerate(order, 1):
        r = rows[i]
        f.write(f"{r[0]}\t{r[1]}\t{r[8]}\t{int(score[i])}\t{rank}\n")
