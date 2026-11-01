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

Les mots sans enregistrement et les phrases sont lus par la voix du système.

Reconstruire le paquet : `fuera-cli voices creditos.tsv <dossier des WAV> voces-es.fvox`. Les WAV sont en mono, 16 bits,
16 kHz.

The pack holds Spanish word recordings by Lingua Libre volunteers (Wikimedia Commons), each under its own licence
(CC BY-SA 4.0, CC BY 4.0 or CC0). The pack as a whole is CC BY-SA 4.0. Speakers and source files: `creditos.tsv`.
