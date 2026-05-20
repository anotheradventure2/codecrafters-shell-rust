use std::{env, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};

pub fn list_executables() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap()
        .split(':')
        .flat_map(get_executables)
        .collect()
}

// Find executable by name - returns None if not found
pub fn find_executable<'a>(name: &str, path: &'a [PathBuf]) -> Option<&'a PathBuf> {
    path.iter()
        .find(|p| p.file_name().map(|f| f == name).unwrap_or(false))
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
                && full_path.metadata().unwrap().permissions().mode() & 0o111 != 0
            {
                Some(full_path)
            } else {
                None
            }
        })
        .collect()
}