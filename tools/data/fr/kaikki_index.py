"""Index the French part of English Wiktionary (kaikki.org JSONL, CC BY-SA) for the content checks.

Usage: python3 -I kaikki_index.py kaikki.org-dictionary-French.jsonl OUTDIR

Writes, in OUTDIR:
  words.tsv   word  pos  gender(m/f/mf/-)  glosses(first three, " | ")  ipa  form_of
  verbs.tsv   lemma  tense  person  forms(comma)   (simple tenses + past participle + auxiliary)
"""
import json, sys, os

src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
POS = {"noun": "n", "verb": "v", "adj": "adj", "adv": "adv", "prep": "prep", "conj": "conj", "pron": "pron",
       "det": "det", "article": "det", "num": "num", "intj": "interj", "phrase": "expr", "prep_phrase": "expr",
       "adv_phrase": "expr", "proverb": "expr", "name": "name", "suffix": "-", "prefix": "-", "particle": "adv",
       "contraction": "det", "character": "-", "symbol": "-"}
TENSES = {("present", "indicative"): "present", ("imperfect", "indicative"): "imparfait",
          ("future", "indicative"): "futur", ("conditional",): "conditionnel", ("present", "subjunctive"): "subjonctif"}
PERS = {("first-person", "singular"): "je", ("second-person", "singular"): "tu", ("third-person", "singular"): "il",
        ("first-person", "plural"): "nous", ("second-person", "plural"): "vous", ("third-person", "plural"): "ils"}

w = open(os.path.join(out, "words.tsv"), "w")
v = open(os.path.join(out, "verbs.tsv"), "w")
w.write("word\tpos\tgender\tglosses\tipa\tform_of\n")
v.write("lemma\ttense\tperson\tforms\n")
for line in open(src, encoding="utf-8"):
    try:
        d = json.loads(line)
    except Exception:
        continue
    if d.get("lang_code") != "fr":
        continue
    word, pos = d.get("word", ""), POS.get(d.get("pos", ""), "-")
    gender = "-"
    tags = set()
    for ht in d.get("head_templates", []) or []:
        a = ht.get("args", {})
        g = a.get("1") or a.get("g") or ""
        if g in ("m", "f", "mf", "m-p", "f-p", "mfbysense", "m or f"):
            gender = {"m-p": "m", "f-p": "f", "mfbysense": "mf", "m or f": "mf"}.get(g, g)
    for s in d.get("senses", []) or []:
        tags.update(s.get("tags", []) or [])
    if gender == "-" and pos == "n":
        if "masculine" in tags and "feminine" in tags:
            gender = "mf"
        elif "masculine" in tags:
            gender = "m"
        elif "feminine" in tags:
            gender = "f"
    glosses, form_of = [], ""
    for s in d.get("senses", []) or []:
        for g in s.get("glosses", []) or []:
            if g not in glosses:
                glosses.append(g)
        for fo in s.get("form_of", []) or []:
            form_of = form_of or fo.get("word", "")
        for fo in s.get("alt_of", []) or []:
            form_of = form_of or fo.get("word", "")
    ipa = ""
    for snd in d.get("sounds", []) or []:
        if snd.get("ipa"):
            ipa = snd["ipa"]
            break
    gl = " | ".join(x.replace("\t", " ").replace("\n", " ") for x in glosses[:3])
    w.write(f"{word}\t{pos}\t{gender}\t{gl}\t{ipa}\t{form_of}\n")
    if pos == "v" and not form_of:
        slots = {}
        for f in d.get("forms", []) or []:
            t = set(f.get("tags", []) or [])
            form = f.get("form", "")
            if not form or "multiword-construction" in t:
                if "infinitive" in t and "multiword-construction" in t:
                    aux = form.split(" ")[0]
                    if aux in ("avoir", "être"):
                        slots.setdefault(("aux", "-"), []).append(aux)
                continue
            if t == {"participle", "past"}:
                slots.setdefault(("pp", "-"), []).append(form)
                continue
            for tk, tn in TENSES.items():
                if set(tk) <= t and not ("imperfect" in t and tn == "present") and "historic" not in t and "imperative" not in t:
                    if tn == "present" and ("imperfect" in t or "subjunctive" in t):
                        continue
                    if tn == "subjonctif" and "imperfect" in t:
                        continue
                    for pk, pn in PERS.items():
                        if set(pk) <= t:
                            slots.setdefault((tn, pn), [])
                            if form not in slots[(tn, pn)]:
                                slots[(tn, pn)].append(form)
        for (tn, pn), fs in slots.items():
            v.write(f"{word}\t{tn}\t{pn}\t{','.join(dict.fromkeys(fs))}\n")
