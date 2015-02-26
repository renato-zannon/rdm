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

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct IssueStatus {
    id: u32,
    name: String,
}

impl IssueStatus {
    pub fn into_pair(self) -> (u32, String) { (self.id, self.name) }
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct User {
    id: u32,
    login: String,
    firstname: String,
    lastname: String,
}
