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

use crate::{db::DB, errors::GHDError};

use super::{rest, types::GithubUser};

/// Find out who I am, based on the provided API token. Returns a
/// `GithubUser` struct with the user's information.
///
/// # Arguments
///
/// * `token` - String containing an API Token.
///
pub async fn whoami(token: &String) -> Result<GithubUser, reqwest::StatusCode> {
    let ghreq = rest::GithubRequest::new(token);
    let req = ghreq.get("/user");
    match ghreq.send::<rest::GithubUserReply>(req).await {
        Ok(res) => Ok(user_reply_to_user(res)),
        Err(err) => Err(err),
    }
}

/// Returns a user from the database, if it exists.
///
/// # Arguments
///
/// * `db` - The GHD Database handle.
/// * `login` - A String representing the user login.
///
pub async fn get_user_by_login(
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

/// Checks whether a given user exists in the database.
///
/// # Arguments
///
/// * `db` - The GHD Database handle.
/// * `login` - A String representing the user login.
///
pub async fn user_exists(db: &DB, login: &String) -> bool {
    match get_user_by_login(&db, &login).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Add a user to the GHD database. This function requires a transaction.
/// Presumes the user does not exist in the database.
///
/// # Arguments
///
/// * `tx` - The sqlx transaction to piggy-back on.
/// * `user` - The user being added.
///
pub async fn add_user_to_db(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user: &GithubUser,
) {
    sqlx::query(
        "
        INSERT into users (id, login, name, avatar_url)
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

pub async fn update_user_refresh(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    userid: &i64,
    when: &chrono::DateTime<chrono::Utc>,
) {
    let now = when.timestamp();
    sqlx::query("UPDATE user_refresh SET refresh_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&userid)
        .execute(&mut *tx)
        .await
        .unwrap_or_else(|err| {
            panic!("Error updating user {} refresh table: {}", userid, err);
        });
}

/// Obtain GHD's main user.
///
/// # Arguments
///
/// * `db` - The GHD Database handle.
///
pub async fn get_main_user(db: &DB) -> Result<GithubUser, GHDError> {
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
            debug!("has user: {}", res.login);
            res
        }
        Err(_) => {
            warn!("no user found!");
            return Err(GHDError::UserNotSetError);
        }
    };

    Ok(val)
}

/// Obtain a Vector of all tracked users.
///
/// * `db` - The GHD Database handle.
///
pub async fn get_tracked_users(db: &DB) -> Result<Vec<GithubUser>, GHDError> {
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

/// Translated a REST `GithubUserReply` to a `GithubUser`.
///
pub fn user_reply_to_user(res: rest::GithubUserReply) -> GithubUser {
    GithubUser {
        login: res.login,
        id: res.id,
        avatar_url: res.avatar_url,
        name: res.name,
    }
}
