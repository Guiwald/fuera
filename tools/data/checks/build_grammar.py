#!/usr/bin/env python3
"""Build assets/es/grammar/*.tsv from ' | '-delimited sources (python3 -I)."""
import sys
import unicodedata

SRC = '/tmp/claude-0/-home-claude/e07d58d4-7dbb-5e9c-900d-435cc048a19d/scratchpad/src'
OUT = '/home/claude/fuera/assets/es/grammar'
NAMES = ['ser_estar', 'por_para', 'pret_imp', 'subjuntivo', 'falsos_amigos', 'genero', 'pseudo']

for name in (sys.argv[1:] or NAMES):
    out = []
    for i, line in enumerate(open(f'{SRC}/{name}.psv', encoding='utf-8'), 1):
        line = line.rstrip('\n')
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        cols = [c.strip() for c in line.split('|')]
        out.append('\t'.join(cols))
    text = unicodedata.normalize('NFC', '\n'.join(out) + '\n')
    with open(f'{OUT}/{name}.tsv', 'w', encoding='utf-8', newline='\n') as f:
        f.write(text)
    print(f'{name}.tsv: {len(out) - 1} rows')
