"""Index French entries of the French Wiktionary (kaikki.org raw wiktextract data, CC BY-SA) for the content checks.

Usage: zcat raw-wiktextract-data.jsonl.gz | python3 -I frwikt_index.py > frwikt.tsv

Columns: word  pos  gender  es(translations, |)  en(translations, |)  ipa  h_aspire(1/0)  homophones(1/0)
"""
import json, sys
POS = {"noun": "n", "verb": "v", "adj": "adj", "adv": "adv", "prep": "prep", "conj": "conj", "pron": "pron",
       "det": "det", "article": "det", "num": "num", "intj": "interj", "phrase": "expr", "name": "name"}
print("word\tpos\tgender\tes\ten\tipa\th_aspire\thomophones")
for line in sys.stdin:
    if '"lang_code": "fr"' not in line[:200] and '"lang_code":"fr"' not in line[:200]:
        # cheap prefilter; fall back to parsing when unsure
        if '"lang_code": "fr"' not in line:
            continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    if d.get("lang_code") != "fr":
        continue
    word, pos = d.get("word", ""), POS.get(d.get("pos", ""), d.get("pos", "-"))
    tags = set(d.get("tags", []) or [])
    for s in d.get("senses", []) or []:
        pass
    g = "mf" if {"masculine", "feminine"} <= tags else "m" if "masculine" in tags else "f" if "feminine" in tags else "-"
    es, en = [], []
    for t in d.get("translations", []) or []:
        w = (t.get("word") or "").strip()
        if not w:
            continue
        if t.get("lang_code") == "es" and w not in es:
            es.append(w)
        elif t.get("lang_code") == "en" and w not in en:
            en.append(w)
    ipa = ""
    for s in d.get("sounds", []) or []:
        i = s.get("ipa", "")
        if i.startswith("\\") and i.endswith("\\") and not ipa:
            ipa = i.strip("\\")
    cats = " ".join(d.get("categories", []) or [])
    ha = "1" if "h aspiré" in cats else "0"
    ho = "1" if "homophones" in cats else "0"
    clean = lambda xs: "|".join(x.replace("\t", " ") for x in xs[:12])
    print(f"{word}\t{pos}\t{g}\t{clean(es)}\t{clean(en)}\t{ipa}\t{ha}\t{ho}")
