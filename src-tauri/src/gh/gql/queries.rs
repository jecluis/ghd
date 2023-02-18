#![allow(clippy::all, warnings)]
pub struct UserInfo;
pub mod user_info {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "UserInfo";
    pub const QUERY : & str = "query UserInfo($login: String!) {\n  user(login: $login) {\n    name\n    login\n    avatarUrl\n    databaseId\n    pullRequests(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        totalCommentsCount\n        createdAt\n        updatedAt\n        isDraft\n        milestone {\n          id\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n      }\n    }\n    issues(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        updatedAt\n        author {\n          __typename\n          login\n        }\n        participants(first: 1) {\n          totalCount\n        }\n        assignees(first: 10) {\n          nodes {\n            login\n          }\n        }\n      }\n    }\n  }\n}\n" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    type DateTime = crate::gh::gql::custom_types::DateTime;
    type URI = crate::gh::gql::custom_types::URI;
    #[derive(Debug)]
    pub enum PullRequestReviewDecision {
        APPROVED,
        CHANGES_REQUESTED,
        REVIEW_REQUIRED,
        Other(String),
    }
    impl ::serde::Serialize for PullRequestReviewDecision {
        fn serialize<S: serde::Serializer>(
            &self,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                PullRequestReviewDecision::APPROVED => "APPROVED",
                PullRequestReviewDecision::CHANGES_REQUESTED => {
                    "CHANGES_REQUESTED"
                }
                PullRequestReviewDecision::REVIEW_REQUIRED => "REVIEW_REQUIRED",
                PullRequestReviewDecision::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for PullRequestReviewDecision {
        fn deserialize<D: ::serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "APPROVED" => Ok(PullRequestReviewDecision::APPROVED),
                "CHANGES_REQUESTED" => {
                    Ok(PullRequestReviewDecision::CHANGES_REQUESTED)
                }
                "REVIEW_REQUIRED" => {
                    Ok(PullRequestReviewDecision::REVIEW_REQUIRED)
                }
                _ => Ok(PullRequestReviewDecision::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub login: String,
    }
    impl Variables {}
    #[derive(Deserialize, Debug)]
    pub struct ResponseData {
        pub user: Option<UserInfoUser>,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUser {
        pub name: Option<String>,
        pub login: String,
        #[serde(rename = "avatarUrl")]
        pub avatar_url: URI,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        #[serde(rename = "pullRequests")]
        pub pull_requests: UserInfoUserPullRequests,
        pub issues: UserInfoUserIssues,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserPullRequests {
        pub nodes: Option<Vec<Option<UserInfoUserPullRequestsNodes>>>,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserPullRequestsNodes {
        pub title: String,
        pub number: Int,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        #[serde(rename = "totalCommentsCount")]
        pub total_comments_count: Option<Int>,
        #[serde(rename = "createdAt")]
        pub created_at: DateTime,
        #[serde(rename = "updatedAt")]
        pub updated_at: DateTime,
        #[serde(rename = "isDraft")]
        pub is_draft: Boolean,
        pub milestone: Option<UserInfoUserPullRequestsNodesMilestone>,
        pub repository: UserInfoUserPullRequestsNodesRepository,
        #[serde(rename = "reviewDecision")]
        pub review_decision: Option<PullRequestReviewDecision>,
        pub reviews: Option<UserInfoUserPullRequestsNodesReviews>,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserPullRequestsNodesMilestone {
        pub id: ID,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserPullRequestsNodesRepository {
        pub owner: UserInfoUserPullRequestsNodesRepositoryOwner,
        pub name: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserPullRequestsNodesRepositoryOwner {
        pub login: String,
        #[serde(flatten)]
        pub on: UserInfoUserPullRequestsNodesRepositoryOwnerOn,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum UserInfoUserPullRequestsNodesRepositoryOwnerOn {
        Organization,
        User,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserPullRequestsNodesReviews {
        #[serde(rename = "totalCount")]
        pub total_count: Int,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserIssues {
        pub nodes: Option<Vec<Option<UserInfoUserIssuesNodes>>>,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserIssuesNodes {
        pub title: String,
        pub number: Int,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        #[serde(rename = "updatedAt")]
        pub updated_at: DateTime,
        pub author: Option<UserInfoUserIssuesNodesAuthor>,
        pub participants: UserInfoUserIssuesNodesParticipants,
        pub assignees: UserInfoUserIssuesNodesAssignees,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserIssuesNodesAuthor {
        pub login: String,
        #[serde(flatten)]
        pub on: UserInfoUserIssuesNodesAuthorOn,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum UserInfoUserIssuesNodesAuthorOn {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserIssuesNodesParticipants {
        #[serde(rename = "totalCount")]
        pub total_count: Int,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserIssuesNodesAssignees {
        pub nodes: Option<Vec<Option<UserInfoUserIssuesNodesAssigneesNodes>>>,
    }
    #[derive(Deserialize, Debug)]
    pub struct UserInfoUserIssuesNodesAssigneesNodes {
        pub login: String,
    }
}
impl graphql_client::GraphQLQuery for UserInfo {
    type Variables = user_info::Variables;
    type ResponseData = user_info::ResponseData;
    fn build_query(
        variables: Self::Variables,
    ) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: user_info::QUERY,
            operation_name: user_info::OPERATION_NAME,
        }
    }
}
