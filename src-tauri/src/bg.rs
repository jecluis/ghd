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

use crate::{
    db::DB,
    events,
    gh::{self, Github},
    ManagedState,
};
use tauri::Manager;

mod types;

pub struct BGTask {}

impl BGTask {
    pub fn new() -> Self {
        BGTask {}
    }

    pub async fn run(self: &mut Self, app: tauri::AppHandle) {
        let window = app.get_window("main").unwrap();
        let mstate = app.try_state::<ManagedState>().unwrap();

        let mut n = 1;
        loop {
            let state = &mstate.state().await;
            let db = &state.db;
            let _cfg = &state.config;
            let gh = &state.gh;

            println!("background task iteration #{}", n);
            window.emit("iteration", n).unwrap();
            n += 1;

            if !has_token(&gh, &db).await {
                self.sleep_for_a_bit().await;
                continue;
            }

            let to_refresh = gh::refresh::get_to_refresh_users(&db).await;
            for user in &to_refresh {
                println!("should refresh user '{}'", user.login);
                match gh.refresh_user(&db, &user.login).await {
                    Ok(true) => {
                        println!("refreshed user '{}'", user.login);
                        events::emit_user_data_update(&window, &user.login);
                    }
                    Ok(false) => {}
                    Err(crate::errors::GHDError::BadTokenError) => {
                        println!("invalidate token");
                        gh.invalidate_token(&db).await;
                        continue;
                    }
                    Err(err) => {
                        println!(
                            "error refreshing user '{}': {:?}",
                            user.login, err,
                        );
                    }
                }
            }

            let users = match gh::users::get_tracked_users(&db).await {
                Ok(res) => res,
                Err(err) => {
                    panic!("Unable to obtain tracked users: {:?}", err);
                }
            };

            for user in &users {
                if gh::refresh::should_refresh_user(&db, &user.login).await {}
            }

            self.sleep_for_a_bit().await;
        }
    }

    async fn sleep_for_a_bit(self: &Self) {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn has_token(gh: &Github, db: &DB) -> bool {
    match &gh.get_token(&db).await {
        Ok(_) => true,
        Err(_) => false,
    }
}
