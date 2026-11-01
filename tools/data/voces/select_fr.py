"""Chooses one Lingua Libre recording per French word of the game (assets/fr/voces/voces-fr.fvox).

Usage: python3 -I select_fr.py targets_fr.txt titles_fra.txt ll_locations.json > candidates_fr.json

Where a speaker lives comes from the French Wiktionary: the Lingua Libre bot adds each recording there with the
speaker's place (`France (Lyon)`, `Suisse (Lausanne)`, `Canada (Shawinigan)`), extracted by wiktextract
(ll_locations.json: speaker → place → count). Order of preference: France outside the South, then the South of
France, then Switzerland and Belgium. Québec, other countries and speakers with no known place are left out.
Inside a tier, the speakers with the most recordings of the game's words come first (fewer voices, more even).
"""
import json, re, sys, collections

SOUTH = ["Toulouse", "Aude", "Hérault", "Occitanie", "Céret", "Cerdans", "Millau", "Cesseras", "Montpouillan", "Bordeaux",
         "Lot-et-Garonne", "Agen", "Marseille", "Provence", "Nice", "Montpellier", "Perpignan", "Pyrénées", "Nîmes",
         "Sud de la France", "Gard", "Var", "Gironde", "Landes", "Béziers", "Narbonne", "Ariège", "Tarn", "Aveyron"]

def tier(places):
    """0 France (not the South), 1 South of France, 2 Switzerland/Belgium, None otherwise."""
    if not places:
        return None
    top = max(places.items(), key=lambda x: x[1])[0]
    total = sum(places.values())
    known = {k: v for k, v in places.items() if k != "?"}
    if not known:
        return None
    top = max(known.items(), key=lambda x: x[1])[0]
    if re.search(r"Canada|Québec|Quebec|Italie|Belgique|Suisse|Sénégal|Cameroun|Congo|Haïti|Maroc|Algérie|Tunisie|Liban|États-Unis|Allemagne", top) is None and "France" in top:
        return 1 if any(s in top for s in SOUTH) else 0
    if re.search(r"Suisse|Belgique", top):
        return 2
    return None

def norm(s):
    s = s.strip().lower().replace("’", "'").replace("_", " ")
    return re.sub(r"\s*\(.*?\)\s*", " ", s).strip()

targets = [l.strip() for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
tset = {norm(t): t for t in targets}
places = json.load(open(sys.argv[3], encoding="utf-8"))
by = collections.defaultdict(list)
hits = collections.Counter()
for t in open(sys.argv[2], encoding="utf-8"):
    t = t.strip()
    m = re.match(r"File:LL-Q150 \(fra\)-(.+?)-(.+)\.(wav|ogg|flac|mp3)$", t)
    if not m:
        continue
    n = norm(m.group(2))
    if n in tset:
        by[tset[n]].append((m.group(1), t))
        hits[m.group(1)] += 1
plan = {}
for word, cands in by.items():
    ok = []
    for sp, title in cands:
        tr = tier(places.get(sp, {}))
        if tr is None:
            continue
        ok.append((tr, -hits[sp], sp, title))
    ok.sort()
    if ok:
        plan[word] = [{"speaker": sp, "title": title, "tier": tr} for tr, _, sp, title in ok[:4]]
json.dump(plan, sys.stdout, ensure_ascii=False, indent=0)
first = collections.Counter(v[0]["speaker"] for v in plan.values())
print(f"words {len(targets)}, with a candidate {len(plan)}; by tier {collections.Counter(v[0]['tier'] for v in plan.values())}", file=sys.stderr)
print(first.most_common(20), file=sys.stderr)
