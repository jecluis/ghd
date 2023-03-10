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

use crate::{common, db::DB, errors::GHDError};

use super::types::{Issue, PullRequest, PullRequestTableEntry};

/// Obtain all Pull Requests from the database.
///
pub async fn get_all_prs_from_db(
    db: &DB,
) -> Result<Vec<PullRequestTableEntry>, GHDError> {
    match sqlx::query_as::<_, PullRequestTableEntry>(
        "
        SELECT
            issues.*, pull_requests.is_draft, pull_requests.review_decision,
            pull_requests.merged_at
        FROM
            pull_requests LEFT JOIN issues
        ON
            pull_requests.id = issues.id
        ",
    )
    .fetch_all(db.pool())
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            panic!("Unable to obtain pull requests from db: {}", err);
        }
    }
}

/// Obtain all Pull Requests from the provided author `login`.
///
pub async fn get_prs_by_author(
    db: &DB,
    login: &String,
) -> Result<Vec<PullRequestTableEntry>, GHDError> {
    match sqlx::query_as::<_, PullRequestTableEntry>(
        "
        SELECT
            issues.*, pull_requests.is_draft, pull_requests.review_decision,
            pull_requests.merged_at
        FROM
            pull_requests LEFT JOIN issues
        ON
            pull_requests.id = issues.id
        WHERE
            issues.author = ?
        ORDER BY issues.updated_at DESC
        ",
    )
    .bind(&login)
    .fetch_all(db.pool())
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            panic!("Unable to obtain pull requests from db: {}", err);
        }
    }
}

/// Obtain all Pull Requests the provided user `login` is involved with. This
/// means mentions, review requests, authored, or where the user may have
/// commented.
///
pub async fn get_involved_prs(
    db: &DB,
    login: &String,
) -> Result<Vec<PullRequestTableEntry>, GHDError> {
    match sqlx::query_as::<_, PullRequestTableEntry>(
        "
        SELECT
            issues.*, pull_requests.is_draft, pull_requests.merged_at,
            pull_requests.review_decision
        FROM pull_requests INNER JOIN (
            SELECT
                issues.*
            FROM
                issues LEFT JOIN user_issues
            ON
                issues.id = user_issues.issue_id
            WHERE
                user_issues.user_id = (
                    SELECT id FROM users WHERE login = ?
                )
        ) AS
            issues
        ON
            pull_requests.id = issues.id AND issues.author != ?
        ORDER BY issues.updated_at DESC
        ",
    )
    .bind(&login)
    .bind(&login)
    .fetch_all(db.pool())
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            panic!("Unable to obtain data from database: {}", err);
        }
    }
}

/// Insert the given issue into the database.
///
async fn consume_issue(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    issue: &Issue,
) -> Result<(), GHDError> {
    match sqlx::query(
        "
        INSERT OR REPLACE INTO issues (
            id, number, title, author, author_id,
            url, repo_owner, repo_name, state,
            created_at, updated_at, closed_at,
            is_pull_request,
            last_viewed
        ) VALUES (
            ?, ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?,
            ?,
            ?
        )
        ",
    )
    .bind(&issue.id)
    .bind(&issue.number)
    .bind(&issue.title)
    .bind(&issue.author)
    .bind(&issue.author_id)
    .bind(&issue.url)
    .bind(&issue.repo_owner)
    .bind(&issue.repo_name)
    .bind(&issue.state)
    .bind(&issue.created_at.timestamp())
    .bind(&issue.updated_at.timestamp())
    .bind(common::dt_opt_to_ts(&issue.closed_at))
    .bind(&issue.is_pull_request)
    .bind(common::dt_opt_to_ts(&issue.last_viewed))
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            panic!("Unable to consume issue: {}", err);
        }
    };
    Ok(())
}

/// Consume all issues and Pull Requests provided as arguments, writing them to
/// the database, associating them with the provided `userid`.
///
/// # Arguments
///
/// * `tx` - A transaction handle.
/// * `userid` - The user ID to associate the issues and Pull Requests with.
/// * `issues` - A Vector of Issues associated with the provided user.
/// * `prs` - A Vector of Pull Requests associated with the provided user.
///
pub async fn consume_issues(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    userid: &i64,
    issues: &Vec<Issue>,
    prs: &Vec<PullRequest>,
) -> Result<(), GHDError> {
    println!("consuming {} issues, {} prs", issues.len(), prs.len());

    let mut issue_ids: Vec<i64> = vec![];

    for entry in issues {
        match consume_issue(tx, &entry).await {
            Ok(_) => {}
            Err(err) => {
                panic!("unexpected error: {:?}", err);
            }
        };
        issue_ids.push(entry.id);
    }

    for entry in prs {
        match consume_issue(tx, &entry.issue).await {
            Ok(_) => {}
            Err(err) => {
                panic!("unexpected error: {:?}", err);
            }
        };

        // consume pull request
        match sqlx::query(
            "
            INSERT OR REPLACE INTO pull_requests (
                id, is_draft, review_decision, merged_at
            ) VALUES (
                ?, ?, ?, ?
            )
            ",
        )
        .bind(&entry.issue.id)
        .bind(&entry.is_draft)
        .bind(&entry.review_decision)
        .bind(common::dt_opt_to_ts(&entry.merged_at))
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!("unable to consume pull request: {}", err);
            }
        };
        issue_ids.push(entry.issue.id);
    }

    for id in &issue_ids {
        match sqlx::query(
            "
            INSERT OR REPLACE INTO user_issues (
                user_id, issue_id
            ) VALUES (
                ?, ?
            )
            ",
        )
        .bind(&userid)
        .bind(id)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!(
                    "unable to track issue {} for user {}: {}",
                    id, userid, err
                );
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

    match sqlx::query("UPDATE issues SET last_viewed = ? WHERE id = ?")
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
