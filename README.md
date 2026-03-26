# fabric

Matrix-style terminal rain with live system metrics, written in Rust.

## Features

- Matrix-like animated rain in the terminal
- Live system info bar with rotating fields (OS, kernel, memory, uptime, network, and more)
- Multiple visual styles: `braille`, `block`, `binary`, `hex`
- Multiple color themes: `green`, `blue`, `cyan`, `yellow`, `red`, `magenta`, `orange`, `white`, `gray`
- Multiple animation programs: `rain`, `vortex`, `circuit`, `usage`
- Command prompt with tab completion
- Persistent preferences across runs

## Screenshots

![fabric screenshot 1](screenshot-1.png)
![fabric screenshot 2](screenshot-2.png)
![fabric screenshot 3](screenshot-3.png)

## Platform

fabric currently targets Linux environments (it reads metrics from `/proc` and `/etc/os-release`).

## Run

```bash
cargo run --release
```

After startup, type commands in the bottom prompt and press Enter.

## Keyboard Controls

- `Enter`: run command
- `Tab`: autocomplete command/value
- `Backspace`: edit command input
- `Esc`: quit
- `Ctrl+C`: quit

## Commands

```text
help
style [braille|block|binary|hex]
color [green|blue|cyan|yellow|red|magenta|orange|white|gray]
program [rain|vortex|circuit|usage]
p | pause
resume
clear
quit | exit | q
```

## Configuration

Preferences are saved automatically when you change `style`, `color`, or `program`.

Config file path:

- `$XDG_CONFIG_HOME/fabric/preferences.conf`
- fallback: `~/.config/fabric/preferences.conf`

File format:

```text
style=braille
color=green
program=rain
```

## Install

### AUR

```bash
yay -S xeo-fabric
```

If install fails on a fresh system with an error like:

```text
error: no matching package named 'rand' found
```

that usually means the AUR recipe is building with `cargo --frozen` before crates are fetched.
For maintainers, run `cargo fetch --locked` in `prepare()` and then keep `--frozen` in
`build()` and `check()` (see `packaging/aur/PKGBUILD` in this repo).

### From source

```bash
git clone https://github.com/xeoxaz/fabric.git
cd fabric
cargo build --release
./target/release/fabric
```

## License

MIT. See LICENSE.

## Packaging

This repository includes AUR packaging templates for `xeo-fabric` in:

- `packaging/aur/PKGBUILD`
- `packaging/aur/.SRCINFO`

The template uses a `prepare()` step with `cargo fetch --locked` and a shared
`CARGO_HOME` under `$srcdir` so installs work on fresh systems while preserving
reproducible `--frozen` builds.
