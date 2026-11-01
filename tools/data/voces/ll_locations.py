"""Where Lingua Libre speakers live, from the French Wiktionary: the Lingua Libre bot adds each recording there with
the speaker's place (`France (Lyon)`, `Canada (Shawinigan)`). Reads wiktextract's raw French-edition dump.

Usage: zcat raw-wiktextract-data.jsonl.gz | grep -F '"LL-Q150 (fra)' | python3 -I ll_locations.py Q150 fra > ll_locations.json
       (Q1321 spa for Spanish)
"""
import collections, json, re, sys
q, code = sys.argv[1], sys.argv[2]
pat = re.compile(rf"LL-{q} \({code}\)-(.+?)-(.+)\.(wav|ogg|flac)$")
loc = collections.defaultdict(collections.Counter)
for line in sys.stdin:
    try:
        d = json.loads(line)
    except Exception:
        continue
    for s in d.get("sounds", []) or []:
        m = pat.match(s.get("audio", ""))
        if m:
            tags = s.get("raw_tags") or s.get("tags") or []
            loc[m.group(1)][" / ".join(tags) or "?"] += 1
json.dump({k: dict(v) for k, v in loc.items()}, sys.stdout, ensure_ascii=False)
