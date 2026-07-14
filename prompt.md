Build "GameDock", an open-source Android gaming platform for Linux written entirely in Rust. The goal is to make Android gaming on Linux as seamless as Steam makes PC gaming. Do not build an emulator or pirate APK downloader. Instead, build a runtime manager around Waydroid (initial backend) with a pluggable runtime abstraction for future support (Redroid, BlissOS, etc.). Provide both a CLI and a modern egui desktop application sharing the same backend.

Features to implement:

- Automatic runtime installation, initialization, updates, and health checks.
- One-click installation of APK, XAPK, APKS, and APKM packages.
- Official Google Play support by launching the user's own Play Store inside the Android runtime (no redistribution or bypassing licensing).
- Game library with icons, metadata, search, favorites, categories, and recently played.
- One-click launch with automatic runtime startup if needed.
- Desktop launcher integration for GNOME, KDE, Hyprland, XFCE, Cinnamon, and other freedesktop-compliant environments.
- Controller support (Xbox, DualSense, DualShock, Switch Pro, 8BitDo) with configurable mappings.
- Keyboard and mouse mapping profiles similar to BlueStacks for games that benefit from them.
- Automatic GameMode integration, MangoHUD support, CPU governor tuning, graphics optimization, and optional FPS overlay.
- Backup and restore of installed apps and game data.
- Automatic update checking for installed applications where possible.
- Plugin architecture for additional runtimes and future features.
- Clean layered architecture with crates for CLI, GUI, core library, runtime manager, installer, launcher, optimizer, controller manager, Linux integration, and plugin SDK.
- High-quality documentation, tests, CI, packaging for Arch Linux, Flatpak, AppImage, and Debian packages.

The project should prioritize reliability, native Linux integration, legal compliance, and an excellent user experience over attempting to replace Google Play or Android itself.

This is a project that would genuinely solve a real pain point for Linux users while staying within legal and technical boundaries. It also has room to grow into something much larger than a simple launcher.
