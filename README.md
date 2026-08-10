<!--suppress HtmlDeprecatedAttribute -->
<p align="center">
  <img src="assets/logo.svg" alt="nf-installer" width="120">
</p>

<h1 align="center">nf-installer</h1>

<p align="center">Install fonts from your terminal, without the copy-and-cache dance.</p>

<p align="center">
  <a href="https://github.com/KarnesTH/nf-installer/actions/workflows/ci.yaml">
    <img src="https://github.com/KarnesTH/nf-installer/actions/workflows/ci.yaml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/KarnesTH/nf-installer/releases/latest">
    <img src="https://img.shields.io/github/v/release/KarnesTH/nf-installer" alt="Latest release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  </a>
  <img src="https://img.shields.io/badge/rust-2024%20edition-orange?logo=rust" alt="Rust">
</p>

---

On most Linux systems there is no "install font" button. You download an archive, unpack it into
`~/.local/share/fonts`, run `fc-cache -f -v`, and clean up the leftovers yourself. Every time, for every font.

`nf-installer` does that for you: pick a font, and it is installed. It started out as a Nerd Fonts installer — hence the
name — and now handles Google Fonts and fonts you already downloaded yourself, so read it as *nice fonts* if you like.

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

Run it without arguments, and it asks what you want to do:

```bash
nf-installer
```

| Command               | Description                                     |
|-----------------------|-------------------------------------------------|
| `nerd [-f NAME]`      | Install a Nerd Font                             |
| `google [-f NAME]`    | Install a Google Font                           |
| `local <PATH>`        | Install from an archive, font file or directory |
| `list`                | List the installed fonts                        |
| `uninstall [-f NAME]` | Remove an installed font                        |

### Nerd Fonts

```bash
nf-installer nerd -f Hack
nf-installer nerd -r          # ignore the cached list and reload
```

### Google Fonts

Install into the system like any other font:

```bash
nf-installer google -f Roboto
```

Or drop web assets into a project instead — woff2 files, a ready-made `fonts.css` and the font license:

```bash
nf-installer google -f Roboto --web
nf-installer google -f Roboto --web -o public/fonts
nf-installer google -f Roboto --web --weights 400,700,700italic
```

Without `-o` the files go to `assets/fonts`.

### Local files

```bash
nf-installer local ~/Downloads/Comic.zip
nf-installer local ~/Downloads/loose-fonts/ -n MyFonts
```

Works with a zip, a single `.ttf`/`.otf`, or a directory of them. The name is taken from the path unless you pass `-n`.

### Removing

```bash
nf-installer list
nf-installer uninstall -f Hack
```

## How it works

Fonts are installed into their own directory under `~/.local/share/fonts/<FontName>/` and registered with
`fc-cache -f -v`. Uninstalling removes that directory and refreshes the cache again. Downloaded archives live in the
temp directory and are deleted afterward, including when the extraction fails — files you pass to `local` are of course,
left alone.

Font lists are fetched once and then cached, so repeated runs start instantly instead of hitting the network:

- **Location:** `~/.cache/nf-installer/nerd_fonts.json` and `google_fonts.json`
- **Lifetime:** 7 days, after which the list is fetched again
- **Offline:** if the refresh fails but a cached list exists, the stale list is used and a warning is printed
- **Corrupt or missing:** treated as a cache miss, not as an error

Delete those files at any time — they are rebuilt on the next run.

Nerd Fonts come from [nerdfonts.com](https://www.nerdfonts.com/font-downloads), Google Fonts through the public API of
[google-webfonts-helper](https://gwfh.mranftl.com/).

## Requirements

- `fontconfig` — provides `fc-cache`, present on virtually every desktop Linux
- Rust toolchain, if you build from source

## License

MIT — see [LICENSE](LICENSE).

The fonts themselves are not part of this project and keep their own licenses. When you use `--web`, the license of the
family is written next to the font files, because those end up in your repository.

---

Built with [Rust](https://www.rust-lang.org/) and 💜 by [KarnesTH](https://github.com/KarnesTH).