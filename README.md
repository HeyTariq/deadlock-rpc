<div align="center">
  <img src="assets/icon.png" alt="Deadlock RPC" width="120" />

  # Deadlock Rich Presence for Discord

  Show your Deadlock hero, match mode, and game phase on your Discord profile automatically. Free, open-source, native Rust binary for Windows and Linux. No runtime required.

  [![Latest Release](https://img.shields.io/github/v/release/HeyTariq/deadlock-rpc?&label=release)](https://github.com/HeyTariq/deadlock-rpc/releases/latest)
  [![Downloads](https://img.shields.io/github/downloads/HeyTariq/deadlock-rpc/total?)](https://github.com/HeyTariq/deadlock-rpc/releases)
  [![License](https://img.shields.io/github/license/HeyTariq/deadlock-rpc?)](LICENSE)
  [![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?&logo=rust)](https://www.rust-lang.org)
  [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-blue?)](https://github.com/HeyTariq/deadlock-rpc/releases/latest)

</div>

## Contents

- [Preview](#preview)
- [Features](#features)
- [Installation](#installation)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
  - [General](#general)
  - [Presence](#presence)
  - [Per-phase status strings](#per-phase-status-strings)
  - [Images](#images)
  - [Template variables](#template-variables)
  - [Examples](#examples)
- [Building from Source](#building-from-source)
- [Disclaimer](#disclaimer)

## Preview

<div align="center">
  <img src="assets/demo.gif" alt="Deadlock Discord Rich Presence showing hero portrait, match mode, and game phase updating live on a Discord profile card" />
</div>

## Features

- **Hero portrait display:** shows hero name and portrait on your Discord presence card with three art styles (normal, gloat, critical)
- **Game phase tracking:** Hideout, In Queue, Match Intro, In Match, Post Match, Spectating
- **Match mode detection:** Standard, Street Brawl, Training Range, and more
- **Read-only and VAC safe:** reads only the game log file, no memory access, no code injection, no game files modified
- **Auto-launch and auto-exit:** launches Deadlock automatically and closes when the game does
- **Native Rust binary:** no Python, no Node.js, no runtime dependencies, single self-contained executable
- **Statlocker.gg button:** optional clickable button on your Deadlock Discord presence linking to your match history
- **Deeply customizable:** presence text, timer, hero portrait style, poll rate, and more via `config.toml`

## Installation

1. Go to the [Releases](../../releases) page
2. Download and extract the zip for your platform:
   - **Windows:** `deadlock-rpc-setup-windows-x86_64.zip`
   - **Linux:** `deadlock-rpc-setup-linux-x86_64.zip`
3. Run the binary inside the extracted folder:
   - **Windows:** double-click `deadlock-rpc.exe`
   - **Linux:** `chmod +x deadlock-rpc && ./deadlock-rpc`
4. A shortcut named **Deadlock RPC** is created in the extracted folder. Move it to your desktop or wherever is convenient
5. Deadlock launches with Rich Presence active

From this point forward, use the **Deadlock RPC** shortcut instead of launching Deadlock directly. Be sure to keep the executable within the extracted folder as it writes logs to the `logs/` directory.

> [!TIP]
> Add the Deadlock RPC executable as a non-Steam game in your Steam launcher so you can launch it directly from your library. See [Steam's guide](https://help.steampowered.com/en/faqs/view/4B8B-9697-2338-40EC) for instructions.

### Windows SmartScreen

Windows may show a **"Windows protected your PC"** warning on first run. This is because the executable is unsigned, not because it contains malware. Click **More info → Run anyway** to proceed, or [build from source](#building-from-source) to verify the binary yourself.

## How It Works

Deadlock RPC launches the game with the `-condebug` flag, which causes Deadlock to write its console output to a log file. The app monitors this file in real time, parsing log lines to detect hero selection, map loads, game phase transitions, and match mode. Detected state is pushed to Discord via its IPC protocol, updating your Discord status live as you play.

No game memory is read, no files are modified, and no network traffic is intercepted. It is entirely read-only with respect to the game, making it VAC safe by design.

## Configuration

A **`config.toml`** is included in the release zip next to the executable with all options and their defaults. Edit it with any text editor — changes take effect on the next launch. Any key you omit falls back to its default, and any key added in a new release is automatically written to your file with its default value.

> [!NOTE]
> If your config file is corrupt or causes issues, delete it and launch the application again to regenerate it with all defaults.

When a release renames or restructures config keys, the release includes a migration that automatically updates your config on the next launch, no manual re-apply needed.

### General

| Key | Default | Description |
|-----|---------|-------------|
| `general.launch_game_on_start` | `true` | Launch Deadlock on startup. |
| `general.exit_when_game_closes` | `true` | Exit when the game closes. |
| `general.game_log_poll_interval_ms` | `500` | How often (ms) to check the game log. Lower = faster updates. |
| `general.discord_update_interval_s` | `5` | How often (seconds) to refresh the Discord presence card. |

### Presence

| Key | Default | Description |
|-----|---------|-------------|
| `presence.show_elapsed_timer` | `true` | Show the elapsed time counter. |
| `presence.show_hero_image` | `true` | Show the hero image and name. |
| `presence.show_statlocker_button` | `false` | Show a "View on Statlocker" button linking to your match history. Only visible to other Discord users, not yourself. |
| `presence.hero_portrait_style` | `"normal"` | Hero portrait art style. Options: `"normal"`, `"gloat"` (celebration crop), `"critical"` (combat crop). |
| `presence.details_with_hero` | `"Playing as {hero}"` | Top line when a hero is known. |
| `presence.details_without_hero` | `"{phase}"` | Top line when no hero is known. |

### Per-phase status strings

| Key | Default |
|-----|---------|
| `presence.status.game_not_running` | `"Not Running"` |
| `presence.status.in_main_menu` | `"Browsing the Main Menu"` |
| `presence.status.in_hideout` | `"In the Hideout"` |
| `presence.status.in_matchmaking` | `"Searching for a Match..."` |
| `presence.status.loading_into_match` | `"{mode} - Loading into Match"` |
| `presence.status.in_match` | `"In Match: {mode}"` |
| `presence.status.match_location_label` | `"the Cursed Apple"` |
| `presence.status.post_match` | `"Reviewing Match Results"` |
| `presence.status.spectating` | `"Spectating a Match"` |

### Images

| Key | Default | Description |
|-----|---------|-------------|
| `images.fallback_large_image` | `"deadlock_logo"` | Large image asset when no hero is shown. |
| `images.fallback_large_image_tooltip` | `"Deadlock"` | Tooltip for the large image. |
| `images.corner_image` | `"deadlock_logo"` | Small corner overlay image asset. |
| `images.corner_image_tooltip` | `"Deadlock"` | Tooltip for the small corner image. |

### Template variables

| Variable | Available in | Value |
|----------|-------------|-------|
| `{hero}` | `details_with_hero`, `in_hideout` | Hero display name, e.g. `Vindicta` |
| `{phase}` | `details_without_hero` | Phase label, e.g. `Post Match` |
| `{mode}` | `loading_into_match`, `in_match` | Match mode, e.g. `Standard Match` |
| `{location}` | `in_match` | Value of `match_location_label` |

### Examples

```toml
# Minimal presence — no hero name, no timer
[presence]
show_elapsed_timer = false
details_with_hero  = "Playing Deadlock"
details_without_hero = "Playing Deadlock"

# Custom in-match status
[presence.status]
in_match = "Grinding {mode}"
in_matchmaking = "Waiting for a game..."

# Keep the app open after the game closes
[general]
exit_when_game_closes = false
```

## Building from Source

Requires [Rust](https://rustup.rs) stable.

```bash
git clone https://github.com/HeyTariq/deadlock-rpc.git
cd deadlock-rpc
cargo build --release
./target/release/deadlock-rpc
```

## Disclaimer

Not affiliated with, endorsed by, or connected to Valve Corporation. **Deadlock**, all hero names, images, and related assets are the property of **Valve Corporation**. Hero images displayed in Discord are sourced from the community-maintained [Deadlock API](https://deadlock-api.com) and remain the property of Valve. This project does not distribute or claim ownership of any Valve assets.
