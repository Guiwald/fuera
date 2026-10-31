"""Apply cuts/moves to drafts -> draft_final.txt (pipe). usage: apply.py"""
import sys, glob
cuts = set(); moves = {}
for line in open('/tmp/claude-0/checks/cuts.txt', encoding='utf-8'):
    line = line.strip()
    if not line or line.startswith('#'):
        continue
    pos, words = line.split(':', 1)
    ws = words.split('|') if '|' in words else words.split()
    for w in ws:
        cuts.add((w.strip(), pos))
try:
    for line in open('/tmp/claude-0/checks/moves.txt', encoding='utf-8'):
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        band, pos, words = line.split(':', 2)
        for w in words.split():
            moves[(w, pos)] = band
except FileNotFoundError:
    pass
restore = set()
for line in open('/tmp/claude-0/checks/restore.txt', encoding='utf-8'):
    line = line.strip()
    if not line or line.startswith('#'):
        continue
    pos, words = line.split(':', 1)
    for w in (words.split('|') if '|' in words else words.split()):
        restore.add((w.strip(), pos))
cuts -= restore
alex = set()
for l in open('/home/claude/fuera/assets/es/lexicon_a.tsv', encoding='utf-8').read().split('\n')[1:]:
    if l:
        q = l.split('\t'); alex.add((q[0], q[1]))
overlap = []
rows = []
seen_cut = set()
for fn in sorted(glob.glob('/tmp/claude-0/checks/draft/*.txt')):
    for line in open(fn, encoding='utf-8'):
        line = line.rstrip('\n')
        if not line.strip():
            continue
        p = line.split('|')
        key = (p[0], p[1])
        if key in cuts:
            seen_cut.add(key); continue
        if key in alex:
            overlap.append(key); continue
        if key in moves:
            p[5] = moves[key]; moves_used = True
        rows.append(p)
for c in cuts - seen_cut:
    print('cut not found:', c)
for m in set(moves) - {(r[0], r[1]) for r in rows}:
    print('move not found:', m)
with open('/tmp/claude-0/checks/draft_final.txt', 'w', encoding='utf-8') as o:
    for p in rows:
        o.write('|'.join(p) + '\n')
import collections
c = collections.Counter(r[5] for r in rows); cp = collections.Counter((r[1], r[5]) for r in rows)
print('removed (in lexicon_a):', len(overlap)); print(len(rows), dict(c)); print(sorted(cp.items()))
