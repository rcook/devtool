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
use crate::version::parse_version;
use anyhow::{anyhow, bail, Result};
use std::path::PathBuf;
use std::process::Command;
use swiss_army_knife::{read_toml_file_edit, safe_write_file};
use toml_edit::value;

const INITIAL_VERSION_STR: &str = "v0.0.0";

#[derive(Debug)]
enum ProjectInfo {
    Cargo { cargo_toml_path: PathBuf },
    Other,
}

impl ProjectInfo {
    fn infer(app: &App) -> Self {
        let cargo_toml_path = app.git.dir.join("Cargo.toml");
        if cargo_toml_path.is_file() {
            Self::Cargo { cargo_toml_path }
        } else {
            Self::Other
        }
    }
}

pub fn bump_version(app: &App) -> Result<()> {
    if app.git.read_config("user.name")?.is_none() {
        bail!("Git user name is not set")
    }

    if app.git.read_config("user.email")?.is_none() {
        bail!("Git e-mail address is not set")
    }

    let branch = app.git.get_current_branch()?;
    if branch != "main" && branch != "master" {
        bail!("Must be on the \"main\" or \"master\" branch")
    }

    if !app.git.status(false)?.is_empty() {
        bail!("Git working directory is not clean: please revert or commit pending changes and try again")
    }

    if app.git.get_upstream(&branch)?.is_none() {
        bail!(
            "Branch {} has no upstream set: set with git push -u origin {} or similar",
            branch,
            branch
        );
    }

    let new_version = match app.git.describe()? {
        Some(description) => {
            if description.offset.is_none() {
                bail!("No commits since most recent tag \"{}\"", description.tag)
            }

            match parse_version(&description.tag) {
                Some(mut version) => {
                    println!("description={:#?}", description);
                    version.increment();
                    version
                }
                None => bail!("Cannot parse tag \"{}\" as version", description.tag),
            }
        }
        None => parse_version(INITIAL_VERSION_STR).expect("must be valid"),
    };

    let project_info = ProjectInfo::infer(app);
    println!("project_info={:#?}", project_info);

    if let ProjectInfo::Cargo { cargo_toml_path } = project_info {
        let mut doc = read_toml_file_edit(&cargo_toml_path)?;
        let package = doc
            .as_table_mut()
            .get_mut("package")
            .ok_or(anyhow!("Expected \"package\" table"))?
            .as_table_mut()
            .ok_or(anyhow!("\"package\" must be a table"))?;

        let mut new_cargo_version = new_version.dupe();
        new_cargo_version.set_prefix(false);
        _ = package.insert("version", value(format!("{}", new_cargo_version)));

        let result = doc.to_string();
        safe_write_file(&cargo_toml_path, result, true)?;

        let cargo_lock_path = app.git.dir.join("Cargo.lock");
        if app.git.is_tracked(&cargo_lock_path)?
            && !Command::new("cargo")
                .arg("build")
                .arg("--manifest-path")
                .arg(&cargo_toml_path)
                .status()?
                .success()
        {
            bail!("cargo build failed")
        }

        app.git.add(&cargo_toml_path)?;
        app.git.add(&cargo_lock_path)?;

        app.git
            .commit(format!("Bump version to {}", new_cargo_version))?;
        println!("Bump Cargo package version to {}", new_cargo_version);
    }

    let tag = new_version.to_string();
    app.git.create_annotated_tag(&tag)?;
    println!("Created tag {}", tag);

    app.git.push_all()?;
    println!("Pushed commits and tags");

    Ok(())
}
