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

use log::info;

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
            AND
            issues.archived_at IS NULL
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
                AND issues.archived_at IS NULL
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
    info!("consuming {} issues, {} prs", issues.len(), prs.len());

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

/// Marks a specific Pull Request as having been viewed.
///
/// This is a helper function that performs a single update within a
/// transaction.
///
/// # Arguments
///
/// * `tx` - The transaction to perform the update as part of.
/// * `prid` - The Pull Request's database ID.
///
async fn _mark_viewed(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    prid: &i64,
) -> Result<(), GHDError> {
    let now = chrono::Utc::now().timestamp();

    match sqlx::query("UPDATE issues SET last_viewed = ? WHERE id = ?")
        .bind(&now)
        .bind(&prid)
        .execute(&mut *tx)
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

/// Marks a specified Pull Request as having been viewed.
///
/// # Arguments
///
/// * `db` - A GHD Database handle.
/// * `prid` - The Pull Request's database ID.
///
pub async fn mark_viewed(db: &DB, prid: &i64) -> Result<(), GHDError> {
    let mut tx = match db.pool().begin().await {
        Ok(res) => res,
        Err(err) => {
            panic!("Error starting transaction to mark PR viewed: {}", err);
        }
    };

    match _mark_viewed(&mut tx, prid).await {
        Ok(_) => {}
        Err(err) => return Err(err),
    };

    tx.commit().await.unwrap_or_else(|err| {
        panic!("Unable to commit transaction to mark PR viewed: {}", err);
    });
    Ok(())
}

/// Mark multiple Pull Requests as having been viewed.
///
/// # Arguments:
///
/// * `db` - A GHD Database handle.
/// * `prs` - A Vector containing one or more Pull Request database IDs.
///
pub async fn mark_viewed_many(db: &DB, prs: &Vec<i64>) -> Result<(), GHDError> {
    let mut tx = db.pool().begin().await.unwrap_or_else(|err| {
        panic!(
            "Error starting transaction to mark multiple PRs as viewed: {}",
            err
        );
    });

    for prid in prs {
        match _mark_viewed(&mut tx, prid).await {
            Ok(_) => {}
            Err(err) => {
                tx.rollback().await.unwrap_or_else(|err| {
                    panic!("Unable to rollback broken transaction: {}", err);
                });
                return Err(err);
            }
        };
    }

    tx.commit().await.unwrap_or_else(|err| {
        panic!("Unable to commit transaction to mark PRs viewed: {}", err);
    });

    Ok(())
}

/// Mark a specified Issue as having been archived.
///
pub async fn archive_issue(db: &DB, issue_id: &i64) -> Result<(), GHDError> {
    let now = chrono::Utc::now().timestamp();

    match sqlx::query("UPDATE issues SET archived_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&issue_id)
        .execute(db.pool())
        .await
    {
        Ok(_) => {}
        Err(sqlx::Error::RowNotFound) => {
            return Err(GHDError::NotFoundError);
        }
        Err(err) => {
            panic!("Unexpected error archiving issue '{}': {}", issue_id, err);
        }
    };

    Ok(())
}
