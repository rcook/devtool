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
use log::trace;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str::from_utf8;

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

    fn ok(self) -> Result<Self> {
        if !self.succeeded {
            match self.exit_code {
                Some(code) => bail!("{} failed with exit code {}", self.command, code),
                None => bail!("{} failed", self.command),
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

    pub fn describe(&self) -> Result<Option<GitDescription>> {
        let result = self.run("describe", |_| {})?;

        if result.exit_code == Some(128) && result.stderr.contains("cannot describe anything") {
            return Ok(None);
        }

        Ok(GitDescription::parse(result.ok()?.stdout))
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let result = self
            .run("branch", |c| {
                c.arg("--show-current");
            })?
            .ok()?;
        Ok(result.stdout)
    }

    pub fn get_upstream(&self, branch: &str) -> Result<Option<String>> {
        let result = self.run("rev-parse", |c| {
            c.arg("--abbrev-ref");
            c.arg(format!("{}@{{upstream}}", branch));
        })?;

        if result.exit_code == Some(128) && result.stderr.contains("no upstream") {
            return Ok(None);
        }

        Ok(Some(result.ok()?.stdout))
    }

    pub fn create_annotated_tag(&self, tag: &str) -> Result<()> {
        self.run("tag", |c| {
            c.arg("--annotate");
            c.arg(tag);
            c.arg("--message");
            c.arg(tag);
        })?
        .ok()?;
        Ok(())
    }

    pub fn push_all(&self) -> Result<()> {
        self.run("push", |c| {
            c.arg("--follow-tags");
        })?
        .ok()?;
        Ok(())
    }

    pub fn status(&self, ignored: bool) -> Result<String> {
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

    pub fn add<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.run("add", |c| {
            c.arg(path.as_ref());
        })?
        .ok()?;
        Ok(())
    }

    pub fn commit<S>(&self, message: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let result = self.run("commit", |c| {
            c.arg("--message");
            c.arg(message.as_ref());
        })?;

        if result.exit_code == Some(128) && result.stderr.contains("tell me who you are") {
            bail!("E-mail address and/or name is not set in Git repo")
        }

        result.ok()?;
        Ok(())
    }

    pub fn read_config<S>(&self, name: S) -> Result<Option<String>>
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

    pub fn is_tracked<P>(&self, path: P) -> Result<bool>
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

    fn run<F>(&self, command: &str, build: F) -> Result<CommandResult>
    where
        F: FnOnce(&mut Command),
    {
        let mut c = Command::new("git");
        c.arg("-C");
        c.arg(&self.dir);
        c.arg(command);
        build(&mut c);

        let command_str = format!("{:?}", c);
        let result = CommandResult::from_output(command, &c.output()?)?;
        trace!(
            "command={}, exit_code={:?}, stdout=[{}], stderr=[{}]",
            command_str,
            result.exit_code,
            result.stdout,
            result.stderr
        );

        Ok(result)
    }
}
