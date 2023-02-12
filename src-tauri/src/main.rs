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

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

mod bg;
mod config;
mod db;
mod gh;
mod gh_types;
mod paths;

struct ManagedDB {
    db: tokio::sync::Mutex<db::DB>,
}
struct ManagedConfig {
    config: tokio::sync::Mutex<config::Config>,
}

impl ManagedDB {
    pub async fn db(self: &Self) -> tokio::sync::MutexGuard<'_, db::DB> {
        self.db.lock().await
    }
}

impl ManagedConfig {
    pub async fn config(
        self: &Self,
    ) -> tokio::sync::MutexGuard<'_, config::Config> {
        self.config.lock().await
    }
}

#[tauri::command]
async fn greet(name: &str) -> Result<String, ()> {
    println!("called 'greet'");
    Ok(format!("Hello, {}!", name))
}

#[tauri::command]
async fn set_api_token(
    token: String,
    mdb: tauri::State<'_, ManagedDB>,
    mcfg: tauri::State<'_, ManagedConfig>,
) -> Result<bool, ()> {
    println!("set token to {}", token);

    let db = mdb.db().await;
    let mut cfg = mcfg.config().await;
    cfg.set_api_token(&db, &token).await.unwrap_or_else(|err| {
        panic!("Unable to set API Token! Error: {}", err);
    });

    Ok(true)
}

#[tauri::command]
async fn get_api_token(
    mcfg: tauri::State<'_, ManagedConfig>,
) -> Result<String, ()> {
    let token = match &mcfg.config().await.api_token {
        Some(t) => t.clone(),
        None => String::default(),
    };

    Ok(token)
}

async fn setup_paths() -> paths::Paths {
    paths::Paths::default().init().await
}

async fn setup_db(path: &std::path::PathBuf) -> db::DB {
    let mut handle = db::DB::new(&path).setup().await;
    handle.connect().await;

    handle
}

async fn setup_config(db: &db::DB) -> config::Config {
    let mut cfg = config::Config::default();
    cfg.init(db).await;

    cfg
}


#[tokio::main]
async fn main() {
    let paths = setup_paths().await;
    let db_handle = setup_db(&paths.db_path).await;
    let cfg = setup_config(&db_handle).await;

    println!("  user data dir: {}", paths.data_dir.display());
    println!("user config dir: {}", paths.config_dir.display());
    println!("  database path: {}", paths.db_path.display());

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let mdb = ManagedDB { db: tokio::sync::Mutex::new(db_handle) };

    tauri::Builder::default()
        .manage(paths)
        .manage(mdb)
        .manage(ManagedConfig {
            config: tokio::sync::Mutex::new(cfg),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            set_api_token,
            get_api_token,
        ])
        .setup(|app| {
            let handle = app.app_handle();
            // let window = app.get_window("main").unwrap();
            tokio::spawn(async move {
                let mut bgtask = bg::BGTask::new();
                bgtask.run(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
