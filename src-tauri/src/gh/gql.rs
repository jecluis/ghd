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

use self::queries::{
    search_issues::{
        self, IssueState, PullRequestReviewDecision, PullRequestState,
        SearchIssuesSearchNodes, SearchIssuesSearchNodesOnIssue,
        SearchIssuesSearchNodesOnIssueAuthor,
        SearchIssuesSearchNodesOnPullRequest,
        SearchIssuesSearchNodesOnPullRequestAuthor, UserFragment,
    },
    SearchIssues,
};

use super::types::{Issue, PullRequest, UserUpdate};

#[derive(serde::Deserialize, Debug)]
struct GQLResData<T> {
    pub data: T,
}

struct GithubGQLRequest {
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

    /// Obtain the result from the `UserInfo` GraphQL query. This will likely go
    /// away soon, as we are relying on a different query.
    ///
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

    /// Obtain all open issues involving the specified user `login`. This means
    /// issues (and pull requests) where the user is the author, has been
    /// mentioned, has been asked for a review, or has commented.
    ///
    /// # Arguments
    ///
    /// * `login` - String containing the user's login handle.
    ///
    pub async fn get_user_open_issues(
        self: &Self,
        login: &String,
    ) -> search_issues::ResponseData {
        let q = format!("involves:{} is:open", login);
        self.get_search_issues_data(&q).await
    }

    /// Obtain all issues involving the specified user `login` that have been
    /// updated since the specified date. This means issues (and pull requests)
    /// where the user is the author, has been mentioned, has been asked for a
    /// review, or has commented.
    ///
    /// # Arguments
    ///
    /// * `login` - String containing the user's login handle.
    /// * `since` - String containing the date and time since which we should
    ///   look for updates. This String must comply with RFC 3339.
    ///
    pub async fn get_user_update(
        self: &Self,
        login: &String,
        since: &String,
    ) -> search_issues::ResponseData {
        let q = format!("involves:{} updated:>{}", login, since);
        self.get_search_issues_data(&q).await
    }

    /// Obtain issues matching the specified query. This function is auxiliary
    /// and implements the common functionality for the `get_user_open_issues()`
    /// and `get_user_update()` functions.
    ///
    /// # Arguments
    ///
    /// * `query` - String containing the query to be used for searching issues.
    ///
    async fn get_search_issues_data(
        self: &Self,
        query: &String,
    ) -> search_issues::ResponseData {
        let vars = search_issues::Variables { q: query.clone() };
        let response_data: search_issues::ResponseData = match self
            .execute::<SearchIssues, search_issues::ResponseData>(vars)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                panic!("error: {:?}", err);
            }
        };

        response_data
    }
}

/// Obtain all open issues for the provided `login`. This includes Pull
/// Requests, and is not limited to issues authored by the user, but includes
/// all issues with which the user may be involved (authored, mentioned, etc.).
///
pub async fn get_user_open_issues(
    token: &String,
    login: &String,
) -> Result<UserUpdate, GHDError> {
    let res = GithubGQLRequest::new(&token)
        .get_user_open_issues(&login)
        .await;

    process_user_search_results(&res)
}

/// Obtain Pull Request and Issue updates for provided `login` since the
/// provided date `since`.
///
/// # Arguments
///
/// * `token` - String containing the Github API Token.
/// * `login` - String containing the user to obtain an update for.
/// * `since` - Date since which updates should be looked for.
///
pub async fn get_user_updates(
    token: &String,
    login: &String,
    since: &chrono::DateTime<chrono::Utc>,
) -> Result<UserUpdate, GHDError> {
    let since_str = since.to_rfc3339();
    let res = GithubGQLRequest::new(&token)
        .get_user_update(&login, &since_str)
        .await;

    process_user_search_results(&res)
}

/// Process the resulting data from the GraphQL call into something that the
/// calling layer may understand a bit better, returning a struct containing
/// both issues and pull requests resulting from the original query.
///
fn process_user_search_results(
    res: &search_issues::ResponseData,
) -> Result<UserUpdate, GHDError> {
    let nodes = match &res.search.nodes {
        None => {
            panic!("Unexpected null nodes for user update!");
        }
        Some(v) => v,
    };

    let mut pr_lst: Vec<PullRequest> = vec![];
    let mut issue_lst: Vec<Issue> = vec![];

    for n in nodes {
        let issue = match &n {
            None => {
                continue;
            }
            Some(SearchIssuesSearchNodes::PullRequest(entry)) => {
                get_issue_from_pull_request(&entry)
            }
            Some(SearchIssuesSearchNodes::Issue(entry)) => {
                get_issue_from_issue(&entry)
            }
            Some(_) => {
                panic!("unexpected node type!");
            }
        };

        if let Some(SearchIssuesSearchNodes::PullRequest(entry)) = &n {
            pr_lst.push(PullRequest {
                issue,
                is_draft: entry.is_draft,
                review_decision: match &entry.review_decision {
                    None => String::from("none"),
                    Some(PullRequestReviewDecision::APPROVED) => {
                        String::from("approved")
                    }
                    Some(PullRequestReviewDecision::CHANGES_REQUESTED) => {
                        String::from("changes_requested")
                    }
                    Some(PullRequestReviewDecision::REVIEW_REQUIRED) => {
                        String::from("review_required")
                    }
                    Some(PullRequestReviewDecision::Other(v)) => v.clone(),
                },
                merged_at: entry.merged_at,
            });
        } else if let Some(SearchIssuesSearchNodes::Issue(_)) = &n {
            issue_lst.push(issue);
        } else {
            panic!("should not have gotten here!");
        }
    }

    Ok(UserUpdate {
        when: chrono::Utc::now(),
        prs: pr_lst,
        issues: issue_lst,
    })
}

/// Obtain an `Issue` from the provided GraphQL issue node.
///
fn get_issue_from_issue(node: &SearchIssuesSearchNodesOnIssue) -> Issue {
    let (username, userid) = match &node.author {
        None => {
            panic!("author not defined for pull request!");
        }
        Some(SearchIssuesSearchNodesOnIssueAuthor::User(user)) => {
            get_username_and_id(user)
        }
        Some(_) => {
            panic!("unexpected author user type!");
        }
    };

    Issue {
        id: get_id(node.database_id),
        title: node.title.clone(),
        number: node.number,
        author: username.clone(),
        author_id: userid,
        url: node.url.clone(),
        repo_name: node.repository.name.clone(),
        repo_owner: node.repository.owner.login.clone(),
        state: match &node.state {
            IssueState::OPEN => String::from("open"),
            IssueState::CLOSED => String::from("closed"),
            IssueState::Other(v) => v.clone(),
        },
        created_at: node.created_at,
        updated_at: node.updated_at,
        closed_at: node.closed_at,
        is_pull_request: false,
        last_viewed: None,
    }
}

/// Obtain the `Issue` associated with the provided GraphQL pull request node.
///
fn get_issue_from_pull_request(
    node: &SearchIssuesSearchNodesOnPullRequest,
) -> Issue {
    let (username, userid) = match &node.author {
        None => {
            panic!("author not defined for pull request!");
        }
        Some(SearchIssuesSearchNodesOnPullRequestAuthor::User(user)) => {
            get_username_and_id(user)
        }
        Some(_) => {
            panic!("unexpected author user type!");
        }
    };

    Issue {
        id: get_id(node.database_id),
        title: node.title.clone(),
        number: node.number,
        author: username.clone(),
        author_id: userid,
        url: node.url.clone(),
        repo_name: node.repository.name.clone(),
        repo_owner: node.repository.owner.login.clone(),
        state: match &node.state {
            PullRequestState::OPEN => String::from("open"),
            PullRequestState::CLOSED => String::from("closed"),
            PullRequestState::MERGED => String::from("merged"),
            PullRequestState::Other(v) => v.clone(),
        },
        created_at: node.created_at,
        updated_at: node.updated_at,
        closed_at: node.closed_at,
        is_pull_request: true,
        last_viewed: None,
    }
}

/// Obtain a user `login` and `id` from a given GraphQL `User Fragment`.
///
fn get_username_and_id(user: &UserFragment) -> (String, i64) {
    let id = get_id(user.database_id);

    (user.login.clone(), id)
}

/// Obtain an `id` from a provided optional ID. Typically this will be a helper
/// call to reduce code overhead when translating GraphQL structs to something
/// else, and used solely when it's expected that the provided `Option<i64>` is
/// not `None`.
///
fn get_id(v: Option<i64>) -> i64 {
    match v {
        None => {
            panic!("id not defined!");
        }
        Some(id) => id,
    }
}
