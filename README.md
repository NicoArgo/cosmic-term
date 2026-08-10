# cosmic-term — POP Flow fork

Fork of [pop-os/cosmic-term](https://github.com/pop-os/cosmic-term) for the
**POP Flow** suite. Upstream's README follows the POP Flow section below.

## POP Flow: appearance per directory

Each folder can have an identity of its own — **a name and a color** —
persisted independently of the global settings and of the other folders. It
applies when a terminal opens in that folder, and live when you `cd` into it.

The color scheme and the transparency can also be pinned per folder, but only
by editing the rules file: they were taken out of the dialog, which is about
telling one project's terminals from another's, not about re-theming each one.

**A rule covers one folder.** Each directory has its own identity and does not
hand it down: a rule on `~/projects` says nothing about `~/projects/foo`, which
keeps the global appearance until it gets a rule of its own. Set
`include_subdirs: true` on a rule when you do want it to cover a whole tree.

A rule only overrides the fields it actually sets. Set a color and nothing else,
and title and transparency keep inheriting — including from the global settings,
so moving the global opacity still moves every folder that did not pin its own.

### Setting it up

Right-click in a terminal → **Create a rule for this folder** makes a rule for
the folder you are in and opens it, ready to be named and colored. It captures
nothing else: a folder keeps following the global theme until you tell it not
to. **File → Directory rules...** reopens the page later.

The list shows each folder by name — in its own color — rather than by path;
the full path is in the rule's own editor.

Each field can be left inheriting, which is what keeps folders independent.

### Editing the rules file directly

Rules live in the COSMIC config store, one file per key:

```
~/.config/cosmic/com.system76.CosmicTerm/v1/dir_rules
```

Editing it by hand still works, and the file is watched, so open terminals react
immediately:

```ron
{
    1: (path: "~/projects", opacity: Some(85), syntax_theme_dark: Some("Dracula")),
    2: (path: "~/projects/prod", tab_title: Some("PROD"), accent: Some("#ff0000")),
    3: (path: "/srv", include_subdirs: true, syntax_theme_dark: Some("Solarized Dark")),
}
```

The key is any number, unique per rule. Fields:

| Field | Default | Meaning |
|---|---|---|
| `path` | — | Absolute, or starting with `~`. Required. |
| `include_subdirs` | `false` | Opt in to covering everything below `path` too. |
| `enabled` | `true` | Lets a rule be parked instead of deleted. |
| `syntax_theme_dark` | inherit | Color scheme name, as shown in *View → Color schemes*. File only. |
| `syntax_theme_light` | inherit | Same, for light mode. File only. |
| `opacity` | inherit | `0`–`100`. File only. |
| `tab_title` | inherit | The folder's name. `{title}` is replaced by the running program's title. |
| `accent` | inherit | The folder's color, `"#rrggbb"`: window accent, stripe, terminal cursor, and what `--resolve-rule` reports. |

A rule written before the folder color and the cursor color merged still loads:
its `cursor` is read once as the folder's color and folded into `accent` the
next time the terminal starts.

With the rules above: `~/projects` is Dracula at 85%, `~/projects/prod` is titled
`PROD` in red — accent, stripe and cursor (and *not* Dracula, since rule 1
stops at its own folder), and `~/projects/foo` looks like every other folder. Everything under
`/srv` is Solarized Dark, because rule 3 opted into its subtree.

When rules do overlap — only possible once one opts into a subtree — the most
specific wins: a folder's own rule beats a tree reaching down into it.

Matching is by path component, not string prefix — a rule on `/home/a` does not
capture `/home/ab`.

## POP Flow: the folder's identity

`accent` and `tab_title` answer a different question from the rest: not *how does
the text look* but *which terminal am I in*. A folder that sets them gets

- the window's **accent** — active tab, focus, hover, action buttons — in its
  color, while the header bar and menus keep the system's grey;
- a thin **stripe** at the top of the window, in the exact color;
- its **name** on the tab.

The accent goes through COSMIC's own accent machinery, which rebuilds from *your*
theme — so nothing else about your customization is lost — and normalizes the
color's lightness for contrast. That is why the stripe, which has nothing read
against it, is the one place the literal hex appears.

### `{title}`

A name replaces the running program's title outright, which is what you want for
a tab that should say one thing and stay there. When you want both, put `{title}`
in the name:

```ron
2: (path: "~/projects/prod", tab_title: Some("PROD — {title}"), accent: Some("#ff5252")),
```

The tab then reads `PROD — vim main.rs`, and collapses to just `PROD` when the
program has set no title — no dangling separator.

### Asking from outside the terminal

Anything that wants to label a folder the same way can ask:

```console
$ cosmic-term --resolve-rule ~/projects/prod
RULE_NAME='PROD'
RULE_ACCENT='#FF5252'
RULE_ACCENT_RGB='255;82;82'
```

Shell-quoted, so `eval` is safe, and it prints **nothing** for a folder with no
rule — which is what lets a caller keep its own default:

```bash
eval "$(cosmic-term --resolve-rule "$PWD")"
name=${RULE_NAME:-$(basename "$PWD")}
```

`RULE_ACCENT_RGB` is the same color as an ANSI triplet, ready to drop into an
escape sequence. It reads the config directly and exits before starting a GUI, so
it is cheap enough for a prompt or a status bar.

The POP Flow suite uses this to drive the Claude Code statusline from the same
rule that colors the window — see `.claude/statusline.sh` at the workspace root.

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
