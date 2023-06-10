//! sslchains
//!
//! A tool to identify related SSL keys, CSRs, and certificates.
//!
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

use std::process;
use std::env;

mod arguments;
mod chain;
mod display;
mod keys;
mod options;

fn main()
{
    // Get command line options.
    let options = options::Options::new();

    // Print help and exit.
    if options.print_help
    {
        help();
    }

    // Get command line arguments.
    let args = match arguments::process(&options)
    {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e.to_string());
            process::exit(1);
        }
    };

    // Sort expanded arguments.
    //args.sort();

    // Build chains from the arguments.
    let chains = match chain::build(args)
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.to_string());
            process::exit(2);
        }
    };

    // Display output.
    match options.display_mode
    {
        options::OptionsDisplayMode::OneLine => display::oneline(chains),
        _ => display::default(chains)
    }
}

fn help()
{
    println!("\nUsage");
    println!("\t{} [-hlL] [path [...]]", env::current_exe().unwrap().to_str().unwrap());
    println!("\t\t-h\tPrint this help menu.");
    println!("\t\t-H\tProcess hidden files and directories.");
    println!("\t\t-l\tOutput each chain as a row of values.");
    println!("\t\t-L\tOutput each chain as a row of values (header excluded).");
    println!("\t\t-r\tProcess arguments recursively.");
    println!("\t\t-S\tFollow symbolic links.");
    println!("\t\t-U\tProcess an unlimited number of file paths.");
    println!("\t\t-X\tCross filesystem boundaries.");
    process::exit(3);
}

