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
mod args;
mod commands;
mod git;
mod result;
mod version;

use clap::Parser;

use crate::args::{Args, Command};
use crate::commands::{generate_ignore, increment_tag, show_description};
use crate::result::{Error, Result};
use colored::Colorize;
use std::env::current_dir;
use std::process::exit;

fn main() {
    match run() {
        Ok(()) => exit(0),
        Err(Error::Reportable { message }) => println!("{}", message.red()),
        Err(e) => println!("{}", format!("Unhandled error: {:#?}", e).red()),
    }
}

fn run() -> Result<()> {
    let cwd = current_dir()?;
    let args = Args::parse();
    let git_dir = args.git_dir.unwrap_or(cwd);
    match args.command {
        Command::GenerateIgnore => generate_ignore(git_dir)?,
        Command::IncrementTag => increment_tag(git_dir)?,
        Command::ShowDescription => show_description(git_dir)?,
    }
    Ok(())
}
