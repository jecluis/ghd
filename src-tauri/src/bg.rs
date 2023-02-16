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
    common,
    db::DB,
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

            let users = match gh.get_tracked_users(&db).await {
                Ok(res) => res,
                Err(err) => {
                    panic!("Unable to obtain tracked users: {:?}", err);
                }
            };

            for user in &users {
                if gh.should_refresh_user(&db, &user.login).await {
                    match gh.refresh_user(&db, &user.login).await {
                        Ok(()) => {}
                        Err(err) => {
                            println!(
                                "error refreshing user '{}': {:?}",
                                &user.login, err
                            );
                        }
                    }
                }
            }

            self.sleep_for_a_bit().await;
        }
    }

    async fn sleep_for_a_bit(self: &Self) {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // fn emit_prs_update(self: &Self, window: &tauri::Window) {
    //     let prlist = get_pull_requests(&self.state.pull_requests);
    //     print_pull_requests(&prlist);
    //     window.emit("pull_requests_update", &prlist).unwrap();
    // }
}

async fn get_token(gh: &Github, db: &DB) -> String {
    match &gh.get_token(&db).await {
        Ok(t) => t.clone(),
        Err(_) => String::default(),
    }
}

async fn has_token(gh: &Github, db: &DB) -> bool {
    match &gh.get_token(&db).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_pull_requests(prs: &Vec<gh::prs::PullRequestEntry>) -> types::PRList {
    let mut prlist = types::PRList::default();

    for pr in prs {
        let created = common::ts_to_datetime(pr.created_at).unwrap();
        let diff = chrono::Utc::now().signed_duration_since(created);

        let secs = diff.num_seconds();
        let hours = diff.num_hours();
        let days = diff.num_days();
        let weeks = diff.num_weeks();
        let months = days / 30;

        let age = if months > 0 {
            format!("{} months", months)
        } else if weeks > 0 {
            format!("{} weeks", weeks)
        } else if days > 0 {
            format!("{} days", days)
        } else if hours > 0 {
            format!("{} hours", hours)
        } else {
            format!("{} secs", secs)
        };

        prlist.entries.push(types::PREntry {
            id: pr.number,
            title: pr.title.clone(),
            age_str: age,
        });
    }

    prlist
}

fn print_pull_requests(prs: &types::PRList) {
    for pr in &prs.entries {
        println!("{:>5}  {}  ({} ago)", pr.id, pr.title, pr.age_str);
    }
}
