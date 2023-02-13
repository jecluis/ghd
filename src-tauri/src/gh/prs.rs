// Copyright 2023 Joao Eduardo Luis <joao@abysmo.io>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{types::GithubRequest, Github};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PullRequestEntry {
    pub url: String,
    pub html_url: String,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub draft: bool,
    pub milestone: Option<String>,
    pub comments: u32,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PullRequestSearchResult {
    pub total_count: u32,
    pub incomplete_results: bool,
    pub items: Vec<PullRequestEntry>,
}

pub async fn get(
    token: &String,
    user: &String,
) -> Result<Vec<PullRequestEntry>, reqwest::StatusCode> {
    let qstr = format!("type:pr state:open author:{}", user);
    let ghreq = GithubRequest::new(token);
    let req = ghreq.get("/search/issues").query(&[("q", qstr)]);

    match ghreq.send::<PullRequestSearchResult>(req).await {
        Ok(res) => {
            return Ok(res.items);
        }
        Err(err) => Err(err),
    }
}
