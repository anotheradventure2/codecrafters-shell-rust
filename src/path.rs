use std::{
    env,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub fn list_executables() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap()
        .split(':')
        .flat_map(get_executables)
        .collect()
}

fn get_executables(dir: &str) -> Vec<PathBuf> {
    Path::new(dir)
        .read_dir()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|e| {
            let full_path = e.path();
            if full_path.is_file()
                && full_path
                    .metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
            {
                Some(full_path)
            } else {
                None
            }
        })
        .collect()
}
