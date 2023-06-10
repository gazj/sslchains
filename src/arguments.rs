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

use std::env;
use std::io;
use walkdir::{ DirEntry, WalkDir };

use crate::options::Options;

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with(".") && s != "." && s != "..")
         .unwrap_or(false)
}

pub fn process(options: &Options) -> Result<Vec<String>, io::Error>
{
    let mut args = vec![];

    for arg in env::args().skip(options.index) // Skip executable name and options.
    {
        args.push(arg);
    }

    if args.len() == 0
    {
        args.push(".".to_string());
    }

    expand(args, options)
}

/**
 * Expand command line arguments.
 *
 * This method is separated for testability.
 */
fn expand(args: Vec<String>, options: &Options) -> Result<Vec<String>, io::Error>
{
    let mut expanded: Vec<String> = vec![];

    // If the recursive option is used, expand arguments
    // by walking the filesystem hierarchy from each argument.
    for arg in args
    {
        // println!("{}", arg.to_string());
        for entry in WalkDir::new(arg.to_string())

            // Optionally follow symbolic links.
            .follow_links(options.follow_symlinks)

            // Optionally recurse up to 100 directories deep.
            .max_depth(if options.recursive { 100 } else { 1 })

            // Optionally cross filesystem boundaries.
            .same_file_system(options.same_file_system)

            // Convert to an Iterator.
            .into_iter()

            // Optionally include hidden files.
            .filter_entry(|e| options.include_hidden_files || !is_hidden(e))

            // Skip inaccessible files.
            .filter_map(|e| e.ok())
        {
            // Add only files to the expanded items list.
            if let Ok(md) = entry.metadata()
            {
                if md.is_file()
                {
                    expanded.push(entry.path().to_str().unwrap().to_string());
                }
            }

            // Do not continue if expanded file listing exceeds limit.
            if expanded.len() > 10_000 && !options.disable_file_limit
            {
                eprintln!("File count (10,000 paths) exceeded. Try using path arguments which contain fewer files.");
                std::process::exit(5);
            }
        }
    }

    Ok(expanded)
}

#[cfg(test)]
mod test
{
    use super::*;
    use crate::options::{ Options, OptionsDisplayMode };

    #[test]
    fn expands_arguments()
    {
        let args = vec![
            String::from("test"),
            String::from("Cargo.toml"),
        ];

        let opts = Options {
            print_help: false,
            display_mode: OptionsDisplayMode::Default,
            follow_symlinks: false,
            include_hidden_files: false,
            recursive: false,
            same_file_system: true,
            suppress_oneline_header: false,
            index: 0
        };

        match expand(args, &opts)
        {
            Ok(x) => {
                assert_eq!(
                    x,
                    vec![
                        String::from("test/file2"),
                        String::from("test/file1"),
                        String::from("Cargo.toml"),
                    ]
                );
            },
            Err(x) => assert!(false, "{}", x)
        }
    }

    #[test]
    fn expands_arguments_recursively()
    {
        let args = vec![
            String::from("test"),
            String::from("Cargo.toml"),
        ];

        let opts = Options {
            print_help: false,
            display_mode: OptionsDisplayMode::Default,
            follow_symlinks: false,
            include_hidden_files: false,
            recursive: true,
            same_file_system: true,
            suppress_oneline_header: false,
            index: 0
        };

        match expand(args, &opts)
        {
            Ok(x) => {
                assert_eq!(
                    x,
                    vec![
                        String::from("test/file2"),
                        String::from("test/dir/file3"),
                        String::from("test/dir/file4"),
                        String::from("test/file1"),
                        String::from("Cargo.toml"),
                    ]
                );
            },
            Err(x) => assert!(false, "{}", x)
        }
    }

    #[test]
    fn expands_arguments_with_hidden()
    {
        let args = vec![
            String::from("test"),
            String::from("Cargo.toml"),
        ];

        let opts = Options {
            print_help: false,
            display_mode: OptionsDisplayMode::Default,
            follow_symlinks: false,
            include_hidden_files: true,
            recursive: false,
            same_file_system: true,
            suppress_oneline_header: false,
            index: 0
        };

        match expand(args, &opts)
        {
            Ok(x) => {
                assert_eq!(
                    x,
                    vec![
                        String::from("test/file2"),
                        String::from("test/file1"),
                        String::from("test/.hidden_file"),
                        String::from("Cargo.toml"),
                    ]
                );
            },
            Err(x) => assert!(false, "{}", x)
        }
    }
}
