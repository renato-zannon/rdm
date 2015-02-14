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

use std::old_io::{File, IoError};
use std::error::{Error, FromError};
use std::{env, fmt};

use rustc_serialize::json;
use url::Url;

#[derive(RustcDecodable, Debug)]
pub struct Config {
    pub redmine_key: String,
    pub redmine_url: Url,
}

pub enum ConfigError {
    Loading(IoError),
    Parsing(json::DecoderError),
    NoConfigFile { searched_paths: Vec<Path> },
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

impl FromError<IoError> for ConfigError {
    fn from_error(err: IoError) -> ConfigError {
        ConfigError::Loading(err)
    }
}

impl FromError<json::DecoderError> for ConfigError {
    fn from_error(err: json::DecoderError) -> ConfigError {
        ConfigError::Parsing(err)
    }
}

pub fn get() -> Result<Config, ConfigError> {
    use std::old_io::fs::PathExtensions;
    use std::iter::Unfold;

    let possible_paths: Vec<Path> = Unfold::new(env::current_dir().ok(), |current_dir| {
        current_dir.clone().map(|dir| {
            let mut next = dir.clone();

            if next.pop() {
                *current_dir = Some(next);
            } else {
                *current_dir = None;
            }

            dir
        })
    }).map(|path| path.join(".rdm.json")).collect();

    let maybe_path = possible_paths.iter().find(|p| p.exists());

    let path = match maybe_path {
        Some(path) => path,
        None       => {
            return Err(ConfigError::NoConfigFile { searched_paths: possible_paths.clone() });
        }
    };

    let config_src = try!(File::open(path).read_to_string());

    json::decode(&config_src).map_err(FromError::from_error)
}
