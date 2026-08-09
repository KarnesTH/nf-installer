<p align="center">
  <img src="assets/logo.svg" alt="nf-installer" width="120">
</p>

<h1 align="center">nf-installer</h1>

<p align="center">Install Nerd Fonts from your terminal, without the copy-and-cache dance.</p>

<p align="center">
  <a href="https://github.com/KarnesTH/nf-installer/actions/workflows/ci.yml">
    <img src="https://github.com/KarnesTH/nf-installer/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/KarnesTH/nf-installer/releases/latest">
    <img src="https://img.shields.io/github/v/release/KarnesTH/nf-installer" alt="Latest release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/KarnesTH/nf-installer" alt="License">
  </a>
  <img src="https://img.shields.io/badge/rust-2024%20edition-orange?logo=rust" alt="Rust">
</p>

---

On most Linux systems there is no "install font" button. You download an archive from
[nerdfonts.com](https://www.nerdfonts.com/font-downloads), unpack it into `~/.local/share/fonts`, run `fc-cache -f -v`,
and clean up the leftovers yourself. Every time, for every font.

`nf-installer` does that for you: pick a font, and it is installed.

## Installation

```bash
curl -LsSf https://raw.githubusercontent.com/KarnesTH/nf-installer/main/install.sh | sh
```

This drops the binary into `~/.local/bin`. Make sure that directory is in your `PATH`.

Or build it yourself:

```bash
git clone https://github.com/KarnesTH/nf-installer.git
cd nf-installer
cargo build --release
```

## Usage

Pick a font from the list and install it:

```bash
nf-installer
```

Install one by name, without the prompt:

```bash
nf-installer --font-name Hack
```

Remove an installed font:

```bash
nf-installer --uninstall
```

| Flag                 | Short | Description                           |
|----------------------|-------|---------------------------------------|
| `--font-name <NAME>` | `-f`  | Nerd Font name                        |
| `--refresh`          | `-r`  | Ignore cache and reload the font list |
| `--uninstall`        | `-u`  | Uninstall an installed Nerd Font      |
| `--help`             | `-h`  | Show all options                      |

`--font-name` combines with `--uninstall`, so `nf-installer -u -f Hack` removes Hack directly.

## How it works

Fonts are installed into their own directory under the user font directory —
`~/.local/share/fonts/<FontName>/` on Linux, `~/Library/Fonts/<FontName>/` on macOS — and registered with
`fc-cache -f -v`. Uninstalling removes that directory and refreshes the cache again. The downloaded archive lives in the
temp directory and is deleted afterwards, including when the extraction fails.

The font list is scraped from nerdfonts.com once and then cached locally, so repeated runs start instantly instead of
hitting the network:

- **Location:** `~/.cache/nf-installer/nerd_fonts.json`
- **Lifetime:** 7 days, after which the list is fetched again
- **Offline:** if the refresh fails but a cached list exists, the stale list is used and a warning is printed
- **Corrupt or missing:** treated as a cache miss, not as an error

Delete the file at any time — it is rebuilt on the next run.

## Requirements

- `fontconfig` — provides `fc-cache`, present on virtually every desktop Linux
- Rust toolchain, if you build from source

## License

MIT — see [LICENSE](LICENSE).

---

Built with [Rust](https://www.rust-lang.org/) and 💜 by [KarnesTH](https://github.com/KarnesTH).