# ADR 0007 — Le cours comme paramètre : français, espagnol d'Amérique latine, retours

- Statut : acceptée (01/11/2026)
- Contexte : version 1.1. Guillaume veut que le copain d'Alice, hispanophone du Honduras, apprenne le français, et
  ajouter l'espagnol d'Amérique latine. Le cadrage complet (23 questions, réponses de Guillaume) est dans le projet :
  `fuera/PLAN-1.1.md`.

## Décision

1. **Un cours = la langue apprise + sa variante**, plus la langue des sens. Trois cours : espagnol d'Espagne,
   espagnol d'Amérique latine, français de France. Sens en français ou en anglais pour l'espagnol, en espagnol ou en
   anglais pour le français (`Target::bases`).
2. **Progrès rangés par langue apprise** dans la sauvegarde : l'espagnol garde les clés de la 1.0 (`m.…`, `l.…`), le
   français prend le préfixe `fr.` (`fr.m.…`, `fr.l.…`). Une sauvegarde 1.0.1 s'ouvre donc telle quelle, en
   « espagnol d'Espagne ». Les deux variantes d'espagnol partagent leurs progrès : seuls les mots remplacés (*carro*
   au lieu de *coche*) sont des cartes à part. Un futur préfixe de profil (1.2) se posera devant.
3. **Un seul réglage « Espagnol : Espagne / Amérique latine »**, par défaut d'après la langue du système (es-ES →
   Espagne, autre pays hispanophone → Amérique latine). Il choisit la variante enseignée *et* les gloses espagnoles
   du cours de français, écrites `Espagne|Amérique` quand le mot courant diffère (`coche|carro`).
4. **Espagnol d'Amérique latine construit à partir de l'espagnol d'Espagne** au chargement (`Content::to_latam`) :
   - `assets/es/latam.tsv` remplace un mot (`coche → carro`, avec genre ou gloses si besoin) ou l'écarte (`-` :
     *vosotros*, *vale*, *guay*…) ;
   - toute phrase de grammaire qui contient une forme de *vosotros*, un mot remplacé (avec son pluriel ou ses formes
     conjuguées : *coches*, *cogimos*), ou une expression d'Espagne (*echar de menos*) est écartée ; une forme commune
     à un verbe d'usage général n'écarte rien (*venga* de *venir*, *mando* de *mandar*) ;
   - `assets/es/latam_skip.txt` écarte quelques phrases relues à la main (passé composé du jour, *quedar*,
     *rellenar*…) ;
   - les portes des verbes n'ont plus *vosotros* ; la troisième personne du pluriel devient « ellos /
     ustedes ». Le voseo n'est pas enseigné.
5. **Choisir son cours** : l'accueil propose « Apprendre une langue » ; un sélecteur montre les trois cours, celui
   qui correspond à la langue de l'appareil en couleur (un appareil en espagnol propose le français). Le bouton
   d'accueil prend ensuite le nom du cours.
6. **Voix rangées par paquet** (`voces-es`, `voces-es-419`, `voces-fr`) : tous embarqués dans les applis (Mac,
   Windows, Linux, Android), téléchargés à la demande sur le web, seulement pour le cours joué. La voix du système
   suit le cours (es-ES, es-MX, fr-FR), Android compris. Un paquet « fr-CA » pourra s'ajouter plus tard avec un réglage
   « Accent ».
7. **Retours sans serveur ni compte** : un bouton « Donner mon avis » (accueil de la langue, Réglages) ouvre la feuille
   de partage du système (Android : `ACTION_SEND` ; navigateur de téléphone : `navigator.share`) avec un message
   prérempli : version, plateforme, cours, niveau, mots signalés. Le joueur choisit l'appli et la personne, et peut
   parler avec le micro de sa messagerie. Sur ordinateur, le message s'ouvre dans le logiciel de courrier, adressé à
   `fuera@librejeu.fr` (redirection OVH vers la boîte de Guillaume). Un drapeau sur la carte de correction, sur le
   résumé de manche et dans le cahier signale un mot (« la voix sonne mal », « traduction bizarre », « autre chose ») ;
   les mots signalés depuis 14 jours partent avec chaque message (ils restent dans la sauvegarde, au plus 60). Une
   seule invitation, après la première séance complète.

## Pourquoi pas…

- **…des profils dès la 1.1** : le copain joue sur son propre téléphone, et une personne qui apprend deux langues a
  déjà deux progrès séparés. Les profils viendront avec le mode enfant, où un enfant joue sur l'appareil d'un parent.
- **…un enregistrement de la voix dans le jeu** : micro à autoriser sur chaque plateforme, partage de fichier sans
  Java sur Android, appli Mac non signée. Le micro de WhatsApp fait le même travail.
- **…un deuxième lexique espagnol pour l'Amérique latine** : 95 % des mots et des phrases sont communs ; une
  surcouche relue est plus sûre qu'une copie qui divergerait.
