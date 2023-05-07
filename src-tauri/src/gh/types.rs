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

/// Describes a user, as it is kept in the database.
///
#[derive(sqlx::FromRow, serde::Serialize, Clone)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub avatar_url: String,
}

#[derive(sqlx::FromRow)]
pub struct IssueTableEntry {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub author_id: i64,
    pub url: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
    pub is_pull_request: bool,
    pub last_viewed: Option<i64>,
}

/// A Pull Request Table Entry includes all columns in the `IssueTableEntry`
/// struct, because it always must be the result of a `JOIN` between the
/// `issues` table and the `pull_requests` table.
///
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PullRequestTableEntry {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub author_id: i64,
    pub url: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
    pub is_pull_request: bool,
    pub last_viewed: Option<i64>,
    pub is_draft: bool,
    pub review_decision: String,
    pub merged_at: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub struct UserIssuesTableEntry {
    pub user_id: i64,
    pub issue_id: i64,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Issue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub author_id: i64,
    pub url: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub state: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_pull_request: bool,
    pub last_viewed: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PullRequest {
    pub issue: Issue,
    pub is_draft: bool,
    pub review_decision: String,
    pub merged_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct UserUpdate {
    pub when: chrono::DateTime<chrono::Utc>,
    pub issues: Vec<Issue>,
    pub prs: Vec<PullRequest>,
}

/// Represents a Pull Request's detailed information.
///
#[derive(serde::Serialize)]
pub struct PullRequestInfo {
    pub number: i64,
    pub title: String,
    pub body_html: String,
    pub author: GithubUser,
    pub repo_owner: String,
    pub repo_name: String,
    pub url: String,
    pub state: String,
    pub is_draft: bool,
    pub milestone: Option<Milestone>,
    pub labels: Vec<Label>,
    pub total_comments: i64,
    pub participants: Vec<GithubUser>,
    pub reviews: Vec<UserReview>,
}

/// Represents a milestone.
///
#[derive(serde::Serialize)]
pub struct Milestone {
    pub title: String,
    pub state: String,
    pub due_on: Option<chrono::DateTime<chrono::Utc>>,
    pub due_on_ts: Option<i64>,
}

/// Represents a label.
///
#[derive(serde::Serialize)]
pub struct Label {
    pub color: String,
    pub name: String,
}

/// Represents a User review
///
#[derive(serde::Serialize)]
pub struct UserReview {
    pub author: GithubUser,
    pub state: String,
}
