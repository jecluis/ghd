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

use sqlx::{sqlite::SqliteQueryResult, SqlitePool};

pub struct DBPR {
    pool: Option<SqlitePool>,
}

impl Default for DBPR {
    fn default() -> Self {
        DBPR { pool: None }
    }
}

impl DBPR {
    pub async fn setup(self: &Self, uri: &str) {
        match create_db_schema(uri).await {
            Ok(_) => println!("pull requests setup successfully."),
            Err(err) => panic!("error setting pull requests table: {}", err),
        };
    }
}

async fn create_db_schema(uri: &str) -> Result<SqliteQueryResult, sqlx::Error> {
    let pool = SqlitePool::connect(uri).await?;
    let query = "
    CREATE TABLE IF NOT EXISTS pull_request (
        id          INTEGER PRIMARY KEY,
        title       TEXT NOT NULL,
        author      TEXT NOT NULL,
        created_at  INTEGER,
        updated_at  INTEGER,
        closed_at   INTEGER,
        merged_at   INTEGER,
        comments    INTEGER
    );
    ";

    let result = sqlx::query(&query).execute(&pool).await;
    pool.close().await;

    result
}
