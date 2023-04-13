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
use crate::result::{reportable, Result};
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

pub struct Git {
    pub dir: PathBuf,
}

impl Git {
    pub fn new<P>(dir: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self { dir: dir.into() }
    }

    pub fn describe(&self) -> Result<Option<GitDescription>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("describe")
            .output()?;
        let exit_code = output.status.code();
        let stderr = from_utf8(output.stderr.as_slice())?.trim();

        if exit_code == Some(128) && stderr.contains("cannot describe anything") {
            return Ok(None);
        }

        if !output.status.success() {
            return Err(reportable(match exit_code {
                Some(code) => format!("git describe failed with exit code {}", code),
                None => String::from("git describe failed"),
            }));
        }

        let stdout = from_utf8(output.stdout.as_slice())?.trim();
        Ok(GitDescription::parse(stdout))
    }

    pub fn rev_parse_abbrev_ref(&self) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()?;
        if !output.status.success() {
            let exit_code = output.status.code();
            return Err(reportable(match exit_code {
                Some(code) => format!("git rev-parse failed with exit code {}", code),
                None => String::from("git rev-parse failed"),
            }));
        }

        let s = from_utf8(output.stdout.as_slice())?.trim();
        Ok(String::from(s))
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
        if !output.status.success() {
            let exit_code = output.status.code();
            return Err(reportable(match exit_code {
                Some(code) => format!("git tag failed with exit code {}", code),
                None => String::from("git tag failed"),
            }));
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
        if !output.status.success() {
            let exit_code = output.status.code();
            return Err(reportable(match exit_code {
                Some(code) => format!("git push failed with exit code {}", code),
                None => String::from("git push failed"),
            }));
        }

        Ok(())
    }

    pub fn status_ignored(&self) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .arg("status")
            .arg("--porcelain")
            .arg("--ignored")
            .output()?;
        if !output.status.success() {
            let exit_code = output.status.code();
            return Err(reportable(match exit_code {
                Some(code) => format!("git status failed with exit code {}", code),
                None => String::from("git status failed"),
            }));
        }

        Ok(String::from(from_utf8(output.stdout.as_slice())?))
    }
}
