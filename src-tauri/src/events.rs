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

use crate::gh::{types::GithubUser, types::PullRequestEntry};

pub const EV_ITERATION: &str = "iteration";
pub const EV_PULL_REQUESTS_UPDATE: &str = "pull_requests_update";
pub const EV_USER_DATA_UPDATE: &str = "user_data_update";
pub const EV_USER_UPDATE: &str = "user_update";
pub const EV_TOKEN_SET: &str = "token_set";

#[derive(serde::Serialize, Clone)]
struct PullRequestUpdatePayload<'a> {
    user: &'a GithubUser,
    prs: &'a Vec<PullRequestEntry>,
}

pub fn emit<S>(w: &tauri::Window, ev: &str, payload: S)
where
    S: serde::Serialize + Clone,
{
    w.emit(ev, payload).unwrap();
}

pub fn emit_token_set(w: &tauri::Window) {
    emit(w, EV_TOKEN_SET, true);
}

pub fn emit_user_update(w: &tauri::Window, user: &GithubUser) {
    println!("emit user update for {}", user.login);
    emit(w, EV_USER_UPDATE, user);
}

pub fn emit_pull_request_update(
    w: &tauri::Window,
    user: &GithubUser,
    prs: &Vec<PullRequestEntry>,
) {
    println!(
        "emit pull request update for {} ({} entries)",
        user.login,
        prs.len()
    );
    emit(
        w,
        EV_PULL_REQUESTS_UPDATE,
        PullRequestUpdatePayload { user, prs },
    );
}

pub fn emit_user_data_update(w: &tauri::Window, login: &String) {
    println!("emite user data update for '{}'", login);
    emit(w, EV_USER_DATA_UPDATE, login);
}
