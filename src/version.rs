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
