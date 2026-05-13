// Copyright (c) 2023 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use crate::app::App;
use anyhow::Result;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ProjectInfo {
    pub cargo_toml_paths: Vec<PathBuf>,
    pub pyproject_toml_paths: Vec<PathBuf>,
}

impl ProjectInfo {
    pub fn infer(app: &App) -> Result<Self> {
        let cargo_toml_paths = Self::walk(
            &app.git.dir,
            |p| p.is_file() && p.file_name().is_some_and(|x| x == "Cargo.toml"),
            &[OsStr::new(".git"), OsStr::new("target")],
        )?;
        let pyproject_toml_paths = Self::walk(
            &app.git.dir,
            |p| p.is_file() && p.file_name().is_some_and(|x| x == "pyproject.toml"),
            &[OsStr::new(".git"), OsStr::new("target")],
        )?;

        Ok(Self {
            cargo_toml_paths,
            pyproject_toml_paths,
        })
    }

    fn walk<P>(start_dir: &Path, predicate: P, ignore_dirs: &[&OsStr]) -> Result<Vec<PathBuf>>
    where
        P: Fn(&Path) -> bool,
    {
        fn helper<P>(
            paths: &mut Vec<PathBuf>,
            start_dir: &Path,
            predicate: &P,
            ignore_dirs_set: &HashSet<&OsStr>,
        ) -> Result<()>
        where
            P: Fn(&Path) -> bool,
        {
            for result in read_dir(start_dir)? {
                let entry = result?;
                let path = entry.path();

                if entry.file_type()?.is_dir()
                    && path
                        .file_name()
                        .is_none_or(|x| !ignore_dirs_set.contains(x))
                {
                    helper(paths, &path, predicate, ignore_dirs_set)?;
                }

                if predicate(&path) {
                    paths.push(path);
                }
            }

            Ok(())
        }

        let mut paths = Vec::new();
        let ignore_dirs_set = ignore_dirs.iter().copied().collect::<HashSet<_>>();
        helper(&mut paths, start_dir, &predicate, &ignore_dirs_set)?;
        paths.sort();

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectInfo;
    use crate::app::App;

    #[test]
    fn infer_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert!(info.cargo_toml_paths.is_empty());
        assert!(info.pyproject_toml_paths.is_empty());
    }

    #[test]
    fn infer_finds_cargo_toml_at_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert_eq!(1, info.cargo_toml_paths.len());
        assert!(info.cargo_toml_paths[0].ends_with("Cargo.toml"));
    }

    #[test]
    fn infer_finds_nested_cargo_tomls_sorted() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::create_dir(dir.path().join("b")).unwrap();
        std::fs::create_dir(dir.path().join("a")).unwrap();
        std::fs::write(dir.path().join("b/Cargo.toml"), "").unwrap();
        std::fs::write(dir.path().join("a/Cargo.toml"), "").unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert_eq!(2, info.cargo_toml_paths.len());
        assert!(info.cargo_toml_paths[0] < info.cargo_toml_paths[1]);
    }

    #[test]
    fn infer_ignores_dot_git_dir() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();
        std::fs::write(git_dir.join("Cargo.toml"), "").unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert!(info.cargo_toml_paths.is_empty());
    }

    #[test]
    fn infer_ignores_target_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let target_dir = dir.path().join("target");
        std::fs::create_dir(&target_dir).unwrap();
        std::fs::write(target_dir.join("Cargo.toml"), "").unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert!(info.cargo_toml_paths.is_empty());
    }

    #[test]
    fn infer_finds_pyproject_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "[project]").unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert!(info.cargo_toml_paths.is_empty());
        assert_eq!(1, info.pyproject_toml_paths.len());
    }

    #[test]
    fn infer_finds_both_manifest_types() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        let app = App::new(dir.path());
        let info = ProjectInfo::infer(&app).unwrap();
        assert_eq!(1, info.cargo_toml_paths.len());
        assert_eq!(1, info.pyproject_toml_paths.len());
    }
}
