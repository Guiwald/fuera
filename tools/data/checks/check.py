"""Validate lexicon_b.tsv against SPEC-es.md section 1 and kaikki. usage: check.py file.tsv [lexicon_a.tsv]"""
import sys, collections, os
sys.path.insert(0, '/tmp/claude-0/checks')
from common import *

fn = sys.argv[1]
lines = open(fn, encoding='utf-8').read().split('\n')
if lines and lines[-1] == '':
    lines.pop()
errs, warns = [], []
if lines[0].split('\t') != HEADER:
    errs.append('bad header')
kk = load_kaikki()
rows = []
for i, line in enumerate(lines[1:], 2):
    p = line.split('\t')
    if len(p) != 9:
        errs.append(f'L{i}: {len(p)} columns'); continue
    es, pos, g, fr, en, band, theme, pic, note = p
    rows.append(p)
    tag = f'L{i} {es} ({pos})'
    if not es or es != es.strip() or '  ' in es:
        errs.append(f'{tag}: bad es')
    if any(c in es for c in '¡!¿?.,;'):
        errs.append(f'{tag}: punctuation in es')
    if pos not in POS: errs.append(f'{tag}: bad pos')
    if band not in ('B1', 'B2'): errs.append(f'{tag}: bad band {band}')
    if theme not in THEMES: errs.append(f'{tag}: bad theme {theme}')
    if pos == 'n' and g not in ('m', 'f', 'mf'): errs.append(f'{tag}: noun without gender')
    if pos != 'n' and g: errs.append(f'{tag}: gender on non-noun')
    if not fr or not en: errs.append(f'{tag}: empty gloss')
    if pos == 'v' and not en.startswith('to '): errs.append(f'{tag}: verb en without "to"')
    if pos == 'v' and not (es.endswith('ar') or es.endswith('er') or es.endswith('ir') or es.endswith('ír') or es.endswith('se')):
        errs.append(f'{tag}: not an infinitive')
    if not is_single_emoji(pic): errs.append(f'{tag}: bad pic {pic!r}')
    for c in fr + en + note:
        if c in '\t\r\n': errs.append(f'{tag}: control char')
    if len(fr.split()) > 4 or len(en.split()) > 5: warns.append(f'{tag}: long gloss fr={fr!r} en={en!r}')
    if len(note) > 70: warns.append(f'{tag}: long note ({len(note)})')
    # kaikki checks
    if pos == 'expr' or ' ' in es:
        if pos != 'expr': warns.append(f'{tag}: multiword but pos {pos}')
        continue
    ents = kk.get(es, [])
    kp = {e[0] for e in ents}
    if not ents:
        errs.append(f'{tag}: NOT IN KAIKKI'); continue
    if not (kp & KPOS[pos]):
        errs.append(f'{tag}: kaikki pos mismatch: has {sorted(kp)}'); continue
    if pos == 'n':
        kg = {e[1] for e in ents if e[0] == 'noun'} - {'-'}
        if not kg:
            warns.append(f'{tag}: kaikki has no gender')
        elif g not in kg:
            if g == 'mf' and {'m', 'f'} <= kg: pass
            else: errs.append(f'{tag}: gender {g} vs kaikki {sorted(kg)}')
        elif 'mf' in kg and g != 'mf':
            warns.append(f'{tag}: kaikki also says mf (wrote {g})')
dup = collections.Counter((r[0], r[1]) for r in rows)
for k, c in dup.items():
    if c > 1: errs.append(f'duplicate {k}')
# order by band then frequency (non-increasing within band)
prev = None
for r in rows:
    if prev and prev[5] == 'B2' and r[5] == 'B1': errs.append(f'band order: {r[0]} after B2 rows')
    prev = r
if len(sys.argv) > 2 and os.path.exists(sys.argv[2]):
    a = {tuple(l.split('\t')[:2]) for l in open(sys.argv[2], encoding='utf-8').read().split('\n')[1:] if l}
    ov = [(r[0], r[1]) for r in rows if (r[0], r[1]) in a]
    print('overlap with lexicon_a:', len(ov), ov)
c = collections.Counter(r[5] for r in rows)
print(f'rows={len(rows)} bands={dict(c)} errors={len(errs)} warnings={len(warns)}')
print('\n'.join(errs)); print('-- warnings'); print('\n'.join(warns))
