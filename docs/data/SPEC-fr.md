# French content — data formats (¡Fuera!, French course)

All files are UTF-8, tab-separated (TSV), one header line, no quoting. They live in `assets/fr/`. French = **France**,
**traditional spelling** (*connaître, événement, oignon, il cédera*; never the 1990 variants). Every value must be
accurate: these files teach a language. When unsure about an entry, leave it out.

Learners: adults/teens whose base language is **Spanish** (first learner: a young man from Honduras, absolute
beginner) or **English**. Meanings must be short (1–3 words, the main everyday meaning), natural, and fit on a game
tile.

**Spanish glosses** follow one rule: when Spain and Latin America use different everyday words, write both as
`Spain|LatAm` (`coche|carro`, `ordenador|computadora`, `móvil|celular`, `zumo|jugo`, `patata|papa`,
`conducir|manejar`, `gafas|lentes`, `piso|apartamento`, `melocotón|durazno`, `enfadado|enojado`). The LatAm side is
"neutral" Latin-American Spanish (Mexico / Central America, understood everywhere): no Rioplatense or Caribbean-only
words. Otherwise write one gloss valid on both sides. Never use *vosotros* forms; use *tú* or *ustedes*. Avoid
*coger* (vulgar in much of Latin America): use *tomar* / *agarrar* as appropriate.

## Reference data on disk (licences: CC BY-SA)

- `/home/claude/data/fr/candidates.tsv` — French lemmas by frequency (OpenSubtitles 2018 forms folded onto English
  Wiktionary lemmas): `rank lemma pos gender count en_wiktionary ipa`. **Noisy**: homographs and misanalyses
  (*pas* as a noun, *dame* very high, *est* as an adjective, proper names like *catherine*). Use your judgement.
- `/home/claude/data/fr/idx/words.tsv` — English Wiktionary, French entries: `word pos gender glosses ipa form_of`.
- `/home/claude/data/fr/idx/frwikt.tsv` — French Wiktionary, French entries: `word pos gender es en ipa h_aspire
  homophones` (`es`, `en`: its translation lists, `|`-separated).
- `/home/claude/data/fr/es_reverse.tsv` — the game's Spanish lexicon read backwards: `fr_gloss es pos gender en band
  theme pic` (useful for Spanish glosses, themes and emojis; the French gloss there is a meaning of the Spanish word,
  so check it fits).
- `/home/claude/data/fr/fr_50k.txt` — French word-form frequencies, most frequent first.

## 1. Lexicon — `lexicon_a.tsv` (A1–A2) and `lexicon_b.tsv` (B1–B2)

Columns: `fr	pos	gender	es	en	band	theme	pic	note`

- `fr`: the lemma as taught. Nouns **without article**, singular (plural-only nouns in the plural with note `pl.`:
  *gens*, *vacances*, *lunettes*). Adjectives in the masculine singular. Verbs in the infinitive; pronominal verbs as
  `se lever` / `s'appeler` only when the pronominal meaning is the one taught (*s'appeler* = *llamarse*).
- `pos`: one of `n v adj adv prep conj pron det num interj expr`.
- `gender`: nouns only: `m`, `f`, or `mf` (same form for both: *élève*, *enfant*, *journaliste*). Empty otherwise.
- `es`: Spanish gloss (see the Spain|LatAm rule above), no article, verbs in the infinitive.
- `en`: English gloss, no article, verbs with `to` (`to eat`).
- `band`: `A1`, `A2`, `B1`, `B2` (frequency first, CEFR intuition for the cut-offs; A1 must contain the classic
  beginner sets: colours, numbers 1–20, days, months, family, body, food, house, clothes, common animals, weather).
- `theme`: exactly one of: `personas familia cuerpo salud comida bebida casa ropa ciudad transporte viaje naturaleza
  animales clima tiempo trabajo estudios ocio deporte emociones comunicacion compras dinero tecnologia sociedad
  cultura abstracto cantidad colores acciones descripcion gramatica` (same list as the Spanish course).
- `pic`: **one** standard Unicode emoji when the word is concrete and the emoji is unambiguous (🐶 *chien*, 🍎
  *pomme*, 🚗 *voiture*). Empty otherwise. Never a flag, a skin-tone or ZWJ sequence: a single code point (an optional
  U+FE0F is fine).
- `note`: optional, short, `;`-separated:
  - `h aspiré` for nouns/adjectives whose *h* blocks elision and liaison (*héros*, *haricot*, *hache*, *honte*,
    *hasard*, *haut*, *huit*…) — **required**, the game builds *le héros* vs *l'homme* from it;
  - `pl.` for plural-only nouns;
  - `f: actrice` for an irregular feminine (nouns/adjectives: *beau → belle*, *acteur → actrice*);
  - `irrégulier` for irregular verbs;
  - `≠ ES: …` (false friend for Spanish speakers, e.g. *large* `≠ ES: largo = long`), `≠ EN: …`;
  - `familier` for informal words (*boulot*, *bagnole*: keep few, B1–B2 only). No vulgar words.

No duplicates (same `fr` + `pos`). Order by frequency inside each band.

## 2. Grammar sets — `grammar/`

Sentences are short (≤ 10 words), everyday, unambiguous, in correct French. `es`/`en` are natural translations of
the whole sentence, the Spanish one following the Spain|LatAm rule when a word differs. `why`: a 2–5 word rule in
French (`verbe de mouvement`, `habitude`, `négation`, `après il faut que`…). `band`: A1–B2.

- `genre.tsv`: `fr	gender	es	en	band	note` — nouns whose gender surprises learners: gender different from the
  Spanish word (*le lait / la leche*, *la mer / el mar*, *le nez / la nariz*, *la douleur / el dolor*, *la couleur*,
  *le sang*, *le sel*, *la dent*, *l'équipe* f, *la méthode*, *la planète*, *le signal*, *le miel*, *la fin*?…),
  and classic traps for everyone (*la plage*, *la page*, *l'image* f, *le musée*, *le lycée*, *la peau*, *l'eau* f,
  *le silence*, *le problème*, *la souris*, *le camion*, *l'avion* m…). Note: `la leche` style hints for
  Spanish speakers (the Spanish word with its article) when the gender differs.
- `etre_avoir.tsv`: `sentence	answer	form	es	en	band	why` — a passé composé sentence with `___` for the
  auxiliary: `Nous ___ partis tôt.` `answer` = `être` or `avoir`; `form` = the conjugated auxiliary that fills the
  gap (`sommes`, `ai`, `t'es` for *tu t'es levé*…: write the gap so that the filled sentence is correct, e.g.
  `Tu ___ levé tôt ?` with form `t'es`). Include: movement verbs (aller, venir, partir, arriver, entrer, rester,
  tomber, naître, mourir…), pronominal verbs (se lever, s'habiller, se tromper…), ordinary avoir verbs, *être* and
  *avoir* themselves (*j'ai été*, *j'ai eu*), and the double verbs with both answers in different sentences
  (*Je suis sorti* / *J'ai sorti la poubelle*, *monter*, *descendre*, *rentrer*, *passer*, *retourner*). About
  half être, half avoir.
- `partitifs.tsv`: `sentence	answer	other	es	en	band	why` — a gap for the article: `answer` one of `du`,
  `de la`, `de l'`, `des`, `de`, `d'` and also `le`, `la`, `l'`, `les`, `un`, `une` when the sentence needs them
  (*J'aime ___ café.* → `le`); `other` = the tempting wrong form (`du` for *J'aime le café*, `du` for *pas de
  pain*, `le` or `Ø`... use `de la`/`du`/`le` etc., never empty). The gap must be unambiguous. Cover: food and
  drink (*du pain, de la confiture, de l'eau, des pommes*), negation (*pas de*, *pas d'*), quantities (*beaucoup
  de*, *un peu de*, *un kilo de*), taste/preference with the definite article (*j'aime le*, *je déteste les*),
  abstract partitives (*du courage*, *de la chance*), *faire du sport / de la musique*. Around 90 rows.
- `passe.tsv`: `sentence	answer	other	es	en	band	why` — a sentence with `___` and the infinitive in brackets at the
  end (`Hier, je ___ au cinéma. (aller)`), `answer` the right form, `other` the same verb in the other tense
  (passé composé ↔ imparfait). Forms written as they fill the gap (subject pronoun outside the gap; `suis allé`,
  `allais`; with elision write the gap after `j'`? No: put the pronoun in the sentence and conjugate after it, and
  avoid sentences where elision would change the pronoun, e.g. avoid `je ___ (aimer)`). Context must make the answer
  unambiguous (time markers, habit vs. one-off, background vs. event). Use the masculine unless the sentence says
  otherwise (*Elle ___ partie*). About 90 rows, half each.
- `subjonctif.tsv`: same columns as `passe.tsv`; `other` is the indicative present (or future when natural). Mix
  triggers (*il faut que, vouloir que, avant que, pour que, bien que, je doute que, c'est dommage que, je suis
  content que*) with indicative contexts (*je sais que, je pense que, parce que, après que* (indicative!), *il est sûr
  que*) so both answers occur. About 100 rows.
- `faux_amis.tsv`: `fr	es_true	es_trap	en_true	en_trap	band	note` — French words that mislead Spanish and/or
  English speakers: the real meaning and the tempting wrong meaning in each base language (empty trap when it is no
  false friend for that language). Examples ES: *large* (ancho / largo), *salir* (ensuciar / salir), *quitter*
  (dejar / quitar), *entendre* (oír / entender), *attendre* (esperar / atender), *rester* (quedarse / restar),
  *constipé* (resfriado / estreñido), *débile*, *carte*, *tirer*, *subir*, *discuter*, *exprimer*, *molester*,
  *vaso*… EN: *librairie*, *actuellement*, *assister*, *blesser*, *coin*, *déception*, *éventuellement*, *journée*,
  *lecture*, *location*, *monnaie*, *préservatif*, *sensible*, *sympathique*, *chair*, *pain*, *sale*, *rester*…
  About 80 rows, at least 50 with a Spanish trap.
- `pseudo.tsv`: `pseudo` — 200 plausible-looking French **non-words** for the "Ça existe ?" placement test, built
  by changing one or two letters of real words while keeping French spelling rules (*chapole*, *fraisure*,
  *pomblier*…). Each must be absent from `fr_50k.txt`, `words.tsv` and `frwikt.tsv`.

## Checks every author must run before handing back

`python3 -I /home/claude/fuera/tools/data/fr/check_fr.py <file>` — format, part of speech and gender against both
Wiktionaries, Spanish/English glosses against the Wiktionary translations (flags only), conjugated forms against the
game's conjugator for `passe.tsv` / `subjonctif.tsv` / `etre_avoir.tsv`. Every flag must be either fixed or answered
in a short `doubts` list handed back with the file.
