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
            |p| p.is_file() && p.file_name().map_or(false, |x| x == "Cargo.toml"),
            &[OsStr::new(".git"), OsStr::new("target")],
        )?;
        let pyproject_toml_paths = Self::walk(
            &app.git.dir,
            |p| p.is_file() && p.file_name().map_or(false, |x| x == "pyproject.toml"),
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

                if path.is_dir()
                    && path
                        .file_name()
                        .map_or(true, |x| !ignore_dirs_set.contains(x))
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
