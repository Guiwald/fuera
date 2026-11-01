import json, urllib.request, urllib.parse, time, sys
UA = "fuera/1.0 (https://github.com/Guiwald/fuera)"
API = "https://commons.wikimedia.org/w/api.php"
params = {"action": "query", "list": "categorymembers", "cmtitle": "Category:Lingua Libre pronunciation-spa",
          "cmlimit": "500", "cmtype": "file", "format": "json"}
titles = []
cont = {}
while True:
    q = dict(params); q.update(cont)
    req = urllib.request.Request(API + "?" + urllib.parse.urlencode(q), headers={"User-Agent": UA})
    for attempt in range(5):
        try:
            d = json.load(urllib.request.urlopen(req, timeout=60)); break
        except Exception as e:
            print("retry", attempt, e, file=sys.stderr); time.sleep(3 * (attempt + 1))
    else:
        raise SystemExit("gave up")
    titles += [m["title"] for m in d["query"]["categorymembers"]]
    if "continue" not in d: break
    cont = d["continue"]
    time.sleep(0.2)
open("titles.txt", "w").write("\n".join(titles) + "\n")
print(len(titles))
