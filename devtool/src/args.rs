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
use clap::{ArgAction, Parser, Subcommand};
use devtool_version::Version;
use log::LevelFilter;
use path_absolutize::Absolutize;
use std::path::PathBuf;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE_HOME_PAGE: &str = env!("CARGO_PKG_HOMEPAGE");
const PACKAGE_BUILD_VERSION: Option<&str> = option_env!("RUST_TOOL_ACTION_BUILD_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = PACKAGE_NAME,
    version = PACKAGE_VERSION,
    about = format!("{} {}", PACKAGE_DESCRIPTION, PACKAGE_VERSION),
    after_help = format!("{}{}", PACKAGE_HOME_PAGE, PACKAGE_BUILD_VERSION.map(|x| format!("\n\n{}", x)).unwrap_or(String::from("")))
)]
pub struct Args {
    #[arg(global = true, help = "Detailed logging", long = "detailed")]
    pub detailed: bool,

    #[arg(
        global = true,
        help = "Logging level filter",
        short = 'l',
        long = "level",
        default_value_t = LevelFilter::Info
    )]
    pub log_level: LevelFilter,

    #[arg(global = true, help = "Path to Git repository", short = 'd', long = "dir", value_parser = parse_absolute_path)]
    pub git_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        name = "bump-version",
        about = "Update Cargo.toml/pyproject.toml version, generate new Git tag and push"
    )]
    BumpVersion {
        #[arg(help = "Version number to bump to")]
        version: Option<Version>,

        #[arg(help = "Do not push commits and tags", long = "no-push-all", action = ArgAction::SetFalse)]
        push_all: bool,

        #[arg(
            help = "Push commits and tags",
            long = "push-all",
            overrides_with = "push_all"
        )]
        _no_push_all: bool,
    },

    #[command(name = "gen-config", about = "Generate devtool configuration file")]
    GenerateConfig,

    #[command(name = "gen-ignore", about = "Generate .gitignore file")]
    GenerateIgnore,

    #[command(name = "scratch", about = "(Experimental)")]
    Scratch,

    #[command(
        name = "show-description",
        about = "Show Git description and commit information"
    )]
    ShowDescription,
}

fn parse_absolute_path(s: &str) -> Result<PathBuf, String> {
    PathBuf::from(s)
        .absolutize()
        .map_err(|_| String::from("invalid path"))
        .map(|x| x.to_path_buf())
}
