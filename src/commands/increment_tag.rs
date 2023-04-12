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
use crate::result::Result;
use crate::version::parse_version;
use crate::{git::Git, result::reportable};
use std::path::{Path, PathBuf};

const FIRST_TAG: &'static str = "v0.0.0";

pub fn increment_tag<P>(git_dir: P) -> Result<()>
where
    P: AsRef<Path> + Into<PathBuf>,
{
    println!("git_dir={git_dir}", git_dir = git_dir.as_ref().display());
    let git = Git::new(git_dir.as_ref());

    let branch = git.rev_parse_abbrev_ref()?;
    if branch != "main" && branch != "master" {
        return Err(reportable("Must be on the \"main\" or \"master\" branch"));
    }

    let tag = match git.describe()? {
        Some(description) => {
            if description.offset == None {
                return Err(reportable(format!(
                    "No commits since most recent tag \"{}\"",
                    description.tag
                )));
            }

            match parse_version(&description.tag) {
                Some(mut version) => {
                    println!("description={:#?}", description);
                    version.increment();
                    version.to_string()
                }
                None => {
                    return Err(reportable(format!(
                        "Cannot parse tag \"{}\" as version",
                        description.tag
                    )))
                }
            }
        }
        None => String::from(FIRST_TAG),
    };

    git.tag_a(&tag)?;
    println!("Created tag {}", tag);

    git.push_follow_tags()?;
    println!("Pushed commits and tags");

    Ok(())
}
