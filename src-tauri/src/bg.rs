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
    config::Config,
    db::DB,
    gh::{self, Github},
    ManagedState,
};
use tauri::Manager;

mod types;

const PULL_UPDATE_INTERVAL: i64 = 60; // one minute

struct State {
    last_pull_update: chrono::DateTime<chrono::Utc>,
    pull_requests: Vec<gh::prs::PullRequestEntry>,
}

pub struct BGTask {
    state: State,
}

impl BGTask {
    pub fn new() -> Self {
        BGTask {
            state: State {
                last_pull_update: chrono::DateTime::<chrono::Utc>::MIN_UTC,
                pull_requests: Vec::new(),
            },
        }
    }

    pub async fn run(self: &mut Self, app: tauri::AppHandle) {
        let window = app.get_window("main").unwrap();
        let mstate = app.try_state::<ManagedState>().unwrap();
        let state = &mstate.state().await;
        let db = &state.db;
        let cfg = &state.config;
        let gh = &state.gh;

        let mut n = 1;
        loop {
            println!("background task iteration #{}", n);
            window.emit("iteration", n).unwrap();
            n += 1;

            let token = match self.try_get_token(&cfg, &db).await {
                Ok(t) => {
                    println!("token: {}", t);
                    t
                }
                Err(_) => {
                    self.sleep_for_a_bit().await;
                    continue;
                }
            };

            if self.maybe_get_prs(&gh, &token).await {
                self.emit_prs_update(&window);
            }

            self.sleep_for_a_bit().await;
        }
    }

    async fn try_get_token(
        self: &Self,
        cfg: &Config,
        db: &DB,
    ) -> Result<String, ()> {
        let t = get_token(&cfg, &db).await;
        if !t.is_empty() {
            return Ok(t.clone());
        }
        Err(())
    }

    async fn sleep_for_a_bit(self: &Self) {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    async fn maybe_get_prs(
        self: &mut Self,
        gh: &Github,
        token: &String,
    ) -> bool {
        if !has_expired(&self.state.last_pull_update, PULL_UPDATE_INTERVAL) {
            return false;
        }

        self.state.last_pull_update = chrono::Utc::now();
        match gh.get_pulls(token).await {
            Err(err) => {
                println!("Error obtaining pull requests: {}", err);
                match err {
                    reqwest::StatusCode::UNAUTHORIZED => {
                        println!("bad token!");
                    }
                    _ => {}
                };
                return false;
            }
            Ok(prs) => {
                println!("success obtaining pull requests!");
                self.state.pull_requests = prs;
                return true;
            }
        };
    }

    fn emit_prs_update(self: &Self, window: &tauri::Window) {
        let prlist = get_pull_requests(&self.state.pull_requests);
        print_pull_requests(&prlist);
        window.emit("pull_requests_update", &prlist).unwrap();
    }
}

async fn get_token(cfg: &Config, db: &DB) -> String {
    match &cfg.get_api_token(&db).await {
        Ok(t) => t.clone(),
        Err(_) => String::default(),
    }
}

fn has_expired(t: &chrono::DateTime<chrono::Utc>, secs: i64) -> bool {
    let now = chrono::Utc::now();
    let dt = match t.checked_add_signed(chrono::Duration::seconds(secs)) {
        Some(v) => v,
        None => now,
    };
    return dt < now;
}

fn get_pull_requests(prs: &Vec<gh::prs::PullRequestEntry>) -> types::PRList {
    let mut prlist = types::PRList::default();

    for pr in prs {
        let created =
            chrono::DateTime::parse_from_rfc3339(&pr.created_at).unwrap();
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
