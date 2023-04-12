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
use crate::result::Result;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

pub struct Git {
    dir: PathBuf,
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
        let s = from_utf8(output.stdout.as_slice())?.trim();
        Ok(GitDescription::parse(s))
    }
}
