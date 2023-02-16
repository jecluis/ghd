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

use crate::{db::DB, errors::GHDError};

use super::{api, types::GithubRequest, Github};

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct PullRequestEntry {
    pub id: i64,
    pub author: String,
    pub url: String,
    pub html_url: String,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub draft: bool,
    pub milestone: Option<String>,
    pub comments: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
    pub merged_at: Option<i64>,
}

impl PullRequestEntry {
    pub fn from_api_entry(entry: &api::PullRequestSearchAPIEntry) -> Self {
        PullRequestEntry {
            id: entry.id,
            author: entry.user.login.clone(),
            url: entry.url.clone(),
            html_url: entry.html_url.clone(),
            number: entry.number,
            title: entry.title.clone(),
            state: entry.state.clone(),
            draft: entry.draft,
            milestone: match &entry.milestone {
                Some(m) => Some(m.title.clone()),
                None => None,
            },
            comments: entry.comments,
            created_at: entry.created_at.timestamp(),
            updated_at: entry.updated_at.timestamp(),
            closed_at: match entry.closed_at {
                Some(dt) => Some(dt.timestamp()),
                None => None,
            },
            merged_at: match entry.pull_request.merged_at {
                Some(dt) => Some(dt.timestamp()),
                None => None,
            },
        }
    }
}

#[derive(serde::Deserialize)]
pub struct PullRequestSearchResult {
    pub total_count: u32,
    pub incomplete_results: bool,
    pub items: Vec<api::PullRequestSearchAPIEntry>,
}

pub async fn fetch_by_author(
    db: &DB,
    gh: &Github,
    login: &String,
) -> Result<Vec<PullRequestEntry>, GHDError> {
    let token = match gh.get_token(&db).await {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let qstr = format!("type:pr state:open author:{}", login);
    let ghreq = GithubRequest::new(&token);
    let req = ghreq.get("/search/issues").query(&[("q", qstr)]);

    match ghreq.send::<PullRequestSearchResult>(req).await {
        Ok(res) => {
            let mut v = Vec::<PullRequestEntry>::new();
            for entry in &res.items {
                v.push(PullRequestEntry::from_api_entry(entry));
            }
            Ok(v)
        }
        Err(err) => {
            panic!(
                "Unable to obtain pull requests for user {}: {}",
                login, err
            );
        }
    }
}

pub async fn get_by_author(
    db: &DB,
    gh: &Github,
    login: &String,
) -> Result<Vec<PullRequestEntry>, GHDError> {
    if gh.should_refresh_user(&db, &login).await {
        match gh.refresh_user(&db, &login).await {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
    }

    match get_prs_from_db(&db, &login).await {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

async fn get_prs_from_db(
    db: &DB,
    login: &String,
) -> Result<Vec<PullRequestEntry>, GHDError> {
    match sqlx::query_as::<_, PullRequestEntry>(
        "SELECT * FROM pull_requests WHERE login = ?",
    )
    .bind(login)
    .fetch_all(db.pool())
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            panic!("Unable to obtain pull requests from db: {}", err);
        }
    }
}
