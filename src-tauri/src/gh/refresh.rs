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

use super::users;

const USER_REFRESH_INTERVAL: i64 = 60;

/// Obtain `chrono::DateTime` from when the user was last refreshed.
///
/// # Arguments
///
/// * `db` - The GHD Database handle.
/// * `userid` - The user's database ID.
///
pub async fn get_user_refresh(
    db: &DB,
    userid: &i64,
) -> Result<chrono::DateTime<chrono::Utc>, GHDError> {
    match sqlx::query_scalar::<_, i64>(
        "SELECT refresh_at FROM user_refresh WHERE id = ?",
    )
    .bind(&userid)
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

/// Check whether we should refresh a given user, by login.
///
/// # Arguments
///
/// * `db` - The GHD Database handle.
/// * `login` - A String representing the user login.
///
pub async fn should_refresh_user(db: &DB, login: &String) -> bool {
    let userid: i64 = match users::get_user_by_login(&db, &login).await {
        Err(GHDError::UserNotFoundError) => {
            return false;
        }
        Err(err) => {
            panic!("unexpected error: {:?}!", err);
        }
        Ok(user) => user.id,
    };

    match get_user_refresh(&db, &userid).await {
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
