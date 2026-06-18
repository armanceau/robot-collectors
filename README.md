# Robot Collectors

Simulation de collecte de ressources en terminal, développée en Rust avec [Ratatui](https://ratatui.rs/).

## Lancer le projet

### Prérequis

- [Rust](https://rustup.rs/) (édition 2024)

### Démarrage

```bash
cargo run
```

La carte est générée aléatoirement à chaque lancement. Appuyez sur **n'importe quelle touche** pour quitter.

## Simulation

Des robots autonomes explorent une carte générée procéduralement et collectent des ressources.

### Carte

La carte est générée avec du **bruit de Perlin** pour placer les obstacles. Les ressources sont placées aléatoirement avec des quantités entre 50 et 200 unités.

### Robots

| Type | Symbole | Couleur | Comportement |
|------|---------|---------|--------------|
| Éclaireur | `x` | Rouge | Explore aléatoirement, découvre et partage les ressources |
| Collecteur | `o` | Magenta | Navigue vers les ressources connues, collecte, retourne à la base |

### Légende visuelle

| Symbole | Couleur | Signification |
|---------|---------|---------------|
| `O` | Cyan clair | Obstacle |
| `E` | Vert | Source d'énergie |
| `C` | Magenta clair | Gisement de cristaux |
| `#` | Vert clair | Base centrale |
| `.` | Gris foncé | Case vide |

## Architecture

Le projet utilise les **threads** et les **canaux `mpsc`** de Rust pour une architecture concurrente :

- **2 threads éclaireurs** — explorent la carte et envoient les découvertes au coordinator
- **2 threads collecteurs** — naviguent via BFS vers les ressources connues, collectent, retournent à la base
- **1 thread coordinator** — reçoit les messages, met à jour l'état partagé (`known_resources`, totaux)
- **Thread principal** — boucle Ratatui qui lit l'état partagé (`Arc<RwLock<SimState>>`) et rafraîchit l'affichage

## Structure du projet

```
src/
├── main.rs           # Interface Ratatui + boucle de rendu
├── simulation.rs     # Logique robots, threads, communication
├── map_generation.rs # Génération de carte (Perlin noise)
└── lib.rs            # Exports publics
```
