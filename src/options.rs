//! Copyright (C) 2022 Gaz J.
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

extern crate getopt;

use getopt::{Opt, Parser};

#[derive(Debug)]
pub enum OptionsDisplayMode
{
    Default,
    OneLine,
}

/// Represents the state/usage of all command line options.
#[derive(Debug)]
pub struct Options
{
    // Print help and exit.
    pub print_help: bool,

    // Disable the file count limit.
    pub disable_file_limit: bool,

    // Determines the output format.
    pub display_mode: OptionsDisplayMode,

    // Determine whether to follow symbolic links.
    pub follow_symlinks: bool,

    // Determine whether to process hidden files.
    pub include_hidden_files: bool,

    // Determine whether to recursively traverse the directory arguments.
    pub recursive: bool,

    // Determine whether to cross filesystem boundaries.
    pub same_file_system: bool,

    // If the display mode is OneLine, use this option
    // to suppress the header row.
    pub suppress_oneline_header: bool,

    // Argument index where options end.
    // Use this to split env::args for argument processing.
    // e.g. args.split_off(options.index)
    pub index: usize
}

impl Options
{
    pub fn new() -> Options
    {
        let args: Vec<String> = std::env::args().collect();
        let mut opts = Parser::new(&args, "hHlLrSUX");

        let mut instance = Options {
            print_help: false,
            disable_file_limit: false,
            display_mode: OptionsDisplayMode::Default,
            follow_symlinks: false,
            include_hidden_files: false,
            recursive: false,
            same_file_system: true,
            suppress_oneline_header: false,
            index: 0
        };

        Options::process_input(&mut instance, &mut opts).unwrap();

        instance.index = opts.index();

        instance
    }

    fn process_input(instance: &mut Options, opts: &mut Parser) -> Result<(), Box<dyn std::error::Error>>
    {
        loop {
            match opts.next().transpose()? {
                None => break,
                Some(opt) => match opt {
                    Opt('h', None) => instance.print_help = true,
                    Opt('H', None) => instance.include_hidden_files = true,
                    Opt('l', None) => instance.display_mode = OptionsDisplayMode::OneLine,
                    Opt('L', None) => {
                        instance.display_mode = OptionsDisplayMode::OneLine;
                        instance.suppress_oneline_header = true;
                    },
                    Opt('r', None) => instance.recursive = true,
                    Opt('S', None) => instance.follow_symlinks = true,
                    Opt('U', None) => instance.disable_file_limit = true,
                    Opt('X', None) => instance.same_file_system = false,
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }

}
