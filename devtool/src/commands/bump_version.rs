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
use anyhow::{Result, bail};
use devtool_git::Git;
use devtool_version::Version;
use joatmon::{read_toml_file_edit, safe_write_file};
use log::info;
use path_absolutize::Absolutize;
use std::io::Result as IOResult;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;
use toml_edit::value;

static INITIAL_VERSION: LazyLock<Version> =
    LazyLock::new(|| "v0.0.0".parse::<Version>().expect("init: must succeed"));

struct RollbackGuard<'a> {
    git: &'a Git,
    original_head: String,
    tag: Option<String>,
    disarmed: bool,
}

impl<'a> RollbackGuard<'a> {
    const fn new(git: &'a Git, original_head: String) -> Self {
        Self {
            git,
            original_head,
            tag: None,
            disarmed: false,
        }
    }

    fn set_tag(&mut self, tag: String) {
        self.tag = Some(tag);
    }

    const fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for RollbackGuard<'_> {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }
        eprintln!("Rolling back version bump...");
        if self.git.reset_hard(&self.original_head).is_err() {
            eprintln!("Warning: failed to reset to {}", self.original_head);
        }
        if let Some(tag) = &self.tag
            && self.git.delete_tag(tag).is_err()
        {
            eprintln!("Warning: failed to delete tag {tag}");
        }
    }
}

pub fn bump_version(app: &App, version: Option<&Version>, push_all: bool) -> Result<()> {
    // Phase 1: Validation — no mutations, no rollback needed

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
        bail!(
            "Git working directory is not clean: please revert or commit pending changes and try again"
        )
    }

    let upstream = app.git.get_upstream(&branch)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Branch {} has no upstream set: set with git push -u origin {} or similar",
            branch,
            branch
        )
    })?;

    info!("Fetching from remote...");
    app.git.fetch()?;

    info!("Rebasing on {upstream}...");
    app.git.rebase(&upstream)?;

    let project_info = app.read_config()?.map_or_else(
        || ProjectInfo::infer(app),
        |c| {
            let cargo_toml_paths = c
                .cargo_toml_paths
                .into_iter()
                .map(|p| p.absolutize_from(&app.git.dir).map(|p| p.to_path_buf()))
                .collect::<IOResult<Vec<_>>>()?;
            let pyproject_toml_paths = c
                .pyproject_toml_paths
                .into_iter()
                .map(|p| p.absolutize_from(&app.git.dir).map(|p| p.to_path_buf()))
                .collect::<IOResult<Vec<_>>>()?;
            Ok(ProjectInfo {
                cargo_toml_paths,
                pyproject_toml_paths,
            })
        },
    )?;

    let mut new_version = if let Some(version) = version {
        version.clone()
    } else {
        get_new_version(app, &INITIAL_VERSION)?
    };

    let tag = new_version.to_string();
    if app.git.rev_parse(&format!("refs/tags/{tag}"))?.is_some() {
        bail!("Tag \"{tag}\" already exists")
    }

    // Phase 2: Mutation — under RollbackGuard

    let original_head = app.git.head_sha()?;
    let mut guard = RollbackGuard::new(&app.git, original_head);

    new_version.set_prefix(false);

    let mut file_change = false;

    for path in &project_info.cargo_toml_paths {
        if update_cargo_toml(app, path, &new_version)? {
            file_change = true;
        }
    }

    if file_change {
        regenerate_cargo_lock(app)?;
    }

    for path in &project_info.pyproject_toml_paths {
        if update_pyproject_toml(app, path, &new_version)? {
            file_change = true;
        }
    }

    if file_change && app.git.has_staged_changes()? {
        app.git.commit(format!("Bump version to {new_version}"))?;
        println!("Bumped version to {new_version}");
    }

    app.git.create_annotated_tag(&tag)?;
    guard.set_tag(tag.clone());
    println!("Created tag {tag}");

    // Phase 3: Finalize

    if push_all {
        app.git.push_all()?;
        println!("Pushed commits and tags");
    } else {
        println!("Skipping push of commits and tags");
    }

    guard.disarm();
    Ok(())
}

fn get_new_version(app: &App, default: &Version) -> Result<Version> {
    Ok(match app.git.describe()? {
        Some(description) => {
            if description.offset.is_none() {
                bail!("No commits since most recent tag \"{}\"", description.tag)
            }

            let mut version = description.tag.parse::<Version>()?;
            version.increment();
            version
        }
        None => default.clone(),
    })
}

fn update_cargo_toml(app: &App, path: &Path, new_version: &Version) -> Result<bool> {
    let mut doc = read_toml_file_edit(path)?;

    if let Some(package) = doc
        .as_table_mut()
        .get_mut("package")
        .and_then(toml_edit::Item::as_table_mut)
    {
        _ = package.insert("version", value(format!("{new_version}")));
        let result = doc.to_string();
        safe_write_file(path, result, true)?;
        app.git.add(path)?;
        return Ok(true);
    }

    if let Some(workspace) = doc
        .as_table_mut()
        .get_mut("workspace")
        .and_then(toml_edit::Item::as_table_mut)
        && let Some(package) = workspace
            .get_mut("package")
            .and_then(toml_edit::Item::as_table_mut)
    {
        _ = package.insert("version", value(format!("{new_version}")));
        let result = doc.to_string();
        safe_write_file(path, result, true)?;
        app.git.add(path)?;
        return Ok(true);
    }

    Ok(false)
}

fn regenerate_cargo_lock(app: &App) -> Result<()> {
    let cargo_toml_path = app.git.dir.join("Cargo.toml");
    let cargo_lock_path = app.git.dir.join("Cargo.lock");
    if app.git.is_tracked(&cargo_toml_path)? && app.git.is_tracked(&cargo_lock_path)? {
        if !Command::new("cargo")
            .arg("generate-lockfile")
            .arg("--manifest-path")
            .arg(&cargo_toml_path)
            .status()?
            .success()
        {
            bail!("cargo generate-lockfile failed")
        }

        app.git.add(&cargo_lock_path)?;
    }

    Ok(())
}

fn update_pyproject_toml(app: &App, path: &Path, new_version: &Version) -> Result<bool> {
    let mut doc = read_toml_file_edit(path)?;

    if let Some(package) = doc
        .as_table_mut()
        .get_mut("project")
        .and_then(toml_edit::Item::as_table_mut)
    {
        _ = package.insert("version", value(format!("{new_version}")));
        let result = doc.to_string();
        safe_write_file(path, result, true)?;
        app.git.add(path)?;
        return Ok(true);
    }

    Ok(false)
}
