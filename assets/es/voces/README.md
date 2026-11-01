# Voix enregistrées / Recorded voices

`voces-es.fvox` contient des prononciations de mots espagnols enregistrées par les bénévoles de
[Lingua Libre](https://lingualibre.org) et publiées sur Wikimedia Commons. Chaque enregistrement garde sa licence
d'origine (CC BY-SA 4.0, CC BY 4.0 ou CC0). Le paquet est diffusé sous CC BY-SA 4.0. `creditos.tsv` donne, pour chaque
mot, la personne qui l'a enregistré, la licence et le fichier d'origine sur Commons.

**Modifications** : son ramené en mono 16 kHz, silences du début et de la fin coupés, volume harmonisé (EBU R128,
−18 LUFS), compression IMA ADPCM (format décrit dans `crates/learn/src/voice.rs`).

**Choix des voix** (espagnol d'Espagne) :

- Un mot prend d'abord l'enregistrement d'une personne d'Espagne. Sont reconnus comme tels les locuteurs soutenus par
  Wikimedia España ou qui l'indiquent sur leur page : Rodelar, Millars, ElsaBornFree, MiguelAlanCS, Mentxuwiki,
  Centenoyespelta.
- Sinon, une autre voix n'est retenue que si le mot ne contient **aucune lettre prononcée différemment en Espagne et
  en Amérique latine** : pas de *z*, de *c* devant *e*/*i*, de *ll* ni de *y* consonne.
- Les enregistrements classés « Non-native Spanish pronunciation » sont exclus.

Depuis la 1.1, les enregistrements de deux locuteurs vivant en France (Eavqwiki, Nicolas Lopez de Silanes WMFr) sont
retirés : ce sont très probablement des voix non natives. Il reste 1 470 mots (70 % du lexique).

Les mots sans enregistrement et les phrases sont lus par la voix du système.

## Espagnol d'Amérique latine : `voces-es-419.fvox`

1 769 mots (86 % du lexique de la variante), crédits dans `creditos-419.tsv`.

- L'origine des locuteurs vient du Wiktionnaire français, où le robot de Lingua Libre range chaque enregistrement
  avec le lieu du locuteur, plus ceux vérifiés à la main pour la 1.0.1 (Precision27, Sinaloa : Mexique ; Rubýñ :
  Costa Rica).
- Ordre de préférence : Mexique et Amérique centrale (Precision27, Sinaloa, Txtdgtl, Rubýñ, BMM66 du Honduras…) ;
  puis espagnol des Andes (Bogotá, Lima : Emanuelps27, Rodrigo5260, Rdrg109) ; puis Caraïbes (Venezuela, côte
  colombienne : Marreromarco, AdrianAbdulBaha) ; puis Río de la Plata et Chili, seulement pour les mots sans *ll*
  ni *y* consonne.
- Un mot sans voix d'Amérique garde sa voix d'Espagne s'il ne contient aucune lettre prononcée autrement des deux
  côtés (pas de *z*, pas de *c* devant *e*/*i*).

Reconstruire un paquet : `fuera-cli voices creditos.tsv <dossier des WAV> voces-es.fvox` (ou `creditos-419.tsv` →
`voces-es-419.fvox`). Les WAV sont en mono, 16 bits, 16 kHz ; scripts dans `tools/data/voces/`.

The packs hold Spanish word recordings by Lingua Libre volunteers (Wikimedia Commons), each under its own licence
(CC BY-SA 4.0, CC BY 4.0 or CC0): Spain first in `voces-es.fvox`, Latin America first in `voces-es-419.fvox`. The
packs as a whole are CC BY-SA 4.0. Speakers and source files: `creditos.tsv` and `creditos-419.tsv`.
