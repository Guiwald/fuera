"""French conjugation fixture for the conjugator test (crates/learn/tests/data/forms_fr.tsv).

Usage: python3 -I build_forms.py idx/verbs.tsv lemfreq.tsv N [extra_verbs.txt] > forms_fr.tsv

Takes the N most frequent verbs of lemfreq.tsv (plus an optional list), keeps the slots extracted from English
Wiktionary (present, imparfait, futur, conditionnel, subjonctif, past participle, auxiliary).
"""
import sys
verbs_path, freq_path, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
extra = [l.strip() for l in open(sys.argv[4], encoding="utf-8")] if len(sys.argv) > 4 else []
table = {}
for i, line in enumerate(open(verbs_path, encoding="utf-8")):
    if i == 0:
        continue
    lemma, tense, person, forms = line.rstrip("\n").split("\t")
    fs = [f for f in forms.split(",") if f and not f.startswith("/") and " " not in f]
    if fs:
        # Several entries (etymologies) per verb: accept the forms of all of them.
        slot = table.setdefault(lemma, {}).setdefault((tense, person), [])
        slot.extend(f for f in fs if f not in slot)
order = []
for i, line in enumerate(open(freq_path, encoding="utf-8")):
    if i == 0:
        continue
    c = line.split("\t")
    if c[1] == "v" and c[0] in table and c[0] not in order:
        order.append(c[0])
# 1990 spellings (*connaitre*), vulgar or misanalysed lemmas are left out.
SKIP = {"connaitre", "paraitre", "apparaitre", "disparaitre", "reconnaitre", "naitre", "foutre", "soir", "luire", "celer",
        "paître", "minuter", "ouvrer", "verrer", "ailler", "policer", "terrer", "musiquer", "contrer", "tabler", "priser", "saler"}
order = [v for v in order if v not in SKIP]
chosen = order[:n] + [v for v in extra if v in table and v not in order[:n]]
print("infinitive\ttense\tperson\tforms")
for v in chosen:
    for (t, p), fs in sorted(table[v].items()):
        print(f"{v}\t{t}\t{p}\t{','.join(fs)}")
