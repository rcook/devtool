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
use super::GitDescription;
use anyhow::anyhow;
use log::trace;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::result::Result as StdResult;
use std::str::from_utf8;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("command {0} failed with exit code {1}")]
    CommandFailedWithCode(String, i32),

    #[error("command {0} failed")]
    CommandFailed(String),

    #[error("e-mail or name is not configured in Git")]
    EmailOrNameNotConfigured,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type GitResult<T> = StdResult<T, GitError>;

const GIT_FATAL_EXIT_CODE: i32 = 128;

#[derive(Debug)]
pub struct Git {
    pub dir: PathBuf,
}

struct CommandResult {
    command: String,
    succeeded: bool,
    exit_code: Option<i32>,
    stderr: String,
    stdout: String,
}

impl CommandResult {
    fn from_output<S>(command: S, output: &Output) -> GitResult<Self>
    where
        S: Into<String>,
    {
        Ok(Self {
            command: command.into(),
            succeeded: output.status.success(),
            exit_code: output.status.code(),
            stderr: String::from(
                from_utf8(output.stderr.as_slice())
                    .map_err(|e| GitError::Other(anyhow!(e)))?
                    .trim(),
            ),
            stdout: String::from(
                from_utf8(output.stdout.as_slice())
                    .map_err(|e| GitError::Other(anyhow!(e)))?
                    .trim(),
            ),
        })
    }

    #[allow(clippy::missing_const_for_fn)]
    fn ok(self) -> GitResult<Self> {
        if !self.succeeded {
            match self.exit_code {
                Some(code) => return Err(GitError::CommandFailedWithCode(self.command, code)),
                None => return Err(GitError::CommandFailed(self.command)),
            };
        }
        Ok(self)
    }
}

impl Git {
    pub fn new<P>(dir: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self { dir: dir.into() }
    }

    pub fn describe(&self) -> GitResult<Option<GitDescription>> {
        let result = self.run("describe", |_| {})?;

        if result.exit_code == Some(GIT_FATAL_EXIT_CODE)
            && result.stderr.contains("cannot describe anything")
        {
            return Ok(None);
        }

        Ok(GitDescription::parse(result.ok()?.stdout))
    }

    pub fn get_current_branch(&self) -> GitResult<String> {
        let result = self
            .run("branch", |c| {
                c.arg("--show-current");
            })?
            .ok()?;
        Ok(result.stdout)
    }

    pub fn get_upstream(&self, branch: &str) -> GitResult<Option<String>> {
        let result = self.run("rev-parse", |c| {
            c.arg("--abbrev-ref");
            c.arg(format!("{branch}@{{upstream}}"));
        })?;

        if result.exit_code == Some(GIT_FATAL_EXIT_CODE) && result.stderr.contains("no upstream") {
            return Ok(None);
        }

        Ok(Some(result.ok()?.stdout))
    }

    pub fn create_annotated_tag(&self, tag: &str) -> GitResult<()> {
        self.run("tag", |c| {
            c.arg("--annotate");
            c.arg(tag);
            c.arg("--message");
            c.arg(tag);
        })?
        .ok()?;
        Ok(())
    }

    pub fn push_all(&self) -> GitResult<()> {
        self.run("push", |c| {
            c.arg("--follow-tags");
        })?
        .ok()?;
        Ok(())
    }

    pub fn status(&self, ignored: bool) -> GitResult<String> {
        let result = self
            .run("status", |c| {
                c.arg("--porcelain");
                if ignored {
                    c.arg("--ignored");
                }
            })?
            .ok()?;
        Ok(result.stdout)
    }

    pub fn add<P>(&self, path: P) -> GitResult<()>
    where
        P: AsRef<Path>,
    {
        self.run("add", |c| {
            c.arg(path.as_ref());
        })?
        .ok()?;
        Ok(())
    }

    pub fn commit<S>(&self, message: S) -> GitResult<()>
    where
        S: AsRef<str>,
    {
        let result = self.run("commit", |c| {
            c.arg("--message");
            c.arg(message.as_ref());
        })?;

        if result.exit_code == Some(GIT_FATAL_EXIT_CODE)
            && result.stderr.contains("tell me who you are")
        {
            return Err(GitError::EmailOrNameNotConfigured);
        }

        result.ok()?;
        Ok(())
    }

    pub fn read_config<S>(&self, name: S) -> GitResult<Option<String>>
    where
        S: AsRef<str>,
    {
        let result = self.run("config", |c| {
            c.arg(name.as_ref());
        })?;

        if result.exit_code == Some(1) && result.stdout.is_empty() {
            return Ok(None);
        }

        Ok(Some(result.ok()?.stdout))
    }

    pub fn is_tracked<P>(&self, path: P) -> GitResult<bool>
    where
        P: AsRef<Path>,
    {
        let result = self
            .run("ls-files", |c| {
                c.arg(path.as_ref());
            })?
            .ok()?;
        Ok(!result.stdout.is_empty())
    }

    pub fn fetch(&self) -> GitResult<()> {
        self.run("fetch", |_| {})?.ok()?;
        Ok(())
    }

    pub fn rebase(&self, upstream: &str) -> GitResult<()> {
        self.run("rebase", |c| {
            c.arg(upstream);
        })?
        .ok()?;
        Ok(())
    }

    pub fn rev_parse(&self, rev: &str) -> GitResult<Option<String>> {
        let result = self.run("rev-parse", |c| {
            c.arg("--verify");
            c.arg(rev);
        })?;

        if !result.succeeded {
            return Ok(None);
        }

        Ok(Some(result.stdout))
    }

    pub fn head_sha(&self) -> GitResult<String> {
        let result = self
            .run("rev-parse", |c| {
                c.arg("HEAD");
            })?
            .ok()?;
        Ok(result.stdout)
    }

    pub fn reset_hard(&self, target: &str) -> GitResult<()> {
        self.run("reset", |c| {
            c.arg("--hard");
            c.arg(target);
        })?
        .ok()?;
        Ok(())
    }

    pub fn delete_tag(&self, tag: &str) -> GitResult<()> {
        self.run("tag", |c| {
            c.arg("-d");
            c.arg(tag);
        })?
        .ok()?;
        Ok(())
    }

    pub fn has_staged_changes(&self) -> GitResult<bool> {
        let result = self.run("diff", |c| {
            c.arg("--cached");
            c.arg("--quiet");
        })?;
        Ok(!result.succeeded)
    }

    fn run<F>(&self, command: &str, build: F) -> GitResult<CommandResult>
    where
        F: FnOnce(&mut Command),
    {
        let mut c = Command::new("git");
        c.arg("-C");
        c.arg(&self.dir);
        c.arg(command);
        build(&mut c);

        let command_str = format!("{c:?}");
        let result = CommandResult::from_output(
            command,
            &c.output().map_err(|e| GitError::Other(anyhow!(e)))?,
        )?;
        trace!(
            "command={}, exit_code={:?}, stdout=[{}], stderr=[{}]",
            command_str, result.exit_code, result.stdout, result.stderr
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Git, GitError};
    use std::path::PathBuf;

    #[test]
    fn new_stores_dir() {
        let git = Git::new("/some/path");
        assert_eq!(PathBuf::from("/some/path"), git.dir);
    }

    #[test]
    fn error_display_command_failed_with_code() {
        let err = GitError::CommandFailedWithCode(String::from("status"), 128);
        assert_eq!("command status failed with exit code 128", err.to_string());
    }

    #[test]
    fn error_display_command_failed() {
        let err = GitError::CommandFailed(String::from("push"));
        assert_eq!("command push failed", err.to_string());
    }

    #[test]
    fn error_display_email_or_name_not_configured() {
        let err = GitError::EmailOrNameNotConfigured;
        assert_eq!("e-mail or name is not configured in Git", err.to_string());
    }

    #[test]
    fn status_on_clean_repo() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir.path())
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        let status = git.status(false).unwrap();
        assert!(status.is_empty());
    }

    #[test]
    fn get_current_branch_on_new_repo() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir.path())
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        let branch = git.get_current_branch().unwrap();
        assert_eq!("main", branch);
    }

    #[test]
    fn describe_returns_none_with_no_tags() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir.path())
            .output()
            .unwrap();
        std::fs::write(dir.path().join("file.txt"), "hello").unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@test.com",
                "commit",
                "-m",
                "init",
            ])
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        let description = git.describe().unwrap();
        assert!(description.is_none());
    }

    #[test]
    fn is_tracked_false_for_untracked_file() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir.path())
            .output()
            .unwrap();
        std::fs::write(dir.path().join("untracked.txt"), "hello").unwrap();
        let git = Git::new(dir.path());
        assert!(!git.is_tracked(dir.path().join("untracked.txt")).unwrap());
    }

    #[test]
    fn is_tracked_true_for_tracked_file() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir.path())
            .output()
            .unwrap();
        let file_path = dir.path().join("tracked.txt");
        std::fs::write(&file_path, "hello").unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["add", "tracked.txt"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@test.com",
                "commit",
                "-m",
                "init",
            ])
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        assert!(git.is_tracked(&file_path).unwrap());
    }

    #[test]
    fn read_config_returns_none_for_unset_key() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir.path())
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        let value = git.read_config("user.nonexistent-key-12345").unwrap();
        assert!(value.is_none());
    }

    fn init_repo_with_commit(dir: &std::path::Path) {
        std::process::Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(dir)
            .output()
            .unwrap();
        std::fs::write(dir.join("file.txt"), "hello").unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args(["add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@test.com",
                "commit",
                "-m",
                "init",
            ])
            .output()
            .unwrap();
    }

    #[test]
    fn rev_parse_returns_none_for_nonexistent_ref() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        let git = Git::new(dir.path());
        assert!(git.rev_parse("refs/tags/v99.99.99").unwrap().is_none());
    }

    #[test]
    fn rev_parse_returns_some_for_existing_ref() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["tag", "v1.0.0"])
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        assert!(git.rev_parse("refs/tags/v1.0.0").unwrap().is_some());
    }

    #[test]
    fn head_sha_returns_valid_sha() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        let git = Git::new(dir.path());
        let sha = git.head_sha().unwrap();
        assert_eq!(40, sha.len());
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn reset_hard_moves_head() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        let git = Git::new(dir.path());
        let first_sha = git.head_sha().unwrap();

        std::fs::write(dir.path().join("second.txt"), "world").unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@test.com",
                "commit",
                "-m",
                "second",
            ])
            .output()
            .unwrap();
        assert_ne!(first_sha, git.head_sha().unwrap());

        git.reset_hard(&first_sha).unwrap();
        assert_eq!(first_sha, git.head_sha().unwrap());
    }

    #[test]
    fn delete_tag_removes_tag() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["config", "user.name", "Test"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["config", "user.email", "test@test.com"])
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        git.create_annotated_tag("v1.0.0").unwrap();
        assert!(git.rev_parse("refs/tags/v1.0.0").unwrap().is_some());
        git.delete_tag("v1.0.0").unwrap();
        assert!(git.rev_parse("refs/tags/v1.0.0").unwrap().is_none());
    }

    #[test]
    fn has_staged_changes_false_when_clean() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        let git = Git::new(dir.path());
        assert!(!git.has_staged_changes().unwrap());
    }

    #[test]
    fn has_staged_changes_true_when_staged() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        std::fs::write(dir.path().join("new.txt"), "new").unwrap();
        std::process::Command::new("git")
            .args(["-C"])
            .arg(dir.path())
            .args(["add", "new.txt"])
            .output()
            .unwrap();
        let git = Git::new(dir.path());
        assert!(git.has_staged_changes().unwrap());
    }
}
