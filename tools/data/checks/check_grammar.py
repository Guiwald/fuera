#!/usr/bin/env python3
"""Checker for assets/es/grammar/*.tsv (spec: docs/data/SPEC-es.md section 2).

Run:  python3 -I /tmp/claude-0/checks/check_grammar.py
Exit code 0 = no failures (warnings are printed for human review).
"""
import collections
import re
import sys
import unicodedata

ROOT = sys.argv[1] if len(sys.argv) > 1 else '/home/claude/fuera/assets/es/grammar'
DATA = '/home/claude/data'

HEADERS = {
    'ser_estar.tsv': ['phrase', 'answer', 'fr', 'en', 'band', 'why'],
    'por_para.tsv': ['phrase', 'answer', 'fr', 'en', 'band', 'why'],
    'pret_imp.tsv': ['sentence', 'answer', 'other', 'fr', 'en', 'band', 'why'],
    'subjuntivo.tsv': ['sentence', 'answer', 'other', 'fr', 'en', 'band', 'why'],
    'falsos_amigos.tsv': ['es', 'fr_true', 'fr_trap', 'en_true', 'en_trap', 'band', 'note'],
    'genero.tsv': ['es', 'gender', 'fr', 'en', 'band', 'note'],
    'pseudo.tsv': ['pseudo'],
}
TARGET = {'ser_estar.tsv': 90, 'por_para.tsv': 90, 'pret_imp.tsv': 90, 'subjuntivo.tsv': 110,
          'falsos_amigos.tsv': 70, 'genero.tsv': 70, 'pseudo.tsv': 200}
OPTIONAL = {'fr_trap', 'en_trap', 'note'}
GAP_COL = {'por_para.tsv': 'phrase', 'pret_imp.tsv': 'sentence', 'subjuntivo.tsv': 'sentence'}
BANDS = {'A2', 'B1', 'B2'}

fails = []
warns = []


def fail(f, n, msg):
    fails.append(f'{f}:{n}: {msg}')


def warn(f, n, msg):
    warns.append(f'{f}:{n}: {msg}')


def strip_acc(s):
    return ''.join(c for c in unicodedata.normalize('NFD', s) if unicodedata.category(c) != 'Mn')


def phon(w):
    """Rough Spanish sound key (b=v, silent h, seseo, yeismo, ge=je) to catch homophones of real words."""
    w = strip_acc(w.lower()).replace('h', '')
    w = re.sub(r'qu(?=[ei])', 'k', w)
    w = re.sub(r'gu(?=[ei])', 'g', w)
    w = re.sub(r'g(?=[ei])', 'j', w)
    w = re.sub(r'c(?=[ei])', 's', w)
    w = w.replace('c', 'k').replace('z', 's').replace('v', 'b').replace('ll', 'y').replace('x', 'ks')
    return re.sub(r'y(?![aeiou])', 'i', w)


CODA_OK = {'c': 'ctn', 'p': 'tcs', 'b': 'stjv', 'g': 'nm', 'd': 'mv', 't': 'ml', 'm': 'bpn'}
ONSET_CL = {'bl', 'br', 'cl', 'cr', 'dr', 'fl', 'fr', 'gl', 'gr', 'pl', 'pr', 'tr'}


def cluster_ok(c):
    """c = a word-internal run of consonant letters; True if it is a normal Spanish coda+onset."""
    c = c.replace('ch', 'C').replace('ll', 'L').replace('rr', 'R').replace('qu', 'Q')
    if len(c) == 1:
        return True
    if len(c) == 3:
        return c[0] in 'nlrsb' and c[1:] in ONSET_CL or c in ('nst', 'bst', 'nsp', 'rst')
    if len(c) != 2:
        return False
    a, b = c
    if c in ONSET_CL:
        return True
    if b in 'CLRQ':
        return a in 'nlrs' and not (a in 'nls' and b == 'R')
    if a in 'nlr':
        return not (a == 'n' and b in 'bp')
    if a == 's':
        return b != 'r'
    return b in CODA_OK.get(a, '')


# ---------------------------------------------------------------- loading
def load_tsv(name):
    path = f'{ROOT}/{name}'
    try:
        raw = open(path, 'rb').read()
    except OSError as e:
        fail(name, 0, f'cannot open: {e}')
        return []
    if raw.startswith(b'\xef\xbb\xbf'):
        fail(name, 0, 'BOM present')
    if b'\r' in raw:
        fail(name, 0, 'CR characters present')
    try:
        text = raw.decode('utf-8')
    except UnicodeDecodeError as e:
        fail(name, 0, f'not UTF-8: {e}')
        return []
    if not text.endswith('\n'):
        fail(name, 0, 'no trailing newline')
    if unicodedata.normalize('NFC', text) != text:
        fail(name, 0, 'text is not NFC-normalised')
    lines = text.split('\n')
    if lines and lines[-1] == '':
        lines.pop()
    hdr = HEADERS[name]
    if not lines or lines[0].split('\t') != hdr:
        fail(name, 1, f'header must be {hdr!r}, got {lines[0] if lines else None!r}')
        return []
    rows = []
    for i, line in enumerate(lines[1:], start=2):
        if line.strip() == '':
            fail(name, i, 'blank line')
            continue
        cols = line.split('\t')
        if len(cols) != len(hdr):
            fail(name, i, f'{len(cols)} columns, expected {len(hdr)}')
            continue
        row = dict(zip(hdr, cols))
        row['_n'] = i
        for k in hdr:
            v = row[k]
            if v != v.strip() or '  ' in v:
                fail(name, i, f'stray whitespace in {k}: {v!r}')
            if v == '' and k not in OPTIONAL:
                fail(name, i, f'empty {k}')
            if '"' in v:
                fail(name, i, f'double quote in {k} (spec: no quoting)')
        rows.append(row)
    if len(rows) != TARGET[name]:
        fail(name, 0, f'{len(rows)} rows, target {TARGET[name]}')
    # duplicates: whole row and key column (normalised)
    seen_row, seen_key = {}, {}
    key = hdr[0]
    for r in rows:
        whole = tuple(r[k] for k in hdr)
        if whole in seen_row:
            fail(name, r['_n'], f'duplicate row (first at line {seen_row[whole]})')
        seen_row.setdefault(whole, r['_n'])
        nk = re.sub(r'[^\w ]', '', r[key].lower()).strip()
        if nk in seen_key:
            fail(name, r['_n'], f'duplicate {key} {r[key]!r} (first at line {seen_key[nk]})')
        seen_key.setdefault(nk, r['_n'])
    # bands
    if 'band' in hdr:
        for r in rows:
            if r['band'] not in BANDS:
                fail(name, r['_n'], f'band {r["band"]!r} not in {sorted(BANDS)}')
    # gaps: exactly one "___" in the gap column, no underscores anywhere else
    gcol = GAP_COL.get(name)
    for r in rows:
        for k in hdr:
            runs = re.findall(r'_+', r[k])
            if k == gcol:
                if runs != ['___']:
                    fail(name, r['_n'], f'{k} must contain exactly one "___" gap, found {runs}')
            elif runs:
                fail(name, r['_n'], f'underscore in {k}: {r[k]!r}')
    # why: short rule (1-5 words)
    if 'why' in hdr:
        for r in rows:
            nw = len(r['why'].split())
            if not 1 <= nw <= 5:
                fail(name, r['_n'], f'why has {nw} words (want 1-5): {r["why"]!r}')
    return rows


print('loading reference data ...', file=sys.stderr)
words = collections.defaultdict(list)          # word -> [(pos, gender)]
for line in open(f'{DATA}/kaikki/words.tsv', encoding='utf-8'):
    p = line.rstrip('\n').split('\t')
    if len(p) >= 3 and p[0] != 'word':
        words[p[0]].append((p[1], p[2]))
freq = set()
for line in open(f'{DATA}/es_50k.txt', encoding='utf-8'):
    p = line.split()
    if p:
        freq.add(p[0].lower())

# ------------------------------------------------------------------ files
data = {n: load_tsv(n) for n in HEADERS}

needed_lemmas = set()
for n in ('pret_imp.tsv', 'subjuntivo.tsv'):
    for r in data[n]:
        m = re.search(r'\(([^()]+)\)$', r['sentence'])
        if m:
            needed_lemmas.add(m.group(1))

forms = collections.defaultdict(lambda: collections.defaultdict(list))   # lemma -> form -> [tagset]
all_forms = set()
for line in open(f'{DATA}/kaikki/forms.tsv', encoding='utf-8'):
    p = line.rstrip('\n').split('\t')
    if len(p) < 3:
        continue
    all_forms.add(p[1].lower())
    if p[0] in needed_lemmas:
        forms[p[0]][p[1]].append(frozenset(p[2].split(',')))

# ------------------------------------------------------------ ser / estar
rows = data['ser_estar.tsv']
cnt = collections.Counter(r['answer'] for r in rows)
for r in rows:
    if r['answer'] not in ('ser', 'estar'):
        fail('ser_estar.tsv', r['_n'], f'answer {r["answer"]!r}')
print(f'ser_estar: {dict(cnt)}')
if rows and abs(cnt['ser'] - cnt['estar']) > 4:
    fail('ser_estar.tsv', 0, f'unbalanced {dict(cnt)}')

# --------------------------------------------------------------- por/para
rows = data['por_para.tsv']
cnt = collections.Counter(r['answer'] for r in rows)
for r in rows:
    if r['answer'] not in ('por', 'para'):
        fail('por_para.tsv', r['_n'], f'answer {r["answer"]!r}')
    if re.search(r'\b(por|para)\b', r['phrase'], re.I):
        warn('por_para.tsv', r['_n'], f'phrase also contains por/para outside the gap: {r["phrase"]!r}')
print(f'por_para: {dict(cnt)}')
if rows and abs(cnt['por'] - cnt['para']) > 4:
    fail('por_para.tsv', 0, f'unbalanced {dict(cnt)}')

# ------------------------------------------------- verb forms (pret/subj)
PERSONS = ('first-person', 'second-person', 'third-person')
NUMBERS = ('singular', 'plural')


def person_number(ts):
    return (tuple(p for p in PERSONS if p in ts), tuple(x for x in NUMBERS if x in ts))


def usable(ts):
    return not (ts & {'combined-form', 'imperative', 'vos-form', 'error-unrecognized-form', 'obsolete',
                      'archaic', 'participle', 'gerund', 'infinitive', 'class'})


def cls_pret_imp(ts):
    if 'subjunctive' in ts:
        return None
    if 'preterite' in ts:
        return 'pret'
    if 'imperfect' in ts and 'indicative' in ts:
        return 'imp'
    return None


def cls_mood(ts):
    if 'subjunctive' in ts:
        return 'subj'
    if 'indicative' in ts:
        return 'ind'
    return None


def check_verbs(name, classify, want_pair, maxwords=None):
    rows = data[name]
    tally = collections.Counter()
    detail = collections.Counter()
    for r in rows:
        n = r['_n']
        m = re.fullmatch(r'(.*\S) \(([a-záéíóúñ]+)\)', r['sentence'])
        if not m:
            fail(name, n, f'sentence must end with " (infinitive)": {r["sentence"]!r}')
            continue
        body, lemma = m.groups()
        if '___' not in body:
            fail(name, n, 'gap is not in the sentence body')
        if not re.search(r'(ar|er|ir|ír)$', lemma):
            fail(name, n, f'bracket {lemma!r} is not an infinitive')
        if body.split()[0] == '___':
            warn(name, n, 'sentence starts with the gap (answer would need a capital)')
        if maxwords:
            nw = len([t for t in body.split() if re.search(r'\w', t)])
            if nw > maxwords:
                fail(name, n, f'{nw} words (max {maxwords}): {body!r}')
        a, o = r['answer'], r['other']
        if a == o:
            fail(name, n, 'answer == other')
        if lemma not in forms:
            fail(name, n, f'lemma {lemma!r} not found in forms.tsv')
            continue
        fa = [t for t in forms[lemma].get(a, []) if usable(t)]
        fo = [t for t in forms[lemma].get(o, []) if usable(t)]
        if not fa:
            fail(name, n, f'answer {a!r} is not a (non-voseo) form of {lemma}')
        if not fo:
            fail(name, n, f'other {o!r} is not a (non-voseo) form of {lemma}')
        if not (fa and fo):
            continue
        pairs = set()
        for ta in fa:
            for to in fo:
                ca, co = classify(ta), classify(to)
                if ca and co and {ca, co} == want_pair and person_number(ta) == person_number(to):
                    pairs.add((ca, person_number(ta), 'imperfect' in ta and 'subjunctive' in ta))
        if not pairs:
            fail(name, n, f'{a!r}/{o!r} are not the same person/number in opposite {sorted(want_pair)} '
                          f'(answer tags {sorted(map(sorted, fa))}, other tags {sorted(map(sorted, fo))})')
            continue
        kinds = {p[0] for p in pairs}
        if len(kinds) > 1:
            fail(name, n, f'answer {a!r} classifies both ways: {kinds}')
            continue
        tally[kinds.pop()] += 1
        if any(p[2] for p in pairs):
            detail['answer is imperfect subjunctive'] += 1
    return tally, detail


t, _ = check_verbs('pret_imp.tsv', cls_pret_imp, {'pret', 'imp'}, maxwords=9)
print(f'pret_imp answers: {dict(t)}')
if sum(t.values()) and abs(t['pret'] - t['imp']) > 4:
    fail('pret_imp.tsv', 0, f'unbalanced {dict(t)}')

t, d = check_verbs('subjuntivo.tsv', cls_mood, {'subj', 'ind'})
tot = sum(t.values())
print(f'subjuntivo answers: {dict(t)} ({100 * t["subj"] / tot if tot else 0:.0f}% subj); {dict(d)}')
if tot and not 0.55 <= t['subj'] / tot <= 0.65:
    fail('subjuntivo.tsv', 0, f'subjunctive share {t["subj"]}/{tot} outside 55-65%')

# ---------------------------------------------------------- falsos amigos
rows = data['falsos_amigos.tsv']
nfr = sum(1 for r in rows if r['fr_trap'])
nen = sum(1 for r in rows if r['en_trap'])
for r in rows:
    if not (r['fr_trap'] or r['en_trap']):
        fail('falsos_amigos.tsv', r['_n'], f'{r["es"]}: no trap for either language')
    if ' ' not in r['es'] and r['es'] not in words:
        fail('falsos_amigos.tsv', r['_n'], f'{r["es"]!r} not in kaikki words.tsv')
    for k in ('fr_true', 'fr_trap', 'en_true', 'en_trap'):
        if len(r[k].split()) > 4:
            warn('falsos_amigos.tsv', r['_n'], f'{k} longer than 4 words: {r[k]!r}')
print(f'falsos_amigos: {len(rows)} rows, {nfr} with FR trap, {nen} with EN trap, '
      f'{sum(1 for r in rows if r["fr_trap"] and r["en_trap"])} with both')
if rows and (nfr < 20 or nen < 20):
    fail('falsos_amigos.tsv', 0, 'need traps for both French and English speakers (>=20 each)')

# ------------------------------------------------------------------ genero
rows = data['genero.tsv']
for r in rows:
    n, es, g = r['_n'], r['es'], r['gender']
    if g not in ('m', 'f'):
        fail('genero.tsv', n, f'gender {g!r} (want m or f)')
        continue
    gs = {gen for pos, gen in words.get(es, []) if pos == 'noun'}
    if not gs:
        fail('genero.tsv', n, f'{es!r} not a noun in kaikki words.tsv')
    elif g not in gs and not gs & {'mf', '-'}:
        fail('genero.tsv', n, f'{es}: gender {g} disagrees with kaikki {sorted(gs)}')
    elif gs != {g}:
        warn('genero.tsv', n, f'{es}: wrote {g}, kaikki has {sorted(gs)} ("-" = m or f same meaning) - double-check')
    if g == 'f' and re.match(r'h?[aá]', es) and f'el {es}' not in r['note']:
        warn('genero.tsv', n, f'{es}: feminine starting with a-/ha- but note lacks "el {es}"')
print(f'genero: {collections.Counter(r["gender"] for r in rows)}')

# ------------------------------------------------------------------ pseudo
rows = data['pseudo.tsv']
known = freq | set(w.lower() for w in words) | all_forms
known_stripped = {strip_acc(w) for w in known}
known_phon = {phon(w) for w in known}
ONSET2 = {'bl', 'br', 'cl', 'cr', 'dr', 'fl', 'fr', 'gl', 'gr', 'pl', 'pr', 'tr', 'ch', 'll'}
V = set('aeiouáéíóú')
endings = collections.Counter()
for r in rows:
    n, w = r['_n'], r['pseudo']
    if not re.fullmatch(r'[a-zñáéíóúü]+', w):
        fail('pseudo.tsv', n, f'{w!r}: only lowercase Spanish letters allowed')
        continue
    if not 4 <= len(w) <= 9:
        fail('pseudo.tsv', n, f'{w!r}: length {len(w)} not in 4-9')
    if w in known:
        fail('pseudo.tsv', n, f'{w!r} is a real word (es_50k / words.tsv / forms.tsv)')
    elif strip_acc(w) in known_stripped:
        fail('pseudo.tsv', n, f'{w!r} differs from a real word only by accents')
    elif phon(w) in known_phon:
        fail('pseudo.tsv', n, f'{w!r} sounds like a real word (b/v, h, c/z/s, ll/y, g/j)')
    for m in re.finditer(r'[^aeiouáéíóúü]+', w):
        if 0 < m.start() and m.end() < len(w) and not cluster_ok(m.group(0)):
            fail('pseudo.tsv', n, f'{w!r}: un-Spanish consonant cluster {m.group(0)!r}')
    # Spanish spelling rules
    bad = []
    if re.search(r'[kw]', w):
        bad.append('k/w')
    if re.search(r'q(?!u[eiéí])', w):
        bad.append('q not followed by ue/ui')
    if re.search(r'z[eiéí]', w):
        bad.append('ze/zi (use ce/ci)')
    if re.search(r'n[bp]', w):
        bad.append('nb/np (use mb/mp)')
    if re.search(r'm[^aeiouáéíóúbpn]', w):
        bad.append('m + consonant other than b/p/n')
    for m in re.finditer(r'(.)\1', w):
        if m.group(0) not in ('rr', 'll', 'cc', 'ee', 'oo'):
            bad.append(f'double {m.group(0)}')
    if re.search(r'cc(?![eiéí])', w):
        bad.append('cc not before e/i')
    if w.startswith('rr') or w.endswith('rr') or re.search(r'[nls]rr', w):
        bad.append('misplaced rr')
    if w[-1] not in V and w[-1] not in 'dlnrszy':
        bad.append(f'final -{w[-1]}')
    if re.match(r's[^aeiouáéíóú]', w):
        bad.append('initial s+consonant')
    if not w[0] in V and not w[1] in V and w[:2] not in ONSET2:
        bad.append(f'initial cluster {w[:2]}')
    if re.search(r'[aeiouáéíóú]{3}', w):
        bad.append('three vowels in a row')
    acc = re.findall(r'[áéíóú]', w)
    if len(acc) > 1:
        bad.append('several accents')
    if re.search(r'[cs]ion$', w):
        bad.append('-cion/-sion needs an accent')
    if acc and not re.search(r'(ción|sión|ón|ía|ío|ías|íos|án|én|ín|ún)$', w):
        bad.append('accent outside a final stressed syllable / hiatus')
    if bad:
        fail('pseudo.tsv', n, f'{w!r}: ' + ', '.join(bad))
    for e in ('ción', 'dad', 'ar', 'er', 'ir', 'oso', 'ero', 'o', 'a', 'e', 'al', 'ón', 'l', 'n', 'r', 's', 'z'):
        if w.endswith(e):
            endings[e] += 1
            break
    else:
        endings['other'] += 1
print(f'pseudo endings: {dict(endings)}; lengths: {dict(sorted(collections.Counter(len(r["pseudo"]) for r in rows).items()))}')
for e in ('ción', 'ar', 'o', 'a', 'dad'):
    if rows and endings[e] < 8:
        fail('pseudo.tsv', 0, f'too few -{e} endings ({endings[e]})')

# ------------------------------------------------------------------ report
print()
for n in HEADERS:
    print(f'{n:20s} {len(data[n]):4d} rows (target {TARGET[n]})')
print()
for w in warns:
    print('WARN', w)
for f in fails:
    print('FAIL', f)
print(f'\n{len(fails)} failure(s), {len(warns)} warning(s)')
sys.exit(1 if fails else 0)
