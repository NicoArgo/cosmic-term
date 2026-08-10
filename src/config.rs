// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use cosmic_text::{Metrics, Stretch, Weight};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::{fl, localize::LANGUAGE_SORTER, shortcuts::Shortcuts};

pub const CONFIG_VERSION: u64 = 1;
pub const COSMIC_THEME_DARK: &str = "COSMIC Dark";
pub const COSMIC_THEME_LIGHT: &str = "COSMIC Light";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ColorSchemeKind {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ColorSchemeId(pub u64);

//TODO: there is a lot of extra code to keep the exported color scheme clean,
//consider how to reduce this
fn de_color_opt<'de, D>(deserializer: D) -> Result<Option<HexColor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hex_color: HexColor = Deserialize::deserialize(deserializer)?;
    Ok(Some(hex_color))
}

fn ser_color_opt<S>(hex_color_opt: &Option<HexColor>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::Error as _;
    match hex_color_opt {
        Some(hex_color) => Serialize::serialize(hex_color, serializer),
        None => Err(S::Error::custom("ser_color_opt called with None")),
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorSchemeAnsi {
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub black: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub red: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub green: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub yellow: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub blue: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub magenta: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub cyan: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub white: Option<HexColor>,
}

impl ColorSchemeAnsi {
    pub fn is_empty(&self) -> bool {
        self.black.is_none()
            && self.red.is_none()
            && self.green.is_none()
            && self.yellow.is_none()
            && self.blue.is_none()
            && self.magenta.is_none()
            && self.cyan.is_none()
            && self.white.is_none()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorScheme {
    pub name: String,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub foreground: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub background: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub cursor: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub bright_foreground: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub dim_foreground: Option<HexColor>,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub normal: ColorSchemeAnsi,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub bright: ColorSchemeAnsi,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub dim: ColorSchemeAnsi,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProfileId(pub u64);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub syntax_theme_dark: String,
    #[serde(default)]
    pub syntax_theme_light: String,
    #[serde(default)]
    pub tab_title: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default)]
    pub drain_on_exit: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: fl!("new-profile"),
            command: String::new(),
            syntax_theme_dark: COSMIC_THEME_DARK.to_string(),
            syntax_theme_light: COSMIC_THEME_LIGHT.to_string(),
            tab_title: String::new(),
            working_directory: String::new(),
            drain_on_exit: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DirRuleId(pub u64);

/// An appearance bound to a directory, so a folder can look different from the
/// rest without touching the global settings.
///
/// **A rule covers one folder.** Each directory carries its own identity and
/// does not hand it down: a rule on `~/projects` says nothing about
/// `~/projects/foo`, which keeps the global appearance until it is given a rule
/// of its own. Covering a whole tree is available per rule via
/// [`Self::include_subdirs`], but it is opt-in rather than the default.
///
/// Every appearance field is an `Option` where `None` means "no opinion,
/// inherit". That is what keeps folders independent from each other and from
/// the global settings: a rule that only sets a color leaves opacity, title and
/// cursor alone, and changing the global opacity still moves every folder that
/// did not pin its own.
///
/// Kept separate from [`Profile`] on purpose: a profile says *what to run*, a
/// rule says *how it looks*. Folding the two together would mean inventing a
/// profile every time you just wanted to paint a directory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DirRule {
    /// Directory this rule applies to. Absolute, except for a leading `~`,
    /// which is expanded when the rule is matched (so the config stays portable
    /// between machines with different user names).
    pub path: String,
    /// Opt in to covering everything below `path` as well. Off by default: a
    /// folder's appearance is its own, not something its children pick up.
    pub include_subdirs: bool,
    /// Lets a rule be kept but parked, instead of having to delete it.
    pub enabled: bool,
    pub syntax_theme_dark: Option<String>,
    pub syntax_theme_light: Option<String>,
    pub opacity: Option<u8>,
    /// The folder's name. Titles the tab, and — via `--resolve-rule` — labels
    /// the folder anywhere outside the terminal that cares to ask.
    ///
    /// A `{title}` in here is replaced by whatever the running program set the
    /// title to, so a folder can carry a fixed name *and* still show what is
    /// happening in it. Without the placeholder the name replaces the program's
    /// title outright, which is how this behaved before the placeholder existed.
    pub tab_title: Option<String>,
    /// The folder's color — the only one it has. It paints the accent of the
    /// window chrome, the stripe at the top, the terminal's cursor, and,
    /// through `--resolve-rule`, whatever asks from outside (the Claude
    /// statusline today).
    ///
    /// One field because it is one fact. The cursor used to be picked
    /// separately, and every rule that had one set it to the value the accent
    /// already carried: the same decision, made twice, with two chances to
    /// disagree.
    pub accent: Option<HexColor>,
    /// Superseded by [`Self::accent`], and read only so that a rule written
    /// before the two merged keeps its color — see [`Self::color`]. Never
    /// written back: the first save after this rule is touched drops it.
    #[serde(default, rename = "cursor", skip_serializing)]
    pub legacy_cursor: Option<HexColor>,
}

impl Default for DirRule {
    fn default() -> Self {
        Self {
            path: String::new(),
            // Each folder has its own identity: painting one must not silently
            // repaint everything under it.
            include_subdirs: false,
            enabled: true,
            syntax_theme_dark: None,
            syntax_theme_light: None,
            opacity: None,
            tab_title: None,
            accent: None,
            legacy_cursor: None,
        }
    }
}

/// The title a tab shows, given the folder's name and the title the running
/// program last set.
///
/// The placeholder is what lets a fixed folder name coexist with a live title
/// instead of erasing it. When a name has no placeholder the program's title is
/// dropped, which is the pre-placeholder behaviour and still the right answer
/// for someone who wants the tab to say one thing and stay there.
///
/// An empty program title collapses the placeholder and tidies up the separator
/// it leaves behind, so a fresh shell reads `POP FLOW` rather than `POP FLOW —`.
pub fn render_tab_title(name: &str, program_title: Option<&str>) -> String {
    const PLACEHOLDER: &str = "{title}";

    if !name.contains(PLACEHOLDER) {
        return name.to_string();
    }

    let program_title = program_title.map(str::trim).unwrap_or("");
    let rendered = name.replace(PLACEHOLDER, program_title);

    if program_title.is_empty() {
        // Trim the separator the empty placeholder orphaned, from either side:
        // the placeholder is as likely to lead (`{title} — POP FLOW`) as to
        // trail. Only punctuation used as a separator goes; letters stay.
        rendered
            .trim()
            .trim_matches(|c: char| c.is_whitespace() || matches!(c, '—' | '-' | '–' | ':' | '|' | '·'))
            .to_string()
    } else {
        rendered.trim().to_string()
    }
}

impl DirRule {
    /// The folder's color, wherever it happens to be written. Reading it
    /// through here is what keeps a rule from before the merge — one that only
    /// ever set a cursor color — from looking like a folder with no color.
    pub fn color(&self) -> Option<HexColor> {
        self.accent.or(self.legacy_cursor)
    }

    /// The rule's path with a leading `~` expanded, or `None` when it cannot be
    /// resolved to an absolute path. A relative path has no stable meaning here
    /// — it would follow whatever directory the terminal happens to be in — so
    /// it is treated as "matches nothing" rather than guessed at.
    pub fn absolute_path(&self) -> Option<PathBuf> {
        let trimmed = self.path.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix('~') {
            // Only `~` and `~/...` are ours to expand; `~other-user` is a shell
            // construct we do not implement, and silently reading it as this
            // user's home would point the rule at the wrong directory.
            if rest.is_empty() || rest.starts_with('/') {
                let home = std::env::home_dir()?;
                return Some(home.join(rest.trim_start_matches('/')));
            }
            return None;
        }

        let path = PathBuf::from(trimmed);
        path.is_absolute().then_some(path)
    }

    /// The name to fall back on when the rule pins everything about a folder
    /// except what to call it.
    ///
    /// A rule that is switched on says the folder has an identity worth
    /// showing. Left without a title, though, the window heading would say
    /// "COSMIC Terminal" and anything asking from outside would reach for its
    /// own hardcoded default — so the one folder ends up with two names, or
    /// with none. Naming it after itself keeps every surface telling the same
    /// story, and costs nothing to override: type a title and this stops
    /// applying.
    ///
    /// Upper case because that is how these labels get written by hand
    /// ("POP FLOW"), and because it reads as a name for the place rather than
    /// as a path component echoed back.
    ///
    /// Derived from the *rule's* path, not from the terminal's directory: a
    /// rule covering a subtree names the folder it covers, so everything under
    /// it answers with one name instead of renaming itself on every `cd`.
    pub fn derived_title(&self) -> Option<String> {
        self.folder_name().map(|name| name.to_uppercase())
    }

    /// The last component of the rule's path — the folder's own name, as it is
    /// written on disk. What the rules list shows, since the full path is long,
    /// mostly shared between rules, and already one click away in the editor.
    pub fn folder_name(&self) -> Option<String> {
        let path = self.absolute_path()?;
        let name = path.file_name()?.to_str()?.trim();
        (!name.is_empty()).then(|| name.to_string())
    }
}

/// How well a rule path covers `cwd`, as the number of path components matched,
/// or `None` when the rule does not apply at all.
///
/// Comparison is component by component rather than by string prefix, so a rule
/// on `/home/a` does not capture `/home/ab`.
fn match_depth(rule_path: &Path, cwd: &Path, include_subdirs: bool) -> Option<usize> {
    let mut depth = 0;
    let mut cwd_components = cwd.components();

    for rule_component in rule_path.components() {
        if cwd_components.next()? != rule_component {
            return None;
        }
        depth += 1;
    }

    // The rule path ran out first: cwd sits below it.
    if cwd_components.next().is_some() && !include_subdirs {
        return None;
    }

    Some(depth)
}

/// The rule that applies to `cwd`, if any.
///
/// Normally that is the rule naming this exact directory: folders do not hand
/// their appearance down to their children. Rules that opted into
/// [`DirRule::include_subdirs`] can also reach `cwd` from above, and when
/// several rules reach it the most specific wins — measured in path components
/// matched, so a subtree rule on `~/projects/prod` beats one on `~/projects`,
/// and the directory's own rule beats both. Ties (only possible between rules
/// with the same path) go to the lowest id, so the result never depends on
/// iteration luck.
pub fn resolve_dir_rule(rules: &BTreeMap<DirRuleId, DirRule>, cwd: &Path) -> Option<DirRuleId> {
    let mut best: Option<(usize, DirRuleId)> = None;

    // BTreeMap iterates in ascending id order and we only replace on a strictly
    // deeper match, so the lowest id naturally survives a tie.
    for (id, rule) in rules {
        if !rule.enabled {
            continue;
        }
        let Some(rule_path) = rule.absolute_path() else {
            continue;
        };
        let Some(depth) = match_depth(&rule_path, cwd, rule.include_subdirs) else {
            continue;
        };
        if best.is_none_or(|(best_depth, _)| depth > best_depth) {
            best = Some((depth, *id));
        }
    }

    best.map(|(_, id)| id)
}

/// The appearance a terminal actually renders with, after the directory rule,
/// the profile and the global settings have been layered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Appearance {
    pub syntax_theme: String,
    pub opacity: u8,
    pub tab_title: Option<String>,
    /// The folder's color: the window accent, the stripe, and the cursor, all
    /// from one place.
    ///
    /// `None` means "no folder identity here" — the window keeps the system
    /// accent, grows no stripe and keeps the color scheme's own cursor, so a
    /// terminal without a rule looks exactly like it did before rules existed.
    pub accent: Option<HexColor>,
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub app_theme: AppTheme,
    pub color_schemes_dark: BTreeMap<ColorSchemeId, ColorScheme>,
    pub color_schemes_light: BTreeMap<ColorSchemeId, ColorScheme>,
    pub font_name: String,
    pub font_size: u16,
    pub font_weight: u16,
    pub dim_font_weight: u16,
    pub bold_font_weight: u16,
    pub font_stretch: u16,
    pub font_size_zoom_step_mul_100: u16,
    pub opacity: u8,
    /// Per-directory appearance overrides. See [`DirRule`].
    #[serde(default)]
    pub dir_rules: BTreeMap<DirRuleId, DirRule>,
    pub profiles: BTreeMap<ProfileId, Profile>,
    pub show_headerbar: bool,
    pub show_pane_borders: bool,
    pub use_bright_bold: bool,
    pub syntax_theme_dark: String,
    pub syntax_theme_light: String,
    pub focus_follow_mouse: bool,
    #[serde(default)]
    pub tab_new_inherit_working_directory: bool,
    pub default_profile: Option<ProfileId>,
    #[serde(default)]
    pub shortcuts_custom: Shortcuts,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            bold_font_weight: Weight::BOLD.0,
            color_schemes_dark: BTreeMap::new(),
            color_schemes_light: BTreeMap::new(),
            dim_font_weight: Weight::NORMAL.0,
            focus_follow_mouse: false,
            tab_new_inherit_working_directory: false,
            font_name: "Noto Sans Mono".to_string(),
            font_size: 14,
            font_size_zoom_step_mul_100: 100,
            font_stretch: Stretch::Normal.to_number(),
            font_weight: Weight::NORMAL.0,
            opacity: 100,
            dir_rules: BTreeMap::new(),
            profiles: BTreeMap::new(),
            show_headerbar: true,
            show_pane_borders: false,
            syntax_theme_dark: COSMIC_THEME_DARK.to_string(),
            syntax_theme_light: COSMIC_THEME_LIGHT.to_string(),
            use_bright_bold: false,
            default_profile: None,
            shortcuts_custom: Shortcuts::default(),
        }
    }
}

impl Config {
    pub fn color_schemes(
        &self,
        color_scheme_kind: ColorSchemeKind,
    ) -> &BTreeMap<ColorSchemeId, ColorScheme> {
        match color_scheme_kind {
            ColorSchemeKind::Dark => &self.color_schemes_dark,
            ColorSchemeKind::Light => &self.color_schemes_light,
        }
    }

    pub fn color_schemes_mut(
        &mut self,
        color_scheme_kind: ColorSchemeKind,
    ) -> &mut BTreeMap<ColorSchemeId, ColorScheme> {
        match color_scheme_kind {
            ColorSchemeKind::Dark => &mut self.color_schemes_dark,
            ColorSchemeKind::Light => &mut self.color_schemes_light,
        }
    }

    pub fn color_scheme_kind(&self, system_theme: &theme::Theme) -> ColorSchemeKind {
        match self.app_theme {
            AppTheme::Dark => ColorSchemeKind::Dark,
            AppTheme::Light => ColorSchemeKind::Light,
            AppTheme::System => {
                if system_theme.theme_type.is_dark() {
                    ColorSchemeKind::Dark
                } else {
                    ColorSchemeKind::Light
                }
            }
        }
    }

    // Get a sorted and adjusted for duplicates list of color scheme names and ids
    pub fn color_scheme_names(
        &self,
        color_scheme_kind: ColorSchemeKind,
    ) -> Vec<(String, ColorSchemeId)> {
        let color_schemes = self.color_schemes(color_scheme_kind);
        let mut color_scheme_names =
            Vec::<(String, ColorSchemeId)>::with_capacity(color_schemes.len());
        for (color_scheme_id, color_scheme) in color_schemes {
            let mut name = color_scheme.name.clone();

            let mut copies = 1;
            while color_scheme_names.iter().any(|x| x.0 == name) {
                copies += 1;
                name = format!("{} ({})", color_scheme.name, copies);
            }

            color_scheme_names.push((name, *color_scheme_id));
        }
        color_scheme_names.sort_by(|a, b| LANGUAGE_SORTER.compare(&a.0, &b.0));
        color_scheme_names
    }

    fn font_size_adjusted(&self, zoom_adj: i8) -> f32 {
        let font_size = f32::from(self.font_size).max(1.0);
        let adj = f32::from(zoom_adj);
        let adj_step = f32::from(self.font_size_zoom_step_mul_100) / 100.0;
        (font_size + adj * adj_step).max(1.0)
    }

    // Calculate metrics from font size
    pub fn metrics(&self, zoom_adj: i8) -> Metrics {
        let font_size = self.font_size_adjusted(zoom_adj);
        let line_height = (font_size * 1.4).ceil();
        Metrics::new(font_size, line_height)
    }

    pub fn opacity_ratio(&self) -> f32 {
        f32::from(self.opacity) / 100.0
    }

    // Get a sorted and adjusted for duplicates list of profile names and ids
    pub fn profile_names(&self) -> Vec<(String, ProfileId)> {
        let mut profile_names = Vec::<(String, ProfileId)>::with_capacity(self.profiles.len());
        for (profile_id, profile) in &self.profiles {
            let mut name = profile.name.clone();

            let mut copies = 1;
            while profile_names.iter().any(|x| x.0 == name) {
                copies += 1;
                name = format!("{} ({})", profile.name, copies);
            }

            profile_names.push((name, *profile_id));
        }
        profile_names.sort_by(|a, b| LANGUAGE_SORTER.compare(&a.0, &b.0));
        profile_names
    }

    // Get current syntax theme based on dark mode
    pub fn syntax_theme(
        &self,
        color_scheme_kind: ColorSchemeKind,
        profile_id_opt: Option<ProfileId>,
    ) -> (String, ColorSchemeKind) {
        let theme_name = match profile_id_opt.and_then(|profile_id| self.profiles.get(&profile_id))
        {
            Some(profile) => match color_scheme_kind {
                ColorSchemeKind::Dark => profile.syntax_theme_dark.clone(),
                ColorSchemeKind::Light => profile.syntax_theme_light.clone(),
            },
            None => match color_scheme_kind {
                ColorSchemeKind::Dark => self.syntax_theme_dark.clone(),
                ColorSchemeKind::Light => self.syntax_theme_light.clone(),
            },
        };
        (theme_name, color_scheme_kind)
    }

    /// Layer directory rule over profile over global, field by field.
    ///
    /// Field by field is the point: a rule that only pins a color must not drag
    /// the other three along with it, otherwise every rule would silently
    /// freeze the whole appearance of its folder.
    /// Rules listed by path, for the settings UI. Sorted by path rather than by
    /// id so the list reads like a directory listing instead of like insertion
    /// history.
    pub fn dir_rule_paths(&self) -> Vec<(String, DirRuleId)> {
        let mut paths: Vec<(String, DirRuleId)> = self
            .dir_rules
            .iter()
            .map(|(id, rule)| (rule.path.clone(), *id))
            .collect();
        paths.sort_by(|a, b| LANGUAGE_SORTER.compare(&a.0, &b.0));
        paths
    }

    /// The rule already covering `path` exactly, if there is one. Used by the
    /// "create a rule for this folder" action so it edits the folder's rule
    /// instead of stacking a second one on the same directory.
    pub fn dir_rule_for_exact_path(&self, path: &Path) -> Option<DirRuleId> {
        self.dir_rules
            .iter()
            .find(|(_, rule)| rule.absolute_path().as_deref() == Some(path))
            .map(|(id, _)| *id)
    }

    /// A fresh rule for a folder, carrying nothing but the folder.
    ///
    /// It used to freeze the current color scheme and transparency too. Those
    /// left the dialog, and a rule that silently pins settings nothing on
    /// screen can show or undo is worse than no rule: the folder would quietly
    /// stop following the global theme with no way back. What is left is the
    /// identity — name and color — which is what the rule is for.
    pub fn dir_rule_from_current(&self, path: String) -> DirRule {
        DirRule {
            path,
            ..Default::default()
        }
    }

    /// The next free rule id.
    pub fn next_dir_rule_id(&self) -> DirRuleId {
        self.dir_rules
            .last_key_value()
            .map(|(id, _)| DirRuleId(id.0 + 1))
            .unwrap_or_default()
    }

    /// The opacity a directory rule pins for this folder, if it pins one.
    ///
    /// Kept separate from [`Self::effective_appearance`] because the view needs
    /// to tell "this folder asked for 85%" apart from "nobody asked, use the
    /// global 85%" — under blur those two must behave differently.
    pub fn dir_rule_opacity(&self, dir_rule_id_opt: Option<DirRuleId>) -> Option<u8> {
        dir_rule_id_opt
            .and_then(|id| self.dir_rules.get(&id))
            .and_then(|rule| rule.opacity)
    }

    /// Opacity for a terminal: the directory rule's if it has one, else the
    /// global. Profiles carry no opacity of their own.
    pub fn effective_opacity(&self, dir_rule_id_opt: Option<DirRuleId>) -> u8 {
        self.dir_rule_opacity(dir_rule_id_opt)
            .unwrap_or(self.opacity)
    }

    pub fn effective_appearance(
        &self,
        color_scheme_kind: ColorSchemeKind,
        profile_id_opt: Option<ProfileId>,
        dir_rule_id_opt: Option<DirRuleId>,
    ) -> Appearance {
        let rule_opt = dir_rule_id_opt.and_then(|id| self.dir_rules.get(&id));
        let profile_opt = profile_id_opt.and_then(|id| self.profiles.get(&id));

        let rule_theme = rule_opt.and_then(|rule| match color_scheme_kind {
            ColorSchemeKind::Dark => rule.syntax_theme_dark.clone(),
            ColorSchemeKind::Light => rule.syntax_theme_light.clone(),
        });

        Appearance {
            // `syntax_theme` already resolves profile over global, so the rule
            // is the only layer left to add on top.
            syntax_theme: rule_theme
                .unwrap_or_else(|| self.syntax_theme(color_scheme_kind, profile_id_opt).0),
            opacity: self.effective_opacity(dir_rule_id_opt),
            tab_title: rule_opt
                .and_then(|rule| rule.tab_title.clone())
                .or_else(|| {
                    profile_opt
                        .map(|profile| profile.tab_title.clone())
                        .filter(|title| !title.is_empty())
                })
                .filter(|title| !title.is_empty())
                // Last, so it only fills a gap: a title written on the rule or
                // on the profile is a choice, and a choice outranks a name we
                // inferred. See [`DirRule::derived_title`].
                .or_else(|| rule_opt.and_then(DirRule::derived_title)),
            // Rule-only on purpose: a profile says *what to run*, and the same
            // profile is meant to be reusable across folders. Letting it carry
            // an identity color would make two folders running the same profile
            // claim the same identity.
            accent: rule_opt.and_then(DirRule::color),
        }
    }

    /// The name and identity color a directory carries, for callers outside the
    /// terminal — the statusline being the one that exists today.
    ///
    /// The name comes back with `{title}` already collapsed: a caller that is
    /// not a terminal has no program title to substitute, and leaking the raw
    /// placeholder into a status bar would be worse than dropping it.
    pub fn dir_identity(&self, dir: &Path) -> (Option<String>, Option<HexColor>) {
        let Some(rule) = resolve_dir_rule(&self.dir_rules, dir).and_then(|id| self.dir_rules.get(&id))
        else {
            return (None, None);
        };

        let name = rule
            .tab_title
            .as_deref()
            .map(|name| render_tab_title(name, None))
            .filter(|name| !name.is_empty())
            // Same fallback, same order as the tab's, so the status bar and the
            // window heading cannot end up calling one folder two things.
            .or_else(|| rule.derived_title());

        (name, rule.color())
    }

    /// Folds the old separate cursor color into the folder's single color, and
    /// says whether anything moved.
    ///
    /// Needed because the old field is read but never written: without this,
    /// the next save of an untouched rule would quietly drop a color the user
    /// had chosen. Called once at startup, so the file is rewritten on our
    /// terms rather than as a side effect of some unrelated edit.
    pub fn migrate_dir_rule_colors(&mut self) -> bool {
        let mut moved = false;
        for rule in self.dir_rules.values_mut() {
            if let Some(legacy) = rule.legacy_cursor.take() {
                // A rule that set both keeps the accent: that is the one the
                // chrome and the statusline were already showing.
                if rule.accent.is_none() {
                    rule.accent = Some(legacy);
                }
                moved = true;
            }
        }
        moved
    }

    pub fn typed_font_stretch(&self) -> Stretch {
        macro_rules! populate_num_typed_map {
            ($($stretch:ident,)+) => {
                let mut map = BTreeMap::new();
                $(map.insert(Stretch::$stretch.to_number(), Stretch::$stretch);)+
                map
            };
        }

        static NUM_TO_TYPED_MAP: OnceLock<BTreeMap<u16, Stretch>> = OnceLock::new();

        NUM_TO_TYPED_MAP.get_or_init(|| {
            populate_num_typed_map! {
                UltraCondensed, ExtraCondensed, Condensed, SemiCondensed,
                Normal, SemiExpanded, Expanded, ExtraExpanded, UltraExpanded,
            }
        })[&self.font_stretch]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(path: &str) -> DirRule {
        DirRule {
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn rules(list: &[(u64, DirRule)]) -> BTreeMap<DirRuleId, DirRule> {
        list.iter()
            .map(|(id, rule)| (DirRuleId(*id), rule.clone()))
            .collect()
    }

    #[test]
    fn a_rule_covers_only_its_own_folder() {
        // Each folder has its own identity: painting ~/projects must leave the
        // projects inside it alone.
        let rules = rules(&[(1, rule("/home/nico/projects"))]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects")),
            Some(DirRuleId(1))
        );
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/foo")),
            None
        );
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/foo/bar")),
            None
        );
    }

    #[test]
    fn a_rule_can_opt_into_covering_its_subtree() {
        let rules = rules(&[(
            1,
            DirRule {
                include_subdirs: true,
                ..rule("/home/nico/projects")
            },
        )]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/foo/bar")),
            Some(DirRuleId(1))
        );
    }

    #[test]
    fn a_folders_own_rule_beats_a_subtree_reaching_it() {
        // Only reachable once someone opts into subtrees: the folder's own
        // identity must still win over the tree it happens to sit in.
        let rules = rules(&[
            (
                1,
                DirRule {
                    include_subdirs: true,
                    ..rule("/home/nico/projects")
                },
            ),
            (2, rule("/home/nico/projects/prod")),
        ]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/prod")),
            Some(DirRuleId(2))
        );
        // And a folder with no rule of its own falls to the subtree rule.
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/dev")),
            Some(DirRuleId(1))
        );
        // The deeper rule does not cover a subtree, so its children are the
        // outer tree's again.
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/prod/src")),
            Some(DirRuleId(1))
        );
    }

    #[test]
    fn matching_is_by_path_component_not_string_prefix() {
        // `/home/a` must not capture `/home/ab`. String-prefix matching would,
        // and would paint an unrelated folder.
        let rules = rules(&[(1, rule("/home/a"))]);
        assert_eq!(resolve_dir_rule(&rules, Path::new("/home/ab")), None);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/a")),
            Some(DirRuleId(1))
        );
    }

    #[test]
    fn a_disabled_rule_never_matches() {
        let rules = rules(&[(
            1,
            DirRule {
                enabled: false,
                ..rule("/home/nico")
            },
        )]);
        assert_eq!(resolve_dir_rule(&rules, Path::new("/home/nico")), None);
    }

    #[test]
    fn a_parked_rule_does_not_shadow_a_live_one() {
        // Disabling a folder's own rule has to hand it back to the subtree rule
        // reaching it, not leave it unmatched.
        let rules = rules(&[
            (
                1,
                DirRule {
                    include_subdirs: true,
                    ..rule("/home/nico")
                },
            ),
            (
                2,
                DirRule {
                    enabled: false,
                    ..rule("/home/nico/projects")
                },
            ),
        ]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects")),
            Some(DirRuleId(1))
        );
    }

    #[test]
    fn overlapping_rules_resolve_deterministically() {
        // Two rules on the same path: the answer must not depend on iteration
        // order, or the terminal would flicker between them across restarts.
        let rules = rules(&[(7, rule("/home/nico")), (3, rule("/home/nico"))]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico")),
            Some(DirRuleId(3))
        );
    }

    #[test]
    fn an_unmatched_directory_has_no_rule() {
        let rules = rules(&[(1, rule("/home/nico/projects"))]);
        assert_eq!(resolve_dir_rule(&rules, Path::new("/etc")), None);
        assert_eq!(resolve_dir_rule(&BTreeMap::new(), Path::new("/etc")), None);
    }

    #[test]
    fn a_tilde_path_expands_to_the_home_directory() {
        let Some(home) = std::env::home_dir() else {
            return;
        };
        let rules = rules(&[(1, rule("~/projects"))]);
        assert_eq!(
            resolve_dir_rule(&rules, &home.join("projects")),
            Some(DirRuleId(1))
        );
    }

    #[test]
    fn unusable_rule_paths_match_nothing() {
        // A relative path would follow whatever directory the terminal is in,
        // and `~other` is a shell construct we do not implement — neither may
        // be silently reinterpreted into a real directory.
        assert_eq!(rule("projects").absolute_path(), None);
        assert_eq!(rule("").absolute_path(), None);
        assert_eq!(rule("   ").absolute_path(), None);
        assert_eq!(rule("~otheruser/projects").absolute_path(), None);
    }

    #[test]
    fn a_rule_only_overrides_the_fields_it_sets() {
        // The independence guarantee: a rule that pins a color must leave
        // opacity and cursor inheriting from the layers below it.
        //
        // The title is the one exception, and deliberately so: there is no
        // layer below it to inherit *from* — an untitled folder falls through
        // to "COSMIC Terminal", which names the program and not the place. So
        // an enabled rule always yields a name. See [`DirRule::derived_title`].
        let mut config = Config::default();
        config.opacity = 90;
        config.syntax_theme_dark = "Global Dark".to_string();
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                syntax_theme_dark: Some("Rule Dark".to_string()),
                ..rule("/home/nico")
            },
        );

        let appearance =
            config.effective_appearance(ColorSchemeKind::Dark, None, Some(DirRuleId(1)));
        assert_eq!(appearance.syntax_theme, "Rule Dark");
        assert_eq!(appearance.opacity, 90, "opacity must still come from global");
        assert_eq!(appearance.tab_title.as_deref(), Some("NICO"));
        assert_eq!(appearance.accent, None);
    }

    #[test]
    fn a_rule_outranks_its_profile_field_by_field() {
        let mut config = Config::default();
        config.syntax_theme_dark = "Global Dark".to_string();
        config.profiles.insert(
            ProfileId(1),
            Profile {
                syntax_theme_dark: "Profile Dark".to_string(),
                tab_title: "Profile title".to_string(),
                ..Default::default()
            },
        );
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                // Sets a title but no theme: the theme must fall through to the
                // profile, not skip it and land on the global.
                tab_title: Some("Rule title".to_string()),
                ..rule("/home/nico")
            },
        );

        let appearance = config.effective_appearance(
            ColorSchemeKind::Dark,
            Some(ProfileId(1)),
            Some(DirRuleId(1)),
        );
        assert_eq!(appearance.syntax_theme, "Profile Dark");
        assert_eq!(appearance.tab_title.as_deref(), Some("Rule title"));
    }

    #[test]
    fn no_rule_leaves_the_existing_behaviour_untouched() {
        // F1 must be inert until something starts resolving rules.
        let mut config = Config::default();
        config.opacity = 75;
        config.syntax_theme_light = "Global Light".to_string();

        let appearance = config.effective_appearance(ColorSchemeKind::Light, None, None);
        assert_eq!(appearance.syntax_theme, "Global Light");
        assert_eq!(appearance.opacity, 75);
        assert_eq!(appearance.tab_title, None);
        assert_eq!(appearance.accent, None);
    }

    #[test]
    fn the_documented_example_parses() {
        // The snippet README.md tells people to type. If this drifts, the only
        // documented way to use the feature today stops working.
        let written: BTreeMap<DirRuleId, DirRule> = ron::from_str(
            r##"{
                1: (path: "~/projects", opacity: Some(85), syntax_theme_dark: Some("Dracula")),
                2: (path: "~/projects/prod", tab_title: Some("PROD"), accent: Some("#ff0000")),
                3: (path: "/srv", include_subdirs: true, syntax_theme_dark: Some("Solarized Dark")),
            }"##,
        )
        .expect("the example in the docs must parse");

        assert_eq!(written[&DirRuleId(1)].opacity, Some(85));
        assert_eq!(
            written[&DirRuleId(2)].color(),
            Some(HexColor::rgb(0xff, 0x00, 0x00))
        );
        assert!(written[&DirRuleId(3)].include_subdirs);

        // And the documented outcome holds: rule 1 stops at its own folder.
        let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("/home/nobody"));
        assert_eq!(
            resolve_dir_rule(&written, &home.join("projects/foo")),
            None,
            "a folder without a rule must keep the global appearance"
        );
        assert_eq!(
            resolve_dir_rule(&written, Path::new("/srv/anything/deep")),
            Some(DirRuleId(3)),
            "the subtree rule must reach down"
        );
    }

    #[test]
    fn saving_an_appearance_twice_edits_one_rule_instead_of_stacking() {
        // "Use this appearance here" has to find the folder's existing rule.
        // Adding a second rule on the same path would leave a dead entry that
        // the resolver ignores, and the user would see their edit do nothing.
        let mut config = Config::default();
        config
            .dir_rules
            .insert(DirRuleId(1), rule("/home/nico/projects"));
        config.dir_rules.insert(DirRuleId(2), rule("/srv"));

        assert_eq!(
            config.dir_rule_for_exact_path(Path::new("/home/nico/projects")),
            Some(DirRuleId(1))
        );
        assert_eq!(config.dir_rule_for_exact_path(Path::new("/srv")), Some(DirRuleId(2)));
        // A folder merely *covered* by a rule is not the same as one the rule
        // names: saving there must make a new rule, not edit the parent's.
        assert_eq!(
            config.dir_rule_for_exact_path(Path::new("/home/nico/projects/foo")),
            None
        );
    }

    #[test]
    fn a_new_rule_carries_the_folder_and_nothing_else() {
        // Making a rule for a folder must not quietly freeze the theme and the
        // transparency it happens to have right now: the dialog no longer shows
        // either, so a rule that pinned them could never be unpinned.
        let mut config = Config::default();
        config.opacity = 72;
        config.syntax_theme_dark = "Global Dark".to_string();
        config.syntax_theme_light = "Global Light".to_string();

        let fresh = config.dir_rule_from_current("/srv".to_string());
        assert_eq!(fresh.syntax_theme_dark, None);
        assert_eq!(fresh.syntax_theme_light, None);
        assert_eq!(fresh.opacity, None);
        assert_eq!(fresh.tab_title, None);
        assert_eq!(fresh.accent, None);
        assert!(fresh.enabled);
        assert!(!fresh.include_subdirs, "a rule must not paint the subtree");

        // So the folder keeps following the global settings until it is given
        // something of its own.
        config.dir_rules.insert(DirRuleId(1), fresh);
        assert_eq!(config.effective_opacity(Some(DirRuleId(1))), 72);
        config.opacity = 30;
        assert_eq!(config.effective_opacity(Some(DirRuleId(1))), 30);
    }

    #[test]
    fn the_rules_list_shows_the_folder_name_alone() {
        // The list is a set of identities, not a set of paths — and the paths in
        // it are long and mostly identical to each other.
        assert_eq!(
            rule("/home/nico/Apps Workspace/Pop Flow").folder_name().as_deref(),
            Some("Pop Flow")
        );
        // The title derived for an unnamed folder is the same name, shouted.
        assert_eq!(
            rule("/home/nico/Apps Workspace/Pop Flow").derived_title().as_deref(),
            Some("POP FLOW")
        );
        // A path that names no folder has no name to show; the caller falls
        // back to the path it was given.
        assert_eq!(rule("/").folder_name(), None);
    }

    #[test]
    fn new_rule_ids_do_not_collide_with_existing_ones() {
        let mut config = Config::default();
        assert_eq!(config.next_dir_rule_id(), DirRuleId(0));
        config.dir_rules.insert(DirRuleId(0), rule("/a"));
        config.dir_rules.insert(DirRuleId(4), rule("/b"));
        assert_eq!(config.next_dir_rule_id(), DirRuleId(5));
    }

    #[test]
    fn rules_are_listed_by_path_not_by_insertion_order() {
        let mut config = Config::default();
        config.dir_rules.insert(DirRuleId(1), rule("/srv"));
        config.dir_rules.insert(DirRuleId(2), rule("/etc"));
        config.dir_rules.insert(DirRuleId(3), rule("/home"));

        let listed: Vec<String> = config
            .dir_rule_paths()
            .into_iter()
            .map(|(path, _)| path)
            .collect();
        assert_eq!(listed, ["/etc", "/home", "/srv"]);
    }

    #[test]
    fn a_hand_written_rule_file_loads_with_sane_defaults() {
        // Rules are meant to be editable by hand in the config store, so the
        // minimal thing someone would actually type has to work: naming a path
        // and one field, and getting the documented defaults for the rest.
        let written: BTreeMap<DirRuleId, DirRule> = ron::from_str(
            r#"{
                1: (path: "~/projects", opacity: Some(85)),
            }"#,
        )
        .expect("a minimal hand-written rule must parse");

        let rule = &written[&DirRuleId(1)];
        assert_eq!(rule.path, "~/projects");
        assert_eq!(rule.opacity, Some(85));
        assert!(rule.enabled, "a rule must be live unless it says otherwise");
        assert!(
            !rule.include_subdirs,
            "a folder's appearance must not spread to its children by default"
        );
        assert_eq!(rule.syntax_theme_dark, None);
        assert_eq!(rule.color(), None);
    }

    #[test]
    fn rules_survive_a_round_trip_through_the_config_store() {
        let mut rules = BTreeMap::new();
        rules.insert(
            DirRuleId(1),
            DirRule {
                include_subdirs: false,
                opacity: Some(70),
                accent: Some(HexColor::rgb(0x00, 0xff, 0x00)),
                tab_title: Some("prod".to_string()),
                syntax_theme_dark: Some("Red Alert".to_string()),
                ..rule("/srv/prod")
            },
        );

        let encoded = ron::to_string(&rules).expect("rules must serialize");
        let decoded: BTreeMap<DirRuleId, DirRule> =
            ron::from_str(&encoded).expect("rules must round-trip");
        assert_eq!(decoded, rules);
    }

    #[test]
    fn a_pinned_opacity_is_distinguishable_from_an_inherited_one() {
        // The view needs to tell "this folder asked for 90%" apart from
        // "nobody asked, the global happens to be 90%", because blur treats
        // those two differently.
        let mut config = Config::default();
        config.opacity = 90;
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                opacity: Some(90),
                ..rule("/home/nico/pinned")
            },
        );
        config
            .dir_rules
            .insert(DirRuleId(2), rule("/home/nico/plain"));

        assert_eq!(config.dir_rule_opacity(Some(DirRuleId(1))), Some(90));
        assert_eq!(config.dir_rule_opacity(Some(DirRuleId(2))), None);
        assert_eq!(config.dir_rule_opacity(None), None);

        // Either way the effective value is the same 90.
        assert_eq!(config.effective_opacity(Some(DirRuleId(1))), 90);
        assert_eq!(config.effective_opacity(Some(DirRuleId(2))), 90);
        assert_eq!(config.effective_opacity(None), 90);
    }

    #[test]
    fn the_folder_color_is_the_cursor_color() {
        let mut config = Config::default();
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                accent: Some(HexColor::rgb(0xff, 0x00, 0x00)),
                ..rule("/home/nico")
            },
        );

        let appearance =
            config.effective_appearance(ColorSchemeKind::Dark, None, Some(DirRuleId(1)));
        // One color, and the cursor is painted from it — the terminal reads
        // this same field for both.
        assert_eq!(appearance.accent, Some(HexColor::rgb(0xff, 0x00, 0x00)));
        // And a terminal with no rule keeps whatever its scheme says.
        assert_eq!(
            config
                .effective_appearance(ColorSchemeKind::Dark, None, None)
                .accent,
            None
        );
    }

    #[test]
    fn a_rule_written_before_the_colors_merged_keeps_its_color() {
        // Two fields became one. A rule that had only the old cursor color must
        // still show a color, and the merge must be written back so that the
        // next save does not drop it.
        let mut config = Config::default();
        let mut rules: BTreeMap<DirRuleId, DirRule> = ron::from_str(
            r##"{
                1: (path: "/srv/old", cursor: Some("#ffffff")),
                2: (path: "/srv/both", cursor: Some("#111111"), accent: Some("#222222")),
            }"##,
        )
        .expect("a rule from before the merge must still parse");
        assert_eq!(
            rules[&DirRuleId(1)].color(),
            Some(HexColor::rgb(0xff, 0xff, 0xff))
        );
        // A rule that set both keeps the accent: that is the color the chrome
        // and the statusline were already showing.
        assert_eq!(
            rules[&DirRuleId(2)].color(),
            Some(HexColor::rgb(0x22, 0x22, 0x22))
        );

        config.dir_rules = std::mem::take(&mut rules);
        assert!(config.migrate_dir_rule_colors());
        assert_eq!(
            config.dir_rules[&DirRuleId(1)].accent,
            Some(HexColor::rgb(0xff, 0xff, 0xff))
        );
        assert_eq!(
            config.dir_rules[&DirRuleId(2)].accent,
            Some(HexColor::rgb(0x22, 0x22, 0x22))
        );
        assert!(
            !config.migrate_dir_rule_colors(),
            "a config that has already been merged must not keep rewriting itself"
        );

        // And the old key is gone from what we write.
        let encoded = ron::to_string(&config.dir_rules).expect("rules must serialize");
        assert!(!encoded.contains("cursor"));
    }

    #[test]
    fn an_empty_profile_title_is_not_a_title() {
        // Profile.tab_title is a String, and upstream treats empty as unset.
        let mut config = Config::default();
        config
            .profiles
            .insert(ProfileId(1), Profile::default());

        let appearance =
            config.effective_appearance(ColorSchemeKind::Dark, Some(ProfileId(1)), None);
        assert_eq!(appearance.tab_title, None);
    }

    #[test]
    fn the_accent_comes_from_the_rule_and_nowhere_else() {
        let mut config = Config::default();
        config.profiles.insert(ProfileId(1), Profile::default());
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                accent: Some(HexColor::rgb(0x48, 0xb9, 0xc7)),
                ..rule("/home/nico")
            },
        );

        assert_eq!(
            config
                .effective_appearance(ColorSchemeKind::Dark, Some(ProfileId(1)), Some(DirRuleId(1)))
                .accent,
            Some(HexColor::rgb(0x48, 0xb9, 0xc7))
        );
        // No rule means no identity: the window must keep the system accent
        // rather than invent one.
        assert_eq!(
            config
                .effective_appearance(ColorSchemeKind::Dark, Some(ProfileId(1)), None)
                .accent,
            None
        );
    }

    #[test]
    fn a_name_without_the_placeholder_replaces_the_program_title() {
        // The pre-placeholder behaviour, which someone who wants a tab to say
        // one thing and stay there still depends on.
        assert_eq!(render_tab_title("PROD", Some("vim src/main.rs")), "PROD");
        assert_eq!(render_tab_title("PROD", None), "PROD");
    }

    #[test]
    fn the_placeholder_lets_the_name_and_the_live_title_coexist() {
        assert_eq!(
            render_tab_title("POP FLOW — {title}", Some("vim src/main.rs")),
            "POP FLOW — vim src/main.rs"
        );
        // Either side of the name, because a leading placeholder is as natural
        // to write as a trailing one.
        assert_eq!(
            render_tab_title("{title} · POP FLOW", Some("claude")),
            "claude · POP FLOW"
        );
    }

    #[test]
    fn an_empty_live_title_does_not_leave_a_dangling_separator() {
        // A fresh shell sets no title. Without the tidy-up the tab would read
        // "POP FLOW —", which looks like a bug every time it happens.
        assert_eq!(render_tab_title("POP FLOW — {title}", None), "POP FLOW");
        assert_eq!(render_tab_title("POP FLOW — {title}", Some("  ")), "POP FLOW");
        assert_eq!(render_tab_title("{title} · POP FLOW", None), "POP FLOW");
        // A name that is nothing but the placeholder collapses to nothing, and
        // the caller falls back to its own default.
        assert_eq!(render_tab_title("{title}", None), "");
    }

    #[test]
    fn dir_identity_answers_what_the_statusline_asks() {
        let mut config = Config::default();
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                tab_title: Some("POP FLOW — {title}".to_string()),
                accent: Some(HexColor::rgb(0x48, 0xb9, 0xc7)),
                ..rule("/home/nico/flow")
            },
        );

        // The placeholder is collapsed: a status bar has no program title to
        // put there, and printing "{title}" raw would be worse than dropping it.
        let (name, accent) = config.dir_identity(Path::new("/home/nico/flow"));
        assert_eq!(name.as_deref(), Some("POP FLOW"));
        assert_eq!(accent, Some(HexColor::rgb(0x48, 0xb9, 0xc7)));

        // A folder with no rule has no identity to report, and the caller keeps
        // whatever default it had.
        let (name, accent) = config.dir_identity(Path::new("/home/nico/outra"));
        assert_eq!(name, None);
        assert_eq!(accent, None);
    }

    #[test]
    fn dir_identity_respects_the_one_folder_rule() {
        // Same guarantee as the rest of T1: a rule covers its folder, not the
        // tree below it, unless it opts in.
        let mut config = Config::default();
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                tab_title: Some("POP FLOW".to_string()),
                accent: Some(HexColor::rgb(0x48, 0xb9, 0xc7)),
                ..rule("/home/nico/flow")
            },
        );

        assert_eq!(
            config.dir_identity(Path::new("/home/nico/flow/sub")).0,
            None
        );

        config
            .dir_rules
            .get_mut(&DirRuleId(1))
            .unwrap()
            .include_subdirs = true;
        assert_eq!(
            config.dir_identity(Path::new("/home/nico/flow/sub")).0.as_deref(),
            Some("POP FLOW")
        );
    }

    #[test]
    fn a_titleless_rule_still_names_its_folder() {
        // The case this exists for: a rule that only paints the folder. It is
        // switched on, so the folder has an identity — it just never got a name
        // typed into it.
        let mut config = Config::default();
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                accent: Some(HexColor::rgb(0x48, 0xb9, 0xc7)),
                ..rule("/home/nico/Pop Flow")
            },
        );

        assert_eq!(
            config.dir_identity(Path::new("/home/nico/Pop Flow")).0.as_deref(),
            Some("POP FLOW")
        );

        // The window has to reach the same name by its own path, or the two
        // surfaces disagree about one folder — which is the whole point.
        let appearance =
            config.effective_appearance(ColorSchemeKind::Dark, None, Some(DirRuleId(1)));
        assert_eq!(appearance.tab_title.as_deref(), Some("POP FLOW"));
    }

    #[test]
    fn a_written_title_outranks_the_derived_one() {
        let mut config = Config::default();
        config.profiles.insert(
            ProfileId(7),
            Profile {
                tab_title: "From the profile".to_string(),
                ..Default::default()
            },
        );
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                tab_title: Some("PROD".to_string()),
                ..rule("/home/nico/Pop Flow")
            },
        );

        let appearance = config.effective_appearance(
            ColorSchemeKind::Dark,
            Some(ProfileId(7)),
            Some(DirRuleId(1)),
        );
        assert_eq!(appearance.tab_title.as_deref(), Some("PROD"));

        // With the rule's title cleared the profile's is still a choice someone
        // made, so it comes before a name we inferred.
        config.dir_rules.get_mut(&DirRuleId(1)).unwrap().tab_title = None;
        let appearance = config.effective_appearance(
            ColorSchemeKind::Dark,
            Some(ProfileId(7)),
            Some(DirRuleId(1)),
        );
        assert_eq!(appearance.tab_title.as_deref(), Some("From the profile"));
    }

    #[test]
    fn a_folder_with_no_name_to_derive_gets_none() {
        // Nothing to name it after, so nothing is invented — the caller keeps
        // whatever fallback it already had.
        assert_eq!(rule("/").derived_title(), None);
        assert_eq!(rule("").derived_title(), None);
        // Relative paths never match a directory, so they never name one.
        assert_eq!(rule("projects/prod").derived_title(), None);
    }

    #[test]
    fn a_folder_with_no_rule_is_still_nobodys_business() {
        // The fallback is tied to an *enabled* rule. A disabled one is parked,
        // and a folder nobody wrote a rule for must stay unnamed.
        let mut config = Config::default();
        config.dir_rules.insert(
            DirRuleId(1),
            DirRule {
                enabled: false,
                ..rule("/home/nico/Pop Flow")
            },
        );

        assert_eq!(config.dir_identity(Path::new("/home/nico/Pop Flow")).0, None);
        assert_eq!(config.dir_identity(Path::new("/home/nico/outra")).0, None);
    }
}
