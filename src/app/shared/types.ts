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

import { Observable } from "rxjs";

export enum GHDError {
  TokenNotFoundError,
  BadTokenError,
  UserNotSetError,
  UserNotFoundError,
  NeverRefreshedError,
  BadRequest,
  UnknownError,
  NotFoundError,
  DBVersionInTheFuture,
}

export type GithubUser = {
  id: number;
  login: string;
  name: string;
  avatar_url: string;
};

export type PullRequestEntry = {
  id: number;
  number: number;
  title: string;
  author: string;
  author_id: string;
  url: string;
  repo_owner: string;
  repo_name: string;
  state: string;
  created_at: number;
  updated_at: number;
  closed_at?: number;
  is_pull_request: boolean;
  last_viewed?: number;
  is_draft: boolean;
  review_decision: string;
  merged_at?: number;
};

/// Used in the Dashboard's Pull Request Table
///
export type PRTableEntry = {
  id: number;
  number: number;
  title: string;
  author: string;
  repoOwner: string;
  url: string;
  repoName: string;
  state: string;
  lastUpdate: number;
  lastUpdateObs: Observable<string>;
  reviewDecision: string;
};

/// Used in the Dashboard's Pull Requests Table
///
export type TrackedPRs = {
  toView: PRTableEntry[];
  viewed: PRTableEntry[];
  len: number;
};

/// Represents the information for a specific Pull Request.
///
export type PullRequestInfo = {
  number: number;
  title: string;
  body_html: string;
  author: GithubUser;
  repo_owner: string;
  repo_name: string;
  url: string;
  state: string;
  is_draft: boolean;
  milestone?: Milestone;
  labels: Label[];
  total_comments: number;
  participants: GithubUser[];
  reviews: UserReview[];
};

/// Represents a Milestone
///
export type Milestone = {
  title: string;
  state: string;
  due_on?: string;
  due_on_ts?: number;
};

/// Represents a Label
///
export type Label = {
  color: string;
  name: string;
};

/// Represents a Review from a User
///
export type UserReview = {
  author: GithubUser;
  state: string;
};
