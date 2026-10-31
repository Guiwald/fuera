"""Builds compact indexes from the kaikki.org Spanish extract (Wiktionary, CC BY-SA).

Usage: python3 -I kaikki_index.py kaikki-es.jsonl outdir
Writes:
  words.tsv   word, pos, gender (m/f/mf/-), first English glosses (; separated)
  forms.tsv   lemma, form, tags (comma separated) for verbs (conjugation check)
"""
import json, sys, os

src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
words = open(os.path.join(out, "words.tsv"), "w", encoding="utf-8")
forms = open(os.path.join(out, "forms.tsv"), "w", encoding="utf-8")
words.write("word\tpos\tgender\tgloss\n")
forms.write("lemma\tform\ttags\n")
n = 0
with open(src, encoding="utf-8") as f:
    for line in f:
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("lang_code") != "es":
            continue
        word = e.get("word", "")
        pos = e.get("pos", "")
        senses = e.get("senses", [])
        # Skip pure form-of entries for the word index.
        glosses = []
        form_of = False
        for s in senses:
            if "form_of" in s or "alt_of" in s:
                form_of = True
            for g in s.get("glosses", [])[:1]:
                glosses.append(g)
        g = "-"
        tags = set()
        for h in e.get("head_templates", []):
            args = h.get("args", {})
            for k in ("1", "g", "g2"):
                v = args.get(k, "")
                if v in ("m", "f", "mf", "m-p", "f-p", "mfbysense", "m or f"):
                    tags.add(v.split("-")[0].replace("mfbysense", "mf").replace("m or f", "mf"))
        for s in senses:
            for t in s.get("tags", []):
                if t == "masculine":
                    tags.add("m")
                if t == "feminine":
                    tags.add("f")
        if pos == "noun":
            if "mf" in tags or ("m" in tags and "f" in tags):
                g = "mf"
            elif "m" in tags:
                g = "m"
            elif "f" in tags:
                g = "f"
        if not form_of or glosses:
            words.write(f"{word}\t{pos}\t{g}\t{'; '.join(x.replace(chr(9),' ') for x in glosses[:3])}\n")
        if pos == "verb" and not form_of:
            for fm in e.get("forms", []):
                t = fm.get("tags", [])
                if "table-tags" in t or "inflection-template" in t:
                    continue
                forms.write(f"{word}\t{fm.get('form','')}\t{','.join(t)}\n")
        n += 1
print("entries", n)
