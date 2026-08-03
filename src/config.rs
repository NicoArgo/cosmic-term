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
    /// Whether the rule also covers directories below `path`. With this off the
    /// rule only fires on the exact directory.
    pub include_subdirs: bool,
    /// Lets a rule be kept but parked, instead of having to delete it.
    pub enabled: bool,
    pub syntax_theme_dark: Option<String>,
    pub syntax_theme_light: Option<String>,
    pub opacity: Option<u8>,
    pub tab_title: Option<String>,
    pub cursor: Option<HexColor>,
}

impl Default for DirRule {
    fn default() -> Self {
        Self {
            path: String::new(),
            // A rule on `~/projects` covering only `~/projects` itself and none
            // of the projects inside it would surprise most people.
            include_subdirs: true,
            enabled: true,
            syntax_theme_dark: None,
            syntax_theme_light: None,
            opacity: None,
            tab_title: None,
            cursor: None,
        }
    }
}

impl DirRule {
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
/// The most specific rule wins, measured in path components matched — so a rule
/// on `~/projects/prod` beats one on `~/projects`, and "exact match beats
/// inherited-from-parent" falls out of that for free. Ties (only possible
/// between rules with the same path) go to the lowest id, so the result never
/// depends on iteration luck.
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
    pub cursor: Option<HexColor>,
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
            // Profiles carry no opacity of their own, so this falls straight
            // through to the global value.
            opacity: rule_opt.and_then(|rule| rule.opacity).unwrap_or(self.opacity),
            tab_title: rule_opt
                .and_then(|rule| rule.tab_title.clone())
                .or_else(|| {
                    profile_opt
                        .map(|profile| profile.tab_title.clone())
                        .filter(|title| !title.is_empty())
                })
                .filter(|title| !title.is_empty()),
            cursor: rule_opt.and_then(|rule| rule.cursor),
        }
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
    fn a_rule_covers_its_subdirectories() {
        let rules = rules(&[(1, rule("/home/nico/projects"))]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/foo/bar")),
            Some(DirRuleId(1))
        );
    }

    #[test]
    fn the_most_specific_rule_wins() {
        // The whole point of nesting: a rule deeper in the tree must override
        // the one it sits inside, no matter what order they are stored in.
        let rules = rules(&[
            (1, rule("/home/nico/projects")),
            (2, rule("/home/nico/projects/prod")),
        ]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/prod/src")),
            Some(DirRuleId(2))
        );
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/dev")),
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
    fn subdirectories_can_be_excluded() {
        let rules = rules(&[(
            1,
            DirRule {
                include_subdirs: false,
                ..rule("/home/nico/projects")
            },
        )]);
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects")),
            Some(DirRuleId(1))
        );
        assert_eq!(
            resolve_dir_rule(&rules, Path::new("/home/nico/projects/foo")),
            None
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
        // Disabling the deeper rule has to hand the directory back to the
        // shallower one, not leave it unmatched.
        let rules = rules(&[
            (1, rule("/home/nico")),
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
            resolve_dir_rule(&rules, &home.join("projects/foo")),
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
        // opacity, title and cursor inheriting from the layers below it.
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
        assert_eq!(appearance.tab_title, None);
        assert_eq!(appearance.cursor, None);
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
        assert_eq!(appearance.cursor, None);
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
}
