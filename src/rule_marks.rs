// SPDX-License-Identifier: GPL-3.0-only

//! POP Flow: a folder's rule follows the folder when it moves.
//!
//! A rule names its folder by path, so moving the folder used to strand it:
//! on 23/11/2026, 46 of 85 rules pointed at folders that no longer existed,
//! and the moved folders had been painted again by hand.
//!
//! Two things keep a rule attached now:
//!
//! - The file manager tells the terminal when it moves a folder
//!   (`cosmic-term --move-rules OLD NEW`), and every rule at or below OLD is
//!   moved along: [`move_rules`].
//! - For moves made anywhere else (`mv`, another program), each ruled folder
//!   carries an extended attribute, [`MARK`], holding the path its rule was
//!   made for. A rename keeps it. When a terminal stands in a folder with no
//!   rule whose mark names a rule's path that is gone, the rule moves here:
//!   [`find_moved`].
//!
//! The mark holds a path rather than the rule's id because ids get reused
//! after a delete: a stale id could hand a folder somebody else's color, a
//! stale path can only match a rule that is missing its folder.

use crate::config::{DirRule, DirRuleId};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

pub const MARK: &str = "user.popflow.rule";

/// How far up from the terminal's folder to look for a mark: a rule covering
/// subfolders can sit a few levels above where the terminal stands.
const MAX_LEVELS_UP: usize = 8;

/// The path a folder's mark names, if it has one.
pub fn read_mark(dir: &Path) -> Option<PathBuf> {
    let value = xattr::get(dir, MARK).ok()??;
    (!value.is_empty()).then(|| PathBuf::from(OsStr::from_bytes(&value)))
}

/// Mark `dir` as the folder its rule was made for. Silent on failure: a file
/// system without user attributes just loses the `mv` case, nothing else.
pub fn mark(dir: &Path) {
    if read_mark(dir).as_deref() == Some(dir) {
        return;
    }
    if let Err(err) = xattr::set(dir, MARK, dir.as_os_str().as_bytes()) {
        log::debug!("could not mark {}: {err}", dir.display());
    }
}

/// Mark every folder that has a rule and exists. Run after each save, so a
/// rule made by any route (the menu, the rules page, `--set-rule`) is covered,
/// and rules from before marks existed get theirs the next time anything saves.
pub fn mark_all(rules: &BTreeMap<DirRuleId, DirRule>) {
    for rule in rules.values() {
        if let Some(path) = rule.absolute_path()
            && path.is_dir()
        {
            mark(&path);
        }
    }
}

/// Move every rule at or below `from` to the same place under `to`. Returns
/// how many moved.
pub fn move_rules(rules: &mut BTreeMap<DirRuleId, DirRule>, from: &Path, to: &Path) -> usize {
    let mut moved = 0;
    for rule in rules.values_mut() {
        let Some(path) = rule.absolute_path() else {
            continue;
        };
        let Ok(rest) = path.strip_prefix(from) else {
            continue;
        };
        let new_path = if rest.as_os_str().is_empty() {
            to.to_path_buf()
        } else {
            to.join(rest)
        };
        if let Some(text) = new_path.to_str() {
            rule.path = text.to_string();
            moved += 1;
        }
    }
    moved
}

/// `cwd`, or a folder above it, carries a mark naming a rule whose folder is
/// gone: that folder was moved here. Returns `(old, new)` for [`move_rules`].
///
/// `folder_exists` is a parameter so the decision can be tested without a
/// disk; callers pass `Path::exists`.
pub fn find_moved(
    rules: &BTreeMap<DirRuleId, DirRule>,
    cwd: &Path,
    read_mark: impl Fn(&Path) -> Option<PathBuf>,
    folder_exists: impl Fn(&Path) -> bool,
) -> Option<(PathBuf, PathBuf)> {
    for dir in cwd.ancestors().take(MAX_LEVELS_UP) {
        let Some(old) = read_mark(dir) else {
            continue;
        };
        if old == dir {
            continue;
        }
        let has_rule = rules
            .values()
            .any(|rule| rule.absolute_path().as_deref() == Some(old.as_path()));
        if has_rule && !folder_exists(&old) {
            return Some((old, dir.to_path_buf()));
        }
    }
    None
}

/// What `--repair-rules` would do with a rule whose folder is gone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Repair {
    /// One folder of that name exists, it has no rule, and it is not inside
    /// a folder the rules know is gone: move the rule there.
    Move(PathBuf),
    /// None or several candidates: keep the rule, switched off.
    Disable,
}

/// For each rule pointing at a folder that no longer exists, a repair.
///
/// `marked` maps the path a mark names to the folder carrying it: a folder
/// that says it is the one a rule was made for wins outright. `candidates`
/// maps a folder name to every folder of that name found on disk, the guess
/// for rules older than marks.
pub fn plan_repairs(
    rules: &BTreeMap<DirRuleId, DirRule>,
    marked: &BTreeMap<PathBuf, PathBuf>,
    candidates: &BTreeMap<String, Vec<PathBuf>>,
    folder_exists: impl Fn(&Path) -> bool,
) -> Vec<(DirRuleId, PathBuf, Repair)> {
    let ruled: Vec<PathBuf> = rules.values().filter_map(DirRule::absolute_path).collect();
    let mut plan = Vec::new();
    let mut taken: Vec<PathBuf> = Vec::new();
    for (id, rule) in rules {
        let Some(path) = rule.absolute_path() else {
            continue;
        };
        if folder_exists(&path) {
            continue;
        }
        if !rule.enabled {
            // Already parked: nothing to say about it.
            continue;
        }
        if let Some(to) = marked.get(&path) {
            taken.push(to.clone());
            plan.push((*id, path, Repair::Move(to.clone())));
            continue;
        }
        let repair = rule
            .folder_name()
            .and_then(|name| candidates.get(&name))
            .and_then(|found| match found.as_slice() {
                [only] if !ruled.contains(only) && !taken.contains(only) => Some(only.clone()),
                _ => None,
            })
            .map_or(Repair::Disable, Repair::Move);
        if let Repair::Move(to) = &repair {
            taken.push(to.clone());
        }
        plan.push((*id, path, repair));
    }
    plan
}

/// Every folder under `root`, up to `max_depth` levels down, whose name is in
/// `names`, and every marked folder (as mark -> folder). Skips hidden folders
/// and build or dependency trees, which hold no project folders and would
/// make this slow.
pub fn scan_candidates(
    root: &Path,
    names: &[String],
    max_depth: usize,
) -> (BTreeMap<PathBuf, PathBuf>, BTreeMap<String, Vec<PathBuf>>) {
    let mut marked = BTreeMap::new();
    const SKIP: &[&str] = &["node_modules", "target", "__pycache__", "venv", "dist", "build"];
    let mut found: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut stack = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if !kind.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.starts_with('.') || SKIP.contains(&name) {
                continue;
            }
            let path = entry.path();
            if let Some(mark) = read_mark(&path)
                && mark != path
            {
                marked.insert(mark, path.clone());
            }
            if names.iter().any(|wanted| wanted == name) {
                found.entry(name.to_string()).or_default().push(path.clone());
            }
            if depth + 1 < max_depth {
                stack.push((path, depth + 1));
            }
        }
    }
    (marked, found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_beats_a_guess_by_name() {
        let r = rules(&["/old/cluster"]);
        let mut marked = BTreeMap::new();
        marked.insert(PathBuf::from("/old/cluster"), PathBuf::from("/y/cluster"));
        let mut candidates = BTreeMap::new();
        candidates.insert(
            "cluster".to_string(),
            vec![PathBuf::from("/x/cluster"), PathBuf::from("/y/cluster")],
        );
        assert_eq!(
            plan_repairs(&r, &marked, &candidates, |_| false),
            vec![(DirRuleId(0), "/old/cluster".into(), Repair::Move("/y/cluster".into()))]
        );
    }

    fn rules(paths: &[&str]) -> BTreeMap<DirRuleId, DirRule> {
        paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                (
                    DirRuleId(i as u64),
                    DirRule {
                        path: path.to_string(),
                        ..DirRule::default()
                    },
                )
            })
            .collect()
    }

    fn paths(rules: &BTreeMap<DirRuleId, DirRule>) -> Vec<&str> {
        rules.values().map(|rule| rule.path.as_str()).collect()
    }

    #[test]
    fn moving_a_folder_takes_its_rule_and_its_subfolders_rules() {
        let mut r = rules(&["/a/Proj", "/a/Proj/web", "/a/Projeto", "/b"]);
        assert_eq!(move_rules(&mut r, Path::new("/a/Proj"), Path::new("/c/Proj")), 2);
        assert_eq!(paths(&r), ["/c/Proj", "/c/Proj/web", "/a/Projeto", "/b"]);
    }

    #[test]
    fn a_marked_folder_whose_rule_lost_its_path_is_found() {
        let r = rules(&["/old/Orbitaz"]);
        let found = find_moved(
            &r,
            Path::new("/new/Orbitaz/src"),
            |dir| (dir == Path::new("/new/Orbitaz")).then(|| PathBuf::from("/old/Orbitaz")),
            |_| false,
        );
        assert_eq!(
            found,
            Some((PathBuf::from("/old/Orbitaz"), PathBuf::from("/new/Orbitaz")))
        );
    }

    #[test]
    fn a_copy_does_not_steal_the_rule_while_the_original_is_there() {
        // `cp -a` keeps attributes: the copy says /old/X, but /old/X is still
        // there and keeps its rule.
        let r = rules(&["/old/X"]);
        let found = find_moved(
            &r,
            Path::new("/copy/X"),
            |_| Some(PathBuf::from("/old/X")),
            |_| true,
        );
        assert_eq!(found, None);
    }

    #[test]
    fn a_mark_naming_no_rule_is_ignored() {
        let r = rules(&["/a"]);
        let found = find_moved(&r, Path::new("/b"), |_| Some(PathBuf::from("/gone")), |_| false);
        assert_eq!(found, None);
    }

    #[test]
    fn repairs_move_only_to_a_single_unruled_folder_of_the_same_name() {
        let r = rules(&["/old/Orbitaz", "/old/cluster", "/old/Lost", "/new/Ruled", "/old/Ruled"]);
        let mut candidates = BTreeMap::new();
        candidates.insert("Orbitaz".to_string(), vec![PathBuf::from("/new/Orbitaz")]);
        candidates.insert(
            "cluster".to_string(),
            vec![PathBuf::from("/x/cluster"), PathBuf::from("/y/cluster")],
        );
        candidates.insert("Ruled".to_string(), vec![PathBuf::from("/new/Ruled")]);
        let exists = |p: &Path| p.starts_with("/new");
        let plan = plan_repairs(&r, &BTreeMap::new(), &candidates, exists);
        assert_eq!(
            plan,
            vec![
                (DirRuleId(0), "/old/Orbitaz".into(), Repair::Move("/new/Orbitaz".into())),
                (DirRuleId(1), "/old/cluster".into(), Repair::Disable),
                (DirRuleId(2), "/old/Lost".into(), Repair::Disable),
                (DirRuleId(4), "/old/Ruled".into(), Repair::Disable),
            ]
        );
    }
}
