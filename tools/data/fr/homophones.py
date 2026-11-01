"""Groups of lexicon words that sound the same (same IPA in the French Wiktionary, else the English one):
assets/fr/homophones.txt, one group per line. Listening rounds leave them out.

Usage: python3 -I homophones.py idx/frwikt.tsv idx/words.tsv lexicon_a.tsv lexicon_b.tsv > homophones.txt
"""
import re, sys
from collections import defaultdict
fr_ipa, en_ipa = {}, {}
for i, l in enumerate(open(sys.argv[1], encoding="utf-8")):
    c = l.rstrip("\n").split("\t")
    if i and len(c) >= 6 and c[5] and c[0] not in fr_ipa:
        fr_ipa[c[0]] = c[5]
for i, l in enumerate(open(sys.argv[2], encoding="utf-8")):
    c = l.rstrip("\n").split("\t")
    if i and len(c) >= 6 and c[4] and not c[5] and c[0] not in en_ipa:
        en_ipa[c[0]] = c[4]
def key(ipa):
    return re.sub(r"[./\\\[\]ˈˌ‿ː()]", "", ipa).replace("ɛ", "e").replace("ɑ", "a").replace("ɔ̃", "ɔ̃").strip()
groups = defaultdict(set)
for path in sys.argv[3:]:
    for i, l in enumerate(open(path, encoding="utf-8")):
        c = l.split("\t")
        if i == 0 or " " in c[0]:
            continue
        ipa = fr_ipa.get(c[0]) or en_ipa.get(c[0])
        if ipa:
            groups[key(ipa)].add(c[0])
for k, ws in sorted(groups.items()):
    if len(ws) > 1:
        print(" ".join(sorted(ws)))
