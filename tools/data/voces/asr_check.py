import json, sys, wave, unicodedata, difflib
from vosk import Model, KaldiRecognizer, SetLogLevel
SetLogLevel(-1)
model_dir, manifest, raw = sys.argv[1], sys.argv[2], sys.argv[3]
rows = [l.rstrip("\n").split("\t") for i, l in enumerate(open(manifest, encoding="utf-8")) if i]
words = sorted({r[0].lower() for r in rows})
model = Model(model_dir)
def fold(s):
    return "".join(c for c in unicodedata.normalize("NFD", s.lower()) if unicodedata.category(c) != "Mn").replace("'", " ").replace("-", " ")
bad = []
rec = KaldiRecognizer(model, 16000, json.dumps(words + ["[unk]"], ensure_ascii=False))
for r in rows:
    w = wave.open(f"{raw}/{r[4]}")
    rec.AcceptWaveform(w.readframes(w.getnframes()))
    txt = json.loads(rec.FinalResult()).get("text", "")
    a, b = fold(txt), fold(r[0])
    sim = difflib.SequenceMatcher(None, a, b).ratio()
    if sim < 0.75:
        bad.append((r[0], r[1], txt, round(sim, 2), r[4]))
json.dump(bad, open(sys.argv[4], "w"), ensure_ascii=False, indent=0)
print(len(rows), "checked,", len(bad), "suspicious")
