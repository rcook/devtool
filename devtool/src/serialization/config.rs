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
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "cargo_toml_paths", default)]
    pub cargo_toml_paths: Vec<PathBuf>,

    #[serde(rename = "pyproject_toml_paths", default)]
    pub pyproject_toml_paths: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::path::PathBuf;

    #[test]
    fn default_has_empty_vecs() {
        let config = Config::default();
        assert!(config.cargo_toml_paths.is_empty());
        assert!(config.pyproject_toml_paths.is_empty());
    }

    #[test]
    fn yaml_round_trip() {
        let config = Config {
            cargo_toml_paths: vec![PathBuf::from("Cargo.toml"), PathBuf::from("sub/Cargo.toml")],
            pyproject_toml_paths: vec![PathBuf::from("pyproject.toml")],
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.cargo_toml_paths, deserialized.cargo_toml_paths);
        assert_eq!(
            config.pyproject_toml_paths,
            deserialized.pyproject_toml_paths
        );
    }

    #[test]
    fn deserialize_empty_yaml() {
        let config: Config = serde_yaml::from_str("{}").unwrap();
        assert!(config.cargo_toml_paths.is_empty());
        assert!(config.pyproject_toml_paths.is_empty());
    }

    #[test]
    fn deserialize_partial_yaml() {
        let yaml = "cargo_toml_paths:\n  - Cargo.toml\n";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(vec![PathBuf::from("Cargo.toml")], config.cargo_toml_paths);
        assert!(config.pyproject_toml_paths.is_empty());
    }

    #[test]
    fn deserialize_yaml_with_paths() {
        let yaml = "cargo_toml_paths:\n  - a/Cargo.toml\n  - b/Cargo.toml\npyproject_toml_paths:\n  - pyproject.toml\n";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            vec![PathBuf::from("a/Cargo.toml"), PathBuf::from("b/Cargo.toml")],
            config.cargo_toml_paths
        );
        assert_eq!(
            vec![PathBuf::from("pyproject.toml")],
            config.pyproject_toml_paths
        );
    }
}
