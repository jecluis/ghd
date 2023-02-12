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

use crate::db::DB;

#[derive(sqlx::FromRow)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

pub struct Config {
    pub api_token: Option<String>,
    pub settings: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_token: None,
            settings: HashMap::default(),
        }
    }
}

impl Config {
    pub async fn init(self: &mut Self, db: &DB) {
        let stream = sqlx::query_as::<_, ConfigEntry>("SELECT * FROM settings")
            .fetch_all(db.pool())
            .await
            .unwrap_or_else(|err| {
                panic!("Unable to obtain settings: {}", err);
            });

        for entry in &stream {
            println!("{} = {}", entry.key, entry.value);
            self.settings.insert(entry.key.clone(), entry.value.clone());

            if entry.key == "api_token" {
                self.api_token = Some(entry.value.clone());
            }
        }
    }

    pub async fn set_api_token(
        self: &mut Self,
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

        self.api_token = Some(token.clone());
        self.settings
            .insert(String::from("api_token"), token.clone());

        Ok(())
    }
}

unsafe impl Send for Config {}
