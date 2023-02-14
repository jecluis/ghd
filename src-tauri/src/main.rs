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
mod errors;
mod events;
mod gh;
mod gh_types;
mod paths;
mod state;

struct ManagedState {
    state: tokio::sync::Mutex<state::State>,
}

impl ManagedState {
    pub async fn state(
        self: &Self,
    ) -> tokio::sync::MutexGuard<'_, state::State> {
        self.state.lock().await
    }
}

#[tauri::command]
async fn set_token(
    token: String,
    window: tauri::Window,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<bool, ()> {
    println!("set token to {}", token);

    let state = &mstate.state().await;

    let db = &state.db;
    let gh = &state.gh;
    match gh
        .set_token(&db, &token, |user| {
            events::emit_token_set(&window);
            events::emit_user_update(&window, &user);
        })
        .await
    {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn get_token(
    mstate: tauri::State<'_, ManagedState>,
) -> Result<String, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    let token = match &gh.get_token(&db).await {
        Ok(val) => val.clone(),
        Err(_) => String::default(),
    };

    Ok(token)
}

#[tauri::command]
async fn get_user(
    mstate: tauri::State<'_, ManagedState>,
) -> Result<gh::types::GithubUser, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    match gh.get_user(&db).await {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
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
    config::Config::default()
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

    tauri::Builder::default()
        .manage(ManagedState {
            state: tokio::sync::Mutex::new(state::State {
                config: cfg,
                db: db_handle,
                paths: paths,
                gh: gh::Github::new(),
            }),
        })
        .invoke_handler(tauri::generate_handler![
            set_token, get_token, get_user
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
