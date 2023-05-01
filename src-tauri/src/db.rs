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

use log::{debug, info};
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, SqlitePool};

use crate::errors::GHDError;

/// Current DB Schema version
///
// version 2: add 'invalid' token table column
//
const GHD_DB_VERSION: u32 = 2;

pub struct DB {
    pub uri: String,
    pub pool: Option<SqlitePool>,
}

impl DB {
    pub fn new(path: &std::path::PathBuf) -> DB {
        let uri = format!("sqlite://{}", path.display());

        DB { uri, pool: None }
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
                Ok(_) => info!("Database created successfully."),
                Err(err) => panic!("{}", err),
            };
        } else {
            debug!("database exists, maybe migrate?");
            match maybe_migrate(&self.uri).await {
                Ok(_) => {}
                Err(err) => panic!("error migrating db: {:?}", err),
            };
        }

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
    CREATE TABLE IF NOT EXISTS users (
        id          INTEGER PRIMARY KEY NOT NULL,
        login       TEXT UNIQUE NOT NULL,
        avatar_url  TEXT NOT NULL,
        name        TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS issues (
        id              INTEGER PRIMARY KEY NOT NULL,
        number          INTEGER NOT NULL,
        title           TEXT NOT NULL,
        author          TEXT NOT NULL,
        author_id       INTEGER NOT NULL,
        url             TEXT NOT NULL,
        repo_owner      TEXT NOT NULL,
        repo_name       TEXT NOT NULL,
        state           TEXT NOT NULL,
        created_at      INTEGER NOT NULL,
        updated_at      INTEGER NOT NULL,
        closed_at       INTEGER,
        is_pull_request BOOL NOT NULL,
        last_viewed     INTEGER,
        archived_at     INTEGER
    );
    CREATE TABLE IF NOT EXISTS pull_requests (
        id              INTEGER PRIMARY KEY NOT NULL,
        is_draft        BOOL NOT NULL,
        review_decision TEXT NOT NULL,
        merged_at       INTEGER,
        FOREIGN KEY (id) REFERENCES issues (id)
    );
    CREATE TABLE IF NOT EXISTS user_issues (
        user_id     INTEGER NOT NULL,
        issue_id    INTEGER NOT NULL,
        archived    BOOL NOT NULL,
        PRIMARY KEY (user_id, issue_id),
        FOREIGN KEY (user_id) REFERENCES users (id),
        FOREIGN KEY (issue_id) REFERENCES issues (id)
    );
    CREATE TABLE IF NOT EXISTS user_refresh (
        id          INTEGER PRIMARY KEY NOT NULL,
        refresh_at  INTEGER,
        FOREIGN KEY(id) REFERENCES users(id)
    );
    CREATE TABLE IF NOT EXISTS tokens (
        id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        token       TEXT NOT NULL,
        user_id     INTEGER,
        invalid     BOOL NOT NULL,
        UNIQUE(token, user_id)
    );
    ";

    let result = sqlx::query(&query).execute(&pool).await;
    pool.close().await;

    result
}

async fn maybe_migrate(uri: &str) -> Result<(), GHDError> {
    let pool = SqlitePool::connect(uri).await.unwrap_or_else(|err| {
        panic!("Unable to connect to db at '{}': {}", uri, err);
    });

    let version = match sqlx::query_scalar::<_, u32>(
        "SELECT user_version from pragma_user_version",
    )
    .fetch_one(&pool)
    .await
    {
        Ok(v) => v,
        Err(err) => {
            panic!("unable to obtain db version: {}", err);
        }
    };
    debug!(
        "database at version {}, current {}",
        version, GHD_DB_VERSION
    );

    if version > GHD_DB_VERSION {
        return Err(GHDError::DBVersionInTheFuture);
    } else if version < GHD_DB_VERSION {
        info!("migrate db to latest version...");
        let mut v = version;
        while v < GHD_DB_VERSION {
            let to = v + 1;
            info!("migrate db from version {} to {}", v, to);
            match migrate(&pool, v, to).await {
                Ok(()) => {}
                Err(err) => {
                    panic!(
                        "Error migrating db from version {} to {}: {}",
                        v, to, err
                    );
                }
            };
            v += 1;
        }
    }

    Ok(())
}

async fn migrate(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    from: u32,
    to: u32,
) -> Result<(), sqlx::Error> {
    if from < to - 1 {
        panic!(
            "We can't migrate db versions separated by more than one version!"
        );
    } else if from > to {
        panic!("We can't migrate db from the future!");
    } else if from == to {
        // nothing to do
        return Ok(());
    }

    if from == 0 {
        // migrate version 0 to version 1
        assert_eq!(to, 1);

        let mut tx = pool.begin().await.unwrap_or_else(|err| {
            panic!("unable to start transaction: {}", err);
        });

        let query = "ALTER TABLE issues ADD COLUMN archived_at INTEGER";
        match sqlx::query(&query).execute(&mut tx).await {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to alter table: {}", err);
            }
        };
        match sqlx::query("PRAGMA user_version=1").execute(&mut tx).await {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to increase db version: {}", err);
            }
        };
        match tx.commit().await {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
    } else if from == 1 {
        // migrate version 1 to version 2
        assert_eq!(to, 2);

        let mut tx = pool.begin().await.unwrap_or_else(|err| {
            panic!("unable to start transaction: {}", err);
        });

        match sqlx::query("ALTER TABLE tokens ADD COLUMN invalid BOOL")
            .execute(&mut tx)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to alter table 'tokens': {}", err);
            }
        };
        match sqlx::query("UPDATE tokens SET invalid = True")
            .execute(&mut tx)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!(
                    "Unable to set default token invalid values to True: {}",
                    err
                );
            }
        };
        match sqlx::query(
            "
                UPDATE tokens SET invalid = False
                WHERE id = (SELECT MAX(id) FROM tokens)
            ",
        )
        .execute(&mut tx)
        .await
        {
            Ok(_) => {}
            Err(err) => {
                panic!(
                    "Unable to set latest token invalid value to False: {}",
                    err
                );
            }
        };
        match sqlx::query("PRAGMA user_version=2").execute(&mut tx).await {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to increase db version: {}", err);
            }
        };
        match tx.commit().await {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
    }

    Ok(())
}
