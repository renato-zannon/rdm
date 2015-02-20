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

use url::{Url, UrlParser};

use hyper;
use hyper::header::{self, HeaderFormat};
use hyper::HttpResult;

use rustc_serialize::json;

use user_config::Config;

#[derive(Clone)]
struct RedmineApiKey(String);
impl_header!(RedmineApiKey, "X-Redmine-API-Key", String);

pub struct Client {
    config: Config,
}

struct Request<'a> {
    method: Method,
    body: Option<String>,
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

    pub fn update_issue(&self, number: u32, status: u32) -> Result<(), hyper::HttpError> {
        let response = self.send_request(Request {
            method: Method::Put,
            body: Some(json!({ "issue": { "status_id": (status) } }).to_string()),
            url: self.issue_url(number),
        });

        match response {
            Ok(_)  => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn issue_statuses(&self) -> Result<Vec<(u32, String)>, hyper::HttpError> {
        #[derive(RustcDecodable, Debug)]
        struct IssueStatuses {
            issue_statuses: Vec<IssueStatus>
        }

        #[derive(RustcDecodable, Debug)]
        struct IssueStatus {
            id: u32,
            name: String,
        }

        let maybe_response = self.send_request(Request {
            method: Method::Get,
            body: None,
            url: self.build_url("issue_statuses.json"),
        });

        let response = maybe_response.unwrap().read_to_string().unwrap();
        let parsed: IssueStatuses = json::decode(&response).unwrap();

        Ok(parsed.issue_statuses.into_iter().map(|status| (status.id, status.name)).collect())
    }

    fn send_request<'a>(&self, request: Request) -> HttpResult<hyper::client::Response> {
        let mut client = hyper::Client::new();

        let request_builder = match request.method {
            Method::Get    => client.get(request.url),
            Method::Post   => client.post(request.url),
            Method::Put    => client.put(request.url),
            Method::Delete => client.delete(request.url),
        };

        let request_with_headers = request_builder
            .header(RedmineApiKey(self.config.redmine_key().to_string()))
            .header(header::ContentType("application/json".parse().unwrap()));

        let complete_request = match request.body {
            None        => request_with_headers,
            Some(ref s) => request_with_headers.body(&s[..]),
        };

        complete_request.send()
    }

    fn issue_url(&self, number: u32) -> Url {
        self.build_url(&format!("issues/{}.json", number))
    }

    fn build_url(&self, path: &str) -> Url {
        let request_url = self.config.redmine_url();

        UrlParser::new()
            .base_url(request_url)
            .parse(path)
            .unwrap()
    }
}
