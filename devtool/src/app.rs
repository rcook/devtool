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
use crate::{constants::CONFIG_FILE_NAME, serialization::Config};
use anyhow::Result;
use devtool_git::Git;
use joatmon::{read_yaml_file, safe_write_file};
use std::path::PathBuf;

#[derive(Debug)]
pub struct App {
    pub git: Git,
}

impl App {
    pub fn new<P>(git_dir: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            git: Git::new(git_dir),
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.git.dir.join(CONFIG_FILE_NAME)
    }

    pub fn read_config(&self) -> Result<Option<Config>> {
        // TBD: Complete with time-of-check time-of-use race condition!
        let config_path = self.config_path();
        if config_path.is_file() {
            Ok(Some(read_yaml_file(&config_path)?))
        } else {
            Ok(None)
        }
    }

    pub fn write_config(&self, config: &Config, overwrite: bool) -> Result<()> {
        safe_write_file(
            &self.config_path(),
            serde_yaml::to_string(config)?,
            overwrite,
        )?;
        Ok(())
    }
}
