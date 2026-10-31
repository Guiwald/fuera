# Test data

## `forms.tsv`: Spanish conjugation fixture

Columns: `infinitive`, `tense`, `person`, `form` (tab-separated; lines starting
with `#` are comments). `tense` and `person` use the names of
`fuera_learn::conj::Tense` and `Person`. A slot (infinitive, tense, person) may
have several rows when Wiktionary lists several valid forms (for example
`predeciré` / `prediré`). The test accepts any of them.

Contents: the 400 most frequent infinitives of a Spanish frequency list (a
handful of noun/adjective homographs such as *placer* or *militar* left out), plus
a list of 200 irregular, spelling-change and inherently pronominal verbs
(*quejarse*, *inmiscuirse*, …). The two lists overlap, giving 500 verbs in total. It covers the present,
preterite, imperfect, future, conditional, present subjunctive and `-ra`
imperfect subjunctive, for the six persons with *vosotros*. These rows are left
out: *vos* forms, formal imperatives, the `-se` imperfect subjunctive, the future
subjunctive, regional (non-Spain), archaic or nonstandard variants, and
pronominal duplicates of non-pronominal verbs (`hablar` → "me hablo").

`build_forms.py` regenerates the file:

```sh
python3 -I build_forms.py kaikki/forms.tsv es_50k.txt kaikki-es.jsonl > forms.tsv
```

## Attribution and licence

The verb forms come from [Wiktionary](https://en.wiktionary.org/), as extracted
by [kaikki.org](https://kaikki.org/dictionary/Spanish/) (wiktextract, Tatu
Ylonen). They are available under the
[Creative Commons Attribution-ShareAlike 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
licence (CC BY-SA 4.0). `forms.tsv` is an adapted, filtered extract and is
distributed under the same licence. It is used only as test data and is not
compiled into the library.
