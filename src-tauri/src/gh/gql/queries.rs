#![allow(clippy::all, warnings)]
pub struct UserInfo;
pub mod user_info {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "UserInfo";
    pub const QUERY : & str = "query UserInfo($login: String!) {\n  user(login: $login) {\n    name\n    login\n    avatarUrl\n    databaseId\n    pullRequests(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        totalCommentsCount\n        createdAt\n        updatedAt\n        isDraft\n        milestone {\n          id\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n      }\n    }\n    issues(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        updatedAt\n        author {\n          __typename\n          login\n        }\n        participants(first: 1) {\n          totalCount\n        }\n        assignees(first: 10) {\n          nodes {\n            login\n          }\n        }\n      }\n    }\n  }\n}\n\nfragment UserFragment on User {\n  login\n  databaseId\n  name\n  avatarUrl\n}\n\nquery SearchIssues($q: String!) {\n  search(first: 100, query: $q, type: ISSUE) {\n    nodes {\n      __typename\n      ... on PullRequest {\n        title\n        number\n        databaseId\n        author {\n          __typename\n          ...UserFragment\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        url\n        state\n        isDraft\n        totalCommentsCount\n        reviewRequests(first: 30) {\n          nodes {\n            requestedReviewer {\n              __typename\n              ...UserFragment\n            }\n          }\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n        createdAt\n        updatedAt\n        closedAt\n        mergedAt\n      }\n      ... on Issue {\n        title\n        number\n        databaseId\n        url\n        author {\n          __typename\n          ...UserFragment\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        state\n        createdAt\n        updatedAt\n        closedAt\n        comments(first: 100) {\n          nodes {\n            author {\n              __typename\n              ...UserFragment\n            }\n          }\n        }\n      }\n    }\n  }\n}\n\nquery GetPullRequestInfo($owner: String!, $repo: String!, $prid: Int!) {\n  repository(owner: $owner, name: $repo) {\n    pullRequest(number: $prid) {\n      number\n      title\n      bodyHTML\n      author {\n        __typename\n        ...UserFragment\n      }\n      repository {\n        owner {\n          __typename\n          login\n        }\n        name\n      }\n      url\n      state\n      isDraft\n      milestone {\n        title\n        state\n        dueOn\n      }\n      labels(first: 100) {\n        nodes {\n          color\n          name\n        }\n      }\n      totalCommentsCount\n      comments(first: 100) {\n        nodes {\n          author {\n            __typename\n            ...UserFragment\n          }\n        }\n      }\n      reviews(first: 100) {\n        nodes {\n          author {\n            __typename\n            ...UserFragment\n          }\n          state\n        }\n      }\n      participants(first: 100) {\n        nodes {\n          login\n          name\n          avatarUrl\n          databaseId\n        }\n      }\n    }\n  }\n}\n" ;
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
pub struct SearchIssues;
pub mod search_issues {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "SearchIssues";
    pub const QUERY : & str = "query UserInfo($login: String!) {\n  user(login: $login) {\n    name\n    login\n    avatarUrl\n    databaseId\n    pullRequests(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        totalCommentsCount\n        createdAt\n        updatedAt\n        isDraft\n        milestone {\n          id\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n      }\n    }\n    issues(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        updatedAt\n        author {\n          __typename\n          login\n        }\n        participants(first: 1) {\n          totalCount\n        }\n        assignees(first: 10) {\n          nodes {\n            login\n          }\n        }\n      }\n    }\n  }\n}\n\nfragment UserFragment on User {\n  login\n  databaseId\n  name\n  avatarUrl\n}\n\nquery SearchIssues($q: String!) {\n  search(first: 100, query: $q, type: ISSUE) {\n    nodes {\n      __typename\n      ... on PullRequest {\n        title\n        number\n        databaseId\n        author {\n          __typename\n          ...UserFragment\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        url\n        state\n        isDraft\n        totalCommentsCount\n        reviewRequests(first: 30) {\n          nodes {\n            requestedReviewer {\n              __typename\n              ...UserFragment\n            }\n          }\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n        createdAt\n        updatedAt\n        closedAt\n        mergedAt\n      }\n      ... on Issue {\n        title\n        number\n        databaseId\n        url\n        author {\n          __typename\n          ...UserFragment\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        state\n        createdAt\n        updatedAt\n        closedAt\n        comments(first: 100) {\n          nodes {\n            author {\n              __typename\n              ...UserFragment\n            }\n          }\n        }\n      }\n    }\n  }\n}\n\nquery GetPullRequestInfo($owner: String!, $repo: String!, $prid: Int!) {\n  repository(owner: $owner, name: $repo) {\n    pullRequest(number: $prid) {\n      number\n      title\n      bodyHTML\n      author {\n        __typename\n        ...UserFragment\n      }\n      repository {\n        owner {\n          __typename\n          login\n        }\n        name\n      }\n      url\n      state\n      isDraft\n      milestone {\n        title\n        state\n        dueOn\n      }\n      labels(first: 100) {\n        nodes {\n          color\n          name\n        }\n      }\n      totalCommentsCount\n      comments(first: 100) {\n        nodes {\n          author {\n            __typename\n            ...UserFragment\n          }\n        }\n      }\n      reviews(first: 100) {\n        nodes {\n          author {\n            __typename\n            ...UserFragment\n          }\n          state\n        }\n      }\n      participants(first: 100) {\n        nodes {\n          login\n          name\n          avatarUrl\n          databaseId\n        }\n      }\n    }\n  }\n}\n" ;
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
    pub enum IssueState {
        CLOSED,
        OPEN,
        Other(String),
    }
    impl ::serde::Serialize for IssueState {
        fn serialize<S: serde::Serializer>(
            &self,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                IssueState::CLOSED => "CLOSED",
                IssueState::OPEN => "OPEN",
                IssueState::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for IssueState {
        fn deserialize<D: ::serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "CLOSED" => Ok(IssueState::CLOSED),
                "OPEN" => Ok(IssueState::OPEN),
                _ => Ok(IssueState::Other(s)),
            }
        }
    }
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
    #[derive(Debug)]
    pub enum PullRequestState {
        CLOSED,
        MERGED,
        OPEN,
        Other(String),
    }
    impl ::serde::Serialize for PullRequestState {
        fn serialize<S: serde::Serializer>(
            &self,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                PullRequestState::CLOSED => "CLOSED",
                PullRequestState::MERGED => "MERGED",
                PullRequestState::OPEN => "OPEN",
                PullRequestState::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for PullRequestState {
        fn deserialize<D: ::serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "CLOSED" => Ok(PullRequestState::CLOSED),
                "MERGED" => Ok(PullRequestState::MERGED),
                "OPEN" => Ok(PullRequestState::OPEN),
                _ => Ok(PullRequestState::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub q: String,
    }
    impl Variables {}
    #[derive(Deserialize, Debug)]
    pub struct UserFragment {
        pub login: String,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        pub name: Option<String>,
        #[serde(rename = "avatarUrl")]
        pub avatar_url: URI,
    }
    #[derive(Deserialize, Debug)]
    pub struct ResponseData {
        pub search: SearchIssuesSearch,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearch {
        pub nodes: Option<Vec<Option<SearchIssuesSearchNodes>>>,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodes {
        App,
        Discussion,
        Issue(SearchIssuesSearchNodesOnIssue),
        MarketplaceListing,
        Organization,
        PullRequest(SearchIssuesSearchNodesOnPullRequest),
        Repository,
        User,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnIssue {
        pub title: String,
        pub number: Int,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        pub url: URI,
        pub author: Option<SearchIssuesSearchNodesOnIssueAuthor>,
        pub repository: SearchIssuesSearchNodesOnIssueRepository,
        pub state: IssueState,
        #[serde(rename = "createdAt")]
        pub created_at: DateTime,
        #[serde(rename = "updatedAt")]
        pub updated_at: DateTime,
        #[serde(rename = "closedAt")]
        pub closed_at: Option<DateTime>,
        pub comments: SearchIssuesSearchNodesOnIssueComments,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodesOnIssueAuthor {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User(SearchIssuesSearchNodesOnIssueAuthorOnUser),
    }
    pub type SearchIssuesSearchNodesOnIssueAuthorOnUser = UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnIssueRepository {
        pub owner: SearchIssuesSearchNodesOnIssueRepositoryOwner,
        pub name: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnIssueRepositoryOwner {
        pub login: String,
        #[serde(flatten)]
        pub on: SearchIssuesSearchNodesOnIssueRepositoryOwnerOn,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodesOnIssueRepositoryOwnerOn {
        Organization,
        User,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnIssueComments {
        pub nodes:
            Option<Vec<Option<SearchIssuesSearchNodesOnIssueCommentsNodes>>>,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnIssueCommentsNodes {
        pub author: Option<SearchIssuesSearchNodesOnIssueCommentsNodesAuthor>,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodesOnIssueCommentsNodesAuthor {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User(SearchIssuesSearchNodesOnIssueCommentsNodesAuthorOnUser),
    }
    pub type SearchIssuesSearchNodesOnIssueCommentsNodesAuthorOnUser =
        UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnPullRequest {
        pub title: String,
        pub number: Int,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        pub author: Option<SearchIssuesSearchNodesOnPullRequestAuthor>,
        pub repository: SearchIssuesSearchNodesOnPullRequestRepository,
        pub url: URI,
        pub state: PullRequestState,
        #[serde(rename = "isDraft")]
        pub is_draft: Boolean,
        #[serde(rename = "totalCommentsCount")]
        pub total_comments_count: Option<Int>,
        #[serde(rename = "reviewRequests")]
        pub review_requests:
            Option<SearchIssuesSearchNodesOnPullRequestReviewRequests>,
        #[serde(rename = "reviewDecision")]
        pub review_decision: Option<PullRequestReviewDecision>,
        pub reviews: Option<SearchIssuesSearchNodesOnPullRequestReviews>,
        #[serde(rename = "createdAt")]
        pub created_at: DateTime,
        #[serde(rename = "updatedAt")]
        pub updated_at: DateTime,
        #[serde(rename = "closedAt")]
        pub closed_at: Option<DateTime>,
        #[serde(rename = "mergedAt")]
        pub merged_at: Option<DateTime>,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodesOnPullRequestAuthor {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User(SearchIssuesSearchNodesOnPullRequestAuthorOnUser),
    }
    pub type SearchIssuesSearchNodesOnPullRequestAuthorOnUser = UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnPullRequestRepository {
        pub owner: SearchIssuesSearchNodesOnPullRequestRepositoryOwner,
        pub name: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnPullRequestRepositoryOwner {
        pub login: String,
        #[serde(flatten)]
        pub on: SearchIssuesSearchNodesOnPullRequestRepositoryOwnerOn,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodesOnPullRequestRepositoryOwnerOn {
        Organization,
        User,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnPullRequestReviewRequests {
        pub nodes: Option<
            Vec<
                Option<SearchIssuesSearchNodesOnPullRequestReviewRequestsNodes>,
            >,
        >,
    }
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnPullRequestReviewRequestsNodes { # [serde (rename = "requestedReviewer")] pub requested_reviewer : Option < SearchIssuesSearchNodesOnPullRequestReviewRequestsNodesRequestedReviewer > , }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum SearchIssuesSearchNodesOnPullRequestReviewRequestsNodesRequestedReviewer
    {
        Mannequin , Team , User (SearchIssuesSearchNodesOnPullRequestReviewRequestsNodesRequestedReviewerOnUser) }
    pub type SearchIssuesSearchNodesOnPullRequestReviewRequestsNodesRequestedReviewerOnUser =
        UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct SearchIssuesSearchNodesOnPullRequestReviews {
        #[serde(rename = "totalCount")]
        pub total_count: Int,
    }
}
impl graphql_client::GraphQLQuery for SearchIssues {
    type Variables = search_issues::Variables;
    type ResponseData = search_issues::ResponseData;
    fn build_query(
        variables: Self::Variables,
    ) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: search_issues::QUERY,
            operation_name: search_issues::OPERATION_NAME,
        }
    }
}
pub struct GetPullRequestInfo;
pub mod get_pull_request_info {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "GetPullRequestInfo";
    pub const QUERY : & str = "query UserInfo($login: String!) {\n  user(login: $login) {\n    name\n    login\n    avatarUrl\n    databaseId\n    pullRequests(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        totalCommentsCount\n        createdAt\n        updatedAt\n        isDraft\n        milestone {\n          id\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n      }\n    }\n    issues(first: 30, states: OPEN) {\n      nodes {\n        title\n        number\n        databaseId\n        updatedAt\n        author {\n          __typename\n          login\n        }\n        participants(first: 1) {\n          totalCount\n        }\n        assignees(first: 10) {\n          nodes {\n            login\n          }\n        }\n      }\n    }\n  }\n}\n\nfragment UserFragment on User {\n  login\n  databaseId\n  name\n  avatarUrl\n}\n\nquery SearchIssues($q: String!) {\n  search(first: 100, query: $q, type: ISSUE) {\n    nodes {\n      __typename\n      ... on PullRequest {\n        title\n        number\n        databaseId\n        author {\n          __typename\n          ...UserFragment\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        url\n        state\n        isDraft\n        totalCommentsCount\n        reviewRequests(first: 30) {\n          nodes {\n            requestedReviewer {\n              __typename\n              ...UserFragment\n            }\n          }\n        }\n        reviewDecision\n        reviews(first: 1) {\n          totalCount\n        }\n        createdAt\n        updatedAt\n        closedAt\n        mergedAt\n      }\n      ... on Issue {\n        title\n        number\n        databaseId\n        url\n        author {\n          __typename\n          ...UserFragment\n        }\n        repository {\n          owner {\n            __typename\n            login\n          }\n          name\n        }\n        state\n        createdAt\n        updatedAt\n        closedAt\n        comments(first: 100) {\n          nodes {\n            author {\n              __typename\n              ...UserFragment\n            }\n          }\n        }\n      }\n    }\n  }\n}\n\nquery GetPullRequestInfo($owner: String!, $repo: String!, $prid: Int!) {\n  repository(owner: $owner, name: $repo) {\n    pullRequest(number: $prid) {\n      number\n      title\n      bodyHTML\n      author {\n        __typename\n        ...UserFragment\n      }\n      repository {\n        owner {\n          __typename\n          login\n        }\n        name\n      }\n      url\n      state\n      isDraft\n      milestone {\n        title\n        state\n        dueOn\n      }\n      labels(first: 100) {\n        nodes {\n          color\n          name\n        }\n      }\n      totalCommentsCount\n      comments(first: 100) {\n        nodes {\n          author {\n            __typename\n            ...UserFragment\n          }\n        }\n      }\n      reviews(first: 100) {\n        nodes {\n          author {\n            __typename\n            ...UserFragment\n          }\n          state\n        }\n      }\n      participants(first: 100) {\n        nodes {\n          login\n          name\n          avatarUrl\n          databaseId\n        }\n      }\n    }\n  }\n}\n" ;
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
    type HTML = crate::gh::gql::custom_types::HTML;
    type URI = crate::gh::gql::custom_types::URI;
    #[derive(Debug)]
    pub enum MilestoneState {
        CLOSED,
        OPEN,
        Other(String),
    }
    impl ::serde::Serialize for MilestoneState {
        fn serialize<S: serde::Serializer>(
            &self,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                MilestoneState::CLOSED => "CLOSED",
                MilestoneState::OPEN => "OPEN",
                MilestoneState::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for MilestoneState {
        fn deserialize<D: ::serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "CLOSED" => Ok(MilestoneState::CLOSED),
                "OPEN" => Ok(MilestoneState::OPEN),
                _ => Ok(MilestoneState::Other(s)),
            }
        }
    }
    #[derive(Debug)]
    pub enum PullRequestReviewState {
        APPROVED,
        CHANGES_REQUESTED,
        COMMENTED,
        DISMISSED,
        PENDING,
        Other(String),
    }
    impl ::serde::Serialize for PullRequestReviewState {
        fn serialize<S: serde::Serializer>(
            &self,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                PullRequestReviewState::APPROVED => "APPROVED",
                PullRequestReviewState::CHANGES_REQUESTED => {
                    "CHANGES_REQUESTED"
                }
                PullRequestReviewState::COMMENTED => "COMMENTED",
                PullRequestReviewState::DISMISSED => "DISMISSED",
                PullRequestReviewState::PENDING => "PENDING",
                PullRequestReviewState::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for PullRequestReviewState {
        fn deserialize<D: ::serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "APPROVED" => Ok(PullRequestReviewState::APPROVED),
                "CHANGES_REQUESTED" => {
                    Ok(PullRequestReviewState::CHANGES_REQUESTED)
                }
                "COMMENTED" => Ok(PullRequestReviewState::COMMENTED),
                "DISMISSED" => Ok(PullRequestReviewState::DISMISSED),
                "PENDING" => Ok(PullRequestReviewState::PENDING),
                _ => Ok(PullRequestReviewState::Other(s)),
            }
        }
    }
    #[derive(Debug)]
    pub enum PullRequestState {
        CLOSED,
        MERGED,
        OPEN,
        Other(String),
    }
    impl ::serde::Serialize for PullRequestState {
        fn serialize<S: serde::Serializer>(
            &self,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                PullRequestState::CLOSED => "CLOSED",
                PullRequestState::MERGED => "MERGED",
                PullRequestState::OPEN => "OPEN",
                PullRequestState::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for PullRequestState {
        fn deserialize<D: ::serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "CLOSED" => Ok(PullRequestState::CLOSED),
                "MERGED" => Ok(PullRequestState::MERGED),
                "OPEN" => Ok(PullRequestState::OPEN),
                _ => Ok(PullRequestState::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub owner: String,
        pub repo: String,
        pub prid: Int,
    }
    impl Variables {}
    #[derive(Deserialize, Debug)]
    pub struct UserFragment {
        pub login: String,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
        pub name: Option<String>,
        #[serde(rename = "avatarUrl")]
        pub avatar_url: URI,
    }
    #[derive(Deserialize, Debug)]
    pub struct ResponseData {
        pub repository: Option<GetPullRequestInfoRepository>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepository {
        #[serde(rename = "pullRequest")]
        pub pull_request: Option<GetPullRequestInfoRepositoryPullRequest>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequest {
        pub number: Int,
        pub title: String,
        #[serde(rename = "bodyHTML")]
        pub body_html: HTML,
        pub author: Option<GetPullRequestInfoRepositoryPullRequestAuthor>,
        pub repository: GetPullRequestInfoRepositoryPullRequestRepository,
        pub url: URI,
        pub state: PullRequestState,
        #[serde(rename = "isDraft")]
        pub is_draft: Boolean,
        pub milestone: Option<GetPullRequestInfoRepositoryPullRequestMilestone>,
        pub labels: Option<GetPullRequestInfoRepositoryPullRequestLabels>,
        #[serde(rename = "totalCommentsCount")]
        pub total_comments_count: Option<Int>,
        pub comments: GetPullRequestInfoRepositoryPullRequestComments,
        pub reviews: Option<GetPullRequestInfoRepositoryPullRequestReviews>,
        pub participants: GetPullRequestInfoRepositoryPullRequestParticipants,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum GetPullRequestInfoRepositoryPullRequestAuthor {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User(GetPullRequestInfoRepositoryPullRequestAuthorOnUser),
    }
    pub type GetPullRequestInfoRepositoryPullRequestAuthorOnUser = UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestRepository {
        pub owner: GetPullRequestInfoRepositoryPullRequestRepositoryOwner,
        pub name: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestRepositoryOwner {
        pub login: String,
        #[serde(flatten)]
        pub on: GetPullRequestInfoRepositoryPullRequestRepositoryOwnerOn,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum GetPullRequestInfoRepositoryPullRequestRepositoryOwnerOn {
        Organization,
        User,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestMilestone {
        pub title: String,
        pub state: MilestoneState,
        #[serde(rename = "dueOn")]
        pub due_on: Option<DateTime>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestLabels {
        pub nodes: Option<
            Vec<Option<GetPullRequestInfoRepositoryPullRequestLabelsNodes>>,
        >,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestLabelsNodes {
        pub color: String,
        pub name: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestComments {
        pub nodes: Option<
            Vec<Option<GetPullRequestInfoRepositoryPullRequestCommentsNodes>>,
        >,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestCommentsNodes {
        pub author:
            Option<GetPullRequestInfoRepositoryPullRequestCommentsNodesAuthor>,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum GetPullRequestInfoRepositoryPullRequestCommentsNodesAuthor {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User(GetPullRequestInfoRepositoryPullRequestCommentsNodesAuthorOnUser),
    }
    pub type GetPullRequestInfoRepositoryPullRequestCommentsNodesAuthorOnUser =
        UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestReviews {
        pub nodes: Option<
            Vec<Option<GetPullRequestInfoRepositoryPullRequestReviewsNodes>>,
        >,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestReviewsNodes {
        pub author:
            Option<GetPullRequestInfoRepositoryPullRequestReviewsNodesAuthor>,
        pub state: PullRequestReviewState,
    }
    #[derive(Deserialize, Debug)]
    #[serde(tag = "__typename")]
    pub enum GetPullRequestInfoRepositoryPullRequestReviewsNodesAuthor {
        Bot,
        EnterpriseUserAccount,
        Mannequin,
        Organization,
        User(GetPullRequestInfoRepositoryPullRequestReviewsNodesAuthorOnUser),
    }
    pub type GetPullRequestInfoRepositoryPullRequestReviewsNodesAuthorOnUser =
        UserFragment;
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestParticipants {
        pub nodes: Option<
            Vec<
                Option<
                    GetPullRequestInfoRepositoryPullRequestParticipantsNodes,
                >,
            >,
        >,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPullRequestInfoRepositoryPullRequestParticipantsNodes {
        pub login: String,
        pub name: Option<String>,
        #[serde(rename = "avatarUrl")]
        pub avatar_url: URI,
        #[serde(rename = "databaseId")]
        pub database_id: Option<Int>,
    }
}
impl graphql_client::GraphQLQuery for GetPullRequestInfo {
    type Variables = get_pull_request_info::Variables;
    type ResponseData = get_pull_request_info::ResponseData;
    fn build_query(
        variables: Self::Variables,
    ) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: get_pull_request_info::QUERY,
            operation_name: get_pull_request_info::OPERATION_NAME,
        }
    }
}
