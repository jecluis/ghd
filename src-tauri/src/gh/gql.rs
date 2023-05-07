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

use crate::{
    errors::GHDError,
    gh::types::{Label, UserReview},
};

use self::queries::{
    get_pull_request_info::{
        self, GetPullRequestInfoRepositoryPullRequestAuthor,
    },
    search_issues::{
        self, IssueState, PullRequestReviewDecision, SearchIssuesSearchNodes,
        SearchIssuesSearchNodesOnIssue, SearchIssuesSearchNodesOnIssueAuthor,
        SearchIssuesSearchNodesOnPullRequest,
        SearchIssuesSearchNodesOnPullRequestAuthor, UserFragment,
    },
    GetPullRequestInfo, SearchIssues,
};

use super::types::{
    GithubUser, Issue, Milestone, PullRequest, PullRequestInfo, UserUpdate,
};

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
            reqwest::StatusCode::UNAUTHORIZED => {
                return Err(GHDError::BadTokenError);
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
    ) -> Result<search_issues::ResponseData, GHDError> {
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
    ) -> Result<search_issues::ResponseData, GHDError> {
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
    ) -> Result<search_issues::ResponseData, GHDError> {
        let vars = search_issues::Variables { q: query.clone() };
        let response_data: search_issues::ResponseData = match self
            .execute::<SearchIssues, search_issues::ResponseData>(vars)
            .await
        {
            Ok(res) => res,
            Err(GHDError::UnknownError) => {
                panic!("error: unknown error");
            }
            Err(err) => {
                return Err(err);
            }
        };

        Ok(response_data)
    }

    /// Obtain a given Pull Request's information.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - String containing the Pull Request's repository owner.
    /// * `repo_name` - String containing the Pull Request's repository name.
    /// * `pr_number` - The Pull Request's number.
    ///
    async fn get_pull_request_info(
        self: &Self,
        repo_owner: &String,
        repo_name: &String,
        pr_number: &i64,
    ) -> Result<get_pull_request_info::ResponseData, GHDError> {
        let vars = get_pull_request_info::Variables {
            owner: repo_owner.clone(),
            repo: repo_name.clone(),
            prid: pr_number.clone(),
        };
        let response_data: get_pull_request_info::ResponseData = match self
            .execute::<GetPullRequestInfo, get_pull_request_info::ResponseData>(
                vars,
            )
            .await
        {
            Ok(res) => res,
            Err(GHDError::UnknownError) => {
                panic!("error: unknown error");
            }
            Err(err) => {
                return Err(err);
            }
        };

        Ok(response_data)
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
    let res = match GithubGQLRequest::new(&token)
        .get_user_open_issues(&login)
        .await
    {
        Ok(v) => v,
        Err(err) => return Err(err),
    };

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
    let res = match GithubGQLRequest::new(&token)
        .get_user_update(&login, &since_str)
        .await
    {
        Ok(v) => v,
        Err(err) => return Err(err),
    };

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
            search_issues::PullRequestState::OPEN => String::from("open"),
            search_issues::PullRequestState::CLOSED => String::from("closed"),
            search_issues::PullRequestState::MERGED => String::from("merged"),
            search_issues::PullRequestState::Other(v) => v.clone(),
        },
        created_at: node.created_at,
        updated_at: node.updated_at,
        closed_at: node.closed_at,
        is_pull_request: true,
        last_viewed: None,
    }
}

/// Obtain a given Pull Request's information.
///
pub async fn get_pull_request_info(
    token: &String,
    repo_owner: &String,
    repo_name: &String,
    pr_number: &i64,
) -> Result<PullRequestInfo, GHDError> {
    // helper types
    type PRInfoAuthor = GetPullRequestInfoRepositoryPullRequestAuthor;
    type PRState = get_pull_request_info::PullRequestState;
    type PRMilestoneState = get_pull_request_info::MilestoneState;
    type ReviewAuthor = get_pull_request_info::GetPullRequestInfoRepositoryPullRequestReviewsNodesAuthor;
    type ReviewState = get_pull_request_info::PullRequestReviewState;

    let res = match GithubGQLRequest::new(&token)
        .get_pull_request_info(&repo_owner, &repo_name, &pr_number)
        .await
    {
        Ok(v) => v,
        Err(err) => return Err(err),
    };

    let repo = match res.repository {
        None => return Err(GHDError::RepositoryNotFoundError),
        Some(v) => v,
    };
    let pr = match repo.pull_request {
        None => return Err(GHDError::PullRequestNotFoundError),
        Some(v) => v,
    };

    let unknown_user = GithubUser {
        id: -1,
        login: String::from("unknown"),
        name: String::from("unknown"),
        avatar_url: String::new(),
    };

    let author: GithubUser = match pr.author {
        None => unknown_user.clone(),
        Some(v) => match v {
            PRInfoAuthor::User(user) => GithubUser {
                id: user.database_id.unwrap_or(-1),
                login: user.login,
                name: user.name.unwrap_or(String::from("unknown")),
                avatar_url: user.avatar_url,
            },
            _ => unknown_user.clone(),
        },
    };

    let mut labels: Vec<Label> = vec![];
    if let Some(l) = &pr.labels {
        if let Some(lst) = &l.nodes {
            for entry in lst {
                if let Some(label) = &entry {
                    labels.push(Label {
                        name: label.name.clone(),
                        color: label.color.clone(),
                    });
                }
            }
        }
    }

    let mut participants: Vec<GithubUser> = vec![];
    if let Some(lst) = &pr.participants.nodes {
        for entry in lst {
            if let Some(user) = &entry {
                participants.push(GithubUser {
                    id: match &user.database_id {
                        None => -1,
                        Some(v) => *v,
                    },
                    login: user.login.clone(),
                    name: match &user.name {
                        None => String::from("unknown"),
                        Some(v) => v.clone(),
                    },
                    avatar_url: user.avatar_url.clone(),
                });
            }
        }
    }

    let mut reviews: Vec<UserReview> = vec![];
    if let Some(r) = &pr.reviews {
        if let Some(lst) = &r.nodes {
            for entry in lst {
                if let Some(rev) = &entry {
                    reviews.push(UserReview {
                        author: match &rev.author {
                            None => unknown_user.clone(),
                            Some(ReviewAuthor::User(u)) => GithubUser {
                                id: u.database_id.unwrap_or(-1),
                                login: u.login.clone(),
                                name: match &u.name {
                                    None => String::from("unknown"),
                                    Some(v) => v.clone(),
                                },
                                avatar_url: u.avatar_url.clone(),
                            },
                            Some(_) => unknown_user.clone(),
                        },
                        state: match &rev.state {
                            ReviewState::APPROVED => String::from("approved"),
                            ReviewState::CHANGES_REQUESTED => {
                                String::from("changes_requested")
                            }
                            ReviewState::COMMENTED => String::from("commented"),
                            ReviewState::DISMISSED => String::from("dismissed"),
                            ReviewState::PENDING => String::from("pending"),
                            ReviewState::Other(v) => v.clone(),
                        },
                    });
                }
            }
        }
    }

    Ok(PullRequestInfo {
        number: pr.number,
        title: pr.title,
        body_html: pr.body_html,
        author,
        repo_owner: pr.repository.owner.login,
        repo_name: pr.repository.name,
        url: pr.url,
        state: match &pr.state {
            PRState::OPEN => String::from("open"),
            PRState::CLOSED => String::from("closed"),
            PRState::MERGED => String::from("merged"),
            PRState::Other(v) => v.clone(),
        },
        is_draft: pr.is_draft,
        milestone: match &pr.milestone {
            None => None,
            Some(m) => Some(Milestone {
                title: m.title.clone(),
                state: match &m.state {
                    PRMilestoneState::OPEN => String::from("open"),
                    PRMilestoneState::CLOSED => String::from("closed"),
                    PRMilestoneState::Other(v) => v.clone(),
                },
                due_on: match &m.due_on {
                    None => None,
                    Some(d) => Some(*d),
                },
                due_on_ts: match &m.due_on {
                    None => None,
                    Some(d) => Some(d.timestamp()),
                },
            }),
        },
        labels,
        total_comments: match &pr.total_comments_count {
            None => 0,
            Some(v) => *v,
        },
        participants,
        reviews,
    })
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
