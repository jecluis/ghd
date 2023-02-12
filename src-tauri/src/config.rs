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

use std::collections::HashMap;

use sqlx::Row;

use crate::db::DB;

#[derive(Debug)]
pub enum ConfigError {
    SettingNotFoundError,
    TokenNotFoundError,
}

#[derive(sqlx::FromRow)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

impl Config {
    pub async fn set_api_token(
        self: &Self,
        db: &DB,
        token: &String,
    ) -> Result<(), String> {
        sqlx::query("INSERT OR REPLACE into settings VALUES ('api_token', ?)")
            .bind(token)
            .execute(db.pool())
            .await
            .unwrap_or_else(|err| {
                panic!("Error inserting into database: {}", err);
            });

        Ok(())
    }

    pub async fn get_api_token(
        self: &Self,
        db: &DB,
    ) -> Result<String, ConfigError> {
        let val: Result<sqlx::sqlite::SqliteRow, sqlx::Error> =
            sqlx::query("SELECT value FROM settings WHERE key='api_token'")
                .fetch_one(db.pool())
                .await;
        match &val {
            Ok(res) => {
                match res.try_get("value") {
                    Ok(res) => return Ok(res),
                    Err(err) => {
                        panic!("Unable to obtain token column: {}", err);
                    }
                };
            }
            Err(_) => return Err(ConfigError::TokenNotFoundError),
        }
    }
}
