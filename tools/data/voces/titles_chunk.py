"""Lists a Commons category of Lingua Libre recordings in chunks (`budget` seconds each), resumable.

Usage: python3 titles_chunk.py "Category:Lingua Libre pronunciation-fra" titles_fra.txt 160   (repeat until done)
"""
import json, urllib.request, urllib.parse, time, sys, os
UA = "fuera/1.1 (https://github.com/Guiwald/fuera)"
API = "https://commons.wikimedia.org/w/api.php"
cat, out, budget = sys.argv[1], sys.argv[2], float(sys.argv[3])
state = out + ".cont"
cont = json.load(open(state)) if os.path.exists(state) else {}
if cont == "done":
    print("already done"); sys.exit()
params = {"action": "query", "list": "categorymembers", "cmtitle": cat, "cmlimit": "500", "cmtype": "file", "format": "json"}
t0 = time.time(); n = 0
with open(out, "a") as f:
    while time.time() - t0 < budget:
        q = dict(params); q.update(cont)
        req = urllib.request.Request(API + "?" + urllib.parse.urlencode(q), headers={"User-Agent": UA})
        for attempt in range(4):
            try:
                d = json.load(urllib.request.urlopen(req, timeout=30)); break
            except Exception:
                time.sleep(4 * (attempt + 1))
        else:
            break
        ts = [m["title"] for m in d["query"]["categorymembers"]]
        f.write("".join(t + "\n" for t in ts)); f.flush(); n += len(ts)
        if "continue" not in d:
            cont = "done"; json.dump(cont, open(state, "w")); break
        cont = d["continue"]; json.dump(cont, open(state, "w"))
        time.sleep(0.2)
print("added", n, "state", "done" if cont == "done" else "more", flush=True)
