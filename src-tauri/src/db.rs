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

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, SqlitePool};

pub mod pr;

pub struct DB {
    pub uri: String,
    pub pool: Option<SqlitePool>,
    pub prs: pr::DBPR,
}

impl DB {
    pub fn new(path: &std::path::PathBuf) -> DB {
        let uri = format!("sqlite://{}", path.display());

        DB {
            uri,
            pool: None,
            prs: pr::DBPR::default(),
        }
    }

    pub async fn connect(self: &mut Self) {
        if let Some(_) = self.pool {
            panic!("Attempting to connect to connected database!");
        }

        self.pool =
            Some(SqlitePool::connect(&self.uri).await.unwrap_or_else(|_| {
                panic!("Unable to open database!");
            }));
    }

    pub async fn setup(self: Self) -> Self {
        if !sqlx::Sqlite::database_exists(&self.uri)
            .await
            .unwrap_or(false)
        {
            sqlx::Sqlite::create_database(&self.uri).await.unwrap();
            match create_db_schema(&self.uri).await {
                Ok(_) => println!("Database created successfully."),
                Err(err) => panic!("{}", err),
            };
        }

        self.prs.setup(&self.uri).await;

        self
    }

    pub fn pool(self: &Self) -> &SqlitePool {
        match &self.pool {
            Some(pool) => pool,
            None => {
                panic!("Attempting to obtain pool for unconnected database!");
            }
        }
    }
}

async fn create_db_schema(uri: &str) -> Result<SqliteQueryResult, sqlx::Error> {
    let pool = SqlitePool::connect(uri).await?;
    let query = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS settings (
        key         TEXT PRIMARY KEY NOT NULL,
        value       TEXT NOT NULL
    );
    ";

    let result = sqlx::query(&query).execute(&pool).await;
    pool.close().await;

    result
}
