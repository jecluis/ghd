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

use sqlx::Row;

use crate::{
    common::{self, datetime_to_ts},
    db::DB,
    errors::GHDError,
};

use self::{
    prs::PullRequestEntry,
    types::{GithubRequest, GithubUser, GithubUserReply},
};

pub mod api;
pub mod gql;
pub mod prs;
pub mod types;

const USER_REFRESH_INTERVAL: i64 = 60;

pub struct Github {}

impl Github {
    pub fn new() -> Self {
        Github {}
    }

    pub async fn whoami(
        self: &Self,
        token: &String,
    ) -> Result<GithubUser, reqwest::StatusCode> {
        let ghreq = GithubRequest::new(token);
        let req = ghreq.get("/user");
        match ghreq.send::<GithubUserReply>(req).await {
            Ok(res) => Ok(to_user(res)),
            Err(err) => Err(err),
        }
    }

    pub async fn get_token(self: &Self, db: &DB) -> Result<String, GHDError> {
        let val: Result<sqlx::sqlite::SqliteRow, sqlx::Error> = sqlx::query(
            "
                SELECT token FROM tokens
                WHERE id = (SELECT MAX(id) FROM tokens);
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
            Err(_) => return Err(GHDError::TokenNotFoundError),
        }
    }

    pub async fn set_token<F>(
        self: &Self,
        db: &DB,
        token: &String,
        cb: F,
    ) -> Result<(), GHDError>
    where
        F: FnOnce(&GithubUser),
    {
        println!("setting token {}", token);
        println!("  obtaining user for token");
        let user: GithubUser = match self.whoami(token).await {
            Ok(res) => res,
            Err(err) => {
                return match err {
                    reqwest::StatusCode::FORBIDDEN => {
                        Err(GHDError::BadTokenError)
                    }
                    _ => Err(GHDError::UnknownError),
                };
            }
        };
        println!("  user: {}, {}", user.login, user.name);

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!("Error starting transaction to set token: {}", err);
            }
        };

        add_user_to_db(&mut tx, &user).await;

        sqlx::query(
            "INSERT OR REPLACE into tokens (token, user_id) VALUES (?, ?)",
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
        println!("  user and token have been set!");

        cb(&user);
        Ok(())
    }

    pub async fn get_user(
        self: &Self,
        db: &DB,
    ) -> Result<GithubUser, GHDError> {
        let val: GithubUser = match sqlx::query_as::<_, GithubUser>(
            "
            SELECT id, login, name, avatar_url
            FROM users
            WHERE id = (
                SELECT user_id FROM tokens
                WHERE id = (SELECT MAX(id) FROM tokens)
            )
            ",
        )
        .fetch_one(db.pool())
        .await
        {
            Ok(res) => {
                println!("has user: {}", res.login);
                res
            }
            Err(_) => {
                println!("no user found!");
                return Err(GHDError::UserNotSetError);
            }
        };

        Ok(val)
    }

    pub async fn get_user_by_login(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<GithubUser, GHDError> {
        match get_user_by_login(&db, &login).await {
            Ok(res) => return Ok(res),
            Err(_) => {}
        };

        let token: String = match self.get_token(db).await {
            Ok(t) => t.clone(),
            Err(err) => return Err(err),
        };

        let ghreq = GithubRequest::new(&token);
        let reqstr = format!("/users/{}", login);
        let req = ghreq.get(&reqstr);
        match ghreq.send::<GithubUserReply>(req).await {
            Ok(res) => return Ok(to_user(res)),
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

    pub async fn track_user<F>(
        self: &Self,
        db: &DB,
        login: &String,
        cb: F,
    ) -> Result<GithubUser, GHDError>
    where
        F: FnOnce(&GithubUser),
    {
        match get_user_by_login(&db, &login).await {
            Ok(res) => {
                println!("user {} already exists!", login);
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

        add_user_to_db(&mut tx, &user).await;

        tx.commit().await.unwrap_or_else(|err| {
            panic!("Unable to commit transaction to track new user: {}", err);
        });

        cb(&user);
        Ok(user)
    }

    pub async fn get_tracked_users(
        self: &Self,
        db: &DB,
    ) -> Result<Vec<GithubUser>, GHDError> {
        match sqlx::query_as::<_, GithubUser>(
            "
            SELECT id, login, name, avatar_url FROM users
            ",
        )
        .fetch_all(db.pool())
        .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(GHDError::UnknownError),
        }
    }

    pub async fn refresh_user(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<(), GHDError> {
        // obtain user; if DNE, return error.
        let user = match self.get_user_by_login(&db, &login).await {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        // obtain pull requests by author
        let prs = match prs::fetch_by_author(&db, &self, &login).await {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        let mut tx = match db.pool().begin().await {
            Ok(res) => res,
            Err(err) => {
                panic!("Error starting transaction to update PRs: {}", err);
            }
        };

        for pr in &prs {
            let created_at = pr.created_at;
            let updated_at = pr.updated_at;
            let merged_at = match pr.merged_at {
                Some(v) => v,
                None => -1,
            };
            let closed_at = match pr.closed_at {
                Some(v) => v,
                None => -1,
            };

            match sqlx::query(
                "
                INSERT OR REPLACE into pull_request (
                    id, title, author, created_at,
                    updated_at, closed_at, merged_at, comments
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?
                )
                ",
            )
            .bind(&pr.id)
            .bind(&pr.title)
            .bind(&pr.author)
            .bind(created_at)
            .bind(updated_at)
            .bind(closed_at)
            .bind(merged_at)
            .bind(&pr.comments)
            .execute(&mut tx)
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    panic!("Unable to update PR: {}", err);
                }
            };
        }

        let now = chrono::Utc::now().timestamp();
        match sqlx::query(
            "
            INSERT OR REPLACE into user_refresh (id, refresh_at)
            VALUES (?, ?)
            ",
        )
        .bind(&user.id)
        .bind(&now)
        .execute(&mut tx)
        .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to update user '{}' refresh: {}", login, err);
            }
        };

        tx.commit().await.unwrap_or_else(|err| {
            panic!("Unable to commit transaction to update PRs: {}", err);
        });

        Ok(())
    }

    pub async fn get_user_refresh(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<chrono::DateTime<chrono::Utc>, GHDError> {
        let user = match self.get_user_by_login(&db, &login).await {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match sqlx::query_scalar::<_, i64>(
            "SELECT refresh_at FROM user_refresh WHERE id = ?",
        )
        .bind(&user.id)
        .fetch_one(db.pool())
        .await
        {
            Ok(res) => {
                // if < 0, never refreshed; how do we convey that? Error?
                if res <= 0 {
                    return Err(GHDError::NeverRefreshedError);
                }
                Ok(common::ts_to_datetime(res).unwrap())
            }
            Err(_) => Err(GHDError::UserNotFoundError),
        }
    }

    pub async fn should_refresh_user(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> bool {
        match self.get_user_refresh(&db, &login).await {
            Ok(val) => {
                return common::has_expired(&val, USER_REFRESH_INTERVAL);
            }
            Err(GHDError::NeverRefreshedError) => {
                return true;
            }
            Err(GHDError::UserNotFoundError) => {
                println!("Unable to find user '{}' to refresh!", login);
                return false;
            }
            Err(err) => {
                panic!("Unknown error while checking refresh user: {:?}", err);
            }
        };
    }

    pub async fn get_pulls_by_author(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<Vec<PullRequestEntry>, GHDError> {
        let token = match self.get_token(&db).await {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        prs::get_by_author(&db, &self, &login).await
    }
}

fn to_user(res: GithubUserReply) -> GithubUser {
    GithubUser {
        login: res.login,
        id: res.id,
        avatar_url: res.avatar_url,
        name: res.name,
    }
}

async fn get_user_by_login(
    db: &DB,
    login: &String,
) -> Result<GithubUser, GHDError> {
    match sqlx::query_as::<_, GithubUser>(
        "
        SELECT id, login, name, avatar_url
        FROM users
        WHERE login = ?
        ",
    )
    .bind(&login)
    .fetch_one(db.pool())
    .await
    {
        Ok(res) => Ok(res),
        Err(_) => Err(GHDError::UserNotFoundError),
    }
}

async fn add_user_to_db(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user: &GithubUser,
) {
    sqlx::query(
        "
        INSERT OR REPLACE into users (id, login, name, avatar_url)
        VALUES (?, ?, ?, ?)
        ",
    )
    .bind(&user.id)
    .bind(&user.login)
    .bind(&user.name)
    .bind(&user.avatar_url)
    .execute(&mut *tx)
    .await
    .unwrap_or_else(|err| {
        panic!("Error inserting user into database: {}", err);
    });

    sqlx::query("INSERT into user_refresh (id, refresh_at) VALUES (?, -1)")
        .bind(&user.id)
        .execute(&mut *tx)
        .await
        .unwrap_or_else(|err| {
            panic!("Error inserting user into refresh table: {}", err);
        });
}
