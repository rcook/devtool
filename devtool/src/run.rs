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
use crate::args::{Args, Command};
use crate::commands::{bump_version, generate_config, generate_ignore, scratch, show_description};
use crate::logging::init_logging;
use anyhow::{anyhow, Result};
use clap::Parser;
use joatmon::find_sentinel_dir;
use std::env::current_dir;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = current_dir()?;
    let args = Args::parse();

    init_logging(args.detailed, args.log_level)?;

    let git_dir = args
        .git_dir
        .or_else(|| {
            find_sentinel_dir(Path::new(".git"), &cwd, None).map(|mut dir| {
                dir.pop();
                dir
            })
        })
        .ok_or_else(|| anyhow!("Cannot infer Git project directory"))?;

    let app = App::new(&cwd, git_dir);

    match args.command {
        Command::BumpVersion {
            version,
            push_all,
            _no_push_all,
        } => bump_version(&app, &version, push_all)?,
        Command::GenerateConfig => generate_config(&app)?,
        Command::GenerateIgnore => generate_ignore(&app)?,
        Command::Scratch => scratch(&app),
        Command::ShowDescription => show_description(&app)?,
    }
    Ok(())
}
