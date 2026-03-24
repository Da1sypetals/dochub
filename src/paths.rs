use std::path::{Component, Path, PathBuf};

pub fn normalize_join_input(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
            Component::ParentDir => normalized.push(".."),
            Component::Normal(part) => normalized.push(part),
        }
    }

    normalized
}

pub fn has_git_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == ".git"))
}
