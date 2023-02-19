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

#[derive(serde::Deserialize)]
pub struct GithubAPIUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
}

#[derive(serde::Deserialize)]
pub struct GithubAPILabel {
    pub id: i64,
    pub node_id: String,
    pub url: String,
    pub name: String,
    pub color: String,
}

#[derive(serde::Deserialize)]
pub struct GithubAPIMilestone {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
}

#[derive(serde::Deserialize)]
pub struct GithubAPIPullRequestDesc {
    pub url: String,
    pub merged_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(serde::Deserialize)]
pub struct PullRequestSearchAPIEntry {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub node_id: String,
    pub url: String,
    pub html_url: String,
    pub user: GithubAPIUser,
    pub labels: Vec<GithubAPILabel>,
    pub state: String,
    pub assignees: Option<Vec<GithubAPIUser>>,
    pub milestone: Option<GithubAPIMilestone>,
    pub comments: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub draft: bool,
    pub pull_request: GithubAPIPullRequestDesc,
}
