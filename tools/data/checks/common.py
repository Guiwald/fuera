"""Shared helpers for lexicon_b checks (run with python3 -I)."""
import re, collections, unicodedata

THEMES = set("""personas familia cuerpo salud comida bebida casa ropa ciudad transporte viaje naturaleza animales clima
tiempo trabajo estudios ocio deporte emociones comunicacion compras dinero tecnologia sociedad cultura abstracto
cantidad colores acciones descripcion gramatica""".split())
POS = set("n v adj adv prep conj pron det num interj expr".split())
KPOS = {'n': {'noun'}, 'v': {'verb'}, 'adj': {'adj'}, 'adv': {'adv', 'prep_phrase', 'adv_phrase'},
        'prep': {'prep', 'prep_phrase'}, 'conj': {'conj'}, 'pron': {'pron', 'det'}, 'det': {'det', 'pron'},
        'num': {'num', 'noun'}, 'interj': {'intj'}, 'expr': set()}
HEADER = ['es', 'pos', 'gender', 'fr', 'en', 'band', 'theme', 'pic', 'note']
FORMOF = re.compile(r'(plural|feminine|masculine|inflection|participle|gerund|form|person|singular|infinitive) .*\bof\b', re.I)

def load_freq():
    f = {}
    for line in open('/home/claude/data/es_50k.txt', encoding='utf-8'):
        p = line.split()
        if len(p) == 2:
            f[p[0]] = int(p[1])
    return f

def load_kaikki():
    k = collections.defaultdict(list)  # word -> [(pos, gender, gloss)]
    for line in open('/home/claude/data/kaikki/words.tsv', encoding='utf-8'):
        p = line.rstrip('\n').split('\t')
        if len(p) < 4:
            continue
        k[p[0]].append((p[1], p[2], p[3]))
    return k

def infl_forms(kk):
    """lemma -> set of inflected forms for nouns/adjs (from 'plural of X' etc.)."""
    pat = re.compile(r'^(?:(?:masculine|feminine) )?(?:singular |plural )?(?:plural|feminine|masculine)(?: singular| plural)? of ([^\s:;,]+)')
    m = collections.defaultdict(set)
    for w, entries in kk.items():
        for pos, g, gl in entries:
            if pos in ('noun', 'adj'):
                mm = pat.match(gl)
                if mm:
                    m[mm.group(1)].add(w)
    return m

def verb_forms():
    vf = collections.defaultdict(set)
    for line in open('/home/claude/data/kaikki/forms.tsv', encoding='utf-8'):
        p = line.rstrip('\n').split('\t')
        if len(p) < 3 or ' ' in p[1] or 'vos-form' in p[2]:
            continue
        vf[p[0]].add(p[1])
    return vf

def is_single_emoji(s):
    if not s:
        return True
    s2 = s.replace('️', '')
    if len(s2) != 1:
        return False
    cp = ord(s2)
    return cp >= 0x2190 and unicodedata.category(s2) == 'So'

_cache = {}
def _init():
    _cache['freq'] = freq = load_freq(); kk = load_kaikki(); _cache['inf'] = infl_forms(kk); _cache['vf'] = vf = verb_forms()
    owners = collections.defaultdict(set)
    for l, fs in vf.items():
        for f in fs:
            owners[f].add(l)
    _cache['owners'] = owners
    func = set(); ol = set()
    for w, entries in kk.items():
        for p, g, gl in entries:
            if p != 'verb' and not FORMOF.search(gl):
                ol.add(w)
                if p in ('prep', 'conj', 'adv', 'pron', 'det', 'intj', 'article'):
                    func.add(w)
    _cache['func'] = func; _cache['ol'] = ol
    _cache['verbish'] = {w for w, entries in kk.items() if any(e[0] == 'verb' for e in entries)}

def lemma_freq(es, pos):
    if not _cache:
        _init()
    freq, inf, vf, owners, func = _cache['freq'], _cache['inf'], _cache['vf'], _cache['owners'], _cache['func']
    if pos == 'expr' or ' ' in es:
        if 'expr' not in _cache:
            _cache['expr'] = {l.split('|')[0]: int(l.split('|')[1]) for l in open('/tmp/claude-0/checks/exprfreq.txt', encoding='utf-8') if '|' in l}
        if es in _cache['expr']:
            return _cache['expr'][es]
        print('WARN no expr freq for', es)
        return int(min(freq.get(t, 0) for t in es.split()) / 4)
    if pos == 'v':
        base = es[:-2] if es.endswith('se') and es[:-2] in vf else es
        tot = 0
        for f in vf.get(base, set()) | {base}:
            if f != base and f in _cache['ol']:
                continue
            own = owners.get(f, {base})
            if max(own, key=lambda l: freq.get(l, 0)) == base or len(own) == 1:
                tot += freq.get(f, 0)
        if base != es:
            a, b = freq.get(es, 0), freq.get(base, 0)
            share = min(1.0, max(0.1, a / (a + b))) if a + b else 0.3
            tot = int(tot * share)
        return tot
    if pos in ('n', 'adj'):
        forms = {f for f in inf.get(es, set()) if ' ' not in f and (pos == 'adj' or f.endswith('s'))}
        tot = sum(freq.get(f, 0) for f in forms | {es})
        if es in owners or es in func or es in _cache['verbish']:
            pl = sum(freq.get(f, 0) for f in forms)
            tot = min(tot, pl * 3) if pl else tot // 3
        return tot
    return freq.get(es, 0)
