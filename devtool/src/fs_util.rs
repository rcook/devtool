use anyhow::Result;
use std::fs::{OpenOptions, create_dir_all, write};
use std::io::Write;
use std::path::{Path, PathBuf};

#[must_use]
pub fn find_sentinel_dir(sentinel_name: &Path, start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir;
    let mut count = 30;
    loop {
        if count == 0 {
            return None;
        }

        let sentinel_dir_path = dir.join(sentinel_name);
        if sentinel_dir_path.is_dir() {
            return Some(sentinel_dir_path);
        }

        dir = dir.parent()?;
        count -= 1;
    }
}

#[must_use]
pub fn find_sentinel_file(sentinel_name: &Path, start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir;
    let mut count = 30;
    loop {
        if count == 0 {
            return None;
        }

        let sentinel_file_path = dir.join(sentinel_name);
        if sentinel_file_path.is_file() {
            return Some(sentinel_file_path);
        }

        dir = dir.parent()?;
        count -= 1;
    }
}

pub fn safe_write_file<C>(path: &Path, contents: C, overwrite: bool) -> Result<()>
where
    C: AsRef<[u8]>,
{
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    if overwrite {
        write(path, contents)?;
    } else {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(contents.as_ref())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_sentinel_dir, find_sentinel_file, safe_write_file};
    use std::path::Path;

    #[test]
    fn find_sentinel_dir_found() {
        let dir = tempfile::tempdir().unwrap();
        let start = dir.path().join("a").join("b").join("c");
        let sentinel = dir.path().join("a").join("SENTINEL");
        std::fs::create_dir_all(&start).unwrap();
        std::fs::create_dir_all(&sentinel).unwrap();
        let result = find_sentinel_dir(Path::new("SENTINEL"), &start);
        assert_eq!(Some(sentinel), result);
    }

    #[test]
    fn find_sentinel_dir_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let start = dir.path().join("a").join("b");
        std::fs::create_dir_all(&start).unwrap();
        assert!(find_sentinel_dir(Path::new("SENTINEL"), &start).is_none());
    }

    #[test]
    fn find_sentinel_file_found() {
        let dir = tempfile::tempdir().unwrap();
        let start = dir.path().join("a").join("b");
        let sentinel = dir.path().join("a").join("SENTINEL");
        std::fs::create_dir_all(&start).unwrap();
        std::fs::write(&sentinel, "content").unwrap();
        let result = find_sentinel_file(Path::new("SENTINEL"), &start);
        assert_eq!(Some(sentinel), result);
    }

    #[test]
    fn find_sentinel_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let start = dir.path().join("a").join("b");
        std::fs::create_dir_all(&start).unwrap();
        assert!(find_sentinel_file(Path::new("SENTINEL"), &start).is_none());
    }

    #[test]
    fn safe_write_file_creates_new() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        safe_write_file(&path, "hello", false).unwrap();
        assert_eq!("hello", std::fs::read_to_string(&path).unwrap());
    }

    #[test]
    fn safe_write_file_no_overwrite_fails_if_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "original").unwrap();
        assert!(safe_write_file(&path, "new", false).is_err());
        assert_eq!("original", std::fs::read_to_string(&path).unwrap());
    }

    #[test]
    fn safe_write_file_overwrite_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "original").unwrap();
        safe_write_file(&path, "new", true).unwrap();
        assert_eq!("new", std::fs::read_to_string(&path).unwrap());
    }

    #[test]
    fn safe_write_file_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("file.txt");
        safe_write_file(&path, "nested", false).unwrap();
        assert_eq!("nested", std::fs::read_to_string(&path).unwrap());
    }
}
