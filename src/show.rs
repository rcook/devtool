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
use crate::git_description::GitDescription;
use crate::result::Result;
use crate::version::parse_version;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

pub fn show<P>(git_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let output = Command::new("git")
        .arg("-C")
        .arg(git_dir.as_ref().to_str().expect("must be a valid string"))
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
