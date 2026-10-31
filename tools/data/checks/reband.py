"""Re-band non-expression rows by frequency within POS group (share B1), then apply overrides."""
import sys, collections
sys.path.insert(0, '/tmp/claude-0/checks')
from common import lemma_freq
SHARE = float(sys.argv[1]) if len(sys.argv) > 1 else 0.52
def words(s): return set(s.split('|')) if '|' in s else set(s.split())
force = {'B1': set(), 'B2': set()}
for line in open('/tmp/claude-0/checks/force.txt', encoding='utf-8'):
    line = line.strip()
    if line and not line.startswith('#'):
        b, ws = line.split(':', 1)
        force[b] |= {w.strip() for w in words(ws)}
rows = [l.rstrip('\n').split('|') for l in open('/tmp/claude-0/checks/draft_final.txt', encoding='utf-8') if l.strip()]
grp = lambda pos: pos if pos in ('n', 'v', 'adj', 'expr') else 'func'
by = collections.defaultdict(list)
for r in rows:
    by[grp(r[1])].append(r)
changes = []
for g, rs in by.items():
    if g == 'expr':
        continue
    rs.sort(key=lambda r: -lemma_freq(r[0], r[1]))
    cut = round(len(rs) * SHARE)
    for i, r in enumerate(rs):
        nb = 'B1' if i < cut else 'B2'
        if nb != r[5]:
            changes.append((r[5], nb, r[1], r[0]))
        r[5] = nb
for r in rows:
    for b in ('B1', 'B2'):
        if r[0] in force[b] and r[5] != b:
            r[5] = b
with open('/tmp/claude-0/checks/draft_banded.txt', 'w', encoding='utf-8') as o:
    for r in rows:
        o.write('|'.join(r) + '\n')
c = collections.Counter(r[5] for r in rows)
print(dict(c), len(rows))
print('auto B1->B2:', ' '.join(f"{p}:{e}" for a, b, p, e in changes if a == 'B1' and e not in force['B1']))
print('auto B2->B1:', ' '.join(f"{p}:{e}" for a, b, p, e in changes if a == 'B2' and e not in force['B2']))
