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

use errors::GHDError;
use gh::types::PullRequestInfo;
use log::{debug, error, info, warn};
use tauri::Manager;

mod bg;
mod common;
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
) -> Result<(), u16> {
    debug!("set token to {}", token);

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
        Ok(_) => {}
        Err(err) => {
            error!("error setting token: {:?}", err);
            return Err(err as u16);
        }
    };
    Ok(())
}

#[tauri::command]
async fn get_token(
    window: tauri::Window,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<String, u16> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    let token = match &gh.get_token(&db).await {
        Ok(val) => val.clone(),
        Err(err) => {
            match err {
                GHDError::BadTokenError => {
                    events::emit_token_invalid(&window);
                }
                _ => {}
            };
            return Err(*err as u16);
        }
    };

    Ok(token)
}

#[tauri::command]
async fn get_main_user(
    mstate: tauri::State<'_, ManagedState>,
) -> Result<gh::types::GithubUser, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    match gh::users::get_main_user(&db).await {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn get_tracked_users(
    mstate: tauri::State<'_, ManagedState>,
) -> Result<Vec<gh::types::GithubUser>, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    match gh::users::get_tracked_users(&db).await {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn add_tracked_user(
    username: String,
    window: tauri::Window,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<gh::types::GithubUser, ()> {
    debug!("track new user: {}", username);
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    match gh
        .track_user(&db, &username, |user| {
            events::emit_user_update(&window, &user);
        })
        .await
    {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn check_user_exists(
    username: String,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<gh::types::GithubUser, ()> {
    debug!("check user exist: {}", username);
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    match gh.get_user_by_login(&db, &username).await {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn pr_mark_viewed(
    prid: i64,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<(), ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    match gh.mark_pull_request_viewed(&db, &prid).await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn pr_mark_viewed_many(
    prs: Vec<i64>,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<(), ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;
    match gh.mark_pull_request_viewed_many(&db, &prs).await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn pr_get_list_by_author(
    login: String,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<Vec<gh::types::PullRequestTableEntry>, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;

    match gh.get_pulls_by_author(&db, &login).await {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn pr_get_list_by_involved(
    login: String,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<Vec<gh::types::PullRequestTableEntry>, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;

    match gh.get_involved_pulls(&db, &login).await {
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn pr_get_info(
    prid: i64,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<PullRequestInfo, ()> {
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;

    match gh.get_pull_request_info(&db, &prid).await {
        Ok(res) => Ok(res),
        Err(err) => {
            warn!(
                "Error obtaining pull request info, id: {}, err: {:?}",
                prid, err
            );
            return Err(());
        }
    }
}

#[tauri::command]
async fn archive_issue(
    issue_id: i64,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<(), ()> {
    debug!("Marking issue {} as archived", issue_id);
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;

    match gh.archive_issue(&db, &issue_id).await {
        Ok(_) => {}
        Err(err) => {
            error!("Error archiving issue '{}': {:?}", issue_id, err);
            return Err(());
        }
    };

    Ok(())
}

#[tauri::command]
async fn archive_issue_many(
    issues: Vec<i64>,
    mstate: tauri::State<'_, ManagedState>,
) -> Result<(), ()> {
    debug!("Marking {} issues as archived", issues.len());
    let state = &mstate.state().await;
    let db = &state.db;
    let gh = &state.gh;

    match gh.archive_issue_many(&db, &issues).await {
        Ok(_) => {}
        Err(err) => {
            error!("Error archiving multiple issues: {:?}", err);
            return Err(());
        }
    };
    Ok(())
}

async fn setup_paths() -> paths::Paths {
    paths::Paths::default().init().await
}

async fn setup_db(path: &std::path::PathBuf) -> db::DB {
    let mut handle = db::DB::new(&path).setup().await;
    handle.connect().await;

    handle
}

async fn setup_config() -> config::Config {
    config::Config::default()
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let paths = setup_paths().await;
    let db_handle = setup_db(&paths.db_path).await;
    let cfg = setup_config().await;

    info!("  user data dir: {}", paths.data_dir.display());
    info!("user config dir: {}", paths.config_dir.display());
    info!("  database path: {}", paths.db_path.display());

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
            set_token,
            get_token,
            get_main_user,
            get_tracked_users,
            add_tracked_user,
            check_user_exists,
            pr_mark_viewed,
            pr_mark_viewed_many,
            pr_get_list_by_author,
            pr_get_list_by_involved,
            pr_get_info,
            archive_issue,
            archive_issue_many,
        ])
        .setup(|app| {
            let handle = app.app_handle();
            let pinfo = handle.package_info();
            println!("ghd v{} {}", pinfo.version, pinfo.authors);
            tokio::spawn(async move {
                let mut bgtask = bg::BGTask::new();
                bgtask.run(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
