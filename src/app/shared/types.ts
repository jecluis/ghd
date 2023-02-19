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

export type GithubUser = {
  id: number;
  login: string;
  name: string;
  avatar_url: string;
};

export type PullRequestEntry = {
  id: number;
  author: string;
  author_id: string;
  url: string;
  html_url: string;
  number: number;
  title: string;
  repo_owner: string;
  repo_name: string;
  state: string;
  is_draft: boolean;
  milestone?: string;
  comments: number;
  created_at: number;
  updated_at: number;
  closed_at?: number;
  merged_at?: number;
  last_viewed?: number;
};
