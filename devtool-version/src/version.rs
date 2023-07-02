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
use anyhow::anyhow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::result::Result as StdResult;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VersionParseError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type VersionParseResult<T> = StdResult<T, VersionParseError>;

#[derive(Debug)]
pub struct Version {
    inner: Box<dyn VersionInner>,
}

impl Version {
    pub fn set_prefix(&mut self, value: bool) {
        self.inner.set_prefix(value);
    }

    pub fn increment(&mut self) {
        self.inner.increment();
    }

    #[must_use]
    pub fn dupe(&self) -> Self {
        Self {
            inner: self.inner.dupe(),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = parse_version_inner(s)?;
        Ok(Self { inner })
    }
}

pub trait VersionInner: Debug + Display {
    fn set_prefix(&mut self, value: bool);
    fn increment(&mut self);
    fn dupe(&self) -> Box<dyn VersionInner>;
}

fn parse_version_inner(s: &str) -> VersionParseResult<Box<dyn VersionInner>> {
    let has_prefix = s.starts_with('v');
    let s1 = if has_prefix { &s[1..] } else { s };
    let parts = s1.split('.').collect::<Vec<_>>();

    match parts.len() {
        1 => Ok(Box::new(VersionSingleton {
            has_prefix,
            major: parts[0].parse::<i32>().map_err(|e| anyhow!(e))?,
        })),
        2 => Ok(Box::new(VersionPair {
            has_prefix,
            major: parts[0].parse::<i32>().map_err(|e| anyhow!(e))?,
            minor: parts[1].parse::<i32>().map_err(|e| anyhow!(e))?,
        })),
        3 => Ok(Box::new(VersionTriple {
            has_prefix,
            major: parts[0].parse::<i32>().map_err(|e| anyhow!(e))?,
            minor: parts[1].parse::<i32>().map_err(|e| anyhow!(e))?,
            build: parts[2].parse::<i32>().map_err(|e| anyhow!(e))?,
        })),
        _ => Err(VersionParseError::Other(anyhow!(
            "could not parse {} as version",
            s
        ))),
    }
}

#[derive(Debug)]
struct VersionSingleton {
    has_prefix: bool,
    major: i32,
}

impl VersionInner for VersionSingleton {
    fn set_prefix(&mut self, value: bool) {
        self.has_prefix = value;
    }

    fn increment(&mut self) {
        self.major += 1;
    }

    fn dupe(&self) -> Box<dyn VersionInner> {
        Box::new(Self {
            has_prefix: self.has_prefix,
            major: self.major,
        })
    }
}

impl std::fmt::Display for VersionSingleton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_prefix {
            write!(f, "v")?;
        }
        write!(f, "{major}", major = self.major)
    }
}

#[derive(Debug)]
struct VersionPair {
    has_prefix: bool,
    major: i32,
    minor: i32,
}

impl VersionInner for VersionPair {
    fn set_prefix(&mut self, value: bool) {
        self.has_prefix = value;
    }

    fn increment(&mut self) {
        self.minor += 1;
    }

    fn dupe(&self) -> Box<dyn VersionInner> {
        Box::new(Self {
            has_prefix: self.has_prefix,
            major: self.major,
            minor: self.minor,
        })
    }
}

impl std::fmt::Display for VersionPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_prefix {
            write!(f, "v")?;
        }
        write!(f, "{major}.{minor}", major = self.major, minor = self.minor)
    }
}

#[derive(Debug)]
struct VersionTriple {
    has_prefix: bool,
    major: i32,
    minor: i32,
    build: i32,
}

impl VersionInner for VersionTriple {
    fn set_prefix(&mut self, value: bool) {
        self.has_prefix = value;
    }

    fn increment(&mut self) {
        self.build += 1;
    }

    fn dupe(&self) -> Box<dyn VersionInner> {
        Box::new(Self {
            has_prefix: self.has_prefix,
            major: self.major,
            minor: self.minor,
            build: self.build,
        })
    }
}

impl std::fmt::Display for VersionTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_prefix {
            write!(f, "v")?;
        }
        write!(
            f,
            "{major}.{minor}.{build}",
            major = self.major,
            minor = self.minor,
            build = self.build
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use anyhow::Result;
    use rstest::rstest;

    #[rstest]
    #[case("1", "v1", "2", "1")]
    #[case("1", "v1", "v2", "v1")]
    #[case("1.2", "v1.2", "1.3", "1.2")]
    #[case("1.2", "v1.2", "v1.3", "v1.2")]
    #[case("1.2.3", "v1.2.3", "1.2.4", "1.2.3")]
    #[case("1.2.3", "v1.2.3", "v1.2.4", "v1.2.3")]
    fn basics(
        #[case] expected_no_prefix: &str,
        #[case] expected_prefix: &str,
        #[case] expected_incremented: &str,
        #[case] input: &str,
    ) -> Result<()> {
        let mut version = input.parse::<Version>()?;
        assert_eq!(input, version.to_string());

        version.set_prefix(false);
        assert_eq!(expected_no_prefix, version.to_string());

        version.set_prefix(true);
        assert_eq!(expected_prefix, version.to_string());

        let other_version = version.dupe();
        assert_eq!(version.to_string(), other_version.to_string());

        let mut version = input.parse::<Version>()?;
        version.increment();
        assert_eq!(expected_incremented, version.to_string());

        Ok(())
    }
}
