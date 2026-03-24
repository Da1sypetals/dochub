use crate::config::{self, Config};
use crate::paths::{has_git_component, normalize_join_input};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use strsim::jaro_winkler;
use walkdir::WalkDir;

/// Minimum Jaro–Winkler similarity to offer a "did you mean" prompt (TTY) or
/// hint line (non-TTY).
const FUZZY_HUB_THRESHOLD: f64 = 0.8;

pub fn add(skill_name: &str, path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("{} is not a directory.", path.display()));
    }

    let canonical = fs::canonicalize(path)
        .map_err(|err| format!("Failed to resolve {}: {err}", path.display()))?;

    let mut config = config::load()?;
    if config.hub.contains_key(skill_name) {
        return Err(format!("Hub entry `{skill_name}` already exists."));
    }

    config
        .hub
        .insert(skill_name.to_string(), canonical.display().to_string());
    config::save(&config)?;
    println!("{skill_name}\t{}", canonical.display());
    Ok(())
}

pub fn prune() -> Result<(), String> {
    let mut config = config::load()?;
    let mut removed = Vec::new();

    config.hub.retain(|skill_name, path| {
        if Path::new(path).exists() {
            true
        } else {
            removed.push((skill_name.clone(), path.clone()));
            false
        }
    });

    if removed.is_empty() {
        println!("No entries pruned.");
        return Ok(());
    }

    config::save(&config)?;
    for (skill_name, path) in removed {
        println!("{skill_name}\t{path}");
    }
    Ok(())
}

pub fn sanity() -> Result<(), String> {
    let config = config::load()?;
    let limit_bytes = config.sane_size_bytes();
    let limit_mb = config.sane_size.unwrap_or(16);
    let mut found = false;

    for (skill_name, path) in &config.hub {
        let hub_path = Path::new(path);
        if !hub_path.exists() {
            continue;
        }

        let size = directory_size(hub_path)?;
        if size > limit_bytes {
            found = true;
            println!(
                "{skill_name}\t{}\tsize={}B\tlimit={}MB",
                hub_path.display(),
                size,
                limit_mb
            );
        }
    }

    if !found {
        println!(
            "All entries are within the configured sane size ({}MB).",
            limit_mb
        );
    }

    Ok(())
}

pub fn cp(skill_name: &str, dest: &Path) -> Result<(), String> {
    let config = config::load()?;
    let (resolved_skill_name, source) = resolve_hub_source(&config, skill_name)?;
    let final_root = copy_hub_to(
        &config,
        &resolved_skill_name,
        &source,
        &normalize_join_input(dest),
    )?;
    println!("{}", final_root.display());
    Ok(())
}

pub fn hub_use(skill_name: &str, dest: Option<&Path>) -> Result<(), String> {
    let config = config::load()?;
    let (resolved_skill_name, source) = resolve_hub_source(&config, skill_name)?;

    if config.skill_dir.is_empty() {
        return Err("`skill-dir` is missing or empty in hub.toml.".to_string());
    }

    let base_dest = normalize_join_input(dest.unwrap_or_else(|| Path::new(".")));

    for skill_dir in &config.skill_dir {
        let skill_dir_path = Path::new(skill_dir);
        if skill_dir_path.is_absolute() {
            return Err(format!(
                "`skill-dir` entry `{skill_dir}` must be a relative path."
            ));
        }

        let normalized_skill_dir = normalize_join_input(skill_dir_path);
        let final_root = copy_hub_to(
            &config,
            &resolved_skill_name,
            &source,
            &base_dest.join(normalized_skill_dir),
        )?;
        println!("{}", final_root.display());
    }

    Ok(())
}

pub fn rm(skill_name: &str) -> Result<(), String> {
    let mut config = config::load()?;
    let path = config
        .hub
        .get(skill_name)
        .cloned()
        .ok_or_else(|| format!("Hub entry `{skill_name}` not found."))?;

    print!("Remove `{skill_name}` -> {path}? [y/N]: ");
    io::stdout()
        .flush()
        .map_err(|err| format!("Failed to flush stdout: {err}"))?;

    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .map_err(|err| format!("Failed to read confirmation: {err}"))?;

    let confirmed = matches!(response.trim(), "y" | "Y" | "yes" | "YES" | "Yes");
    if !confirmed {
        println!("Cancelled.");
        return Ok(());
    }

    config.hub.remove(skill_name);
    config::save(&config)?;
    println!("Removed `{skill_name}`.");
    Ok(())
}

pub fn ls(skill_name: Option<&str>) -> Result<(), String> {
    let config = config::load()?;

    let rows = match skill_name {
        Some(skill_name) => {
            let path = config
                .hub
                .get(skill_name)
                .ok_or_else(|| format!("Hub entry `{skill_name}` not found."))?;
            vec![(
                skill_name.to_string(),
                path.clone(),
                display_size(Path::new(path))?,
            )]
        }
        None => config
            .hub
            .iter()
            .map(|(skill_name, path)| {
                Ok((
                    skill_name.clone(),
                    path.clone(),
                    display_size(Path::new(path))?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?,
    };

    let skill_name_width = rows
        .iter()
        .map(|(skill_name, _, _)| skill_name.len())
        .max()
        .unwrap_or(0)
        .max("SKILL_NAME".len());
    let path_width = rows
        .iter()
        .map(|(_, path, _)| path.len())
        .max()
        .unwrap_or(0)
        .max("PATH".len());
    let size_width = rows
        .iter()
        .map(|(_, _, size)| size.len())
        .max()
        .unwrap_or(0)
        .max("SIZE".len());

    println!(
        "{:<skill_name_width$}  {:<path_width$}  {:>size_width$}",
        "SKILL_NAME",
        "PATH",
        "SIZE",
        skill_name_width = skill_name_width,
        path_width = path_width,
        size_width = size_width,
    );
    println!(
        "{:-<skill_name_width$}  {:-<path_width$}  {:-<size_width$}",
        "",
        "",
        "",
        skill_name_width = skill_name_width,
        path_width = path_width,
        size_width = size_width,
    );

    for (skill_name, path, size) in rows {
        println!(
            "{skill_name:<skill_name_width$}  {path:<path_width$}  {size:>size_width$}",
            skill_name_width = skill_name_width,
            path_width = path_width,
            size_width = size_width,
        );
    }

    Ok(())
}

fn best_hub_fuzzy_match<'a>(config: &'a Config, input: &str) -> Option<(&'a str, f64)> {
    let mut best: Option<(&str, f64)> = None;
    for key in config.hub.keys() {
        let score = jaro_winkler(input, key);
        best = match best {
            None => Some((key.as_str(), score)),
            Some((bk, bs)) => {
                if score > bs || (score == bs && key.as_str() < bk) {
                    Some((key.as_str(), score))
                } else {
                    Some((bk, bs))
                }
            }
        };
    }
    best
}

fn read_skill_name_confirm() -> Result<bool, String> {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|err| format!("Failed to read confirmation: {err}"))?;
    let t = line.trim();
    Ok(t.is_empty() || t == "y" || t == "Y")
}

fn hub_path_from_entry(skill_name: &str, source: &str) -> Result<(String, PathBuf), String> {
    let path = PathBuf::from(source);
    if !path.is_dir() {
        return Err(format!(
            "Hub entry `{skill_name}` points to a missing directory: {source}"
        ));
    }
    Ok((skill_name.to_string(), path))
}

fn resolve_hub_source(config: &Config, skill_name: &str) -> Result<(String, PathBuf), String> {
    if let Some(path_str) = config.hub.get(skill_name) {
        return hub_path_from_entry(skill_name, path_str);
    }

    let Some((best_key, score)) = best_hub_fuzzy_match(config, skill_name) else {
        return Err(format!("Hub entry `{skill_name}` not found."));
    };

    if score < FUZZY_HUB_THRESHOLD {
        return Err(format!("Hub entry `{skill_name}` not found."));
    }

    if !io::stdin().is_terminal() {
        eprintln!("Closest skill name: `{best_key}` (similarity {score:.3}).");
        return Err(format!("Hub entry `{skill_name}` not found."));
    }

    eprint!("Did you mean `{best_key}` instead of `{skill_name}`? [y/N]: ");
    io::stderr()
        .flush()
        .map_err(|err| format!("Failed to flush stderr: {err}"))?;

    if !read_skill_name_confirm()? {
        return Err(format!("Hub entry `{skill_name}` not found."));
    }

    let path_str = config
        .hub
        .get(best_key)
        .ok_or_else(|| format!("Hub entry `{best_key}` not found."))?;
    hub_path_from_entry(best_key, path_str)
}

fn copy_hub_to(
    config: &Config,
    skill_name: &str,
    source: &Path,
    dest: &Path,
) -> Result<PathBuf, String> {
    let final_root = dest.join(skill_name).join("content");
    let final_root_abs = absolute_path(&final_root)?;

    if final_root_abs.starts_with(source) {
        return Err("Destination must not be inside the source directory.".to_string());
    }

    fs::create_dir_all(&final_root)
        .map_err(|err| format!("Failed to create {}: {err}", final_root.display()))?;

    let matcher = build_ignore_matcher(source, &config.ignore)?;

    for entry in WalkDir::new(source).min_depth(1) {
        let entry = entry.map_err(|err| format!("Failed to walk {}: {err}", source.display()))?;
        let path = entry.path();
        let rel = path.strip_prefix(source).map_err(|err| {
            format!(
                "Failed to compute relative path for {}: {err}",
                path.display()
            )
        })?;

        if should_skip(path, rel, entry.file_type().is_dir(), &matcher) {
            continue;
        }

        let dest_path = final_root.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)
                .map_err(|err| format!("Failed to create {}: {err}", dest_path.display()))?;
            continue;
        }

        if entry.file_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("Failed to create {}: {err}", parent.display()))?;
            }
            fs::copy(path, &dest_path).map_err(|err| {
                format!(
                    "Failed to copy {} to {}: {err}",
                    path.display(),
                    dest_path.display()
                )
            })?;
        }
    }

    Ok(fs::canonicalize(&final_root).unwrap_or(final_root))
}

fn build_ignore_matcher(source: &Path, patterns: &[String]) -> Result<Gitignore, String> {
    let mut builder = GitignoreBuilder::new(source);
    for pattern in patterns {
        builder
            .add_line(None, pattern)
            .map_err(|err| format!("Invalid ignore pattern `{pattern}`: {err}"))?;
    }

    builder
        .build()
        .map_err(|err| format!("Failed to build ignore patterns: {err}"))
}

fn should_skip(path: &Path, rel: &Path, is_dir: bool, matcher: &Gitignore) -> bool {
    has_git_component(rel)
        || matcher
            .matched_path_or_any_parents(path, is_dir)
            .is_ignore()
}

fn directory_size(path: &Path) -> Result<u64, String> {
    let mut total = 0_u64;

    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|err| format!("Failed to walk {}: {err}", path.display()))?;
        if entry.file_type().is_file() {
            let metadata = entry.metadata().map_err(|err| {
                format!(
                    "Failed to read metadata for {}: {err}",
                    entry.path().display()
                )
            })?;
            total = total.saturating_add(metadata.len());
        }
    }

    Ok(total)
}

fn display_size(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok("missing".to_string());
    }

    Ok(human_readable_size(directory_size(path)?))
}

fn human_readable_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = size as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{size} {}", UNITS[unit_index])
    } else if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let cwd =
        env::current_dir().map_err(|err| format!("Failed to resolve current directory: {err}"))?;
    Ok(cwd.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn best_hub_fuzzy_picks_highest_jaro_winkler() {
        let mut config = Config::default();
        config.hub.insert("alpha".to_string(), "/tmp/a".to_string());
        config
            .hub
            .insert("alphabet".to_string(), "/tmp/b".to_string());

        let (key, score) = best_hub_fuzzy_match(&config, "alphabe").unwrap();
        assert_eq!(key, "alphabet");
        assert!(score > jaro_winkler("alphabe", "alpha"));
    }

    #[test]
    fn best_hub_fuzzy_tie_breaks_lexicographically() {
        let mut config = Config::default();
        config.hub.insert("b".to_string(), "/tmp/b".to_string());
        config.hub.insert("a".to_string(), "/tmp/a".to_string());

        let (key, _) = best_hub_fuzzy_match(&config, "x").unwrap();
        assert_eq!(key, "a");
    }

    #[test]
    fn hub1_typo_meets_fuzzy_threshold_for_cp_hint() {
        assert!(
            jaro_winkler("hub1x", "hub1") >= FUZZY_HUB_THRESHOLD,
            "adjust test string if threshold or metric changes"
        );
    }
}
