<div align="center">
  <img src="assets/icon.png" alt="Deadlock RPC" width="120" />

  # Deadlock Rich Presence for Discord

  Show your Deadlock hero, match mode, and game phase on your Discord profile automatically. Free, open-source, native Rust binary for Windows and Linux.

  [![Latest Release](https://img.shields.io/github/v/release/HeyTariq/deadlock-rpc?&label=release)](https://github.com/HeyTariq/deadlock-rpc/releases/latest)
  [![Downloads](https://img.shields.io/github/downloads/HeyTariq/deadlock-rpc/total?)](https://github.com/HeyTariq/deadlock-rpc/releases)
  [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-blue?)](https://github.com/HeyTariq/deadlock-rpc/releases/latest)
  [![License](https://img.shields.io/github/license/HeyTariq/deadlock-rpc)](LICENSE)

</div>

## Preview

<div align="center">
  <img src="assets/demo.gif" alt="Deadlock Discord Rich Presence showing hero portrait, match mode, and game phase updating live on a Discord profile card" />

  **[Try the interactive demo](https://heytariq.github.io/deadlock-rpc/demo/)**

</div>

## Features

- Shows your hero, game mode, and what you're doing (queueing, in a match, reviewing results) on your Discord profile
- Five hero art styles to choose from
- Safe to use: it only reads Deadlock's log file and never touches the game (VAC safe)
- Opens with Deadlock and closes when you quit
- Change settings from the tray icon, no restart needed
- Optional Statlocker button so friends can check your match history

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

Then pick how you want to launch the game below.

## Launching Deadlock

Rich Presence only works when Deadlock writes its log file, which needs the `-condebug` launch option. Pick one of these ways to launch the game. Keep the extracted folder where it is, since the app writes its logs there.

### 1. Use the Deadlock RPC shortcut (recommended)

On first run the app creates a **Deadlock RPC** shortcut in its folder. Move it to your desktop and use it instead of launching Deadlock directly. It starts the game for you and closes when you quit.

> [!TIP]
> You can also add `deadlock-rpc` as a non-Steam game to launch it from your Steam library. [Steam's guide](https://help.steampowered.com/en/faqs/view/4B8B-9697-2338-40EC) explains how.

### 2. Press Play in Steam as usual

Have Steam start Deadlock RPC every time you press Play. In Steam, right-click Deadlock, open **Properties**, and set **Launch Options** to the full path of the app followed by `--launch-with-command -- %command%`:

```sh
/path/to/deadlock-rpc --launch-with-command -- %command%
```

On Windows, use the path to `deadlock-rpc.exe` and wrap it in quotes if it contains spaces. The app adds `-condebug` for you and closes with the game while `exit_when_game_closes` is on.

### 3. Launch both yourself

Add `-condebug` to Deadlock's **Launch Options** in Steam, then start Deadlock and Deadlock RPC however you like. Run the app with `--no-launch` if you don't want it to start the game.

## Configuration

The tray icon's **Settings** menu lets you toggle the most common options without restarting:

| Setting | Tray label |
|---------|------------|
| `general.launch_game_on_start` | Launch Game on Start |
| `general.exit_when_game_closes` | Exit When Game Closes |
| `general.auto_update` | Auto Update |
| `presence.show_hero_image` | Show Hero Image |
| `presence.show_match_timer` | Show Match Timer |
| `presence.show_statlocker_button` | Show Statlocker Button |
| `presence.hero_portrait_style` | Hero Portrait Style (Normal / Gloat / Critical / Icon / Full Body) |

Changes made through the tray are written to `config.toml` immediately and take effect without a restart. Select **Open Config File** in the tray menu to open the file directly in your default editor for full customization.

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
| `general.auto_update` | `false` | Install available updates automatically instead of showing the update prompt. |
| `general.game_log_poll_interval_ms` | `500` | How often (ms) to check the game log. Lower = faster updates. |
| `general.discord_update_interval_s` | `5` | How often (seconds) to refresh the Discord presence card. |

### Presence

| Key | Default | Description |
|-----|---------|-------------|
| `presence.show_hero_image` | `true` | Show the hero image and name. |
| `presence.show_match_timer` | `true` | Show how long the current match has been running. Shown as a clock, e.g. `07:42` (`1:07:42` past an hour). Added to the in-match status line, or placed with `{timer}` if `presence.status.in_match` uses that variable. |
| `presence.show_statlocker_button` | `false` | Show a "View on Statlocker" button linking to your match history. Only visible to other Discord users, not yourself. |
| `presence.hero_portrait_style` | `"normal"` | Hero portrait art style. Options: `"normal"`, `"gloat"` (celebration crop), `"critical"` (combat crop), `"icon"` (small hero icon), `"fullbody"` (Discord app asset named after the hero key, e.g. `hero_inferno`). |
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
| `presence.status.in_match` | `"In Match: {mode}"` (also supports `{timer}`) |
| `presence.status.match_location_label` | `"the Cursed Apple"` |
| `presence.status.post_match` | `"Reviewing Match Results"` |
| `presence.status.spectating` | `"Spectating a Match"` |

### Images

| Key | Default | Description |
|-----|---------|-------------|
| `images.fallback_large_image` | `"deadlock_logo"` | Large image asset when no hero is shown. |
| `images.fallback_large_image_tooltip` | `"Deadlock RPC"` | Tooltip for the large image. |
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
# Minimal presence: no hero name, no timer
[presence]
show_match_timer = false
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

## How It Works

The `-condebug` option makes Deadlock write its console output to a log file. Deadlock RPC watches that file for your hero, match mode, and game phase, then updates Discord. It doesn't read game memory, modify files, or intercept network traffic.

## CLI Flags

| Flag | Description |
|------|-------------|
| `--no-launch` | Skip launching Deadlock on startup, even if `launch_game_on_start` is enabled in config. |
| `--no-shortcut` | Skip the shortcut install prompt. Useful for automated or CI setups. |
| `--launch-with-command -- <command>` | Launch the game by running `<command>` with `-condebug` appended, instead of going through Steam. Meant for Steam launch options with `%command%`. Implies `--no-shortcut`. |
| `--generate-config` | Write a fresh `config.toml` with all defaults to the current directory, then exit. |

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
