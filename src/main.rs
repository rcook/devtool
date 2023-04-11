mod git_description;
mod result;
mod version;

use crate::git_description::GitDescription;
use crate::result::Result;
use crate::version::parse_version;
use std::process::Command;
use std::str::from_utf8;

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
