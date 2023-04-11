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
pub trait Version: std::fmt::Debug + std::fmt::Display {
    fn increment(&mut self);
}

pub fn parse_version(s: &str) -> Option<Box<dyn Version>> {
    let has_v = s.starts_with('v');
    let s1 = if has_v { &s[1..] } else { s };
    let parts = s1.split('.').collect::<Vec<_>>();

    match parts.len() {
        1 => Some(Box::new(VersionSingleton {
            has_v,
            major: parts[0].parse::<i32>().ok()?,
        })),
        2 => Some(Box::new(VersionPair {
            has_v,
            major: parts[0].parse::<i32>().ok()?,
            minor: parts[1].parse::<i32>().ok()?,
        })),
        3 => Some(Box::new(VersionTriple {
            has_v,
            major: parts[0].parse::<i32>().ok()?,
            minor: parts[1].parse::<i32>().ok()?,
            build: parts[2].parse::<i32>().ok()?,
        })),
        _ => None,
    }
}

#[derive(Debug)]
struct VersionSingleton {
    has_v: bool,
    major: i32,
}

impl Version for VersionSingleton {
    fn increment(&mut self) {
        self.major += 1;
    }
}

impl std::fmt::Display for VersionSingleton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_v {
            write!(f, "v")?;
        }
        write!(f, "{major}", major = self.major)
    }
}

#[derive(Debug)]
struct VersionPair {
    has_v: bool,
    major: i32,
    minor: i32,
}

impl Version for VersionPair {
    fn increment(&mut self) {
        self.minor += 1;
    }
}

impl std::fmt::Display for VersionPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_v {
            write!(f, "v")?;
        }
        write!(f, "{major}.{minor}", major = self.major, minor = self.minor)
    }
}

#[derive(Debug)]
struct VersionTriple {
    has_v: bool,
    major: i32,
    minor: i32,
    build: i32,
}

impl Version for VersionTriple {
    fn increment(&mut self) {
        self.build += 1;
    }
}

impl std::fmt::Display for VersionTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_v {
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
