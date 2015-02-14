/* rdm - A command-line redmine client
 * Copyright (C) 2015 Renato Zannon
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, see <http://www.gnu.org/licenses/>. */

use docopt::{self, Docopt};
use std::{fmt, error};

const USAGE: &'static str = "
Usage:
    rdm --help
    rdm issues [--assigned-to=<user>] [--open|--closed|--status=<status>]
    rdm issue <issue-number> update --status=<status>
    rdm issue <issue-number> close [--status=<status>]

Options
    -h, --help                Show this message
    -s, --status=<status>     A status name (case-insensitive). Optional when closing if the user
                              has the 'default_close_status' setting on the config file.
    -a, --assigned-to=<user>  The user whose issues we are searching. It can be an exact match
                              of the name or a partial, case-insensitive match of the user's name.
    issue-number              The number of an issue
";

#[derive(RustcDecodable)]
#[allow(dead_code)]
struct RawArgs {
    cmd_issue: bool,
    cmd_issues: bool,
    cmd_update: bool,
    cmd_close: bool,

    arg_issue_number: Option<u32>,

    flag_assigned_to: Option<String>,
    flag_status: Option<String>,
    flag_open: bool,
    flag_closed: bool,
    flag_help: bool,
}

#[derive(Debug)]
pub enum Status {
    AllOpen,
    AllClosed,
    Specific(String),
}

#[derive(Debug)]
pub enum Args {
    ListIssues  { assigned_to: Option<String>, status: Status },
    UpdateIssue { number: u32, new_status: String },
    CloseIssue  { number: u32, close_status: Option<String> },
}

#[derive(Debug)]
enum ErrorCause {
    FromDocopt(docopt::Error),
    InconsistentArguments(&'static str),
}

#[derive(Debug)]
pub struct Error {
    cause: ErrorCause
}

impl Error {
    pub fn exit_status(&self) -> i32 {
        use self::ErrorCause::*;

        match self.cause {
            FromDocopt(ref err) if err.fatal() => 1,
            _ => 0,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ErrorCause::*;

        match self.cause {
            FromDocopt(ref err) => {
                if err.fatal() {
                    write!(f, "Argument error: {}", err)
                } else {
                    write!(f, "{}", err)
                }
            },

            InconsistentArguments(msg) => {
                write!(f, "Argument error: {}", msg)
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Argument error"
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::ErrorCause::*;

        match self.cause {
            FromDocopt(ref err) => Some(err),
            InconsistentArguments(_) => None,
        }
    }
}

impl error::FromError<docopt::Error> for Error {
    fn from_error(err: docopt::Error) -> Error {
        Error { cause: ErrorCause::FromDocopt(err) }
    }
}

impl error::FromError<&'static str> for Error {
    fn from_error(message: &'static str) -> Error {
        Error { cause: ErrorCause::InconsistentArguments(message) }
    }
}

pub fn parse() -> Result<Args, Error> {
    let raw: RawArgs = try!(Docopt::new(USAGE).and_then(|d| d.decode()));

    if raw.cmd_issues {
        let status = match raw.flag_status {
            Some(s) => Status::Specific(s),
            None if raw.flag_closed => Status::AllClosed,
            _ => Status::AllOpen,
        };

        return Ok(Args::ListIssues {
            status: status,
            assigned_to: raw.flag_assigned_to,
        });
    }

    let issue_number = raw.arg_issue_number.unwrap();

    if raw.cmd_update {
        match raw.flag_status {
            Some(st) => Ok(Args::UpdateIssue { number: issue_number, new_status: st }),
            None     => Err(error::FromError::from_error("update")),
        }
    } else if raw.cmd_close {
        Ok(Args::CloseIssue { number: issue_number, close_status: raw.flag_status })
    } else {
        unreachable!();
    }
}
