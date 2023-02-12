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

use self::prs::PullRequestEntry;

pub mod prs;

pub struct Github {
    client: reqwest::Client,
}

impl Github {
    pub fn new() -> Self {
        Github {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_pulls(
        self: &Self,
        token: &String,
    ) -> Result<Vec<PullRequestEntry>, reqwest::StatusCode> {
        let user = String::from("jecluis");
        prs::get(&self, token, &user).await
    }

    pub fn get(self: &Self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url)
    }

    pub async fn send(
        self: &Self,
        rb: reqwest::RequestBuilder,
        token: &String,
    ) -> Result<reqwest::Response, reqwest::StatusCode> {
        let req = rb
            .bearer_auth(token)
            .header("User-Agent", "GHD")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .unwrap();

        if req.status() != reqwest::StatusCode::OK {
            return Err(req.status());
        }

        Ok(req)
    }
}
