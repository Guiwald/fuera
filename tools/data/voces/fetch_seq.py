"""Downloads the chosen recordings one at a time (Commons limits the rate), converts each to mono 16 kHz with the
silences trimmed and the loudness evened out. Resumable: run again until it prints "complete".

Usage: python3 fetch_seq.py plan.json outdir [budget_seconds] [all]
"""
import hashlib, json, os, subprocess, sys, time, urllib.parse, urllib.request
UA = "fuera/1.1 (https://github.com/Guiwald/fuera)"
plan, out = json.load(open(sys.argv[1])), sys.argv[2]
budget = float(sys.argv[3]) if len(sys.argv) > 3 else 150
os.makedirs(out, exist_ok=True)
# Downloads land in a scratch folder first (outside the output folder: only the converted WAV goes there).
TMP = os.environ.get("FUERA_TMP", os.path.join(os.path.expanduser("~"), "voces-tmp"))
os.makedirs(TMP, exist_ok=True)
def key(title): return hashlib.sha1(title.encode()).hexdigest()[:16]
AF = "silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.05,areverse,silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.08,areverse,highpass=f=70,loudnorm=I=-18:TP=-2:LRA=11"
titles = [v[0]["title"] for v in plan.values()]
if "all" in sys.argv[4:]:
    titles = [c["title"] for v in plan.values() for c in v[:2]]
todo = [t for t in titles if not os.path.exists(f"{out}/{key(t)}.wav") and not os.path.exists(f"{out}/{key(t)}.fail")]
t0, n, wait = time.time(), 0, 0.0
for title in todo:
    if time.time() - t0 > budget:
        break
    name = title[5:].replace(" ", "_")
    h = hashlib.md5(name.encode()).hexdigest()
    url = f"https://upload.wikimedia.org/wikipedia/commons/{h[0]}/{h[:2]}/{urllib.parse.quote(name)}"
    raw = None
    for a in range(4):
        try:
            raw = urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": UA}), timeout=40).read()
            break
        except urllib.error.HTTPError as e:
            if e.code == 404:
                break
            wait = 5 * (a + 1) if e.code == 429 else 2
            time.sleep(wait)
        except Exception:
            time.sleep(2)
    if raw is None:
        open(f"{out}/{key(title)}.fail", "w").write(title)
        continue
    src = f"{TMP}/{key(title)}.src"
    open(src, "wb").write(raw)
    r = subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", src, "-af", AF, "-ac", "1", "-ar", "16000", "-sample_fmt", "s16", f"{out}/{key(title)}.wav"])
    os.remove(src)
    if r.returncode:
        open(f"{out}/{key(title)}.fail", "w").write(title + " ffmpeg")
    n += 1
    time.sleep(float(os.environ.get("FUERA_PAUSE", "1.2")))
left = len([t for t in titles if not os.path.exists(f"{out}/{key(t)}.wav") and not os.path.exists(f"{out}/{key(t)}.fail")])
print(f"fetched {n}, left {left}" if left else f"fetched {n}, complete", flush=True)
