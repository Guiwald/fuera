"""Build assets/es/lexicon_a.tsv from draft rows (es|pos|gender|fr|en|theme|pic|note|band).

Band = CEFR hint, cross-checked with lemma frequency (freq_rank.py output):
PROMOTE (frequent A2 hints) -> A1, DEMOTE (rare, non-core A1 hints) -> A2.
Rows are ordered by band, then by frequency score (desc).
Usage: python3 -I build_lexicon.py DRAFT_DIR RANK_TSV OUT_TSV
"""
import sys, glob, os

draft_dir, rank_tsv, out = sys.argv[1:4]
PROMOTE = {("quedar", "v"), ("ni", "conj"), ("tipo", "n"), ("traer", "v"), ("recordar", "v"),
           ("usar", "v"), ("olvidar", "v"), ("cambiar", "v")}
DEMOTE = {(w, "n") for w in """plátano zumo falda lápiz tomate ensalada gafas arroz plaza bicicleta nube
          vaca chocolate frío camisa taxi viento diente luna pez oreja fútbol reloj nieve estrella metro
          pollo pescado té río montaña dedo caballo bar fruta deporte billete lluvia campo""".split()}
DEMOTE |= {("tercero", "adj"), ("desayunar", "v")}

freq = {}
for line in open("/home/claude/data/es_50k.txt", encoding="utf-8"):
    p = line.split()
    if len(p) == 2:
        freq[p[0]] = int(p[1])

score = {}
for line in open(rank_tsv, encoding="utf-8"):
    es, pos, band, sc, rk = line.rstrip("\n").split("\t")
    score[(es, pos)] = float(sc)

rows = []
for fn in sorted(glob.glob(os.path.join(draft_dir, "d*.txt"))):
    for line in open(fn, encoding="utf-8"):
        line = line.rstrip("\n")
        if line:
            rows.append(line.split("|"))


SCORE_AS = {("este", "n"): ("oeste", "n")}  # homograph of the determiner: borrow a sibling's score


def est(r):
    es, pos = r[0], r[1]
    if (es, pos) in SCORE_AS:
        return score.get(SCORE_AS[(es, pos)], 0.0)
    s = score.get((es, pos), 0.0)
    if s > 0 and " " not in es:
        return s
    if " " in es:  # multi-word: damped frequency of its rarest word
        return 0.2 * float(min(freq.get(w, 0) for w in es.split()))
    if pos == "expr":  # single-word expression (hay): its own frequency
        return float(freq.get(es, 0))
    if pos == "v" and es.endswith("se"):  # reflexive: fraction of the base verb
        return 0.25 * score.get((es[:-2], "v"), 0.0)
    return s


out_rows = []
for r in rows:
    es, pos, gender, fr, en, theme, pic, note, band = r
    key = (es, pos)
    if key in PROMOTE:
        band = "A1"
    if key in DEMOTE:
        band = "A2"
    out_rows.append((band, -est(r), es, pos, gender, fr, en, theme, pic, note))

out_rows.sort(key=lambda t: (t[0], t[1], t[2]))
with open(out, "w", encoding="utf-8") as f:
    f.write("es\tpos\tgender\tfr\ten\tband\ttheme\tpic\tnote\n")
    for band, _, es, pos, gender, fr, en, theme, pic, note in out_rows:
        f.write("\t".join([es, pos, gender, fr, en, band, theme, pic, note]) + "\n")
print("rows:", len(out_rows), {b: sum(1 for t in out_rows if t[0] == b) for b in ("A1", "A2")})
