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

import { Injectable } from "@angular/core";
import { TauriService } from "./tauri.service";
import { PullRequestEntry } from "../types";
import { BehaviorSubject } from "rxjs";

export type UserPullRequests = {
  own: PullRequestEntry[];
  involved: PullRequestEntry[];
};

@Injectable({
  providedIn: "root",
})
export class UserPullRequestsService {
  private userPullRequests: {
    [id: string]: BehaviorSubject<UserPullRequests>;
  } = {};

  public constructor(private tauriSvc: TauriService) {}

  public async updateUser(login: string): Promise<void> {
    try {
      let own = await this.tauriSvc.getPullRequestsByAuthor(login);
      let involved = await this.tauriSvc.getInvolvedPullRequests(login);

      let subject = this.getSubjectForUser(login);
      subject.next({ own: own, involved: involved });
    } catch (err) {
      console.error(`unable to update user '${login}: `, err);
    }
  }

  private getSubjectForUser(login: string): BehaviorSubject<UserPullRequests> {
    if (!(login in this.userPullRequests)) {
      this.userPullRequests[login] = new BehaviorSubject<UserPullRequests>({
        own: [],
        involved: [],
      });
    }
    return this.userPullRequests[login];
  }

  public getPullRequests(login: string): BehaviorSubject<UserPullRequests> {
    return this.getSubjectForUser(login);
  }
}
