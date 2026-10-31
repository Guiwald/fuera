#!/usr/bin/env python3
"""Rebuild tests/data/forms.tsv (the conjugation fixture) from kaikki.org data.

Usage:
    python3 -I build_forms.py FORMS_TSV ES_50K_TXT KAIKKI_ES_JSONL > forms.tsv

FORMS_TSV        kaikki-derived table "lemma<TAB>form<TAB>tags" (comma-separated tags)
ES_50K_TXT       frequency list "word count" (one per line, most frequent first)
KAIKKI_ES_JSONL  full kaikki.org Spanish dump, only used for the few lemmas
                 absent from FORMS_TSV (e.g. "hacer")

Data: Wiktionary via kaikki.org, CC BY-SA 4.0 (see README.md).
"""
import json
import re
import sys

TOP_N = 400

# Frequent infinitive strings whose frequency comes from a noun/adjective homograph.
NOT_REALLY_VERBS = {"placer", "militar", "nuclear", "solar", "escolar", "regular", "lugar"}

# Irregular / feature-coverage verbs always included (besides the frequency top N).
EXTRA = """
ser estar ir haber dar ver saber caber oír tener salir poder hacer decir querer poner
venir valer traer caer conducir andar deshacer rehacer componer proponer suponer exponer
imponer oponer disponer mantener obtener contener detener retener sostener entretener
prevenir convenir intervenir provenir predecir bendecir maldecir contradecir satisfacer
equivaler sobresalir atraer distraer contraer extraer decaer recaer desoír prever
jugar adquirir reír sonreír freír enviar continuar actuar graduar evaluar situar guiar
fiar criar vaciar ampliar esquiar reunir prohibir aislar rehusar
leer creer poseer proveer construir destruir huir incluir concluir contribuir distribuir
sustituir influir disminuir argüir
conocer parecer nacer ofrecer producir traducir introducir reducir deducir lucir vencer
torcer cocer ejercer esparcir zurcir
seguir conseguir perseguir distinguir extinguir delinquir coger recoger escoger dirigir
exigir elegir corregir fingir proteger averiguar apaciguar
buscar pagar empezar comenzar almorzar cruzar llegar sacar tocar colgar rogar forzar
pensar cerrar despertar sentar entender perder querer encender defender
contar mostrar volar soñar recordar acostar costar probar mover volver devolver resolver
morder doler llover oler errar
pedir servir repetir vestir medir competir impedir despedir
sentir mentir preferir sugerir convertir divertir advertir herir hervir
dormir morir reñir teñir ceñir gruñir bullir zambullir tañer
quejarse resentirse desentenderse inmiscuirse entrometerse despreocuparse dignarse
desvivirse escabullirse regodearse autoimponerse
""".split()

PERSONS = [
    ("first-person", "singular", "Yo"),
    ("second-person", "singular", "Tu"),
    ("third-person", "singular", "El"),
    ("first-person", "plural", "Nosotros"),
    ("second-person", "plural", "Vosotros"),
    ("third-person", "plural", "Ellos"),
]

EXCLUDE_TAGS = {
    "vos-form", "formal", "imperfect-se", "combined-form", "imperative", "negative",
    "with-vos", "with-tú", "second-person-semantically",
    "archaic", "obsolete", "dialectal", "nonstandard", "misspelling", "proscribed",
    "pronunciation-spelling", "colloquial", "vulgar", "hypercorrect", "euphemistic",
    "error-unrecognized-form", "error-unknown-tag",
    "Latin-America", "Uruguay", "Argentina", "Chile", "Venezuela", "Cuba",
    "Central-America", "Puerto-Rico", "Peru", "Paraguay", "Nicaragua", "Mexico",
    "Honduras", "Ecuador", "Costa-Rica",
}

CLITICS = ("me ", "te ", "se ", "nos ", "os ")


def tense_of(tags):
    if "indicative" in tags:
        for t, name in (("present", "Present"), ("preterite", "Preterite"),
                        ("imperfect", "Imperfect"), ("future", "Future"),
                        ("conditional", "Conditional")):
            if t in tags:
                return name
        return None
    if "conditional" in tags:
        return "Conditional"
    if "subjunctive" in tags:
        if "future" in tags:
            return None
        if "present" in tags:
            return "PresentSubjunctive"
        if "imperfect" in tags:
            return "ImperfectSubjunctive"
    return None


def person_of(tags):
    for p, n, name in PERSONS:
        if p in tags and n in tags:
            return name
    return None


def row_of(lemma, form, tags):
    tags = set(tags)
    if tags & EXCLUDE_TAGS:
        return None
    tense = tense_of(tags)
    person = person_of(tags)
    if tense is None or person is None:
        return None
    form = form.strip()
    if not form or form == "-" or "/" in form or "(" in form:
        return None
    pronominal = lemma.endswith("se") and lemma[:-2].endswith(("ar", "er", "ir", "ír"))
    if " " in form and not pronominal:
        return None  # "me hablo"-style pronominal duplicate of a non-pronominal verb
    return (lemma, tense, person, form)


def main():
    forms_tsv, freq_txt, jsonl = sys.argv[1:4]
    verbs = set()
    with open(forms_tsv, encoding="utf-8") as f:
        next(f)
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) == 3 and p[2] == "first-person,indicative,present,singular":
                verbs.add(p[0])

    pat = re.compile(r"^[a-zñáéíóúü]+(ar|er|ir|ír)(se)?$")
    chosen = []
    with open(freq_txt, encoding="utf-8") as f:
        for line in f:
            w = line.split()[0]
            if pat.match(w) and w in verbs and w not in NOT_REALLY_VERBS and w not in chosen:
                chosen.append(w)
                if len(chosen) == TOP_N:
                    break
    wanted = set(chosen) | set(EXTRA)

    rows = set()
    found = set()
    with open(forms_tsv, encoding="utf-8") as f:
        next(f)
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) != 3 or p[0] not in wanted:
                continue
            r = row_of(p[0], p[1], p[2].split(","))
            if r:
                rows.add(r)
                found.add(p[0])

    missing = wanted - found
    if missing:
        needle = re.compile(r'"word": "(%s)"' % "|".join(re.escape(m) for m in missing))
        with open(jsonl, encoding="utf-8") as f:
            for line in f:
                if '"pos": "verb"' not in line or not needle.search(line):
                    continue
                d = json.loads(line)
                if d.get("pos") != "verb" or d.get("word") not in missing:
                    continue
                for fm in d.get("forms", []):
                    if fm.get("source") != "conjugation":
                        continue
                    r = row_of(d["word"], fm.get("form", ""), fm.get("tags", []))
                    if r:
                        rows.add(r)
                        found.add(d["word"])

    still = sorted(wanted - found)
    if still:
        print("warning: no forms for " + " ".join(still), file=sys.stderr)

    order = {"Present": 0, "Preterite": 1, "Imperfect": 2, "Future": 3, "Conditional": 4,
             "PresentSubjunctive": 5, "ImperfectSubjunctive": 6}
    porder = {name: i for i, (_, _, name) in enumerate(PERSONS)}
    out = sys.stdout
    out.write("# Spanish verb forms from Wiktionary via kaikki.org (CC BY-SA 4.0); see README.md\n")
    out.write("infinitive\ttense\tperson\tform\n")
    for r in sorted(rows, key=lambda r: (r[0], order[r[1]], porder[r[2]], r[3])):
        out.write("\t".join(r) + "\n")
    print(f"{len(found)} verbs, {len(rows)} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
