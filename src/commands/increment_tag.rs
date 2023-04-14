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
use crate::app::App;
use crate::version::parse_version;
use anyhow::{bail, Result};

const FIRST_TAG: &str = "v0.0.0";

pub fn increment_tag(app: &App) -> Result<()> {
    let branch = app.git.rev_parse_abbrev_ref()?;
    if branch != "main" && branch != "master" {
        bail!("Must be on the \"main\" or \"master\" branch")
    }

    let tag = match app.git.describe()? {
        Some(description) => {
            if description.offset.is_none() {
                bail!("No commits since most recent tag \"{}\"", description.tag)
            }

            match parse_version(&description.tag) {
                Some(mut version) => {
                    println!("description={:#?}", description);
                    version.increment();
                    version.to_string()
                }
                None => bail!("Cannot parse tag \"{}\" as version", description.tag),
            }
        }
        None => String::from(FIRST_TAG),
    };

    app.git.tag_a(&tag)?;
    println!("Created tag {}", tag);

    app.git.push_follow_tags()?;
    println!("Pushed commits and tags");

    Ok(())
}
