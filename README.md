# GameDock

An open-source Android gaming platform for Linux, written in Rust.

GameDock manages Waydroid (Android on Linux) so you can install and play Android games with one command.

## How it actually works

```
yay -Syu gamedock          # 1. Install GameDock (~5MB binary)
gamedock-cli init --gapps   # 2. Installs Waydroid + downloads Android image (~1GB)
gamedock-cli play-store     # 3. Opens Play Store, sign in, install games
gamedock-cli launch <pkg>   # 4. Play
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

Waydroid and its dependencies are listed as optdepends — GameDock will install them automatically when you run `gamedock-cli init`.

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
# Binaries: target/release/gamedock-cli, target/release/gamedock-gui
```

## First-time setup

### CLI

```bash
# With Play Store (recommended — downloads ~1GB Android image)
gamedock-cli init --gapps

# Without Play Store (lighter)
gamedock-cli init
```

### GUI

```bash
gamedock-gui
```

Shows a setup wizard on first launch. Click "Install with Play Store".

## Play a game

```bash
# Open Play Store inside Android
gamedock-cli play-store

# Or install an APK file
gamedock-cli install ~/Downloads/game.apk

# Launch any installed game
gamedock-cli launch com.supercell.clashofclans

# Check what's installed
gamedock-cli list
```

## Resource usage

**When gaming:** Waydroid container runs, uses ~200-400MB RAM + GPU.

**When not gaming:** Container auto-stops after 5 minutes idle. Zero background processes.

**GameDock itself:** Just a CLI/GUI launcher. No daemon, no background service.

## Disk space

| Component | Size | When downloaded |
|-----------|------|-----------------|
| GameDock binary | ~5MB | At install |
| Waydroid package | ~5MB | At `gamedock-cli init` |
| Android system image | ~800MB-1GB | At `gamedock-cli init --gapps` |
| Games | varies | Per game from Play Store |

Total for a typical setup: ~1-2GB + games.

## CLI commands

```bash
gamedock-cli init [--gapps]     # First-time setup
gamedock-cli status             # Check runtime status
gamedock-cli install <file>     # Install APK/XAPK/APKS/APKM
gamedock-cli launch <pkg>       # Launch a game
gamedock-cli play-store         # Open Play Store
gamedock-cli list               # List installed games
gamedock-cli search <query>     # Search games
gamedock-cli backup <pkg>       # Backup a game
gamedock-cli restore <file>     # Restore from backup
gamedock-cli controller list    # List controllers
gamedock-cli optimize           # Enable GameMode/MangoHUD
gamedock-cli system-info        # Show system info
gamedock-cli completions bash   # Shell completions
```

## Requirements

- Linux with Wayland (Wayland or XWayland)
- Kernel with binder support (most kernels from 2020+)
- ~2GB free disk space
- Internet for first setup only

## License

GPL-3.0-only
