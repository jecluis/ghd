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
    token: String,
    client: reqwest::Client,
}

impl Github {
    pub fn new(token: &String) -> Self {
        Github {
            token: token.clone(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_pulls(
        self: &Self,
    ) -> Result<Vec<PullRequestEntry>, reqwest::StatusCode> {
        let user = String::from("jecluis");
        prs::get(&self, &user).await
    }

    pub fn get(self: &Self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url)
    }

    pub async fn send(
        self: &Self,
        rb: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, reqwest::StatusCode> {
        let req = rb
            .bearer_auth(&self.token)
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
