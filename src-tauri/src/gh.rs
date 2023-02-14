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

use crate::{db::DB, errors::GHDError};

use self::{
    prs::PullRequestEntry,
    types::{GithubRequest, GithubUser, GithubUserReply},
};

pub mod prs;
pub mod types;

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
        .execute(&mut tx)
        .await
        .unwrap_or_else(|err| {
            panic!("Error inserting user into database: {}", err);
        });

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

    pub async fn track_user(
        self: &Self,
        db: &DB,
        login: &String,
    ) -> Result<GithubUser, GHDError> {
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

        sqlx::query(
            "
            INSERT INTO users (id, login, name, avatar_url)
            VALUES (?, ?, ?, ?)
            ",
        )
        .bind(&user.id)
        .bind(&user.login)
        .bind(&user.name)
        .bind(&user.avatar_url)
        .execute(db.pool())
        .await
        .unwrap_or_else(|err| {
            panic!("Unable to commit new user entry to database: {}", err);
        });

        Ok(user)
    }

    pub async fn get_pulls(
        self: &Self,
        token: &String,
    ) -> Result<Vec<PullRequestEntry>, reqwest::StatusCode> {
        let user = String::from("jecluis");
        prs::get(token, &user).await
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
