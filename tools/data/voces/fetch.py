import json, os, re, subprocess, urllib.request, urllib.parse, concurrent.futures, time, sys, hashlib
UA = "fuera/1.0 (https://github.com/Guiwald/fuera)"
plan = json.load(open("plan.json"))
DEADLINE = time.time() + 150
ERR = []
os.makedirs("raw", exist_ok=True)
def key(title): return hashlib.sha1(title.encode()).hexdigest()[:16]
def get(title):
    out = f"raw/{key(title)}.wav"
    if os.path.exists(out): return
    if time.time() > DEADLINE: return
    name = title[5:].replace(" ", "_")
    h = hashlib.md5(name.encode()).hexdigest()
    url = f"https://upload.wikimedia.org/wikipedia/commons/{h[0]}/{h[:2]}/{urllib.parse.quote(name)}"
    for a in range(4):
        try:
            raw = urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": UA}), timeout=60).read(); break
        except Exception as e:
            ERR.append(str(e)[:40]); time.sleep(2 * (a + 1))
    else:
        print("FAIL", title, flush=True); return
    tmp = out + ".src"
    open(tmp, "wb").write(raw)
    # mono 16 kHz, silence trimmed at both ends, loudness normalised
    af = "silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.05,areverse,silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.08,areverse,highpass=f=70,loudnorm=I=-18:TP=-2:LRA=11"
    r = subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", tmp, "-af", af, "-ac", "1", "-ar", "16000", "-sample_fmt", "s16", out])
    os.remove(tmp)
    if r.returncode: print("FFMPEG", title, flush=True)
titles = [v[0]["title"] for v in plan.values()]
if len(sys.argv) > 1 and sys.argv[1] == "all":
    titles = sorted({c["title"] for v in plan.values() for c in v})
with concurrent.futures.ThreadPoolExecutor(int(os.environ.get('T','4'))) as ex:
    for i, _ in enumerate(ex.map(get, titles)):
        if i % 100 == 0: print(i, flush=True)
import collections
print("done", collections.Counter(ERR).most_common(3), flush=True)
