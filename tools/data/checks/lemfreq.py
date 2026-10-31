"""Build approximate lemma frequencies from es_50k + kaikki. Output: lemma pos count rank"""
import sys, re, collections
freq = {}
for i, line in enumerate(open('/home/claude/data/es_50k.txt', encoding='utf-8')):
    p = line.split()
    if len(p) == 2:
        freq[p[0]] = int(p[1])
lem = collections.defaultdict(set)  # (lemma,pos) -> forms
pos_of = collections.defaultdict(set)
genders = {}
pat = re.compile(r'^(?:(?:masculine|feminine) )?(?:singular |plural )?(?:plural|feminine|masculine)(?: singular| plural)? of ([^\s:;,]+(?: [^\s:;,]+)?)')
for line in open('/home/claude/data/kaikki/words.tsv', encoding='utf-8'):
    w, p, g, gl = (line.rstrip('\n').split('\t') + ['', '', '', ''])[:4]
    if p not in ('noun', 'adj', 'adv', 'verb'):
        continue
    if p == 'verb':
        continue
    m = pat.match(gl)
    if m:
        lem[(m.group(1), p)].add(w)
    else:
        lem[(w, p)].add(w)
for line in open('/home/claude/data/kaikki/forms.tsv', encoding='utf-8'):
    l, f, t = (line.rstrip('\n').split('\t') + ['', '', ''])[:3]
    if ' ' in f or 'vos-form' in t:
        continue
    lem[(l, 'verb')].add(f)
out = []
for (l, p), fs in lem.items():
    if p != 'verb' and l not in freq and not any(f in freq for f in fs):
        continue
    c = sum(freq.get(f, 0) for f in fs | {l})
    if c:
        out.append((c, l, p))
out.sort(reverse=True)
with open(sys.argv[1], 'w', encoding='utf-8') as o:
    for i, (c, l, p) in enumerate(out, 1):
        o.write(f"{l}\t{p}\t{c}\t{i}\n")
