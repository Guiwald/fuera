"""Fetches the Commons categories (licence, "Non-native …") of every candidate recording, into a resumable cache.

Usage: python3 cats.py candidates.json [more.json…]   (writes cats.json; run again until it says "complete")
"""
import json, os, sys, time, urllib.parse, urllib.request
UA = "fuera/1.1 (https://github.com/Guiwald/fuera)"
API = "https://commons.wikimedia.org/w/api.php"
titles = sorted({c["title"] for f in sys.argv[1:] for v in json.load(open(f)).values() for c in v})
cache = json.load(open("cats.json")) if os.path.exists("cats.json") else {}
todo = [t for t in titles if t not in cache]
t0 = time.time()
for i in range(0, len(todo), 50):
    if time.time() - t0 > 150:
        break
    chunk = todo[i:i + 50]
    q = {"action": "query", "titles": "|".join(chunk), "prop": "categories", "cllimit": "max", "format": "json"}
    for a in range(5):
        try:
            d = json.load(urllib.request.urlopen(urllib.request.Request(API, data=urllib.parse.urlencode(q).encode(), headers={"User-Agent": UA}), timeout=60))
            break
        except Exception:
            time.sleep(3 * (a + 1))
    else:
        break
    norm = {x["to"]: x["from"] for x in d["query"].get("normalized", [])}
    for p in d["query"]["pages"].values():
        cache[norm.get(p["title"], p["title"])] = [c["title"] for c in p.get("categories", [])]
    json.dump(cache, open("cats.json", "w"))
    time.sleep(0.3)
left = len([t for t in titles if t not in cache])
print("complete" if not left else f"{left} left", flush=True)
