"""Rejects recordings that are too short, too long or too noisy, and lists the words to retry with their next
candidate. Usage: python3 check_clips.py plan.json rawdir > bad.json"""
import json, hashlib, wave, os, sys
import numpy as np
plan, raw = json.load(open(sys.argv[1])), sys.argv[2]
def key(t): return hashlib.sha1(t.encode()).hexdigest()[:16]
def stats(path):
    w = wave.open(path)
    x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(float)
    if len(x) < 400:
        return 0, 0
    n = 320
    fr = np.array([np.sqrt(np.mean(x[i:i + n] ** 2)) + 1e-3 for i in range(0, len(x) - n, n)])
    return len(x) / 16000, 20 * np.log10(np.percentile(fr, 95) / np.percentile(fr, 10))
bad, good, dur = {}, 0, []
for word, cands in plan.items():
    p = f"{raw}/{key(cands[0]['title'])}.wav"
    if not os.path.exists(p):
        bad[word] = "missing"
        continue
    d, snr = stats(p)
    limit = 3.5 if " " not in word else 5.0
    if d < 0.2 or d > limit or snr < 15:
        bad[word] = f"{d:.2f}s {snr:.0f}dB"
        continue
    good += 1
    dur.append(d)
print(f"good {good} bad {len(bad)} mean {np.mean(dur):.2f}s", file=sys.stderr)
json.dump(bad, sys.stdout, ensure_ascii=False, indent=0)
