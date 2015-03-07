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

use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::PathBuf;
use std::thread;

use client::{self, Client};
use models::{IssueStatus, User};
use rustc_serialize::json;

use std::time as std_time;
use time;

pub struct Cache {
    data: CacheData,
    path: PathBuf,
}

impl Cache {
    pub fn new(client: &mut Client) -> Result<Cache, client::Error> {
        let config_path = client.config().path().to_path_buf();
        let cache_path  = config_path.with_file_name(&".rdm-cache.json");

        let cache_fresh = match (config_path.metadata(), cache_path.metadata()) {
            (Ok(config_meta), Ok(cache_meta)) => {
                let newer_than_config = config_meta.modified() <= cache_meta.modified();

                if newer_than_config {
                    let now    = time::now().to_timespec();

                    let cache_modified_sec = cache_meta.modified() / 1_000;
                    let change = time::Timespec::new(cache_modified_sec as i64, 0);

                    (now - change) < std_time::Duration::hours(2)
                } else {
                    false
                }
            },
            _ => false,
        };

        let mut open_options = OpenOptions::new();
        open_options.read(true).write(true);

        let cache_data;

        if cache_fresh {
            let file = open_options.open(&cache_path).unwrap();

            let mut cache_content = String::new();
            BufReader::new(file).read_to_string(&mut cache_content).unwrap();
            cache_data = json::decode(&cache_content).unwrap();
        } else {
            cache_data = CacheData {
                issue_statuses: None,
                users: None,
            };
        }

        Ok(Cache {
            data: cache_data,
            path: cache_path,
        })
    }

    pub fn issue_statuses(&mut self, client: &Client) -> Result<Vec<(u32, String)>, client::Error> {
        let statuses = match self.data.issue_statuses {
            Some(ref statuses) => statuses.clone(),

            None => {
                let statuses = try!(client.issue_statuses());
                self.data.issue_statuses = Some(statuses.clone());
                self.update_cache();
                statuses
            }
        };

        Ok(statuses.map_in_place(|s| s.into_pair()))
    }

    fn update_cache(&self) {
        let mut open_options = OpenOptions::new();
        open_options.write(true).create(true).truncate(true);

        let cache_content = json::encode(&self.data).unwrap();

        let mut file = open_options.open(&self.path).unwrap();
        write!(&mut file, "{}", cache_content).unwrap();
    }

}

#[derive(RustcDecodable, RustcEncodable, Clone)]
struct CacheData {
    issue_statuses: Option<Vec<IssueStatus>>,
    users: Option<Vec<User>>
}
