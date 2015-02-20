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

#![feature(plugin, core, io, path, env, unicode, fs, old_io, collections, std_misc)]
#![plugin(json_macros, docopt_macros)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
extern crate url;
extern crate time;

#[macro_use(impl_header, deref)]
extern crate hyper;

use std::env;

mod args;
mod client;
mod user_config;
mod cache;

use args::{Args, parse};

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

    let mut client = client::Client::new(config.clone());
    let cache = cache::Cache::new(&mut client);

    match args {
        Args::CloseIssue { number, close_status } => {
            let status_id = close_status
                .or_else(move || config.default_close_status().map(|s| s.to_string()))
                .and_then(|name| find_status_id(&cache, &name))
                .expect("Unable to find the status id to close the issue");

            client.update_issue(number, status_id).unwrap();
        },

        Args::UpdateIssue { number, new_status } => {
            let status_id = match find_status_id(&cache, &new_status) {
                Some(id) => id,
                None     => panic!("Unable to find status id for status '{}'", new_status),
            };

            client.update_issue(number, status_id).unwrap();
        },

        _ => unimplemented!(),
    }
}

fn find_status_id(cache: &cache::Cache, status_name: &str) -> Option<u32> {
    let statuses = cache.issue_statuses();

    statuses.into_iter().filter_map(|(id, name)| {
        let matches = status_name.chars().zip(name.chars()).all(|(query_chr, name_chr)| {
            query_chr.to_lowercase() == name_chr.to_lowercase()
        });

        if matches {
            Some(id)
        } else {
            None
        }
    }).next()
}
