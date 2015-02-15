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

use std::fmt;
use url::Url;

use hyper;
use hyper::header::{self, Header, HeaderFormat};
use hyper::HttpResult;

use user_config::Config;

#[derive(Clone)]
struct RedmineApiKey { key: String }

impl Header for RedmineApiKey {
    fn header_name() -> &'static str {
        "X-Redmine-API-Key"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Option<RedmineApiKey> {
        use hyper::header::parsing::from_one_raw_str;

        from_one_raw_str(raw).map(|k| RedmineApiKey { key: k })
    }
}

impl HeaderFormat for RedmineApiKey {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

pub struct Client {
    config: Config,
}

struct Request<'a> {
    method: Method,
    body: String,
    url: Url,
}

enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Client {
    pub fn new(config: Config) -> Client {
        Client { config: config }
    }

    pub fn update_issue(&self, number: u32, status: i32) -> Result<(), hyper::HttpError> {
        let response = self.send_request(Request {
            method: Method::Put,
            body: json!({ "issue": { "status_id": (status) } }).to_string(),
            url: self.issue_url(number),
        });

        match response {
            Ok(_)  => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn send_request<'a>(&self, request: Request) -> HttpResult<hyper::client::Response> {
        let mut client = hyper::Client::new();

        let request_builder = match request.method {
            Method::Get    => client.get(request.url),
            Method::Post   => client.post(request.url),
            Method::Put    => client.put(request.url),
            Method::Delete => client.delete(request.url),
        };

        request_builder
            .header(RedmineApiKey { key: self.config.redmine_key.clone() })
            .header(header::ContentType("application/json".parse().unwrap()))
            .body(&request.body[..])
            .send()
    }

    fn issue_url(&self, number: u32) -> Url {
        let mut request_url = self.config.redmine_url.clone();

        {
            let path = request_url.path_mut().unwrap();
            path.push("issues".to_string());
            path.push(format!("{}.json", number));
        }

        request_url
    }
}
