![Deezer TUI Preview](images/preview.png)

# Deezer TUI

A lightweight Deezer client that lives in your terminal. Browse your playlists
and favorites, discover new music, search the catalog and stream tracks at up
to FLAC quality — all from your keyboard **or your mouse**.

## Features

- Browse playlists, favorites, Home feed and Explore
- Search tracks, albums, artists and playlists
- Queue with repeat, shuffle and crossfade support
- Stream at 128kbps, 320kbps or FLAC
- Album artwork (kitty and ueberzugpp get full quality)
- Discord Rich Presence and MPRIS integration
- **Full mouse support**: click to select, scroll to navigate, click the
  progress bar to seek, click the volume row to adjust it
- Themes: Deezer (default), Spotify and Ncmpcpp — press `T` to cycle
- Interface in **English** or **French**, switchable in Settings

## Controls

    Mouse      Click to select, scroll to navigate, click progress bar to seek
    Arrow Keys Navigate
    TAB        Switch focus
    Enter      Select
    P          Play / Pause
    /          Search
    T          Cycle themes
    Q          Quit

## Getting your Deezer ARL

The app authenticates with your Deezer account using an **ARL** cookie:

1. Log in to [deezer.com](https://www.deezer.com) in your browser.
2. Open **DevTools** (`F12`).
3. Go to **Application** → **Cookies** → `https://www.deezer.com`.
4. Copy the value of the cookie named `arl`.

You'll be prompted for it on first launch. It's stored locally in
`~/.deezer-tui-config.json` and never leaves your machine.

## Credits

Fork of [deezer-tui](https://github.com/Minuga-RC/deezer-tui) by
[Minuga-RC](https://github.com/Minuga-RC). This fork adds:

- Internationalization (English & French)
- Full mouse support and general UX polish