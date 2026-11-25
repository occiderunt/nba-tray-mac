# NBA Tracker

A macOS menu bar app that shows live NBA scores for the teams you follow. Click the
basketball in the menu bar and the day's games are right there. No window, no tabs.

Built with Tauri 2 (Rust) and TypeScript. Scores come from the public ESPN
scoreboard API, so there is no key to configure.


ICONS IN SOURCE ARE FILLER ICONS. EITHER USE THE RELEASE OR RUN `scripts/make-icons.py`.

## What it does

- Menu bar menu grouped into **LIVE NOW**, **FINAL** and **TODAY**
- Live games refresh every 30 seconds; idle days check every 2 minutes
- Settings window (tray menu -> Settings...) to search all 30 teams and follow or
  unfollow them, with team logos
- Following a team is enough to see its games: the opponent does not need to be
  followed
- Click any game to open it on ESPN, click a team to open its ESPN page
- Quiet menu bar icon (template image, adapts to light/dark menu bars)
- No dock icon: the app is an accessory, it lives only in the menu bar
- Followed teams persist to `~/Library/Application Support/NBA Tracker/teams.json`

## Development

```sh
npm install
npm run tauri        # dev build with hot reload
```

## Build

```sh
npm run bundle       # release .app + .dmg in src-tauri/target/release/bundle
```

The built app lands at `src-tauri/target/release/bundle/macos/NBA Tracker.app` and
the installer at `src-tauri/target/release/bundle/dmg/NBA Tracker_0.1.0_aarch64.dmg`.

## Tests

```sh
npm run build && node scripts/check-ui.mjs   # 13 UI checks against a mocked backend
```

## Icons

`scripts/make-icons.py` draws the app icon and the 44px menu bar template icon with
no dependencies. After changing the source art, regenerate the bundle icon set with:

```sh
python3 scripts/make-icons.py
npx tauri icon src-tauri/icons/icon.png
```
