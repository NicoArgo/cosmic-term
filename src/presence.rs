// SPDX-License-Identifier: GPL-3.0-only

//! Tells the panel which folder identity each window is wearing.
//!
//! The panel sees every window's app id and title, but not which process owns
//! it nor which folder its active tab is in — only this process knows that. So
//! each window (one process per window, see `Message::WindowNew`) keeps one
//! small file in `$XDG_RUNTIME_DIR/pop-flow/terminals/<pid>`, and the
//! `cosmic-applet-terminals` panel applet reads them. The title is in the file
//! so the applet can tell which window the file is about.
//!
//! The format is `key=value` lines, so neither side needs a JSON parser for
//! three fields. Values never contain a newline: they are flattened on write.

use hex_color::HexColor;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Presence {
    /// The window title exactly as this process set it.
    pub title: String,
    /// The folder rule's name, when the active tab is in a folder with a rule.
    pub name: Option<String>,
    /// The folder rule's color, same condition.
    pub accent: Option<HexColor>,
}

impl Presence {
    pub fn to_text(&self) -> String {
        let flat = |s: &str| s.replace(['\n', '\r'], " ");
        let mut text = format!("title={}\n", flat(&self.title));
        if let Some(name) = &self.name {
            text.push_str(&format!("name={}\n", flat(name)));
        }
        if let Some(accent) = self.accent {
            text.push_str(&format!("accent={}\n", accent.display_rgb()));
        }
        text
    }
}

fn dir() -> Option<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
    Some(Path::new(&runtime).join("pop-flow/terminals"))
}

fn file() -> Option<PathBuf> {
    Some(dir()?.join(std::process::id().to_string()))
}

/// Publishes what this window looks like now. Written to a temporary name and
/// renamed, so the applet never reads half a file.
pub fn write(presence: &Presence) -> io::Result<()> {
    let (Some(dir), Some(file)) = (dir(), file()) else {
        return Ok(());
    };
    fs::create_dir_all(&dir)?;
    let tmp = dir.join(format!(".{}.tmp", std::process::id()));
    fs::write(&tmp, presence.to_text())?;
    fs::rename(&tmp, &file)
}

/// The window is gone. If the process dies without getting here, the applet
/// drops the file itself once `/proc/<pid>` is gone.
pub fn remove() {
    if let Some(file) = file() {
        let _ = fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_only_what_is_known() {
        let bare = Presence {
            title: "~ — COSMIC Terminal".into(),
            ..Presence::default()
        };
        assert_eq!(bare.to_text(), "title=~ — COSMIC Terminal\n");

        let ruled = Presence {
            title: "POP FLOW — claude — COSMIC Terminal".into(),
            name: Some("POP FLOW".into()),
            accent: Some(HexColor::rgb(0x48, 0xb9, 0xc7)),
        };
        assert_eq!(
            ruled.to_text(),
            "title=POP FLOW — claude — COSMIC Terminal\nname=POP FLOW\naccent=#48B9C7\n"
        );
    }

    #[test]
    fn a_newline_cannot_forge_a_field() {
        let sneaky = Presence {
            title: "a\naccent=#000000".into(),
            ..Presence::default()
        };
        assert_eq!(sneaky.to_text(), "title=a accent=#000000\n");
    }
}
