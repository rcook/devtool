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
mod app;
mod args;
mod commands;
mod git;
mod util;
mod version;

use crate::app::App;
use crate::args::{Args, Command};
use crate::commands::{generate_ignore, increment_tag, scratch, show_description};
use crate::util::infer_git_dir;
use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;
use std::env::current_dir;
use std::process::exit;

fn main() {
    exit(match run() {
        Ok(()) => 0,
        Err(e) => {
            println!("{}", format!("{}", e).bright_red());
            1
        }
    })
}

fn run() -> Result<()> {
    let cwd = current_dir()?;
    let args = Args::parse();
    let git_dir = match args.git_dir.or_else(|| infer_git_dir(&cwd)) {
        Some(d) => d,
        None => bail!("Cannot infer Git project directory"),
    };

    let app = App::new(cwd, git_dir);

    match args.command {
        Command::GenerateIgnore => generate_ignore(&app)?,
        Command::IncrementTag => increment_tag(&app)?,
        Command::Scratch => scratch(&app)?,
        Command::ShowDescription => show_description(&app)?,
    }
    Ok(())
}
