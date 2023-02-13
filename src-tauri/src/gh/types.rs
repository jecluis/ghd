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

pub struct GithubRequest {
    client: reqwest::Client,
    token: String,
}

impl GithubRequest {
    pub fn new(token: &String) -> Self {
        GithubRequest {
            client: reqwest::Client::new(),
            token: token.clone(),
        }
    }

    pub fn get(self: &Self, endpoint: &str) -> reqwest::RequestBuilder {
        let ep = match endpoint.strip_prefix("/") {
            Some(res) => res,
            None => endpoint,
        };

        self.client.get(format!("https://api.github.com/{}", ep))
    }

    pub async fn send<'a, T>(
        self: &Self,
        rb: reqwest::RequestBuilder,
    ) -> Result<T, reqwest::StatusCode>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
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

        Ok(req.json::<T>().await.unwrap())
    }
}

// Users

#[derive(serde::Deserialize)]
pub struct GithubUserReply {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub name: String,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct GithubUser {
    pub login: String,
    pub id: i64,
    pub avatar_url: String,
    pub name: String,
}
