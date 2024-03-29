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

use log::{debug, warn};
use sqlx::Row;

use crate::{db::DB, errors::GHDError};

use self::types::{GithubUser, PullRequestInfo, PullRequestTableEntry};

pub mod api;
pub mod gql;
pub mod prs;
pub mod refresh;
pub mod rest;
pub mod types;
pub mod users;

pub struct Github {}

impl Github {
    /// Obtain new Github instance.
    ///
    pub fn new() -> Self {
        Github {}
    }

    /// Obtain token from the database, if exists. Returns a String if the token
    /// exists, or a `GHDError::TokenNotFoundError` otherwise.
    ///
    /// # Arguments
    ///
    /// * `db` - The GHD Database handle.
    ///
    pub async fn get_token(self: &Self, db: &DB) -> Result<String, GHDError> {
        let val: Result<sqlx::sqlite::SqliteRow, sqlx::Error> = sqlx::query(
            "
                SELECT token FROM tokens
                WHERE id = (SELECT MAX(id) FROM tokens WHERE invalid = False);
            ",
        )
        .fetch_one(db.pool())
        .await;

        match &val {
            Ok(res) => {
                match res.try_get("token") {
                    Ok(res) => return Ok(res),
                    Err(err) => {
                        panic!("Unable to obtain token column: {}", err);
                    }
                };
            }
            Err(_) => {}
        };

        // nothing was found, was it because we don't have a token, or because
        // the one we have is invalid?
        match sqlx::query(
            "
            SELECT token FROM tokens
            WHERE id = (SELECT MAX(id) FROM tokens);
            ",
        )
        .fetch_one(db.pool())
        .await
        {
            Ok(_) => {
                // there is at least one token, so they must be invalid.
                return Err(GHDError::BadTokenError);
            }
            Err(_) => {
                // no tokens available
                return Err(GHDError::TokenNotFoundError);
            }
        };
    }

    /// Set the API Token to be used by GHD. Expects a callback function as
    /// argument, which will be called once the token is properly persisted on
    /// disk.
    ///
    /// # Arguments
    ///
    /// * `db` - The GHD Database handle.
    /// * `token` - String containing the API Token to persist.
    /// * `cb` - Callback function to be called once the Token is persisted.
    ///
    pub async fn set_token<F>(
        self: &Self,
        db: &DB,
        token: &String,
        cb: F,
    ) -> Result<(), GHDError>
    where
        F: FnOnce(&GithubUser),
    {
        debug!("setting token {}", token);
        debug!("  obtaining user for token");
        let user: GithubUser = match users::whoami(token).await {
            Ok(res) => res,
            Err(err) => {
                return match err {
                    reqwest::StatusCode::FORBIDDEN => {
                        Err(GHDError::BadTokenError)
                    }
                    reqwest::StatusCode::UNAUTHORIZED => {
                        Err(GHDError::BadTokenError)
                    }
                    _ => Err(GHDError::UnknownError),
                };
            }
        };
        debug!("  user: {}, {}", user.login, user.name);

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!("Error starting transaction to set token: {}", err);
            }
        };

        let user_exists = users::user_exists(&db, &user.login).await;
        if !user_exists {
            users::add_user_to_db(&mut tx, &user).await;
        }

        sqlx::query(
            "
                INSERT OR REPLACE into tokens (token, user_id, invalid)
                VALUES (?, ?, False)
            ",
        )
        .bind(token)
        .bind(&user.id)
        .execute(&mut tx)
        .await
        .unwrap_or_else(|err| {
            panic!("Error inserting token into database: {}", err);
        });

        tx.commit().await.unwrap_or_else(|err| {
            panic!("Unable to commit transaction to set token: {}", err);
        });
        debug!("  user and token have been set!");

        if !user_exists {
            self.populate_user(&db, &user.login).await.unwrap();
        }

        cb(&user);
        Ok(())
    }

    pub async fn invalidate_token(self: &Self, db: &DB) {
        let query = "
            UPDATE tokens SET invalid = True
            WHERE id = (SELECT MAX(id) FROM tokens WHERE invalid = False)
        ";

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!(
                    "Error starting transaction to invalidate token: {}",
                    err
                );
            }
        };
        sqlx::query(query)
            .execute(&mut tx)
            .await
            .unwrap_or_else(|err| {
                panic!("Error updating token validity: {}", err);
            });
        tx.commit().await.unwrap_or_else(|err| {
            panic!("Unable to commit token invalidation transaction: {}", err);
        });
    }

    /// Obtain user by their login. First tries the database, and will fallback
    /// to a REST call if the user cannot be found in the database.
    ///
    /// # Arguments
    ///
    /// * `db` - The GHD Database handle.
    /// * `login` - String containing the login to obtain.
    ///
    pub async fn get_user_by_login(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<GithubUser, GHDError> {
        match users::get_user_by_login(&db, &login).await {
            Ok(res) => return Ok(res),
            Err(_) => {}
        };

        let token: String = match self.get_token(db).await {
            Ok(t) => t.clone(),
            Err(err) => return Err(err),
        };

        let ghreq = rest::GithubRequest::new(&token);
        let reqstr = format!("/users/{}", login);
        let req = ghreq.get(&reqstr);
        match ghreq.send::<rest::GithubUserReply>(req).await {
            Ok(res) => return Ok(users::user_reply_to_user(res)),
            Err(err) => {
                return match err {
                    reqwest::StatusCode::NOT_FOUND => {
                        Err(GHDError::UserNotFoundError)
                    }
                    _ => Err(GHDError::UnknownError),
                };
            }
        }
    }

    /// Track the specified user by their login. Will first check the database
    /// to ascertain whether the user is already being tracked; if so, return
    /// the existing user. Otherwise, will obtain the user via a REST call. If
    /// the user is ultimately added to the database, will callback the provided
    /// function once the data is persisted.
    ///
    /// # Arguments
    ///
    /// * `db` - The GHD Database handle.
    /// * `login` - String containing the user login to be tracked.
    /// * `cb` - Callback function that will be called if a new user is added
    ///   and the data has been persisted.
    ///
    pub async fn track_user<F>(
        self: &Self,
        db: &DB,
        login: &String,
        cb: F,
    ) -> Result<GithubUser, GHDError>
    where
        F: FnOnce(&GithubUser),
    {
        match users::get_user_by_login(&db, &login).await {
            Ok(res) => {
                debug!("user {} already exists!", login);
                return Ok(res);
            }
            Err(_) => {}
        };

        let user = match self.get_user_by_login(&db, &login).await {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!("Error starting transaction to track user: {}", err);
            }
        };

        users::add_user_to_db(&mut tx, &user).await;

        tx.commit().await.unwrap_or_else(|err| {
            panic!("Unable to commit transaction to track new user: {}", err);
        });

        self.populate_user(&db, &user.login).await.unwrap();

        cb(&user);
        Ok(user)
    }

    /// Populate the database for a newly-added user.
    ///
    /// # Arguments
    ///
    /// * `db` - The GHD Database handle.
    /// * `login` - String containing the login of the user to populate.
    ///
    pub async fn populate_user(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<(), GHDError> {
        // sanity checks: user exists in the database, and last update was
        // never.
        let user = match users::get_user_by_login(&db, &login).await {
            Ok(u) => u,
            Err(GHDError::UserNotFoundError) => {
                panic!(
                    "Trying to populate a user that is not in the database: {}",
                    login
                );
            }
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        };
        match refresh::get_user_refresh(&db, &user.id).await {
            Ok(_) => {
                panic!("User has been previously updated: {}", login);
            }
            Err(GHDError::NeverRefreshedError) => {}
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        };

        // obtain user information through GraphQL API
        let token: String = match self.get_token(&db).await {
            Ok(t) => t.clone(),
            Err(_) => {
                panic!("Token not set!");
            }
        };

        // let res = match gql::get_user_info(&token, &login).await {
        let res = match gql::get_user_open_issues(&token, &login).await {
            Ok(info) => info,
            Err(err) => {
                panic!("Unexpected error populating user from GQL: {:?}", err);
            }
        };

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!("Error starting transaction to populate user: {}", err);
            }
        };

        if let Err(err) =
            prs::consume_issues(&mut tx, &user.id, &res.issues, &res.prs).await
        {
            panic!(
                "Error consuming pull requests when populating user: {:?}",
                err
            );
        };

        users::update_user_refresh(&mut tx, &user.id, &res.when).await;

        tx.commit().await.unwrap_or_else(|err| {
            panic!(
                "Unable to commit populate transaction for user '{}': {}",
                user.login, err
            );
        });

        Ok(())
    }

    /// Refreshes the specified user's data.
    ///
    /// # Arguments
    ///
    /// * `db` - A GHD Database handle.
    /// * `login` - String containing the login of the user to be refreshed.
    ///
    pub async fn refresh_user(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<bool, GHDError> {
        let user = match users::get_user_by_login(&db, &login).await {
            Ok(u) => u,
            Err(GHDError::UserNotFoundError) => {
                panic!(
                    "Trying to refresh a user that is not on the database: {}",
                    login
                );
            }
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        };

        let token = match self.get_token(&db).await {
            Ok(t) => t.clone(),
            Err(_) => {
                panic!("Token not set!");
            }
        };

        let last_update = match refresh::get_user_refresh(&db, &user.id).await {
            Ok(v) => v,
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        };

        let res =
            match gql::get_user_updates(&token, &login, &last_update).await {
                Ok(updates) => updates,
                Err(GHDError::BadTokenError) => {
                    warn!("Token invalid or expired!");
                    return Err(GHDError::BadTokenError);
                }
                Err(err) => {
                    panic!(
                    "Unexpected error obtaining user updates from GQL: {:?}",
                    err
                );
                }
            };

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!("Error starting transaction to update user: {}", err);
            }
        };

        let mut ret = true;
        if res.prs.is_empty() && res.issues.is_empty() {
            debug!("nothing to update for user '{}'.", user.login);
            ret = false;
        }

        if let Err(err) =
            prs::consume_issues(&mut tx, &user.id, &res.issues, &res.prs).await
        {
            panic!(
                "Error updating pull requests for user '{}': {:?}",
                login, err
            );
        }
        users::update_user_refresh(&mut tx, &user.id, &res.when).await;

        tx.commit().await.unwrap_or_else(|err| {
            panic!(
                "Unable to commit update transaction for user '{}': {}",
                user.login, err
            );
        });

        Ok(ret)
    }

    /// Obtain all Pull Requests from the provided author `login`.
    ///
    pub async fn get_pulls_by_author(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<Vec<PullRequestTableEntry>, GHDError> {
        prs::get_prs_by_author(&db, &login).await
    }

    /// Obtain all Pull Requests the provided `login` is involved with, except
    /// those that have been authored by `login`.
    ///
    pub async fn get_involved_pulls(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<Vec<PullRequestTableEntry>, GHDError> {
        prs::get_involved_prs(&db, &login).await
    }

    /// Obtain a specific Pull Request's information.
    ///
    /// # Arguments
    ///
    /// * `db` - A GHD Database handle.
    /// * `prid` - A Pull Request database ID.
    ///
    pub async fn get_pull_request_info(
        self: &Self,
        db: &DB,
        prid: &i64,
    ) -> Result<PullRequestInfo, GHDError> {
        let entry = match prs::get_issue_by_id(&db, &prid).await {
            Err(err) => return Err(err),
            Ok(res) => res,
        };

        let token = match self.get_token(&db).await {
            Ok(t) => t.clone(),
            Err(err) => return Err(err),
        };

        let res = match gql::get_pull_request_info(
            &token,
            &entry.repo_owner,
            &entry.repo_name,
            &entry.number,
        )
        .await
        {
            Ok(r) => r,
            Err(err) => return Err(err),
        };

        Ok(res)
    }

    /// Marks a specified Pull Request as having been viewed.
    ///
    /// # Arguments
    ///
    /// * `db` - A GHD Database handle.
    /// * `prid` - The Pull Request's database ID.
    ///
    pub async fn mark_pull_request_viewed(
        self: &Self,
        db: &DB,
        prid: &i64,
    ) -> Result<(), GHDError> {
        prs::mark_viewed(&db, &prid).await
    }

    /// Marks multiple Pull Requests as having been viewed.
    ///
    /// # Arguments
    ///
    /// * `db` - A GHD Database handle.
    /// * `prs` - A Vector of Pull Request database IDs.
    ///
    pub async fn mark_pull_request_viewed_many(
        self: &Self,
        db: &DB,
        prs: &Vec<i64>,
    ) -> Result<(), GHDError> {
        prs::mark_viewed_many(&db, &prs).await
    }

    /// Archive the provided issue with ID `issue_id`.
    ///
    /// # Arguments
    ///
    /// * `db` - A GHD Database handle.
    /// * `issue_id` - The Issue's database ID.
    ///
    pub async fn archive_issue(
        self: &Self,
        db: &DB,
        issue_id: &i64,
    ) -> Result<(), GHDError> {
        prs::archive_issue(&db, &issue_id).await
    }

    /// Archive multiple issues.
    ///
    /// # Arguments
    ///
    /// * `db` - A GHD Database handle.
    /// * `issues` - A Vector of Issue database IDs.
    ///
    pub async fn archive_issue_many(
        self: &Self,
        db: &DB,
        issues: &Vec<i64>,
    ) -> Result<(), GHDError> {
        prs::archive_issue_many(&db, &issues).await
    }
}
