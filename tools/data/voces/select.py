import re, json, collections, urllib.request, urllib.parse, time, os, sys
UA = "fuera/1.0 (https://github.com/Guiwald/fuera)"
API = "https://commons.wikimedia.org/w/api.php"
SPAIN = ["Rodelar", "Millars", "ElsaBornFree", "MiguelAlanCS", "Mentxuwiki", "Centenoyespelta"]
LATAM = {"Precision27": "MX", "Rubýñ": "CR", "Sinaloa": "MX", "Enohecruzl79": "MX"}
def norm(s):
    s = s.strip().lower()
    return re.sub(r"\s*\(.*?\)\s*", " ", s).strip()
def neutral(w):
    # no letter that Spain and Latin America pronounce differently (distinción, ll/y)
    w = w.lower()
    return not ("z" in w or re.search(r"c[eéií]", w) or "ll" in w or re.search(r"y[aeiouáéíóú]", w))
targets = [l.strip() for l in open("targets.txt") if l.strip()]
tset = {norm(t): t for t in targets}
by = collections.defaultdict(list)
count = collections.Counter()
for t in open("titles.txt"):
    t = t.strip()
    m = re.match(r"File:LL-Q1321 \(spa\)-(.+?)-(.+)\.(wav|ogg|flac|mp3)$", t)
    if not m or m.group(2).startswith("("): continue
    count[m.group(1)] += 1
    n = norm(m.group(2))
    if n in tset: by[tset[n]].append((m.group(1), t))
# categories (licence, non-native) for every candidate, cached
cache = json.load(open("cats.json")) if os.path.exists("cats.json") else {}
todo = sorted({t for v in by.values() for _, t in v} - set(cache))
for i in range(0, len(todo), 50):
    chunk = todo[i:i+50]
    q = {"action": "query", "titles": "|".join(chunk), "prop": "categories", "cllimit": "max", "format": "json"}
    for a in range(5):
        try:
            d = json.load(urllib.request.urlopen(urllib.request.Request(API + "?" + urllib.parse.urlencode(q), headers={"User-Agent": UA}), timeout=60)); break
        except Exception as e:
            time.sleep(3 * (a + 1))
    norm_map = {x["to"]: x["from"] for x in d["query"].get("normalized", [])}
    for p in d["query"]["pages"].values():
        cache[norm_map.get(p["title"], p["title"])] = [c["title"] for c in p.get("categories", [])]
    json.dump(cache, open("cats.json", "w"))
def licence(cats):
    for c in cats:
        if c.startswith("Category:CC-"): return c[9:]
    return "?"
plan = {}
for word, cands in by.items():
    ok = []
    for sp, t in cands:
        cats = cache.get(t, [])
        if any("Non-native" in c for c in cats): continue
        lic = licence(cats)
        if lic not in ("CC-BY-SA-4.0", "CC-BY-SA-3.0", "CC-BY-4.0", "CC-BY-3.0", "CC-Zero"): continue
        if sp in SPAIN: rank = 0
        elif not neutral(word): continue
        elif sp in LATAM: rank = 2
        else: rank = 1
        ok.append((rank, -count[sp], sp, t, lic))
    if ok:
        ok.sort()
        plan[word] = [{"speaker": sp, "title": t, "licence": lic, "spain": r == 0} for r, _, sp, t, lic in ok[:3]]
json.dump(plan, open("plan.json", "w"), ensure_ascii=False, indent=0)
first = collections.Counter(v[0]["speaker"] for v in plan.values())
print("words with a voice:", len(plan), "of", len(targets), "| Spain speaker:", sum(v[0]["spain"] for v in plan.values()))
print(first.most_common(15))
print("licences:", collections.Counter(v[0]["licence"] for v in plan.values()))
