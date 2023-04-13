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
use crate::result::Result;

const UNTRACKED_PREFIX: &str = "?? ";
const IGNORED_PREFIX: &str = "!! ";

pub fn generate_ignore(app: &App) -> Result<()> {
    let s = app.git.status_ignored()?;

    let mut all_dir_paths = Vec::new();
    let mut all_file_paths = Vec::new();
    for line in s.lines().filter_map(is_path_to_ignore) {
        if line.ends_with('/') {
            all_dir_paths.push(line)
        } else {
            all_file_paths.push(line)
        }
    }

    all_dir_paths.sort();
    all_file_paths.sort();

    let mut dir_paths = Vec::new();
    for p in &all_dir_paths {
        if !is_covered_by_dir(&all_dir_paths, p) {
            dir_paths.push(p)
        }
    }

    let mut file_paths = Vec::new();
    for p in &all_file_paths {
        if !is_covered_by_dir(&all_dir_paths, p) {
            file_paths.push(p)
        }
    }

    if !dir_paths.is_empty() {
        println!("# Directories");
        for p in &dir_paths {
            println!("/{}", p)
        }
    }

    if !file_paths.is_empty() {
        println!("# Files");
        for p in &file_paths {
            println!("/{}", p)
        }
    }

    Ok(())
}

fn is_path_to_ignore(line: &str) -> Option<&str> {
    if let Some(s) = line.strip_prefix(UNTRACKED_PREFIX) {
        Some(s)
    } else if let Some(s) = line.strip_prefix(IGNORED_PREFIX) {
        Some(s)
    } else {
        None
    }
}

fn is_covered_by_dir<S>(dir_paths: &Vec<S>, path: &str) -> bool
where
    S: AsRef<str>,
{
    for dir_path in dir_paths {
        if path != dir_path.as_ref() && path.starts_with(dir_path.as_ref()) {
            return true;
        }
    }
    false
}
