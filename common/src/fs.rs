use std::fs;
use std::path::{Path, PathBuf};

pub fn get_path_buf(dir: &Path, suffix: &Vec<&str>) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let p = get_path_buf(path.as_path(), suffix);
                result.extend(p.into_iter());
            } else if path.extension().is_some()
                && suffix.contains(&path.extension().unwrap().to_str().unwrap())
            {
                result.push(path);
            }
        }
    }
    return result;
}