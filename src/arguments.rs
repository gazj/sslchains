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
use std::fs;
use std::io;

pub fn process(from_index: usize) -> Result<Vec<String>, io::Error>
{
    let mut args = vec![];

    for arg in env::args().skip(from_index) // Skip executable name and options.
    {
        args.push(arg);
    }

    if args.len() == 0
    {
        for path in fs::read_dir(".").unwrap()
        {
            let path = path.unwrap().path();

            let md = fs::metadata(&path)?;

            if md.is_file()
            {
                match path.to_str()
                {
                    Some(p) => args.push(p.to_string()),
                    _ => ()

                }
            }
        }
    }

    expand(args)
}

/**
 * Expand command line arguments.
 *
 * This method is separated for testability.
 */
fn expand(args: Vec<String>) -> Result<Vec<String>, io::Error>
{
    let mut expanded: Vec<String> = vec![];

    for arg in args
    {
        let md = fs::metadata(&arg)?;

        if md.is_dir()
        {
            for path in fs::read_dir(&arg).unwrap()
            {
                let path = path.unwrap().path();

                let md = fs::metadata(&path)?;

                if md.is_file()
                {
                    match path.to_str()
                    {
                        Some(p) => expanded.push(p.to_string()),
                        _ => ()

                    }
                }
            }
        }

        else if md.is_file()
        {
            expanded.push(arg);
        }
    }

    Ok(expanded)
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn expands_arguments()
    {
        let args = vec![
            String::from("test"),
            String::from("Cargo.toml"),
            ];

            match expand(args)
            {
                Ok(x) => {
                    assert_eq!(x,
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
}
