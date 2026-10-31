"""Checker for assets/es/lexicon_*.tsv (SPEC-es.md section 1).

Usage: python3 -I check_lexicon.py LEXICON_TSV [KAIKKI_WORDS_TSV]
Exit code 1 if any ERROR. WARN lines are informational (to justify or fix).
"""
import sys, unicodedata, collections

path = sys.argv[1]
kaikki_path = sys.argv[2] if len(sys.argv) > 2 else "/home/claude/data/kaikki/words.tsv"

HEADER = ["es", "pos", "gender", "fr", "en", "band", "theme", "pic", "note"]
POS = set("n v adj adv prep conj pron det num interj expr".split())
BANDS = {"A1", "A2", "B1", "B2"}
THEMES = set("""personas familia cuerpo salud comida bebida casa ropa ciudad transporte viaje naturaleza
animales clima tiempo trabajo estudios ocio deporte emociones comunicacion compras dinero tecnologia
sociedad cultura abstracto cantidad colores acciones descripcion gramatica""".split())
COMPAT = {"n": {"noun"}, "v": {"verb"}, "adj": {"adj"}, "adv": {"adv"}, "prep": {"prep"},
          "conj": {"conj"}, "pron": {"pron"}, "det": {"det", "article"}, "num": {"num"},
          "interj": {"intj"}}

# Reviewed deviations from kaikki (key: (es, pos)) -> reason. Reported as JUSTIFIED, not ERROR.
JUSTIFIED = {
    ("conmigo", "pron"): "RAE: pronombre personal (kaikki tags it adv)",
    ("contigo", "pron"): "RAE: pronombre personal (kaikki tags it adv)",
    ("médico", "n"): "standard m; feminine form is médica (kaikki mf = 'la médico')",
    ("presidente", "n"): "standard m; feminine form is presidenta (noted)",
    ("calor", "n"): "m in standard Spanish; 'la calor' is regional/archaic",
    ("fin", "n"): "m in standard Spanish; 'la fin' is archaic/poetic",
    ("Navidad", "n"): "common noun f per RAE, capitalised as the feast; kaikki files it as a name",
}
errors, warns, justified = [], [], []


def err(ln, msg, key=None):
    if key in JUSTIFIED:
        justified.append(f"JUSTIFIED line {ln}: {msg} -- {JUSTIFIED[key]}")
        return
    errors.append(f"ERROR line {ln}: {msg}")


def warn(ln, msg):
    warns.append(f"WARN  line {ln}: {msg}")


lines = open(path, encoding="utf-8").read().split("\n")
if lines and lines[-1] == "":
    lines.pop()
if not lines or lines[0].split("\t") != HEADER:
    err(1, f"bad header: {lines[0] if lines else '(empty)'}")
rows = []
for i, line in enumerate(lines[1:], start=2):
    cols = line.split("\t")
    if len(cols) != len(HEADER):
        err(i, f"{len(cols)} columns (expected {len(HEADER)}): {line!r}")
        continue
    if any(c != c.strip() for c in cols):
        err(i, "leading/trailing whitespace in a field")
    if any('"' in c for c in cols):
        err(i, "double quote in a field")
    rows.append((i, dict(zip(HEADER, cols))))

# kaikki index: word -> list of (pos, gender)
need = {r["es"] for _, r in rows}
kaikki = collections.defaultdict(list)
with open(kaikki_path, encoding="utf-8") as f:
    for line in f:
        p = line.rstrip("\n").split("\t")
        if len(p) >= 3 and p[0] in need:
            kaikki[p[0]].append((p[1], p[2]))

seen = {}
for ln, r in rows:
    es, pos, g = r["es"], r["pos"], r["gender"]
    if not es or not r["fr"] or not r["en"]:
        err(ln, f"empty es/fr/en: {es!r}")
    if any(unicodedata.normalize("NFC", v) != v for v in r.values()):
        err(ln, f"{es}: field not in Unicode NFC")
    if pos not in POS:
        err(ln, f"{es}: bad pos {pos!r}")
    if r["band"] not in BANDS:
        err(ln, f"{es}: bad band {r['band']!r}")
    if r["theme"] not in THEMES:
        err(ln, f"{es}: bad theme {r['theme']!r}")
    key = (es, pos)
    if key in seen:
        err(ln, f"duplicate {es} ({pos}), first at line {seen[key]}")
    seen[key] = ln
    # gender column
    if pos == "n":
        if g not in ("m", "f", "mf"):
            err(ln, f"{es}: noun needs gender m/f/mf, got {g!r}")
    elif g:
        err(ln, f"{es}: gender {g!r} on non-noun")
    # English verbs with "to"
    if pos == "v" and not r["en"].startswith("to "):
        err(ln, f"{es}: verb gloss must start with 'to ': {r['en']!r}")
    for lang in ("fr", "en"):
        nw = len(r[lang].replace(",", " ").split())
        if nw > 4:
            warn(ln, f"{es}: long {lang} gloss ({nw} words): {r[lang]!r}")
    # kaikki existence / pos compatibility
    entries = kaikki.get(es, [])
    if pos != "expr":
        if not entries:
            err(ln, f"{es} ({pos}): not in kaikki", key)
        elif not any(kp in COMPAT[pos] for kp, _ in entries):
            err(ln, f"{es} ({pos}): kaikki has only {sorted({kp for kp, _ in entries})}", key)
    # noun gender vs kaikki
    if pos == "n" and entries:
        kg = {kgen for kp, kgen in entries if kp == "noun"}
        known = kg - {"-"}
        if g in kg:
            pass
        elif not known:
            warn(ln, f"{es}: kaikki gender unknown ('-'), we say {g}")
        else:
            err(ln, f"{es}: gender {g} but kaikki says {sorted(known)}", key)
    # pic
    pic = r["pic"]
    if pic:
        base = pic[:-1] if pic.endswith("️") else pic
        if len(base) != 1:
            err(ln, f"{es}: pic must be one code point (+FE0F), got {[hex(ord(c)) for c in pic]}")
        else:
            cp = ord(base)
            if 0x1F1E6 <= cp <= 0x1F1FF or 0x1F3FB <= cp <= 0x1F3FF or cp == 0x200D:
                err(ln, f"{es}: flag/skin-tone/ZWJ in pic")
            elif unicodedata.category(base) != "So":
                err(ln, f"{es}: pic {pic!r} is not a symbol (category {unicodedata.category(base)})")
            else:
                try:
                    unicodedata.name(base)
                except ValueError:
                    warn(ln, f"{es}: pic U+{cp:04X} unknown to this Python's Unicode DB")
    if r["note"] and len(r["note"]) > 60:
        warn(ln, f"{es}: long note ({len(r['note'])} chars)")

# order: bands grouped and in order
bands = [r["band"] for _, r in rows]
order = {"A1": 0, "A2": 1, "B1": 2, "B2": 3}
if any(order[a] > order[b] for a, b in zip(bands, bands[1:]) if a in order and b in order):
    errors.append("ERROR: rows are not grouped by band (A1 then A2 …)")

cnt = collections.Counter(bands)
print(f"rows={len(rows)} " + " ".join(f"{b}={cnt[b]}" for b in sorted(cnt)))
print(f"pos: {dict(collections.Counter(r['pos'] for _, r in rows))}")
print(f"with pic: {sum(1 for _, r in rows if r['pic'])}, with note: {sum(1 for _, r in rows if r['note'])}")
for m in errors + justified + warns:
    print(m)
print(f"{len(errors)} error(s), {len(justified)} justified, {len(warns)} warning(s)")
sys.exit(1 if errors else 0)
