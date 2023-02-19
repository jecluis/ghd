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

/// Abstracts REST requests. May be used as one GithubRequest per REST
/// operation, or may be reused.
///
pub struct GithubRequest {
    client: reqwest::Client,
    token: String,
}

impl GithubRequest {
    /// Obtain a new GithubRequest instance.
    ///
    /// # Arguments
    ///
    /// * `token` - String containing the API Token to use.
    ///
    pub fn new(token: &String) -> Self {
        GithubRequest {
            client: reqwest::Client::new(),
            token: token.clone(),
        }
    }

    /// Obtain a `reqwest::RequestBuilder` for a `GET` operation, targeting the
    /// provided `endpoint`.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - String containing the target endpoint; e.g., `/user`.
    ///
    pub fn get(self: &Self, endpoint: &str) -> reqwest::RequestBuilder {
        let ep = match endpoint.strip_prefix("/") {
            Some(res) => res,
            None => endpoint,
        };

        self.client.get(format!("https://api.github.com/{}", ep))
    }

    /// Send the request and return a result containing either the specified
    /// type, or a `reqwest::StatusCode` as an error. Requires an existing
    /// `reqwest::RequestBuilder` to be provided as argument. This function
    /// handles setting headers and the token.
    ///
    /// # Arguments
    ///
    /// * `rb` - The pre-built `reqwest::RequestBuilder` to send to the server.
    ///
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

        let txt = req.text().await.unwrap();

        if std::env::var("GHD_REST_DEBUG").is_ok() {
            println!("REST(send result): {}", txt);
        }

        let res: T = serde_json::from_str(&txt).unwrap();
        Ok(res)
    }
}

/// REST API User Reply
///
#[derive(serde::Deserialize)]
pub struct GithubUserReply {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub name: String,
}
