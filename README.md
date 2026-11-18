# TermLaps

Un terminal moderne pour Linux, écrit en Rust (GTK 4 / libadwaita / VTE).

## Fonctionnalités

- **Ctrl+C intelligent** : si du texte est sélectionné à la souris, `Ctrl+C`
  copie la sélection dans le presse-papiers. Si rien n'est sélectionné,
  `Ctrl+C` est transmis au processus (0x03 → `SIGINT`) et tue donc le
  processus au premier plan, comme dans un terminal classique.
- **Ctrl+V** colle le presse-papiers (`Ctrl+Maj+C` / `Ctrl+Maj+V` continuent
  de fonctionner, gérés par VTE).
- **Onglets** : plusieurs terminaux dans des onglets (`Ctrl+Maj+T` pour en
  ouvrir un nouveau).
- **Vue scindée** : découper le panneau actif en deux terminaux côte à côte
  (`Ctrl+Maj+S`), récursivement.
- **Profils configurables** : chaque profil définit la commande lancée
  (ex. `/bin/bash`) et un thème de couleurs. Thèmes intégrés : **Flexoki**,
  **Monokai**, **Solarized Light**, **Solarized Dark**.

## Raccourcis clavier

| Raccourci | Action |
|---|---|
| `Ctrl+Maj+T` | Nouvel onglet (profil par défaut) |
| `Ctrl+Maj+S` | Scinder le panneau actif en deux |
| `Ctrl+Maj+W` | Fermer le panneau actif (ou l'onglet s'il est seul) |
| `Ctrl+Page_Bas` / `Ctrl+Page_Haut` | Onglet suivant / précédent |
| `Ctrl+C` | Copier la sélection, sinon `SIGINT` au processus |
| `Ctrl+V` | Coller le presse-papiers |
| `Ctrl+Maj+C` / `Ctrl+Maj+V` | Copier / coller (raccourcis VTE classiques) |
| `Ctrl+Q` | Quitter |

Le menu (icône ☰ en haut à gauche) permet d'ouvrir un nouvel onglet avec
chacun des profils définis dans la configuration.

## Configuration

Au premier lancement, un fichier de configuration par défaut est écrit dans :

```
$XDG_CONFIG_HOME/termlaps/config.toml   # soit ~/.config/termlaps/config.toml
```

Exemple (voir aussi `config.example.toml`) :

```toml
[settings]
font = "Monospace 12"
scrollback_lines = 10000

[[profiles]]
name = "Flexoki"
command = "/bin/bash"            # ou ["/usr/bin/fish", "-l"]
theme = "flexoki"                # flexoki | monokai | solarized-light | solarized-dark

[[profiles]]
name = "Zsh solaire"
command = ["/bin/zsh", "-l"]
theme = "solarized-dark"
working_directory = "/home/kronos"
env = ["TERM=xterm-256color"]
font = "JetBrains Mono 11"
```

Chaque onglet et chaque panneau scindé hérite du profil du panneau qu'il
remplace ; un nouvel onglet utilise le dernier profil choisi (ou le premier
profil de la liste).

## Compilation

Dépendances système (ex. Arch/CachyOS) :

```
sudo pacman -S gtk4 libadwaita vte4 base-devel
```

Puis :

```
cargo build --release
./target/release/termlaps
```

## Licence

Ce projet est publié **sans licence** : aucun fichier `LICENSE` n'est fourni
et tous les droits sont réservés.
