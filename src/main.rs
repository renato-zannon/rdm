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

#![feature(plugin, core, io, path, env)]
#![plugin(json_macros, docopt_macros)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
extern crate url;
extern crate hyper;

use std::env;

mod args;
mod client;
mod user_config;

use args::{Args, parse};

const SOLVED_STATUS_ID: i32 = 3;

macro_rules! get_or_exit(
    ($result:expr, $err_p:pat => $err_e:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(e) => {
                println!("{}", e);
                let exit_status = match e { $err_p => $err_e };
                env::set_exit_status(exit_status);
                return;
            }
        }
    }
);

fn main() {
    let args   = get_or_exit!(args::parse(),      e => e.exit_status());
    let config = get_or_exit!(user_config::get(), _ => 1);

    let client = client::Client::new(config);

    match args {
        Args::CloseIssue { number, .. } => {
            client.update_issue(number, SOLVED_STATUS_ID).unwrap();
        },

        _ => unimplemented!(),
    }
}
