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

use super::{api, rest::GithubRequest, types::PullRequestEntry, Github};

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
    if super::refresh::should_refresh_user(&db, &login).await {}

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
        "SELECT * FROM pull_request WHERE author = ?",
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

/// Inserts into the database the Pull Requests provided in the `prs` Vector.
/// This operation is performed as part of a larger transaction, for which a
/// handle should be provided by the caller.
///
/// # Arguments
///
/// * `tx` - A database transaction handle.
/// * `prs` - A Vector containing the Pull Requests to be added to the database.
///
pub async fn consume_prs(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    prs: &Vec<PullRequestEntry>,
) -> Result<(), GHDError> {
    println!("consuming {} pull requests", prs.len());
    for pr in prs {
        match sqlx::query(
            "
            INSERT OR REPLACE INTO pull_request (
                id, number, title, author, author_id, url, html_url,
                repo_owner, repo_name, state, is_draft, milestone,
                created_at, updated_at, closed_at, merged_at, 
                comments, last_viewed
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?
            )
            ",
        )
        .bind(&pr.id)
        .bind(&pr.number)
        .bind(&pr.title)
        .bind(&pr.author)
        .bind(&pr.author_id)
        .bind(&pr.url)
        .bind(&pr.html_url)
        .bind(&pr.repo_owner)
        .bind(&pr.repo_name)
        .bind(&pr.state)
        .bind(&pr.is_draft)
        .bind(&pr.milestone)
        .bind(&pr.created_at)
        .bind(&pr.updated_at)
        .bind(&pr.closed_at)
        .bind(&pr.merged_at)
        .bind(&pr.comments)
        .bind(&pr.last_viewed)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to consume pull request: {}", err);
            }
        };
    }

    Ok(())
}

/// Marks a specified Pull Request as having been viewed.
///
/// # Arguments
///
/// * `db` - A GHD Database handle.
/// * `prid` - The Pull Request's database ID.
///
pub async fn mark_viewed(db: &DB, prid: &i64) -> Result<(), GHDError> {
    let now = chrono::Utc::now().timestamp();

    match sqlx::query("UPDATE pull_request SET last_viewed = ? WHERE id = ?")
        .bind(&now)
        .bind(&prid)
        .execute(db.pool())
        .await
    {
        Ok(_) => {}
        Err(sqlx::Error::RowNotFound) => {
            return Err(GHDError::NotFoundError);
        }
        Err(err) => {
            panic!("Unexpected error marking pr '{}' viewed: {}", prid, err);
        }
    };

    Ok(())
}
