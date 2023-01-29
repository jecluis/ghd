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

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, SqlitePool};

mod state;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn setup_state() -> state::State {
    state::State::default().init()
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

async fn setup_db(dbpath: &std::path::PathBuf) {
    let uri: String = format!("sqlite://{}", dbpath.display());
    if !sqlx::Sqlite::database_exists(&uri).await.unwrap_or(false) {
        sqlx::Sqlite::create_database(&uri).await.unwrap();
        match create_db_schema(&uri).await {
            Ok(_) => println!("Database created successfully."),
            Err(err) => panic!("{}", err),
        };
    }
}

#[tokio::main]
async fn main() {
    let state = setup_state();

    println!("  user data dir: {}", state.data_dir.display());
    println!("user config dir: {}", state.config_dir.display());
    println!("  database path: {}", state.db_path.display());

    setup_db(&state.db_path).await;

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
