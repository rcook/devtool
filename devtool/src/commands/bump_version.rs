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
use crate::project_info::ProjectInfo;
use anyhow::{anyhow, bail, Result};
use devtool_version::Version;
use joatmon::{read_toml_file_edit, safe_write_file};
use lazy_static::lazy_static;
use path_absolutize::Absolutize;
use std::io::Result as IOResult;
use std::path::Path;
use std::process::Command;
use toml_edit::value;

lazy_static! {
    static ref INITIAL_VERSION: Version = "v0.0.0".parse::<Version>().expect("init: must succeed");
}

pub fn bump_version(app: &App, version: &Option<Version>, push_all: bool) -> Result<()> {
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

    let project_info = app.read_config()?.map_or_else(
        || ProjectInfo::infer(app),
        |c| {
            c.cargo_toml_paths
                .into_iter()
                .map(|p| p.absolutize_from(&app.git.dir).map(|p| p.to_path_buf()))
                .collect::<IOResult<Vec<_>>>()
                .map(|cargo_toml_paths| ProjectInfo { cargo_toml_paths })
                .map_err(|e| anyhow!(e))
        },
    )?;

    let new_version = if let Some(version) = version {
        version.clone()
    } else {
        get_new_version(app, &INITIAL_VERSION)?
    };

    println!("project_info={project_info:#?}");
    println!("new_version={new_version}");

    if !project_info.cargo_toml_paths.is_empty() {
        let mut new_cargo_version = new_version.dupe();
        new_cargo_version.set_prefix(false);

        for cargo_toml_path in project_info.cargo_toml_paths {
            update_cargo_toml(app, &cargo_toml_path, &new_cargo_version)?;
        }

        regenerate_cargo_lock(app)?;

        app.git
            .commit(format!("Bump version to {new_cargo_version}"))?;
        println!("Bump Cargo package version to {new_cargo_version}");
    }

    let tag = new_version.to_string();
    app.git.create_annotated_tag(&tag)?;
    println!("Created tag {tag}");

    if push_all {
        app.git.push_all()?;
        println!("Pushed commits and tags");
    } else {
        println!("Skipping push of commits and tags");
    }

    Ok(())
}

fn get_new_version(app: &App, default: &Version) -> Result<Version> {
    Ok(match app.git.describe()? {
        Some(description) => {
            if description.offset.is_none() {
                bail!("No commits since most recent tag \"{}\"", description.tag)
            }

            let mut version = description.tag.parse::<Version>()?;
            println!("description={description:#?}");
            version.increment();
            version
        }
        None => default.clone(),
    })
}

fn update_cargo_toml(app: &App, cargo_toml_path: &Path, new_cargo_version: &Version) -> Result<()> {
    let mut doc = read_toml_file_edit(cargo_toml_path)?;

    if let Some(package) = doc
        .as_table_mut()
        .get_mut("package")
        .and_then(toml_edit::Item::as_table_mut)
    {
        _ = package.insert("version", value(format!("{new_cargo_version}")));
        let result = doc.to_string();
        safe_write_file(cargo_toml_path, result, true)?;
        app.git.add(cargo_toml_path)?;
    }

    Ok(())
}

fn regenerate_cargo_lock(app: &App) -> Result<()> {
    let cargo_toml_path = app.git.dir.join("Cargo.toml");
    let cargo_lock_path = app.git.dir.join("Cargo.lock");
    if app.git.is_tracked(&cargo_toml_path)? && app.git.is_tracked(&cargo_lock_path)? {
        if !Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&cargo_toml_path)
            .status()?
            .success()
        {
            bail!("cargo build failed")
        }

        app.git.add(&cargo_lock_path)?;
    }

    Ok(())
}
