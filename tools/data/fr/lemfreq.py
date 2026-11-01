"""Lemma frequencies for French: FrequencyWords fr_50k (OpenSubtitles 2018, CC BY-SA 4.0) folded onto Wiktionary
lemmas (words.tsv from kaikki_index.py). A form's count is shared between its possible lemmas.

Usage: python3 -I lemfreq.py fr_50k.txt idx/words.tsv > lemfreq.tsv
"""
import sys
from collections import defaultdict

freq_path, words_path = sys.argv[1], sys.argv[2]
entries = defaultdict(list)   # form -> [(lemma, pos)]
info = {}                      # (lemma,pos) -> (gender, glosses, ipa)
for i, line in enumerate(open(words_path, encoding="utf-8")):
    if i == 0:
        continue
    c = line.rstrip("\n").split("\t")
    if len(c) < 6:
        continue
    word, pos, gender, glosses, ipa, form_of = c
    if pos in ("-", "name"):
        continue
    if form_of:
        entries[word].append((form_of, pos))
    else:
        entries[word].append((word, pos))
        k = (word, pos)
        if k not in info or (info[k][0] == "-" and gender != "-"):
            info[k] = (gender, glosses, ipa)
counts = defaultdict(float)
for line in open(freq_path, encoding="utf-8"):
    p = line.split()
    if len(p) != 2:
        continue
    form, n = p[0], int(p[1])
    cands = [e for e in dict.fromkeys(entries.get(form, [])) if e in info]
    if not cands:
        continue
    for e in cands:
        counts[e] += n / len(cands)
print("lemma\tpos\tgender\tcount\tglosses\tipa")
for (lemma, pos), n in sorted(counts.items(), key=lambda x: -x[1]):
    g, gl, ipa = info[(lemma, pos)]
    print(f"{lemma}\t{pos}\t{g}\t{int(n)}\t{gl}\t{ipa}")
