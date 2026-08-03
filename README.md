# cosmic-term — POP Flow fork

Fork of [pop-os/cosmic-term](https://github.com/pop-os/cosmic-term) for the
**POP Flow** suite. Upstream's README follows the POP Flow section below.

## POP Flow: appearance per directory

Each folder can have its own terminal appearance — **color scheme,
transparency, tab title and cursor color** — persisted independently of the
global settings and of the other folders. It applies when a terminal opens in
that folder, and live when you `cd` into it.

**A rule covers one folder.** Each directory has its own identity and does not
hand it down: a rule on `~/projects` says nothing about `~/projects/foo`, which
keeps the global appearance until it gets a rule of its own. Set
`include_subdirs: true` on a rule when you do want it to cover a whole tree.

A rule only overrides the fields it actually sets. Set a color and nothing else,
and transparency, title and cursor keep inheriting — including from the global
settings, so moving the global opacity still moves every folder that did not pin
its own.

### There is no settings UI yet

Rules live in the COSMIC config store, one file per key:

```
~/.config/cosmic/com.system76.CosmicTerm/v1/dir_rules
```

Write it by hand for now (the file is watched, so open terminals react
immediately):

```ron
{
    1: (path: "~/projects", opacity: Some(85), syntax_theme_dark: Some("Dracula")),
    2: (path: "~/projects/prod", tab_title: Some("PROD"), cursor: Some("#ff0000")),
    3: (path: "/srv", include_subdirs: true, syntax_theme_dark: Some("Solarized Dark")),
}
```

The key is any number, unique per rule. Fields:

| Field | Default | Meaning |
|---|---|---|
| `path` | — | Absolute, or starting with `~`. Required. |
| `include_subdirs` | `false` | Opt in to covering everything below `path` too. |
| `enabled` | `true` | Lets a rule be parked instead of deleted. |
| `syntax_theme_dark` | inherit | Color scheme name, as shown in *View → Color schemes*. |
| `syntax_theme_light` | inherit | Same, for light mode. |
| `opacity` | inherit | `0`–`100`. |
| `tab_title` | inherit | Fixed tab title for the folder. |
| `cursor` | scheme's | Cursor color, `"#rrggbb"`. |

With the rules above: `~/projects` is Dracula at 85%, `~/projects/prod` is titled
`PROD` with a red cursor (and *not* Dracula, since rule 1 stops at its own
folder), and `~/projects/foo` looks like every other folder. Everything under
`/srv` is Solarized Dark, because rule 3 opted into its subtree.

When rules do overlap — only possible once one opts into a subtree — the most
specific wins: a folder's own rule beats a tree reaching down into it.

Matching is by path component, not string prefix — a rule on `/home/a` does not
capture `/home/ab`.

### Notes and limits

- **Linux only.** The current directory is read from `/proc/<shell>/cwd`.
- It is the **shell's** directory. `cd /x && vim` does not move the terminal, as
  far as a rule is concerned.
- The check rides on terminal output rather than a timer, so it costs nothing
  while the terminal is idle. In practice the shell's next prompt triggers it.
- **Transparency and blur:** when the COSMIC theme has blur active, the theme
  normally dictates a pane's alpha. A folder's pinned opacity deliberately wins
  over that — otherwise setting it would appear to do nothing.

## Install

```bash
./install.sh              # build, back up the system binary, install
./setup-auto-reapply.sh   # make it survive `apt upgrade` (do this too)
```

Undo with `./uninstall.sh` and `./remove-auto-reapply.sh`.

> **Before the first install**, make sure the system package is current
> (`sudo apt upgrade cosmic-term`). `install.sh` backs up whatever is in
> `/usr/bin` as *the* original, and that backup is what `uninstall.sh` restores
> later. The script warns if it spots a mismatch.

Terminals already open keep running the old binary — **nothing is killed**,
because that would close your shells. Open a new terminal window instead.

---

# cosmic-term

COSMIC terminal emulator, built using [alacritty\_terminal](https://docs.rs/alacritty_terminal) that is provided by the [alacritty](https://github.com/alacritty/alacritty) project. `cosmic-term` provides bidirectional rendering and ligatures with a custom renderer based on [cosmic-text](https://github.com/pop-os/cosmic-text).

The `wgpu` feature, enabled by default, supports GPU rendering using `glyphon`
and `wgpu`. If `wgpu` is not enabled or fails to initialize, then rendering falls
back to using `softbuffer` and `tiny-skia`.

## Color Schemes

Custom color schemes can be imported from the `View -> Color schemes...` menu item.
You can find templates for color schemes in the [color-schemes](color-schemes) folder.
