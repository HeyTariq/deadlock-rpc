<div align="center">
  <img src="assets/icon.png" alt="Deadlock RPC" width="120" />

  # Deadlock Rich Presence for Discord

  Show your Deadlock hero, match mode, and game phase on your Discord profile automatically. Free, open-source, native Rust binary for Windows and Linux.

  [![CI](https://github.com/HeyTariq/deadlock-rpc/actions/workflows/ci.yml/badge.svg)](https://github.com/HeyTariq/deadlock-rpc/actions/workflows/ci.yml)
  [![Latest Release](https://img.shields.io/github/v/release/HeyTariq/deadlock-rpc?&label=release)](https://github.com/HeyTariq/deadlock-rpc/releases/latest)
  [![Downloads](https://img.shields.io/github/downloads/HeyTariq/deadlock-rpc/total?)](https://github.com/HeyTariq/deadlock-rpc/releases)
  [![Last Commit](https://img.shields.io/github/last-commit/HeyTariq/deadlock-rpc)](https://github.com/HeyTariq/deadlock-rpc/commits/main)
  [![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?&logo=rust)](https://www.rust-lang.org)
  [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-blue?)](https://github.com/HeyTariq/deadlock-rpc/releases/latest)

</div>

## Contents

- [Deadlock Rich Presence for Discord](#deadlock-rich-presence-for-discord)
  - [Contents](#contents)
  - [Preview](#preview)
  - [Features](#features)
  - [Installation](#installation)
    - [Windows SmartScreen](#windows-smartscreen)
  - [How It Works](#how-it-works)
  - [Configuration](#configuration)
  - [Building from Source](#building-from-source)
  - [Disclaimer](#disclaimer)

## Preview

<div align="center">
  <img src="assets/demo.gif" alt="Deadlock Discord Rich Presence showing hero portrait, match mode, and game phase updating live on a Discord profile card" />
</div>

## Features

- Live hero portrait, game phase, and match mode on your Discord profile, updating as you play
- Three portrait styles: normal, gloat, and critical
- VAC safe, read-only, zero footprint: reads only the game log, touches nothing else
- Single native binary, no runtime or dependencies required
- Tray icon for quick settings, auto-launches with Deadlock, auto-exits when you close it
- Optional Statlocker.gg button so others can view your match history from your profile

## Installation

**Windows**

1. Download `deadlock-rpc-setup-windows-x86_64.zip` from the [Releases](../../releases) page
2. Extract the zip anywhere on your PC
3. Double-click `deadlock-rpc.exe` inside the extracted folder
4. A **Deadlock RPC** shortcut is created in the folder. Move it to your desktop for easy access

> [!WARNING]
> Windows may show a **"Windows protected your PC"** warning. This happens because the app is not signed, not because it is harmful. Click **More info**, then **Run anyway**. If you'd rather verify the code yourself, [build from source](#building-from-source).

**Linux**

1. Download `deadlock-rpc-setup-linux-x86_64.zip` from the [Releases](../../releases) page
2. Extract the zip and open a terminal in that folder
3. Run `chmod +x deadlock-rpc && ./deadlock-rpc`

**After first run**

From now on, launch Deadlock through the **Deadlock RPC** shortcut instead of directly. This is what activates Rich Presence. Keep the extracted folder in place, the app writes its logs there.

If you prefer to launch Deadlock your own way, add `-condebug` to Deadlock's launch options in Steam (right-click Deadlock in your library, Properties, Launch Options). This generates the log file the app needs to function.

> [!TIP]
> You can add the Deadlock RPC executable as a non-Steam game so you can launch it straight from your Steam library. [Steam's guide](https://help.steampowered.com/en/faqs/view/4B8B-9697-2338-40EC) explains how.

## How It Works

Deadlock RPC launches the game with a flag that makes Deadlock write its console output to a log file. The app watches that file and parses it for hero selection, game phase, and match mode, then pushes the result to Discord in real time. No memory is read, no files are modified, no traffic is intercepted. VAC safe by design.

## Configuration

The tray icon's **Settings** menu lets you toggle the most common options without restarting:

| Setting | Tray label |
|---------|------------|
| `general.launch_game_on_start` | Launch Game on Start |
| `general.exit_when_game_closes` | Exit When Game Closes |
| `presence.show_hero_image` | Show Hero Image |
| `presence.show_statlocker_button` | Show Statlocker Button |
| `presence.hero_portrait_style` | Hero Portrait Style (Normal / Gloat / Critical) |

Changes made through the tray are written to `config.toml` immediately and take effect without a restart. Select **Open Config File** in the Settings menu to open the file directly in your default editor for full customization.

A **`config.toml`** is included in the release zip next to the executable with all options and their defaults. Any key you omit falls back to its default, and any key added in a new release is automatically written to your file. Changes to options not available in the tray take effect on the next launch.

> [!NOTE]
> If your config file is corrupt or causes issues, delete it and launch the application again to regenerate it with all defaults.

When a release renames or restructures config keys, the release includes a migration that automatically updates your config on the next launch, no manual re-apply needed.

<details>
<summary>All config options</summary>

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

</details>

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
