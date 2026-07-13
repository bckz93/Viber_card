# Contribuer à ViberCard

*[Read this in English](./CONTRIBUTING.md)*

ViberCard est pensé pour être forké et étendu — nouvelles sources de données,
nouveaux archétypes. Ce document explique l'architecture et détaille les
fichiers exacts à toucher pour les changements les plus courants.

## Architecture

```
src-tauri/  Backend Rust, Tauri v2
  src/scanner/        un module par source de données, chacun implémente HistorySource
    claude_code.rs
    hermes.rs
    ollama.rs
    mod.rs            le trait HistorySource + ScanError
  src/models.rs        Interaction, Role, Source, ScanResult — la forme
                       partagée que chaque scanner doit produire
  src/scoring.rs       Vec<Interaction> -> PlayerStats (les 5 stats, l'archétype,
                       la punchline, les explications par stat)
  src/snapshot.rs      historique JSONL quotidien de PlayerStats
  src/commands.rs      les fonctions #[tauri::command] — le seul endroit qui
                       appelle run_scan()/scoring::compute() et branche la
                       persistance

src/        Frontend React + TypeScript
  components/CurrentDeck/
    FUTCard.tsx           la card elle-même
    RadarChart.tsx        radar mono-série (utilisé sur la card)
    ArchetypeRules.tsx    le panneau "Class Rules" — miroir lisible par un
                          humain de scoring.rs::archetype_for
    archetypeRules.ts     les données réelles (texte, condition, stats
                          d'exemple) derrière ArchetypeRules.tsx et
                          AllCards — source unique de vérité pour les deux
    StatsRecap.tsx        punchline + liste des explications par stat
    archetypeArt.ts       archétype -> illustration
    archetypeLore.ts      archétype -> texte d'ambiance
    cardTheme.ts          archétype -> {gradient, accent, shineOpacity}
    statMeta.ts           les 5 stats {key, label, icon} — source unique de
                          vérité, réutilisée partout
  components/AllCards/
    AllCards.tsx           galerie de tous les archétypes, générée à partir
                          des stats d'exemple d'archetypeRules.ts — pas de
                          vrais scans
  components/EvolutionProgress/
    EvolutionProgress.tsx     choisit le snapshot de comparaison, affiche les deltas
    EvolutionRadarChart.tsx   radar à deux séries (Avant/Maintenant)
  lib/tauri-api.ts       le *seul* fichier qui importe invoke() depuis
                         @tauri-apps/api/core — chaque commande a un unique
                         wrapper typé ici
  hooks/useImageDataUrl.ts   récupère + recadre une image en carré en
                             data: URI (utilisé pour l'avatar GitHub)
```

## Ajouter une nouvelle source d'historique

Disons que tu veux ajouter le support de Cursor, Aider, Windsurf, ou
n'importe quel autre outil qui garde un historique de chat local.

1. Crée `src-tauri/src/scanner/mon_outil.rs` :

   ```rust
   use super::{HistorySource, ScanError};
   use crate::models::{Interaction, Role, ScanResult, Source};
   use std::path::PathBuf;

   pub struct MonOutilSource {
       pub path: PathBuf,
   }

   impl MonOutilSource {
       pub fn default_path() -> Option<PathBuf> {
           dirs::home_dir().map(|h| h.join(".mon-outil").join("history"))
       }

       pub fn new(path: PathBuf) -> Self {
           Self { path }
       }
   }

   impl HistorySource for MonOutilSource {
       fn scan(&self) -> Result<ScanResult, ScanError> {
           // Parse ton format, construis un Interaction::new(...) pour
           // chaque tour user/assistant. Ignore tout ce qui n'est pas un
           // vrai message (événements système/tool/meta) plutôt que de
           // le compter.
           todo!()
       }
   }
   ```

2. Ajoute `Source::MonOutil` à l'enum dans `models.rs`.
3. Déclare `pub mod mon_outil;` dans `scanner/mod.rs`.
4. Branche-la dans `run_scan()` (`commands.rs`), en suivant exactement le
   même schéma que Claude Code / Hermes / Ollama : en cas d'échec, pousse
   une entrée dans `warnings` et continue, ne propage jamais une erreur dure
   avec `?` hors de `run_scan()` — une source cassée ne doit jamais vider
   toute la card.
5. Ajoute un test unitaire avec quelques `Interaction` fixtures (voir la fin
   de `hermes.rs` ou `snapshot.rs` pour le modèle : un fichier temporaire de
   test, une assertion sur le résultat parsé, nettoyage après coup).

**Contraintes que ton scanner doit respecter**, parce que le moteur de score
en dépend :

- `Interaction::new` a besoin d'un vrai timestamp `DateTime<Utc>`. Si ta
  source n'a pas de timestamp fiable par message (comme l'historique texte
  brut d'Ollama), documente clairement ton approximation — ça fausse
  directement NCT (% nocturne) et SPD (rythme), qui sont tous deux basés sur
  le temps. Ollama utilise actuellement le mtime du fichier pour chaque
  ligne, en l'assumant explicitement comme approximation.
- Ne mappe les rôles que vers `Role::User` / `Role::Assistant` ; jette tout
  le reste (appels d'outils, messages système). Si ta source injecte du
  texte système/boilerplate sous le rôle `user` (Hermes fait ça pour les
  déclencheurs cron/skill), filtre-le — voir la vérification `[IMPORTANT:`
  dans `hermes.rs` pour le modèle. Sinon ça gonfle VOL/EMO avec du texte que
  l'humain n'a jamais tapé.

## Comment les 5 stats sont calculées

Tout se trouve dans `src-tauri/src/scoring.rs`, calculé uniquement sur les
messages `Role::User`, sur une **fenêtre glissante de 7 jours**
(`CARD_WINDOW_DAYS` dans `commands.rs`) — pas ton historique complet. C'est
important : tout l'intérêt d'Evolution Progress est de comparer deux
semaines non-chevauchantes, ce qu'une moyenne sur toute la durée de vie
lisserait complètement.

| Stat | Ce qu'elle mesure | Comment |
|---|---|---|
| **VOL** | Volume / taille du contexte | Moyenne de mots de *prose* par message (voir plus bas), calibrée pour que ~150 mots/message donnent ~99. |
| **SPD** | Rythme des requêtes | Délai médian en secondes entre messages consécutifs dans une même session ; ≤15s ≈ 99, ≥10min ≈ 0. |
| **NCT** | Activité nocturne | % de messages envoyés hors de la plage 6h–22h (UTC — pas encore adapté au fuseau horaire, voir Limitations connues). |
| **SLF** | Autonomie / indépendance | Mélange de longueur de prose et du taux de correspondance avec une liste fixe de mots-clés "ingénierie complexe" (`refactor`, `architecture`, `test`, `async`, …). |
| **EMO** | Frustration/panique | % de messages contenant un mot-clé de frustration, `"!!"`, ou majoritairement en majuscules ("cri"). |

VOL et SLF comptent tous les deux des mots de *prose*, pas des mots bruts —
`prose_word_count()` retire le contenu entre balises ``` et tout token
individuel qui est long (>24 caractères) ou majoritairement non-alphabétique
(`is_prose_token()`), pour que le code/logs/stack traces collés ne gonflent
pas ces stats juste parce qu'ils contiennent beaucoup de mots. C'est
particulièrement sensible avec un petit échantillon : quelques stack traces
collées sur ~80 prompts pouvaient à elles seules faire exploser VOL et SLF
au maximum, même si l'utilisateur avait à peine écrit de la vraie prose.

`total_tokens` (la somme de `content.len() / 4` sur chaque interaction, les
deux rôles confondus, dans la fenêtre) n'est pas une des 5 stats
principales, mais elle conditionne un archétype — voir plus bas.

Tous les seuils sont des **choix arbitraires et assumés pour une stat card
humoristique**, pas une métrique de productivité rigoureuse — dis-le dans ta
description de PR si tu en ajustes un, et explique le *pourquoi*, pas
seulement le nouveau chiffre.

Il n'y a intentionnellement **aucun score global combiné**. Une version
précédente en avait un ; ça s'est révélé incohérent (l'activité nocturne
doit-elle compter comme "bien" ? la frustration doit-elle compter comme
"bien" quand elle est élevée, juste parce que c'est un grand nombre ?). Le
retirer était une décision délibérée — ne réintroduis pas de score combiné
sans résoudre ce problème de polarité pour chaque stat que tu y intègres.

## Comment Evolution Progress obtient sa comparaison

Volontairement **pas** dérivé de `snapshots.jsonl`. Comparer deux snapshots
déjà calculés sur une fenêtre glissante lisserait les chiffres en double
(chaque snapshot est déjà une moyenne sur ses 7 jours précédents, donc deux
snapshots espacés d'une semaine partagent encore 6 de ces 7 jours). À la
place, `get_stats_for_range(start, end)` (`commands.rs`) rescanne et
recalcule les stats à zéro, en sommant uniquement les interactions brutes
qui tombent réellement dans `[start, end)` — le même `scoring::compute()`
utilisé partout ailleurs, juste avec une fenêtre exacte différente :

- **Par défaut ("maintenant" vs "avant")** : `[maintenant-7j, maintenant)`
  vs `[maintenant-14j, maintenant-7j)` — deux semaines glissantes
  adjacentes, non-chevauchantes.
- **Une semaine calendaire précise** (`list_available_weeks()` + choix de
  l'utilisateur dans le menu déroulant) : lundi 00h00 UTC jusqu'au lundi
  suivant, pour toute semaine complète ayant au moins une interaction. La
  semaine en cours, non terminée, n'est jamais listée.

Ça implique que les logs bruts (les fichiers JSONL de Claude Code, par
exemple) doivent encore exister sur le disque pour qu'une semaine soit
comparable — si l'outil sous-jacent a depuis fait tourner/supprimé cet
historique, cette semaine disparaît silencieusement de
`list_available_weeks()`. `snapshots.jsonl` reste un registre local durable
pour ce cas de figure, simplement pas celui qu'Evolution Progress lit
aujourd'hui.

## Comment l'archétype ("classe") est déterminé

`archetype_for()` dans `scoring.rs` est **organisé en paliers mutuellement
exclusifs, pas une liste de priorité.** La condition de chaque palier exclut
déjà tout ce qui est capté par les paliers au-dessus, donc un seul archétype
peut jamais correspondre à un jeu de stats donné — quel archétype "gagne"
n'est jamais un effet de bord de l'ordre. Ça a remplacé une ancienne liste
plate de règles `(condition, nom)` indépendantes qui *ressemblait* à une
liste de priorité mais avait en réalité des conditions qui se chevauchaient
silencieusement : `Token Exterminator` (`SLF>80 && VOL>80`) et
`Self-Reliant Sage` (`SLF>80 && EMO<=30`) matchaient tous les deux dès que
VOL et EMO étaient dans la bonne plage en même temps, et comme Exterminator
arrivait en premier dans la liste, Sage devenait inatteignable pour tout
profil verbeux — un vrai bug, pas une hypothèse. Si tu ajoutes une règle,
préserve cette exclusivité mutuelle ; ne reviens pas à une liste plate.

Les paliers, dans l'ordre :

1. **Moment de la journée** (NCT) — `NCT > 60` → `Nocturnal Panic Coder`
   (EMO > 50) ou `Nocturnal Warrior` (EMO ≤ 50). Une seule stat extrême
   (NCT) ne suffit jamais à elle seule ; EMO distingue un couche-tard calme
   de quelqu'un qui panique à 3h du matin.
2. **Frustration**, pour les profils non-nocturnes — `EMO > 60` →
   `Emo-Driven Coder`.
3. **Le quadrant autonomie-vs-verbosité** (SLF × VOL), pour les profils
   calmes/non-nocturnes :
   - `SLF > 80 && VOL > 80` → `Token Exterminator`, mais *seulement* si
     `total_tokens > TOKEN_EXTERMINATOR_THRESHOLD` (200 000) — l'archétype
     tire son nom des tokens, donc être verbeux et autonome ne suffit pas à
     lui seul. En dessous de ce seuil, ça se résout vers celle des deux
     stats (SLF ou VOL) la plus marquée (`Self-Reliant Sage` ou
     `The Novelist`) plutôt que de retomber vers un palier suivant — deux
     stats au maximum ne doivent jamais finir "Balanced".
   - `SLF > 80 && VOL ≤ 80` → `Self-Reliant Sage`.
   - `VOL > 80 && SLF ≤ 80` → `The Novelist`.
4. **Rythme**, seulement pour la cellule restante du quadrant (`SLF ≤ 80 &&
   VOL ≤ 80`) — `SPD > 80` → `Spam Cannon` (VOL < 40) ou
   `Rapid-Fire Debugger` (VOL ≥ 40).
5. **Repli** — `Balanced Vibe Coder`, atteint seulement une fois que VOL,
   SLF, SPD, NCT et EMO sont *tous* en dessous de leur seuil "extrême". Si
   tu ajoutes une condition qui peut être vraie ici alors qu'une stat est
   encore au maximum, tu as réintroduit le bug "tout retombe sur Balanced".

## Ajouter un nouvel archétype

Il n'y a plus de simple liste plate à laquelle ajouter une ligne — il faut
décider dans quel palier ton archétype se place, et vérifier que sa
condition ne chevauche pas une condition existante dans ce palier (ou, si
ça doit être le cas, que le chevauchement est résolu explicitement plutôt
que par un effet d'ordre accidentel — voir plus haut). Une fois que tu sais
où il va, ces fichiers sont tous indexés par la même chaîne exacte :

1. **`src-tauri/src/scoring.rs`**, `archetype_for()` — ajoute la branche
   dans le bon palier.
2. **Même fichier**, `punchline_for()` — ajoute une blague d'une ligne pour
   la branche correspondante.
3. **Même fichier**, le bloc `#[cfg(test)] mod tests` — ajoute un cas qui
   vérifie que la nouvelle combinaison résout bien vers ton nouvel
   archétype, *et* un cas qui vérifie qu'une combinaison voisine résout
   toujours vers ce qu'elle résolvait avant (le type de test de non-
   régression qui a permis de détecter le bug Sage/Exterminator).
4. **`src/components/CurrentDeck/archetypeRules.ts`**, `ARCHETYPE_RULES` —
   la même condition, écrite en texte lisible (ex. `"NCT > 60 and EMO ≤
   50"`), en incluant ce qu'elle exclut des paliers au-dessus, plus des
   `exampleStats` qui tombent confortablement dans la zone du nouvel
   archétype (pas sur une frontière de seuil). Cette liste unique alimente à
   la fois le panneau "Class Rules" (`ArchetypeRules.tsx`) et la galerie
   "All Cards" (`AllCards.tsx`) — il n'y a par contre aucune source de
   vérité partagée entre Rust et ce fichier ; un commentaire en haut du
   fichier existe spécifiquement pour te rappeler de les garder synchronisés.
5. **`src/components/CurrentDeck/archetypeLore.ts`**, `ARCHETYPE_LORE` — une
   courte phrase d'ambiance, façon Pokédex.
6. **`src/components/CurrentDeck/cardTheme.ts`**, `ARCHETYPE_THEME` — une
   entrée `{ gradient, accent, shineOpacity }` qui correspond à l'ambiance
   de l'archétype (entrées existantes : panique = rouge, nuit calme = bleu
   marine, ruée vers l'or = ambre, emo = rose/violet, sage = sarcelle,
   romancier = sépia, détective = bleu acier, équilibré = irisé doux —
   choisis quelque chose de distinct de tout ça).
7. **Illustration de la card** — commande ou génère une illustration
   (les existantes font ~370×230, sans texte/labels de debug intégrés,
   coins transparents-safe — voir `src/assets/archetypes/*.png` comme
   référence), enregistre-la là, puis importe-la et enregistre-la dans
   `src/components/CurrentDeck/archetypeArt.ts`.

Aucune de ces étapes ne fait planter l'app si elle est oubliée — une entrée
manquante retombe simplement, silencieusement, sur rien (pas d'illustration)
ou sur une valeur par défaut (thème). C'est exactement pour ça qu'une PR qui
ajoute un archétype doit toucher les sept d'un coup ; une classe à moitié
ajoutée a l'air cassée, pas absente.

## Persistance

Tout est en JSON simple sous le dossier de données de l'OS
(`dirs::data_dir()` — `~/.local/share/vibercard/` sous Linux, l'équivalent
sur les autres plateformes) :

- **`snapshots.jsonl`** — une ligne `{ taken_at, stats }` ajoutée par jour
  (au plus une fois par jour, dédupliquée). Un registre local durable,
  indépendant de la rétention de logs propre aux outils sous-jacents —
  **pas** ce qu'Evolution Progress lit (voir plus haut).

Pas de SQLite, pas de base de données externe pour les données propres de
ViberCard — garde ça ainsi sauf raison sérieuse. Le principe : un utilisateur
doit pouvoir faire `cat`, inspecter, ou sauvegarder ses propres données sans
aucun outil.

## Frontière frontend/backend

`src/lib/tauri-api.ts` est le *seul* fichier censé importer `invoke` depuis
`@tauri-apps/api/core`. Chaque commande Tauri a exactement un wrapper typé
là-dedans. N'appelle jamais `invoke()` directement depuis un composant — ça
permet de retrouver par grep, en un seul endroit, toute la surface de
commandes (et ce que le côté Rust doit supporter).

## Tests

- **Backend** : `cd src-tauri && cargo test` — tests unitaires par module
  (parsing des scanners, seuils de scoring, aller-retours de persistance des
  snapshots). `cargo run --example inspect` lance un scan complet sur tes
  vraies données locales et affiche un résumé — pratique pour juger un
  changement de scoring sur de l'historique réel plutôt que sur de simples
  fixtures.
- **Frontend** : `npm run build` (vérification de types tsc + build Vite)
  est le principal filtre de correction aujourd'hui. Il n'y a pas encore de
  suite de tests de composants — une contribution qui en ajoute une (Vitest
  + Testing Library) est bienvenue.
- **Manuel** : `npm run tauri dev` et regarde vraiment la card. Tout ce qui
  touche la mise en page, un archétype, ou le radar chart doit être vérifié
  visuellement, pas juste par le typecheck.

## Limitations connues / bonnes premières contributions

- **NCT n'est calculé qu'en UTC**, pas adapté au fuseau horaire — un
  couche-tard en UTC+9 peut être noté comme s'il travaillait en horaires de
  bureau. Convertir vers l'heure locale dans `score_nct` (et dans le texte
  d'explication du frontend) serait une PR solide et bien délimitée.
- **Les listes de mots-clés frustration/complexité sont un mélange
  français/anglais** et plutôt réduites (`FRUSTRATION_KEYWORDS`,
  `COMPLEXITY_KEYWORDS` dans `scoring.rs`). La vraie frustration de
  quelqu'un qui ne parle ni français ni anglais peut passer inaperçue.
  Étendre ou internationaliser ces listes est bienvenu — garde-les comme de
  simples constantes `&[&str]`, pas besoin d'un fichier de config pour ça.
- **Ollama n'a pas de timestamp par message** — sa contribution à NCT/SPD
  est une approximation documentée (mtime du fichier). Si Ollama propose un
  jour un format d'historique plus riche, `scanner/ollama.rs` est le seul
  fichier à modifier.
- **`is_prose_token()` est une heuristique bon marché, pas une vraie
  détection de langage** — elle repère le contenu collé par la longueur des
  tokens/le ratio de lettres (stack traces façon Java avec des points,
  chemins de fichiers avec un numéro de ligne, hachages hexadécimaux), mais
  un collage court et riche en lettres (une ligne `File "..."` d'une
  traceback Python, par exemple) peut encore passer au travers et compter
  dans VOL/SLF. Un classifieur plus malin (par exemple des heuristiques au
  niveau de la ligne plutôt que du token) est une PR bienvenue, tant que ça
  reste une heuristique rapide et sans dépendance — pas de bibliothèque NLP
  pour une stat card humoristique.

## Pull requests

1. Fork, branche depuis `main`.
2. Garde les PR bien délimitées — un nouvel archétype est une PR, une
   nouvelle source de données en est une autre, un ajustement de seuil de
   scoring en est une troisième. Ne mélange pas des changements sans
   rapport entre eux.
3. Lance `cargo test` et `npm run build` avant d'ouvrir la PR.
4. Explique le *pourquoi*, pas juste le *quoi* — particulièrement pour les
   changements de scoring, puisque chaque seuil ici est un choix assumé et
   le raisonnement est ce dont les futurs contributeurs auront besoin pour
   le remettre en question.
