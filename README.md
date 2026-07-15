# GameDock

An open-source Android gaming platform for Linux, written in Rust.

GameDock manages Waydroid (Android on Linux) so you can install and play Android games with one command.

## How it actually works

```
yay -Syu gamedock          # 1. Install GameDock (~5MB binary)
gamedock init --gapps   # 2. Installs Waydroid + downloads Android image (~1GB)
gamedock play-store     # 3. Opens Play Store, sign in, install games
gamedock launch <pkg>   # 4. Play
```

**Step 2 is the big one.** First run downloads:
- Waydroid package manager (~5MB, via your distro's package manager)
- Android system image with Google Play (~800MB-1GB, cached in `~/.local/share/waydroid/`)

After that, everything is local. No internet needed to launch games.

## Install

### Arch Linux (AUR)

```bash
yay -S gamedock
# or: paru -S gamedock
```

Waydroid and its dependencies are listed as optdepends — GameDock will install them automatically when you run `gamedock init`.

### Flatpak (any distro)

```bash
flatpak-builder --user --install --force-clean build-dir packaging/flatpak/io.github.gamedock.yml
```

### Debian/Ubuntu/Mint/Pop/Kali

```bash
dpkg -i gamedock_0.1.0_amd64.deb
```

### From source

```bash
cargo build --release
# Binaries: target/release/gamedock, target/release/gamedock-gui
```

## First-time setup

### CLI

```bash
# With Play Store (recommended — downloads ~1GB Android image)
gamedock init --gapps

# Without Play Store (lighter)
gamedock init
```

### GUI

```bash
gamedock-gui
```

Shows a setup wizard on first launch. Click "Install with Play Store".

## Play a game

```bash
# Open Play Store inside Android
gamedock play-store

# Or install an APK file
gamedock install ~/Downloads/game.apk

# Launch any installed game
gamedock launch com.supercell.clashofclans

# Check what's installed
gamedock list
```

## Resource usage

**When gaming:** Waydroid container runs, uses ~200-400MB RAM + GPU.

**When not gaming:** Container auto-stops after 5 minutes idle. Zero background processes.

**GameDock itself:** Just a CLI/GUI launcher. No daemon, no background service.

## Disk space

| Component | Size | When downloaded |
|-----------|------|-----------------|
| GameDock binary | ~5MB | At install |
| Waydroid package | ~5MB | At `gamedock init` |
| Android system image | ~800MB-1GB | At `gamedock init --gapps` |
| Games | varies | Per game from Play Store |

Total for a typical setup: ~1-2GB + games.

## CLI commands

```bash
gamedock init [--gapps]     # First-time setup
gamedock status             # Check runtime status
gamedock install <file>     # Install APK/XAPK/APKS/APKM
gamedock launch <pkg>       # Launch a game
gamedock play-store         # Open Play Store
gamedock list               # List installed games
gamedock search <query>     # Search games
gamedock backup <pkg>       # Backup a game
gamedock restore <file>     # Restore from backup
gamedock controller list    # List controllers
gamedock optimize           # Enable GameMode/MangoHUD
gamedock system-info        # Show system info
gamedock completions bash   # Shell completions
```

## Requirements

- Linux with Wayland (Wayland or XWayland)
- Kernel with binder support (most kernels from 2020+)
- ~2GB free disk space
- Internet for first setup only

## License

GPL-3.0-only
