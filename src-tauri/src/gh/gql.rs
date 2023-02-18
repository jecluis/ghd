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

mod custom_types;
mod queries;

use graphql_client::GraphQLQuery;
use queries::{user_info, UserInfo};

use crate::errors::GHDError;

#[derive(serde::Deserialize, Debug)]
struct GQLResData<T> {
    pub data: T,
}

pub struct GithubGQLRequest {
    client: reqwest::Client,
}

impl GithubGQLRequest {
    pub fn new(token: &String) -> Self {
        GithubGQLRequest {
            client: reqwest::Client::builder()
                .user_agent("GHD")
                .default_headers(
                    std::iter::once((
                        reqwest::header::AUTHORIZATION,
                        reqwest::header::HeaderValue::from_str(&format!(
                            "Bearer {}",
                            token
                        ))
                        .unwrap(),
                    ))
                    .collect(),
                )
                .build()
                .unwrap(),
        }
    }

    async fn execute<'a, T, M>(
        self: &Self,
        variables: T::Variables,
    ) -> Result<M, GHDError>
    where
        T: GraphQLQuery,
        M: for<'de> serde::Deserialize<'de> + core::fmt::Debug,
    {
        let debug = std::env::var("GHD_GQL_DEBUG").is_ok();
        let req_body = T::build_query(variables);
        let res = match self
            .client
            .post("https://api.github.com/graphql")
            .json(&req_body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => {
                println!("unknown error from send: {}", err);
                return Err(GHDError::UnknownError);
            }
        };

        match res.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::FORBIDDEN => {
                return Err(GHDError::BadTokenError);
            }
            reqwest::StatusCode::NOT_FOUND => {
                return Err(GHDError::UserNotFoundError);
            }
            reqwest::StatusCode::BAD_REQUEST => {
                return Err(GHDError::BadRequest);
            }
            err => {
                println!("unknown error: {}", err);
                return Err(GHDError::UnknownError);
            }
        };

        let res_body = res.text().await.unwrap_or_else(|err| {
            panic!("Unable to unwrap graphql body result: {}", err);
        });
        if debug {
            println!("res body:\n{}", res_body);
        }

        let res_data: GQLResData<M> = serde_json::from_str(&res_body)
            .unwrap_or_else(|err| {
                panic!("Unable to decode graphql result: {}", err);
            });

        if debug {
            println!("res data: {:?}", res_data);
        }

        Ok(res_data.data)
    }

    pub async fn get_user_info(
        self: &Self,
        login: &String,
    ) -> user_info::ResponseData {
        let vars = user_info::Variables {
            login: login.clone(),
        };
        let response_data: user_info::ResponseData = match self
            .execute::<UserInfo, user_info::ResponseData>(vars)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                panic!("error: {:?}", err);
            }
        };

        println!("data: {:?}", response_data);

        response_data
    }
}
