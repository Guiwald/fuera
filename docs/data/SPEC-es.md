# Spanish content — data formats (¡Fuera!, Spanish mode)

All files are UTF-8, tab-separated (TSV), one header line, no quoting. They live in `assets/es/`.
Spanish = **Spain** (Castilian, with *vosotros*). Latin-American variants go in `note` as `LatAm: …`.
Every value must be accurate: these files teach a language. When unsure about an entry, leave it out.

Learners: adults/teens whose base language is **French or English**; the first learner is a B1/B2 French speaker
living in Spain. Glosses must be short (1–3 words, the main everyday meaning), natural, and fit on a game tile.

## Reference data available on disk (licences: CC BY-SA)

- `/home/claude/data/es_50k.txt` — Spanish word-form frequencies (OpenSubtitles 2018), `form count`, most frequent first.
- `/home/claude/data/kaikki/words.tsv` — Wiktionary (en) index: `word pos gender gloss`. `gender` is m / f / mf / - for
  nouns. Use it to check that each lemma exists with that part of speech and, for nouns, its gender.
- `/home/claude/data/kaikki/forms.tsv` — Wiktionary verb forms: `lemma form tags`.

## 1. Lexicon — `lexicon_a.tsv` (A1–A2) and `lexicon_b.tsv` (B1–B2)

Columns: `es	pos	gender	fr	en	band	theme	pic	note`

- `es`: the lemma as taught. Nouns **without article**, singular (plural-only nouns in plural, with a note).
  Adjectives in masculine singular. Verbs in the infinitive (reflexive verbs as `-se` only when the meaning differs,
  e.g. `irse`, `quedarse`).
- `pos`: one of `n v adj adv prep conj pron det num interj expr`.
- `gender`: nouns only: `m`, `f`, or `mf` (same form for both, e.g. *estudiante*). Empty for other parts of speech.
  Nouns like *agua* (feminine with *el*): gender `f`, note `el agua`.
- `fr`: French gloss, no article (`chien`, `manger`, `rapide`). Several senses → the most common one only.
- `en`: English gloss, no article, verbs with `to` (`to eat`).
- `band`: `A1`, `A2`, `B1`, `B2` — by usefulness and frequency (use `es_50k.txt` as the main signal, CEFR intuition
  for the cut-offs).
- `theme`: exactly one of:
  `personas familia cuerpo salud comida bebida casa ropa ciudad transporte viaje naturaleza animales clima tiempo
  trabajo estudios ocio deporte emociones comunicacion compras dinero tecnologia sociedad cultura abstracto
  cantidad colores acciones descripcion gramatica`
- `pic`: **one** standard Unicode emoji when the word is concrete and the emoji is unambiguous for it (🐶 for *perro*,
  🍎 for *manzana*, 🚗 for *coche*). Empty otherwise. Never a flag, never a person-with-skin-tone sequence, never a
  ZWJ sequence: single code point emoji (an optional U+FE0F variation selector is fine).
- `note`: optional, short: `LatAm: carro`, `irregular`, `≠ FR: …` (false friend for French speakers),
  `≠ EN: …` (for English speakers), `el agua`, `pl.`.

No duplicates (same `es` + `pos`). Order by frequency inside each band.

## 2. Grammar sets — `grammar/`

- `ser_estar.tsv`: `phrase	answer	fr	en	band	why`
  `phrase` = what follows the verb (`cansado`, `de Madrid`, `en casa`, `médico`, `abierto`, `muy alto`…),
  `answer` = `ser` or `estar`, unambiguous in neutral use. `fr`/`en`: translation of the whole sentence with the right
  verb ("je suis fatigué" / "I am tired"). `why`: a 2–5 word rule in Spanish (`estado temporal`, `origen`,
  `profesión`, `ubicación`, `característica`…).
- `por_para.tsv`: `phrase	answer	fr	en	band	why` — `phrase` contains `___` where *por* or *para* goes
  (`gracias ___ todo`, `salgo ___ Madrid mañana`, `es ___ ti`). Unambiguous.
- `pret_imp.tsv`: `sentence	answer	other	fr	en	band	why` — a short sentence with `___` and the infinitive in
  brackets at the end (`Ayer ___ al cine. (ir)`), `answer` the right form (`fui`), `other` the same verb in the
  other tense (`iba`). The context must make the answer unambiguous (time markers, habitual vs. one-off…).
- `subjuntivo.tsv`: same columns as `pret_imp.tsv`; `other` is the indicative form. Mix triggers (*querer que,
  ojalá, para que, cuando* + future, *es importante que, no creo que, aunque*…) with indicative contexts (*sé que,
  creo que, es verdad que, cuando* habitual) so that both answers occur.
- `falsos_amigos.tsv`: `es	fr_true	fr_trap	en_true	en_trap	band	note` — the real meaning and the tempting wrong
  meaning for each base language (empty trap when the word is not a false friend for that language).
- `genero.tsv`: `es	gender	fr	en	band	note` — nouns whose gender surprises learners: *-ma* masculines (*problema*),
  *mano*, *día*, *foto*, *moto*, *radio*, *agua*-type, and nouns whose gender differs from French
  (*la leche* / le lait, *la nariz* / le nez, *el árbol*… only if different!, *la sangre*, *el equipo*…).
- `pseudo.tsv`: `pseudo` — plausible-looking Spanish **non-words** for the "¿existe?" placement test, built by
  changing one or two letters of real words while keeping Spanish spelling rules (*cantelo*, *permuna*…). Each must be
  absent from `es_50k.txt` and from `kaikki/words.tsv`.

## Checks every author must run before handing back

- Nouns: gender agrees with `kaikki/words.tsv` (if kaikki says `mf` and you wrote `m`, double-check).
- Every `es` lemma exists in `kaikki/words.tsv` with a compatible part of speech (multi-word expressions excepted).
- No duplicate rows; TSV has exactly the header's column count on every line.
