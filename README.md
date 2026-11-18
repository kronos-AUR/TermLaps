# TermLaps

A modern terminal for Linux, written in Rust (GTK 4 / libadwaita / VTE).

## Features

- **Smart Ctrl+C**: if text is selected with the mouse, `Ctrl+C` copies the
  selection to the clipboard. If nothing is selected, `Ctrl+C` is passed
  through to the child process as `0x03`, which the line discipline turns
  into `SIGINT` — i.e. it kills the foreground process, like a classic
  terminal. `Ctrl+V` pastes the clipboard (`Ctrl+Shift+C` / `Ctrl+Shift+V`
  keep working too, handled by VTE).
- **Tabs**: several terminals in tabs (`Ctrl+Shift+T` opens a new one).
- **Split view**: split the focused pane into two side-by-side terminals
  (`Ctrl+Shift+S`), recursively. `Ctrl+Shift+W` closes the focused pane (or
  the tab if it is the last one).
- **Configurable profiles**: each profile defines the command to run
  (e.g. `/bin/bash`) and a color theme. Built-in themes: **Flexoki**,
  **Monokai**, **Solarized Light**, **Solarized Dark**.

## Installation

### Install script (Arch, Debian, Fedora and derivatives)

```bash
curl -fsSL https://raw.githubusercontent.com/kronos-AUR/TermLaps/main/install.sh | bash
```

The script detects your distribution, installs the system dependencies,
installs Rust via rustup if `cargo` is missing, builds in release mode and
installs the binary plus a desktop entry (default prefix `~/.local`, or
`/usr/local` when run as root). Options: `--prefix DIR`, `--help`.

### Manual installation

System dependencies:

| Distribution | Packages |
|---|---|
| Arch, CachyOS, Manjaro, EndeavourOS… | `sudo pacman -S base-devel pkgconf gtk4 libadwaita vte4` |
| Debian 12+, Ubuntu 23.10+, Mint, Pop!_OS… | `sudo apt-get install build-essential pkg-config libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev` |
| Fedora, Nobara, RHEL… | `sudo dnf install gcc pkgconf-pkg-config gtk4-devel libadwaita-devel vte291-gtk4-devel` |

Then build and run:

```bash
git clone https://github.com/kronos-AUR/TermLaps.git
cd TermLaps
cargo build --release
./target/release/termlaps
```

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+T` | New tab (default profile) |
| `Ctrl+Shift+S` | Split the focused pane in two |
| `Ctrl+Shift+W` | Close the focused pane (or its tab) |
| `Ctrl+Page_Down` / `Ctrl+Page_Up` | Next / previous tab |
| `Ctrl+C` | Copy the selection, otherwise `SIGINT` to the process |
| `Ctrl+V` | Paste the clipboard |
| `Ctrl+Shift+C` / `Ctrl+Shift+V` | Copy / paste (classic VTE shortcuts) |
| `Ctrl+Q` | Quit |

The menu (☰ icon, top left) opens a new tab with any of the configured
profiles.

## Configuration

On first start, a default configuration file is written to:

```
$XDG_CONFIG_HOME/termlaps/config.toml   # usually ~/.config/termlaps/config.toml
```

Example (see also `config.example.toml`):

```toml
[settings]
font = "Monospace 12"
scrollback_lines = 10000

[[profiles]]
name = "Flexoki"
command = "/bin/bash"            # or ["/usr/bin/fish", "-l"]
theme = "flexoki"                # flexoki | monokai | solarized-light | solarized-dark

[[profiles]]
name = "Solarized zsh"
command = ["/bin/zsh", "-l"]
theme = "solarized-dark"
working_directory = "/home/kronos"
env = ["TERM=xterm-256color"]
font = "JetBrains Mono 11"
```

Each tab — and each split pane — inherits the profile of the pane it comes
from; a new tab uses the last chosen profile (or the first one in the list).

## License

This project is published **without a license**: no `LICENSE` file is
provided and all rights are reserved.
