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
use std::io::{self, BufReader};
use std::fs::File;
use std::path::PathBuf;

use std::error::{Error, FromError};
use std::{env, fmt};

use rustc_serialize::json;
use url::Url;

#[derive(RustcDecodable, Debug, Clone)]
pub struct Config {
    pub redmine_key: String,
    pub redmine_url: Url,
    pub default_close_status: Option<String>,
}

pub enum ConfigError {
    Loading(io::Error),
    Parsing(json::DecoderError),
    NoConfigFile { searched_paths: Vec<PathBuf> },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ConfigError::*;

        let result = write!(f, "Configuration error: ");
        if result.is_err() { return result; }

        match *self {
            Loading(ref err) => write!(f, "{}", err),
            Parsing(ref err) => write!(f, "{}", err),
            NoConfigFile { ref searched_paths } => {
                write!(f, "Unable to find a config file. Searched paths: {:?}", searched_paths)
            }
        }
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        use self::ConfigError::*;

        match *self {
            Loading(_)          => "configuration error: error loading config file",
            Parsing(_)          => "configuration error: syntax error on config file",
            NoConfigFile { .. } => "Unable to find a config file",

        }
    }
}

impl FromError<io::Error> for ConfigError {
    fn from_error(err: io::Error) -> ConfigError {
        ConfigError::Loading(err)
    }
}

impl FromError<json::DecoderError> for ConfigError {
    fn from_error(err: json::DecoderError) -> ConfigError {
        ConfigError::Parsing(err)
    }
}

pub fn get() -> Result<Config, ConfigError> {
    let path = match first_user_config() {
        Ok(path) => path,
        Err(tried_paths) => {
            return Err(ConfigError::NoConfigFile { searched_paths: tried_paths });
        }
    };

    let config_file = try!(File::open(&path));

    let mut config_src = String::new();
    try!(BufReader::new(config_file).read_to_string(&mut config_src));

    json::decode(&config_src).map_err(FromError::from_error)
}

fn first_user_config() -> Result<PathBuf, Vec<PathBuf>> {
    use std::iter::Unfold;

    let cwd: Option<PathBuf> = env::current_dir().ok().map(|old_path| {
        PathBuf::new(&old_path)
    });

    let possible_paths = Unfold::new(cwd, |current_dir| {
        current_dir.clone().map(|dir| {
            let mut next = dir.clone();

            if next.pop() {
                *current_dir = Some(next);
            } else {
                *current_dir = None;
            }

            dir
        })
    }).map(|mut path| { path.push(".rdm.json"); path });

    let mut tried_paths = Vec::new();

    for path in possible_paths {
        if path.exists() {
            return Ok(path);
        } else {
            tried_paths.push(path);
        }
    }

    return Err(tried_paths);
}
