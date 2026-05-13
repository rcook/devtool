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

#[cfg(test)]
mod tests {
    use super::App;
    use crate::serialization::Config;
    use std::path::PathBuf;

    #[test]
    fn new_stores_path() {
        let app = App::new("/some/dir");
        assert_eq!(PathBuf::from("/some/dir"), app.git.dir);
    }

    #[test]
    fn config_path_appends_config_file_name() {
        let app = App::new("/project");
        assert_eq!(PathBuf::from("/project/.devtool.yaml"), app.config_path());
    }

    #[test]
    fn read_config_returns_none_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(dir.path());
        let config = app.read_config().unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn write_and_read_config_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(dir.path());
        let config = Config {
            cargo_toml_paths: vec![PathBuf::from("Cargo.toml")],
            pyproject_toml_paths: vec![],
        };
        app.write_config(&config, false).unwrap();
        let read_back = app.read_config().unwrap().unwrap();
        assert_eq!(config.cargo_toml_paths, read_back.cargo_toml_paths);
        assert_eq!(config.pyproject_toml_paths, read_back.pyproject_toml_paths);
    }

    #[test]
    fn write_config_no_overwrite_fails_when_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(dir.path());
        let config = Config::default();
        app.write_config(&config, false).unwrap();
        let result = app.write_config(&config, false);
        assert!(result.is_err());
    }

    #[test]
    fn write_config_overwrite_succeeds_when_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(dir.path());
        let config = Config::default();
        app.write_config(&config, false).unwrap();
        app.write_config(&config, true).unwrap();
        let read_back = app.read_config().unwrap().unwrap();
        assert!(read_back.cargo_toml_paths.is_empty());
    }
}
