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
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str::from_utf8;

pub struct Git {
    pub dir: PathBuf,
    debug: bool,
}

struct CommandResult {
    command: String,
    succeeded: bool,
    exit_code: Option<i32>,
    stderr: String,
    stdout: String,
}

impl CommandResult {
    fn from_output<S>(command: S, output: &Output) -> Result<Self>
    where
        S: Into<String>,
    {
        Ok(Self {
            command: command.into(),
            succeeded: output.status.success(),
            exit_code: output.status.code(),
            stderr: String::from(from_utf8(output.stderr.as_slice())?.trim()),
            stdout: String::from(from_utf8(output.stdout.as_slice())?.trim()),
        })
    }

    fn dump(&self) -> Result<()> {
        println!(
            "TRACE: [{}] exit_code={:?} stdout=[{}] stderr=[{}]",
            self.command, self.exit_code, self.stdout, self.stderr
        );
        Ok(())
    }

    fn ok(&self) -> Result<()> {
        if !self.succeeded {
            match self.exit_code {
                Some(code) => bail!("{} failed with exit code {}", self.command, code),
                None => bail!("{} failed", self.command),
            };
        }
        Ok(())
    }
}

impl Git {
    pub fn new<P>(dir: P, debug: bool) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            dir: dir.into(),
            debug,
        }
    }

    pub fn describe(&self) -> Result<Option<GitDescription>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("describe")
            .output()?;
        self.trace("describe", &output)?;

        let exit_code = output.status.code();
        let stderr = from_utf8(output.stderr.as_slice())?.trim();

        if exit_code == Some(128) && stderr.contains("cannot describe anything") {
            return Ok(None);
        }

        if !output.status.success() {
            match exit_code {
                Some(code) => bail!("git describe failed with exit code {}", code),
                None => bail!("git describe failed"),
            }
        }

        let stdout = from_utf8(output.stdout.as_slice())?.trim();
        Ok(GitDescription::parse(stdout))
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let result = self.run_git_command("rev-parse", |c| {
            c.arg("--abbrev-ref");
            c.arg("HEAD");
        })?;
        result.ok()?;
        Ok(result.stdout)
    }

    pub fn get_upstream(&self, branch: &str) -> Result<Option<String>> {
        let result = self.run_git_command("rev-parse", |c| {
            c.arg("--abbrev-ref");
            c.arg(format!("{}@{{upstream}}", branch));
        })?;

        if result.exit_code == Some(128) && result.stderr.contains("no upstream") {
            return Ok(None);
        }

        result.ok()?;
        Ok(Some(result.stdout))
    }

    pub fn tag_a(&self, tag: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("tag")
            .arg("--annotate")
            .arg(tag)
            .arg("--message")
            .arg(tag)
            .output()?;
        self.trace("tag", &output)?;

        if !output.status.success() {
            match output.status.code() {
                Some(code) => bail!("git tag failed with exit code {}", code),
                None => bail!("git tag failed"),
            }
        }

        Ok(())
    }

    pub fn push_follow_tags(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("push")
            .arg("--follow-tags")
            .output()?;
        self.trace("push", &output)?;

        if !output.status.success() {
            match output.status.code() {
                Some(code) => bail!("git push failed with exit code {}", code),
                None => bail!("git push failed"),
            }
        }

        Ok(())
    }

    pub fn status(&self, ignored: bool) -> Result<String> {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&self.dir)
            .arg("status")
            .arg("--porcelain");
        if ignored {
            _ = command.arg("--ignored");
        }

        let output = command.output()?;
        self.trace("status", &output)?;

        if !output.status.success() {
            match output.status.code() {
                Some(code) => bail!("git status failed with exit code {}", code),
                None => bail!("git status failed"),
            }
        }

        Ok(String::from(from_utf8(output.stdout.as_slice())?))
    }

    pub fn add<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("add")
            .arg(path.as_ref())
            .output()?;
        self.trace("add", &output)?;

        if !output.status.success() {
            match output.status.code() {
                Some(code) => bail!("git commit failed with exit code {}", code),
                None => bail!("git commit failed"),
            }
        }

        Ok(())
    }

    pub fn commit<S>(&self, message: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("commit")
            .arg("--message")
            .arg(message.as_ref())
            .output()?;
        self.trace("commit", &output)?;

        let exit_code = output.status.code();
        let stderr = from_utf8(output.stderr.as_slice())?.trim();

        if exit_code == Some(128) && stderr.contains("tell me who you are") {
            bail!("E-mail address and/or name is not set in Git repo")
        }

        if !output.status.success() {
            match exit_code {
                Some(code) => bail!("git commit failed with exit code {}", code),
                None => bail!("git commit failed"),
            }
        }

        Ok(())
    }

    pub fn read_config<S>(&self, name: S) -> Result<Option<String>>
    where
        S: AsRef<str>,
    {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("config")
            .arg(name.as_ref())
            .output()?;
        self.trace("config", &output)?;

        let exit_code = output.status.code();
        let stdout = from_utf8(output.stdout.as_slice())?.trim();

        if exit_code == Some(1) && stdout.is_empty() {
            return Ok(None);
        }

        if !output.status.success() {
            match exit_code {
                Some(code) => bail!("git config failed with exit code {}", code),
                None => bail!("git config failed"),
            }
        }

        Ok(Some(String::from(stdout)))
    }

    pub fn is_tracked<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("ls-files")
            .arg(path.as_ref())
            .output()?;
        self.trace("ls-files", &output)?;

        if !output.status.success() {
            match output.status.code() {
                Some(code) => bail!("git config failed with exit code {}", code),
                None => bail!("git config failed"),
            }
        }

        let stdout = from_utf8(output.stdout.as_slice())?.trim();
        Ok(!stdout.is_empty())
    }

    fn run_git_command<F>(&self, command: &str, build: F) -> Result<CommandResult>
    where
        F: FnOnce(&mut Command),
    {
        let mut c = Command::new("git");
        c.arg("-C");
        c.arg(&self.dir);
        c.arg(command);
        build(&mut c);
        let result = CommandResult::from_output(command, &c.output()?)?;
        if self.debug {
            result.dump()?
        }
        Ok(result)
    }

    fn trace(&self, context: &str, output: &Output) -> Result<()> {
        if self.debug {
            let stdout = from_utf8(output.stdout.as_slice())?.trim();
            let stderr = from_utf8(output.stderr.as_slice())?.trim();
            println!(
                "TRACE: [{}] exit_code={:?} stdout=[{}] stderr=[{}]",
                context,
                output.status.code(),
                stdout,
                stderr
            );
        }
        Ok(())
    }
}
