# MudForge

**A modern, cross-platform client for Multi-User Dungeons (MUDs).**
Connect to any MUD with a fast terminal, full GMCP/MSDP support, an automapper,
Lua plugins, triggers/aliases/timers, and a polished UI — on desktop or in your browser.

> This is the public home of MudForge — **downloads, release notes, and issue tracking**.
> The app source is maintained privately; this repo is where you grab builds and report bugs.

---

## ⬇️ Download

Grab the latest build for your platform from the **[Releases page](../../releases/latest)**:

| Platform | File |
|----------|------|
| **macOS (Apple Silicon)** | `MudForge_*_aarch64.dmg` |
| **macOS (Intel)** | `MudForge_*_x64.dmg` |
| **Windows** | `MudForge_*_x64-setup.exe` (or `.msi`) |
| **Linux** | `.AppImage`, `.deb`, or `.rpm` |

The macOS builds are **code-signed and notarized**, and the Windows/macOS apps
**auto-update** themselves from this repo's releases — you'll be prompted when a new
version is available.

Prefer the browser? MudForge also runs as a web app — no install required.

## ✨ Features

- **Fast terminal** with full ANSI/truecolor, ligatures, and a wide range of fonts
- **GMCP & MSDP** protocol support (room info, character vitals, group, mapping, …)
- **Automapper** with pathfinding, speedwalking, and **custom exits** (link rooms via
  commands like `ride bucket` or `open door;;wait(3);;enter portal`)
- **Lua plugin system** with a built-in editor, live syntax checking, widgets, and a rich API
- **Triggers, aliases, timers, macros, variables** — full automation
- **Themes**, including **High Contrast** accessibility themes
- **World files** — export/import your entire setup (`.mfw`), maps and all
- Multi-session, mobile-friendly, and desktop (Windows/macOS/Linux) or web

## 🐛 Reporting bugs & requesting features

Please use the **[Issues tab](../../issues/new/choose)** — there are templates for bug
reports and feature requests. For questions, ideas, and showing off your setups, head to
**[Discussions](../../discussions)**.

When filing a bug, the **app version** (Settings → About, or the title bar) and your
platform help a lot.

## 🔗 Related

- **Plugins:** [MudForge-Plugins](https://github.com/Coffee-Nerd/MudForge-Plugins)
- **Docs:** [mudforge-docs](https://github.com/Coffee-Nerd/mudforge-docs)

---

<sub>MudForge is built with Next.js + Tauri. Desktop builds are signed, notarized, and auto-updating.</sub>
