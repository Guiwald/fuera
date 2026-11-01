"""Checks a French content file of ¡Fuera! (see docs/data/SPEC-fr.md). Run with python3 -I.

Usage: python3 -I check_fr.py FILE [FILE...]

Prints ERROR lines (must be fixed) and FLAG lines (to look at: fix, or explain in the doubts list), then a summary.
Reference data is read from /home/claude/data/fr (see SPEC-fr.md); the conjugator is the game's own
(`fuera-cli conj-fr`).
"""
import os, re, subprocess, sys, unicodedata
from collections import defaultdict

DATA = "/home/claude/data/fr"
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
CLI = os.path.join(ROOT, "target", "release", "fuera-cli")

THEMES = set("""personas familia cuerpo salud comida bebida casa ropa ciudad transporte viaje naturaleza animales clima
tiempo trabajo estudios ocio deporte emociones comunicacion compras dinero tecnologia sociedad cultura abstracto
cantidad colores acciones descripcion gramatica""".split())
POS = set("n v adj adv prep conj pron det num interj expr".split())
BANDS = {"A1", "A2", "B1", "B2"}
COMPAT = {"n": {"n", "noun", "name"}, "v": {"v", "verb"}, "adj": {"adj"}, "adv": {"adv", "expr", "adverb"},
          "prep": {"prep", "expr"}, "conj": {"conj"}, "pron": {"pron", "det"}, "det": {"det", "pron", "article"},
          "num": {"num", "n", "adj", "det"}, "interj": {"interj", "intj", "expr"}, "expr": set()}

errors, flags = [], []


def err(where, msg):
    errors.append(f"ERROR {where}: {msg}")


def flag(where, msg):
    flags.append(f"FLAG  {where}: {msg}")


def load_index():
    en_wikt = defaultdict(list)   # word -> [(pos, gender, glosses)]
    for i, l in enumerate(open(f"{DATA}/idx/words.tsv", encoding="utf-8")):
        if i == 0:
            continue
        c = l.rstrip("\n").split("\t")
        if len(c) >= 6 and not c[5]:
            en_wikt[c[0]].append((c[1], c[2], c[3]))
    fr_wikt = defaultdict(list)   # word -> [(pos, gender, es, en, ipa, h_aspire)]
    for i, l in enumerate(open(f"{DATA}/idx/frwikt.tsv", encoding="utf-8")):
        if i == 0:
            continue
        c = l.rstrip("\n").split("\t")
        if len(c) >= 8:
            fr_wikt[c[0]].append((c[1], c[2], c[3], c[4], c[5], c[6]))
    rev = defaultdict(set)
    for i, l in enumerate(open(f"{DATA}/es_reverse.tsv", encoding="utf-8")):
        if i == 0:
            continue
        c = l.rstrip("\n").split("\t")
        for g in re.split(r"[,;/]", c[0]):
            rev[g.strip().lower()].add(c[1].lower())
    freq = set()
    for l in open(f"{DATA}/fr_50k.txt", encoding="utf-8"):
        p = l.split()
        if p:
            freq.add(p[0])
    return en_wikt, fr_wikt, rev, freq


def norm(s):
    s = s.lower().strip()
    s = re.sub(r"\(.*?\)", "", s)
    s = re.sub(r"^(to |el |la |los |las |un |una |the |a |an )", "", s)
    return s.strip()


def words_of(s):
    return {w for w in re.split(r"[^\wáéíóúüñàâçèêëîïôûùœ'-]+", s.lower()) if len(w) > 2}


def is_single_emoji(s):
    if not s:
        return True
    s2 = s.replace("️", "")
    return len(s2) == 1 and ord(s2) >= 0x2190 and unicodedata.category(s2) == "So"


def spanish_ok(where, es):
    for part in es.split("|"):
        for w in re.findall(r"\w+", part.lower()):
            if re.search(r"(áis|éis)$", w) and w not in ("país", "maíz", "seis", "veis", "dieciséis", "veintiséis"):
                err(where, f"Spanish gloss uses a vosotros form: {part}")
            if w in ("vosotros", "vosotras", "vuestro", "vuestra", "coger", "cojo", "coge"):
                err(where, f"Spanish gloss uses a Spain-only word: {part}")
    if es.count("|") > 1:
        err(where, f"more than one | in the Spanish gloss: {es}")


def check_lexicon(path, rows, header, idx):
    en_wikt, fr_wikt, rev, _ = idx
    want = ["fr", "pos", "gender", "es", "en", "band", "theme", "pic", "note"]
    if header != want:
        err(path, f"header must be {want}")
        return
    seen = set()
    for n, c in rows:
        where = f"{os.path.basename(path)}:{n} {c[0]}"
        if len(c) != 9:
            err(where, f"{len(c)} columns")
            continue
        fr, pos, g, es, en, band, theme, pic, note = c
        if (fr, pos) in seen:
            err(where, "duplicate")
        seen.add((fr, pos))
        if pos not in POS:
            err(where, f"bad pos {pos}")
        if pos == "n" and g not in ("m", "f", "mf"):
            err(where, "noun without gender")
        if pos != "n" and g:
            err(where, "gender on a non-noun")
        if band not in BANDS:
            err(where, f"bad band {band}")
        if theme not in THEMES:
            err(where, f"bad theme {theme}")
        if not is_single_emoji(pic):
            err(where, f"pic is not one emoji: {pic!r}")
        if not es or not en:
            err(where, "missing gloss")
        if pos == "v" and en and not en.startswith("to ") and not en.startswith("(to"):
            flag(where, f"English verb gloss without 'to': {en}")
        spanish_ok(where, es)
        if len(es) > 28 or len(en) > 28:
            flag(where, "long gloss")
        if " " in fr and pos not in ("expr", "v", "adv", "prep", "conj", "n", "interj", "num", "det", "pron"):
            flag(where, "multi-word lemma")
        # Existence and gender in the two Wiktionaries.
        look = fr
        if pos == "v":
            look = re.sub(r"^(se |s')", "", fr)
        k_en = [e for e in en_wikt.get(look, []) if not COMPAT[pos] or e[0] in COMPAT[pos]]
        k_fr = [e for e in fr_wikt.get(look, []) if not COMPAT[pos] or e[0] in COMPAT[pos]]
        if pos != "expr" and " " not in look and not k_en and not k_fr:
            flag(where, f"not found as {pos} in either Wiktionary")
        if pos == "n" and (k_en or k_fr):
            gs = {e[1] for e in k_en + k_fr if e[1] in ("m", "f", "mf")}
            if gs and g not in gs and not (g == "mf" and {"m", "f"} <= gs):
                flag(where, f"gender {g}, Wiktionary says {'/'.join(sorted(gs))}")
        # h aspiré.
        if fr[:1] == "h" and pos in ("n", "adj"):
            ents = [e for e in fr_wikt.get(fr, []) if e[0] in COMPAT[pos]]
            ha = {e[5] for e in ents}
            if ha == {"1"} and "h aspiré" not in note:
                err(where, "Wiktionary says h aspiré: add the note")
            elif "1" in ha and "h aspiré" not in note:
                flag(where, "one Wiktionary entry of this spelling has h aspiré: check")
            if "h aspiré" in note and ha and "1" not in ha:
                flag(where, "note says h aspiré, French Wiktionary does not")
        # Spanish gloss against the French Wiktionary's translations and the game's Spanish lexicon.
        es_known = set()
        for e in k_fr:
            es_known |= {norm(x) for x in e[2].split("|") if x}
        es_known |= rev.get(fr.lower(), set())
        parts = {norm(x) for x in re.split(r"[|,]", es)}
        if es_known and not parts & es_known:
            flag(where, f"Spanish '{es}' not among Wiktionary/lexicon translations ({', '.join(sorted(es_known)[:6])})")
        # English gloss against both Wiktionaries.
        en_known = set()
        for e in k_fr:
            en_known |= {norm(x) for x in e[3].split("|") if x}
        en_words = set()
        for e in k_en:
            en_words |= words_of(e[2])
        ep = {norm(x) for x in re.split(r"[,;]", en)}
        if (en_known or en_words) and not (ep & en_known) and not any(words_of(x) & en_words for x in ep):
            flag(where, f"English '{en}' not found in Wiktionary glosses")


def conj_table(infs):
    out = subprocess.run([CLI, "conj-fr"], input="\n".join(infs), capture_output=True, text=True).stdout
    t = defaultdict(lambda: defaultdict(set))
    for l in out.splitlines():
        inf, tense, person, form = l.split("\t")
        t[inf][tense].add(form)
    return t


def pc_variants(form):
    """`suis allé(e)` → {suis allé, suis allée}; `sommes allé(e)s` → {sommes allés, sommes allées}."""
    if "(e" not in form:
        return {form}
    if "(es)" in form:
        b = form.replace("(e)(es)", "").replace("(es)", "")
        return {b, b + "e", b + "es"}
    base = form.replace("(e)(s)", "\x01").replace("(e)s", "\x02").replace("(e)", "\x03")
    out = set()
    for e in ("", "e"):
        for s_ in ("", "s"):
            out.add(base.replace("\x01", e + s_).replace("\x02", e + "s").replace("\x03", e))
    return out


def check_sentences(path, rows, header, kind):
    want = {"passe": ["sentence", "answer", "other", "es", "en", "band", "why"],
            "subjonctif": ["sentence", "answer", "other", "es", "en", "band", "why"],
            "partitifs": ["sentence", "answer", "other", "es", "en", "band", "why"],
            "etre_avoir": ["sentence", "answer", "form", "es", "en", "band", "why"]}[kind]
    if header != want:
        err(path, f"header must be {want}")
        return
    infs = set()
    for n, c in rows:
        m = re.search(r"\(([^()]+)\)\s*$", c[0]) if c else None
        if m:
            infs.add(m.group(1).strip())
    table = conj_table(sorted(infs)) if infs else {}
    seen = set()
    for n, c in rows:
        where = f"{os.path.basename(path)}:{n} {c[0][:40]}"
        if len(c) != 7:
            err(where, f"{len(c)} columns")
            continue
        sent, ans, other, es, en, band, why = c
        if sent in seen:
            err(where, "duplicate sentence")
        seen.add(sent)
        if sent.count("___") != 1:
            err(where, "the sentence needs exactly one ___")
        if band not in BANDS:
            err(where, f"bad band {band}")
        if not es or not en or not why:
            err(where, "missing translation or why")
        spanish_ok(where, es)
        if len(sent) > 70:
            flag(where, "long sentence")
        if kind in ("passe", "subjonctif"):
            m = re.search(r"\(([^()]+)\)\s*$", sent)
            if not m:
                err(where, "no (infinitive) at the end")
                continue
            inf = m.group(1).strip()
            t = table.get(inf)
            if not t:
                err(where, f"the conjugator does not know {inf}")
                continue
            if kind == "passe":
                pc = set().union(*(pc_variants(f) for f in t["passé composé"]))
                imp = t["imparfait"]
                if ans not in pc | imp:
                    err(where, f"answer {ans!r} is not a passé composé or imparfait form of {inf}")
                if other not in pc | imp:
                    err(where, f"other {other!r} is not a passé composé or imparfait form of {inf}")
                if (ans in pc) == (other in pc):
                    err(where, "answer and other must be one passé composé, one imparfait")
            else:
                sub = t["subjonctif présent"]
                ind = t["présent"] | t["futur simple"] | t["imparfait"] | set().union(*(pc_variants(f) for f in t["passé composé"]))
                if ans not in sub | ind or other not in sub | ind:
                    err(where, f"{ans!r} / {other!r}: not forms of {inf}")
                elif (ans in sub) == (other in sub) and ans not in ind:
                    err(where, "answer and other must be one subjunctive, one indicative")
                if ans in sub and ans in ind:
                    flag(where, f"{ans} is both subjunctive and indicative: the sentence must decide (check)")
        elif kind == "etre_avoir":
            if ans not in ("être", "avoir"):
                err(where, "answer must be être or avoir")
            etre = {"suis", "es", "est", "sommes", "êtes", "sont", "t'es", "s'est", "s'es", "me suis", "nous sommes", "vous êtes", "se sont"}
            avoir = {"ai", "as", "a", "avons", "avez", "ont", "t'as", "m'ai"}
            if ans == "être" and other not in etre:
                err(where, f"form {other!r} is not a present of être")
            if ans == "avoir" and other not in avoir:
                err(where, f"form {other!r} is not a present of avoir")
        elif kind == "partitifs":
            ok = {"du", "de la", "de l'", "des", "de", "d'", "le", "la", "l'", "les", "un", "une", "Ø"}
            if ans not in ok or other not in ok:
                err(where, f"answer/other must be among {sorted(ok)}")
            if ans == other:
                err(where, "answer and other are the same")


def check_genre(path, rows, header, idx):
    want = ["fr", "gender", "es", "en", "band", "note"]
    if header != want:
        err(path, f"header must be {want}")
        return
    en_wikt, fr_wikt, _, _ = idx
    for n, c in rows:
        where = f"{os.path.basename(path)}:{n} {c[0]}"
        if len(c) != 6:
            err(where, f"{len(c)} columns")
            continue
        fr, g, es, en, band, note = c
        if g not in ("m", "f"):
            err(where, "gender must be m or f")
        spanish_ok(where, es)
        gs = {e[1] for e in en_wikt.get(fr, []) + fr_wikt.get(fr, []) if e[0] in ("n", "noun") and e[1] in ("m", "f", "mf")}
        if gs and g not in gs:
            err(where, f"gender {g}, Wiktionary says {'/'.join(sorted(gs))}")
        if not gs:
            flag(where, "noun not found in Wiktionary")


def check_faux(path, rows, header, idx):
    want = ["fr", "es_true", "es_trap", "en_true", "en_trap", "band", "note"]
    if header != want:
        err(path, f"header must be {want}")
        return
    en_wikt, fr_wikt, _, _ = idx
    for n, c in rows:
        where = f"{os.path.basename(path)}:{n} {c[0]}"
        if len(c) != 7:
            err(where, f"{len(c)} columns")
            continue
        fr, et, ep, nt, np_, band, note = c
        if not (ep or np_):
            err(where, "no trap for either language")
        if ep and not et or np_ and not nt:
            err(where, "a trap without its true meaning")
        if fr not in en_wikt and fr not in fr_wikt:
            flag(where, "not found in Wiktionary")
        spanish_ok(where, et)
        if ep:
            spanish_ok(where, ep)


def check_pseudo(path, rows, idx):
    en_wikt, fr_wikt, _, freq = idx
    seen = set()
    for n, c in rows:
        p = c[0]
        where = f"{os.path.basename(path)}:{n} {p}"
        if p in seen:
            err(where, "duplicate")
        seen.add(p)
        if p in freq or p in en_wikt or p in fr_wikt:
            err(where, "this word exists")
        if not re.fullmatch(r"[a-zàâçéèêëîïôûùœ]+", p):
            err(where, "letters only")


def main():
    idx = load_index()
    for path in sys.argv[1:]:
        lines = [l.rstrip("\n") for l in open(path, encoding="utf-8")]
        header = lines[0].split("\t") if lines else []
        rows = [(i + 1, l.split("\t")) for i, l in enumerate(lines[1:], 1) if l.strip()]
        name = os.path.basename(path)
        if name.startswith("lexicon"):
            check_lexicon(path, rows, header, idx)
        elif name.startswith("genre"):
            check_genre(path, rows, header, idx)
        elif name.startswith("faux_amis"):
            check_faux(path, rows, header, idx)
        elif name.startswith("pseudo"):
            check_pseudo(path, rows, idx)
        else:
            kind = name.split(".")[0]
            check_sentences(path, rows, header, kind)
    for e in errors:
        print(e)
    for f in flags:
        print(f)
    print(f"{len(errors)} errors, {len(flags)} flags")
    sys.exit(1 if errors else 0)


main()
