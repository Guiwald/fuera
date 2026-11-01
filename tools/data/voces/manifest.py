"""Writes a voice-pack manifest (`fuera-cli voices` input) from a download plan and the converted recordings.

Usage:
  python3 -I manifest.py fr plan_fr.json raw_fr > credits.tsv
  python3 -I manifest.py es-419 plan_419.json raw_419 targets_419.txt creditos_es.tsv flac_dir > creditos-419.tsv

Columns: text, speaker, licence, Commons file, audio file (in the raw folder). For es-419, a word without a
Latin-American recording keeps its Spain recording (creditos_es.tsv, audio from flac_dir converted to WAV next to
the others) when nothing in it is pronounced differently on the two sides: no z, no c before e/i.
"""
import hashlib, json, os, re, subprocess, sys, wave
import numpy as np

LIC = {"CC-BY-SA-4.0": "CC BY-SA 4.0", "CC-BY-SA-3.0": "CC BY-SA 3.0", "CC-BY-4.0": "CC BY 4.0", "CC-BY-3.0": "CC BY 3.0", "CC-Zero": "CC0"}
def key(t): return hashlib.sha1(t.encode()).hexdigest()[:16]
def ok(path, word):
    try:
        w = wave.open(path)
        x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(float)
    except Exception:
        return False
    if len(x) < 3200:
        return False
    n = 320
    fr = np.array([np.sqrt(np.mean(x[i:i + n] ** 2)) + 1e-3 for i in range(0, len(x) - n, n)])
    snr = 20 * np.log10(np.percentile(fr, 95) / np.percentile(fr, 10))
    d = len(x) / 16000
    return snr >= 15 and d <= (5.0 if " " in word else 3.5)

kind, plan_path, raw = sys.argv[1], sys.argv[2], sys.argv[3]
plan = json.load(open(plan_path, encoding="utf-8"))
rows, done = [], set()
for word, cands in sorted(plan.items()):
    for c in cands[:2]:
        p = f"{raw}/{key(c['title'])}.wav"
        if os.path.exists(p) and ok(p, word):
            url = "https://commons.wikimedia.org/wiki/" + c["title"].replace(" ", "_")
            rows.append((word, c["speaker"], LIC.get(c["licence"], c["licence"]), url, os.path.basename(p)))
            done.add(word)
            break
if kind == "es-419":
    targets = [l.strip() for l in open(sys.argv[4], encoding="utf-8") if l.strip()]
    spain = {}
    for i, l in enumerate(open(sys.argv[5], encoding="utf-8")):
        c = l.rstrip("\n").split("\t")
        if i and len(c) >= 5:
            spain[c[0]] = c
    flac = sys.argv[6]
    for word in targets:
        if word in done or word not in spain:
            continue
        w = word.lower()
        if "z" in w or re.search(r"c[eéií]", w):
            continue
        text, sp, lic, url, audio = spain[word][:5]
        src = os.path.join(flac, audio.replace(".wav", ".flac"))
        dst = os.path.join(raw, "es_" + audio)
        if not os.path.exists(dst):
            subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", src, "-ac", "1", "-ar", "16000", "-sample_fmt", "s16", dst], check=True)
        rows.append((word, sp, lic, url, os.path.basename(dst)))
        done.add(word)
head = "texto\tlocutor\tlicencia\tarchivo_commons\taudio" if kind.startswith("es") else "texte\tlocuteur\tlicence\tfichier_commons\taudio"
print(head)
for r in sorted(rows):
    print("\t".join(r))
print(f"{len(rows)} recordings", file=sys.stderr)
