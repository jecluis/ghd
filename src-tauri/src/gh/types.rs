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

// Users

use super::api;

/// Describes a user, as it is kept in the database.
///
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub avatar_url: String,
}

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct PullRequestEntry {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub author_id: i64,
    pub url: String,
    pub html_url: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub state: String,
    pub is_draft: bool,
    pub milestone: Option<String>,
    pub comments: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
    pub merged_at: Option<i64>,
    pub last_viewed: Option<i64>,
}

impl PullRequestEntry {
    pub fn from_api_entry(entry: &api::PullRequestSearchAPIEntry) -> Self {
        PullRequestEntry {
            id: entry.id,
            author: entry.user.login.clone(),
            author_id: entry.user.id,
            url: entry.url.clone(),
            html_url: entry.html_url.clone(),
            number: entry.number,
            title: entry.title.clone(),
            repo_owner: String::new(),
            repo_name: String::new(),
            state: entry.state.clone(),
            is_draft: entry.draft,
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
            last_viewed: None,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct IssueEntry {
    pub id: i64,
    pub title: String,
    pub number: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub participants: i64,
    pub assignees: Vec<String>,
}

pub struct GithubUserInfo {
    pub user: GithubUser,
    pub prs: Vec<PullRequestEntry>,
    pub issues: Vec<IssueEntry>,
}
