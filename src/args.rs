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
use clap::{Parser, Subcommand};
use default_env::default_env;
use git_version::git_version;
use path_absolutize::Absolutize;
use std::path::PathBuf;

const DEVTOOL_VERSION: &str = default_env!("DEVTOOL_VERSION", git_version!());

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    about = format!("{} {}", env!("CARGO_PKG_DESCRIPTION"), DEVTOOL_VERSION),
    after_help = format!("{}\nhttps://github.com/rcook/devtool", env!["CARGO_PKG_HOMEPAGE"]),
    version = DEVTOOL_VERSION
)]
pub struct Args {
    #[arg(global = true, help = "Path to Git repository", short = 'd', long = "dir", value_parser = parse_absolute_path)]
    pub git_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(name = "generate-ignore", about = "Generate .gitignore file")]
    GenerateIgnore,

    #[command(
        name = "show-description",
        about = "Show Git description and commit information"
    )]
    ShowDescription,

    #[command(
        name = "increment-tag",
        about = "Generate new Git tag by incrementing existing tag"
    )]
    IncrementTag,
}

fn parse_absolute_path(s: &str) -> Result<PathBuf, String> {
    PathBuf::from(s)
        .absolutize()
        .map_err(|_| String::from("invalid path"))
        .map(|x| x.to_path_buf())
}
