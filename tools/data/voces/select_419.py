"""Chooses Latin-American recordings for the Latin-American Spanish pack (assets/es/voces/voces-es-419.fvox).

Usage: python3 -I select_419.py targets_419.txt titles_spa.txt ll_locations_spa.json > candidates_419.json

Speakers' places come from the French Wiktionary (Lingua Libre bot, see select_fr.py), plus the ones checked by hand
for 1.0.1 (Precision27, Sinaloa, Enohecruzl79: Mexico; Rubýñ: Costa Rica). Tiers: Mexico and Central America;
Andean Spanish (Bogotá, Lima, Quito); Caribbean (Venezuela, Colombian coast, Cuba, Puerto Rico); River Plate and Chile
only for words without ll or consonant y (sheísmo). A word without any of these keeps a Spain recording when it has
no letter pronounced differently on the two sides (no z, no c before e/i): build_419.py takes it from the Spain pack.
"""
import json, re, sys, collections

KNOWN = {"Precision27": "Mexique", "Sinaloa": "Mexique", "Enohecruzl79": "Mexique", "Rubýñ": "Costa Rica"}
T0 = r"Mexique|Mexico|Honduras|Costa Rica|Guatemala|Salvador|Nicaragua|Panam"
T1 = r"Bogota|Bogotá|Pérou|Lima|Équateur|Quito|Bolivie|Medell"
T2 = r"Venezuela|Carthag|Cuba|Porto Rico|Puerto Rico|dominicaine|Colombie"
T3 = r"Argentine|Uruguay|Paraguay|Chili|Buenos Aires"

def tier(sp, places):
    if sp in KNOWN:
        return 0
    known = {k: v for k, v in places.get(sp, {}).items() if k != "?"}
    if not known:
        return None
    top = max(known.items(), key=lambda x: x[1])[0]
    for i, rx in enumerate([T0, T1, T2, T3]):
        if re.search(rx, top):
            return i
    return None

def norm(s):
    s = s.strip().lower().replace("_", " ")
    return re.sub(r"\s*\(.*?\)\s*", " ", s).strip()

def sheismo_safe(w):
    return not ("ll" in w or re.search(r"y[aeiouáéíóú]", w))

targets = [l.strip() for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
tset = {norm(t): t for t in targets}
places = json.load(open(sys.argv[3], encoding="utf-8"))
by = collections.defaultdict(list)
hits = collections.Counter()
for t in open(sys.argv[2], encoding="utf-8"):
    t = t.strip()
    m = re.match(r"File:LL-Q1321 \(spa\)-(.+?)-(.+)\.(wav|ogg|flac|mp3)$", t)
    if not m or m.group(2).startswith("("):
        continue
    n = norm(m.group(2))
    if n in tset:
        by[tset[n]].append((m.group(1), t))
        hits[m.group(1)] += 1
plan = {}
for word, cands in by.items():
    ok = []
    for sp, title in cands:
        tr = tier(sp, places)
        if tr is None or (tr == 3 and not sheismo_safe(word.lower())):
            continue
        ok.append((tr, -hits[sp], sp, title))
    ok.sort()
    if ok:
        plan[word] = [{"speaker": sp, "title": title, "tier": tr} for tr, _, sp, title in ok[:3]]
json.dump(plan, sys.stdout, ensure_ascii=False, indent=0)
print(f"words {len(targets)}, with a Latin-American voice {len(plan)}; tiers {collections.Counter(v[0]['tier'] for v in plan.values())}", file=sys.stderr)
print(collections.Counter(v[0]["speaker"] for v in plan.values()).most_common(15), file=sys.stderr)
