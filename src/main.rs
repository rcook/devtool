mod version;

use crate::version::parse_version;
use std::process::Command;
use std::str::from_utf8;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg("/home/rcook/src/isopy")
        .arg("describe")
        .output()?;
    let s = from_utf8(output.stdout.as_slice())?.trim();
    let description = GitDescription::parse(s).expect("must succeed");
    let mut version = parse_version(description.tag.as_str()).expect("must succeed");
    println!("description={description:?}", description = description);
    println!("version={version:?}", version = version);
    version.increment();
    println!("version={version:?}", version = version.to_string());
    Ok(())
}

#[derive(Debug)]
struct Offset {
    commit: String,
    count: i32,
}

#[derive(Debug)]
struct GitDescription {
    description: String,
    tag: String,
    offset: Option<Offset>,
}

impl GitDescription {
    fn parse(s: &str) -> Option<Self> {
        let parts = s.split('-').collect::<Vec<_>>();

        match parts.len() {
            1 => Some(Self {
                description: String::from(s),
                tag: String::from(parts[0]),
                offset: None,
            }),
            3 => Some(Self {
                description: String::from(s),
                tag: String::from(parts[0]),
                offset: Some(Offset {
                    commit: String::from(parts[2]),
                    count: parts[1].parse::<i32>().ok()?,
                }),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use rstest::rstest;

    #[rstest]
    #[case("v0.0.21-1-gdf3eff3")]
    fn test_basics(#[case] input: &str) {
        parse(input);
    }
}
