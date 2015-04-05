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
use std::io::{self, Read};
use std::collections::HashMap;
use url::{Url, UrlParser};
use uuid::Uuid;

use hyper;
use hyper::header;
use hyper::status::{StatusCode, StatusClass};

use rustc_serialize::json;

use user_config::Config;
use models::{User, IssueStatus};

header! {
    (RedmineApiKey, "X-Redmine-API-Key") => [String]
}

pub struct Client {
    config: Config,
}

struct Request {
    method: Method,
    body: Option<String>,
    url: Url,
}

#[derive(Clone, Copy, Debug)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Method::*;

        let desc = match *self {
            Get    => "GET",
            Post   => "POST",
            Put    => "PUT",
            Delete => "DELETE",
        };

        write!(f, "{}", desc)
    }
}

trait PrintableError: ::std::error::Error + fmt::Debug + Send {}
impl<T> PrintableError for T where T: ::std::error::Error + fmt::Debug + Send {}

#[derive(Debug)]
pub enum Error {
    Http(Box<PrintableError>),
    Response(Box<PrintableError>),
    Forbidden(Method, Url),
    Server(Method, Url),
    Unknown(Method, Url, StatusCode),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Http(ref err) => write!(f, "Http error: {}", err),
            Error::Response(ref err) => write!(f, "Invalid response: {}", err),

            Error::Forbidden(method, ref url) => {
                write!(f, "Authorization error: Server denied access to {} {}", method, url)
            },

            Error::Server(method, ref url) => {
                write!(f, "Server-side error: Server returned error on {} {}", method, url)
            },

            Error::Unknown(method, ref url, ref status) => {
                write!(f, "Unkwnown error: Server returned {} on {} {}", status, method, url)
            }
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Http(_)          => "Http error",
            Error::Response(_)      => "Received invalid response",
            Error::Forbidden(_, _)  => "User not authorized to perform action",
            Error::Server(_, _)     => "Server-side error",
            Error::Unknown(_, _, _) => "Unknown error",
        }
    }
}

impl From<hyper::HttpError> for Error {
    fn from(err: hyper::HttpError) -> Error {
        Error::Http(Box::new(err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Http(Box::new(err))
    }
}

impl From<json::DecoderError> for Error {
    fn from(err: json::DecoderError) -> Error {
        Error::Response(Box::new(err))
    }
}

impl Client {
    pub fn config(&self) -> &Config { &self.config }

    pub fn new(config: Config) -> Client {
        Client { config: config }
    }

    pub fn update_issue(&self, number: u32, status: u32) -> Result<(), Error> {
        let body = {
            let mut body       = HashMap::new();
            let mut issue_diff = HashMap::new();

            issue_diff.insert("status_id", status);
            body.insert("issue", issue_diff);

            json::encode(&body).unwrap()
        };

        let _response = try!(self.send_request(Request {
            method: Method::Put,
            body: Some(body),
            url: self.issue_url(number),
        }));

        Ok(())
    }

    pub fn issue_statuses(&self) -> Result<Vec<IssueStatus>, Error> {
        #[derive(RustcDecodable, Debug)]
        struct IssueStatuses {
            issue_statuses: Vec<IssueStatus>
        }

        let mut response = try!(self.send_request(Request {
            method: Method::Get,
            body: None,
            url: self.build_url("issue_statuses.json"),
        }));

        let mut response_contents = String::new();
        try!(response.read_to_string(&mut response_contents));

        let parsed: IssueStatuses = try!(json::decode(&response_contents));

        Ok(parsed.issue_statuses)
    }

    pub fn users(&self) -> Result<Vec<User>, Error> {
        #[derive(RustcDecodable, Debug)]
        struct Users {
            users: Vec<User>
        }

        let mut response = try!(self.send_request(Request {
            method: Method::Get,
            body: None,
            url: self.build_url("users.json"),
        }));

        let mut response_contents = String::new();
        try!(response.read_to_string(&mut response_contents));

        let parsed: Users = try!(json::decode(&response_contents));

        Ok(parsed.users)
    }

    fn send_request<'a>(&self, request: Request) -> Result<hyper::client::Response, Error> {
        let request_id = Uuid::new_v4();
        let mut client = hyper::Client::new();

        debug!("Request {} - {} {}", request_id, request.method, request.url);
        debug!("Request {} - Body: {:?}", request_id, request.body);

        let url = request.url.clone();

        let request_builder = match request.method {
            Method::Get    => client.get(url),
            Method::Post   => client.post(url),
            Method::Put    => client.put(url),
            Method::Delete => client.delete(url),
        };

        let request_with_headers = request_builder
            .header(RedmineApiKey(self.config.redmine_key().to_string()))
            .header(header::ContentType("application/json".parse().unwrap()));

        let complete_request = match request.body {
            None        => request_with_headers,
            Some(ref s) => request_with_headers.body(&s[..]),
        };

        let response = try!(complete_request.send());
        debug!("Request {} - Received response: {}", request_id, response.status);

        match (response.status, response.status.class()) {
            (StatusCode::Forbidden, _) | (StatusCode::Unauthorized, _) => {
                Err(Error::Forbidden(request.method, request.url))
            },

            (_, StatusClass::ServerError) => {
                Err(Error::Server(request.method, request.url))
            },

            (_, StatusClass::ClientError) | (_, StatusClass::NoClass)  => {
                Err(Error::Unknown(request.method, request.url, response.status))
            },

            (_, StatusClass::Success) => Ok(response),

            (status, _) => {
                panic!("Request {} - Response for status {} not implemented", request_id, status);
            }
        }
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
