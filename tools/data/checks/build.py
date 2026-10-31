"""Compute frequency for draft rows; write sorted TSV.  usage: build.py out.tsv draftfiles..."""
import sys
sys.path.insert(0, '/tmp/claude-0/checks')
from common import *

rows = []
for fn in sys.argv[2:]:
    for line in open(fn, encoding='utf-8'):
        line = line.rstrip('\n')
        if not line.strip() or line.startswith('#'):
            continue
        p = line.split('|')
        rows.append(p)
for r in rows:
    r.append(lemma_freq(r[0], r[1]))
order = {'B1': 0, 'B2': 1}
rows.sort(key=lambda r: (order.get(r[5], 9), -r[-1], r[0]))
with open(sys.argv[1], 'w', encoding='utf-8') as o:
    o.write('\t'.join(HEADER) + '\n')
    for r in rows:
        o.write('\t'.join(r[:9]) + '\n')
with open(sys.argv[1] + '.freq', 'w', encoding='utf-8') as o:
    for r in rows:
        o.write(f"{r[-1]}\t{r[5]}\t{r[1]}\t{r[0]}\t{r[3]}\t{r[4]}\n")
print(len(rows), 'rows')
