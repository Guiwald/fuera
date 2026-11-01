import json, hashlib, wave, numpy as np, os, collections
plan = json.load(open("plan.json"))
def key(t): return hashlib.sha1(t.encode()).hexdigest()[:16]
def stats(path):
    w = wave.open(path); x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(float)
    if len(x) < 400: return 0, 0, 0
    n = 320; fr = np.array([np.sqrt(np.mean(x[i:i+n]**2)) + 1e-3 for i in range(0, len(x)-n, n)])
    return len(x) / 16000, 20*np.log10(np.percentile(fr, 95)/np.percentile(fr, 10)), np.abs(x).max()
bad, good, dur = [], 0, []
for word, cands in plan.items():
    p = f"raw/{key(cands[0]['title'])}.wav"
    if not os.path.exists(p): bad.append((word, "missing")); continue
    d, snr, peak = stats(p)
    if d < 0.2 or d > 3.5 or snr < 15: bad.append((word, f"{d:.2f}s {snr:.0f}dB")); continue
    good += 1; dur.append(d)
print("good", good, "bad", len(bad), "mean dur", round(np.mean(dur), 2), "total s", round(sum(dur)))
print(bad[:40])
json.dump(bad, open("bad.json", "w"), ensure_ascii=False)
